use std::collections::BTreeSet;

use tfhe::core_crypto::commons::generators::DeterministicSeeder;
use tfhe::core_crypto::commons::math::random::Seed;
use tfhe::core_crypto::algorithms::lwe_bootstrap_key_conversion::convert_standard_lwe_bootstrap_key_to_fourier;
use tfhe::core_crypto::algorithms::lwe_bootstrap_key_generation::{
    par_allocate_and_generate_new_lwe_bootstrap_key,
    par_allocate_and_generate_new_seeded_lwe_bootstrap_key,
};
use tfhe::core_crypto::algorithms::lwe_programmable_bootstrapping::{
    blind_rotate_and_extract_two_coefficients, blind_rotate_assign,
    generate_programmable_bootstrap_glwe_lut,
    programmable_bootstrap_lwe_ciphertext,
};
use tfhe::core_crypto::algorithms::misc::divide_round;
use tfhe::core_crypto::algorithms::modulus_switch::lwe_ciphertext_centered_binary_modulus_switch;
use tfhe::core_crypto::algorithms::glwe_sample_extraction::extract_lwe_sample_from_glwe_ciphertext;
use tfhe::core_crypto::prelude::*;
const DEFAULT_SAMPLES_PER_MESSAGE: usize = 32;
const DEFAULT_EDGE_WINDOW: usize = 16;
const DEFAULT_BITS: [usize; 5] = [4, 6, 8, 9, 10];

#[derive(Clone, Copy)]
struct PbsParams {
    small_lwe_dimension: LweDimension,
    glwe_dimension: GlweDimension,
    polynomial_size: PolynomialSize,
    lwe_noise_distribution: DynamicDistribution<u64>,
    glwe_noise_distribution: DynamicDistribution<u64>,
    pbs_base_log: DecompositionBaseLog,
    pbs_level: DecompositionLevelCount,
    ciphertext_modulus: CiphertextModulus<u64>,
}

#[derive(Clone, Copy)]
enum MsMode {
    Standard,
    CenteredMean,
}

#[derive(Clone, Copy)]
enum DiagMode {
    StandardIdentity,
    ManyLutIdentityTwoOutputs,
    SdrPbsIdentityBox4,
}

#[derive(Clone)]
struct FailureExample {
    message: u64,
    trial: usize,
    raw: u64,
    decoded: u64,
    signed_error: i64,
}

#[derive(Default)]
struct IdentityStats {
    total_trials: usize,
    exact_trials: usize,
    padding_wrap_trials: usize,
    out_of_range_trials: usize,
    off_by_more_than_1: usize,
    off_by_more_than_2: usize,
    off_by_more_than_4: usize,
    off_by_more_than_8: usize,
    max_abs_error: u64,
    examples: Vec<FailureExample>,
}

impl IdentityStats {
    fn record(
        &mut self,
        expected: u64,
        trial: usize,
        raw: u64,
        decoded: u64,
        error_modulus: u64,
        valid_output_upper_bound: u64,
    ) {
        self.total_trials += 1;

        if raw >= error_modulus {
            self.padding_wrap_trials += 1;
        }

        if decoded >= valid_output_upper_bound {
            self.out_of_range_trials += 1;
        }

        let signed_error = cyclic_signed_error(decoded, expected, error_modulus);
        let abs_error = signed_error.unsigned_abs();
        self.max_abs_error = self.max_abs_error.max(abs_error);

        if signed_error == 0 {
            self.exact_trials += 1;
            return;
        }

        if abs_error > 1 {
            self.off_by_more_than_1 += 1;
        }
        if abs_error > 2 {
            self.off_by_more_than_2 += 1;
        }
        if abs_error > 4 {
            self.off_by_more_than_4 += 1;
        }
        if abs_error > 8 {
            self.off_by_more_than_8 += 1;
        }

        if self.examples.len() < 20 {
            self.examples.push(FailureExample {
                message: expected,
                trial,
                raw,
                decoded,
                signed_error,
            });
        }
    }
}

fn cyclic_signed_error(decoded: u64, expected: u64, modulus: u64) -> i64 {
    let forward = (decoded + modulus - expected) % modulus;
    let backward = (expected + modulus - decoded) % modulus;

    if forward <= backward {
        forward as i64
    } else {
        -(backward as i64)
    }
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
}

fn parse_env_u128(name: &str) -> Option<u128> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u128>().ok())
}

fn parse_bits() -> Vec<usize> {
    let mut bits = Vec::new();

    if let Ok(raw) = std::env::var("DIAG_BITS") {
        for part in raw.split(',') {
            if let Ok(value) = part.trim().parse::<usize>() {
                bits.push(value);
            }
        }
    }

    if bits.is_empty() {
        bits.extend(DEFAULT_BITS);
    }

    bits
}

fn parse_env_usize_list(name: &str, default: &[usize]) -> Vec<usize> {
    let parsed = std::env::var(name)
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|part| part.trim().parse::<usize>().ok())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if parsed.is_empty() {
        default.to_vec()
    } else {
        parsed
    }
}

fn parse_env_bool(name: &str, default: bool) -> bool {
    std::env::var(name)
        .ok()
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => true,
            "0" | "false" | "no" | "off" => false,
            _ => default,
        })
        .unwrap_or(default)
}

fn parse_ms_mode() -> MsMode {
    match std::env::var("DIAG_MS_MODE")
        .unwrap_or_else(|_| "standard".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "centered" | "centered_mean" | "centered-mean" => MsMode::CenteredMean,
        _ => MsMode::Standard,
    }
}

fn parse_diag_mode() -> DiagMode {
    match std::env::var("DIAG_MODE")
        .unwrap_or_else(|_| "standard_identity".to_string())
        .to_ascii_lowercase()
        .as_str()
    {
        "many_lut" | "many_lut_identity" | "many_lut_identity_2" => {
            DiagMode::ManyLutIdentityTwoOutputs
        }
        "sdr" | "sdr_pbs" | "sdr_identity" | "sdr_pbs_identity" => {
            DiagMode::SdrPbsIdentityBox4
        }
        _ => DiagMode::StandardIdentity,
    }
}

fn sample_messages(modulus: u64, edge_window: usize, message_limit: Option<usize>) -> Vec<u64> {
    match message_limit {
        Some(limit) if limit > 0 && limit < modulus as usize => {
            let mut picked = BTreeSet::new();
            let edge = edge_window.min(modulus as usize);

            for msg in 0..edge {
                picked.insert(msg as u64);
            }
            for msg in modulus.saturating_sub(edge as u64)..modulus {
                picked.insert(msg);
            }

            let remaining = limit.saturating_sub(picked.len());
            if remaining > 0 {
                for idx in 0..remaining {
                    let numerator = (idx as u128) * (modulus as u128);
                    let denominator = remaining as u128;
                    let candidate = (numerator / denominator).min((modulus - 1) as u128) as u64;
                    picked.insert(candidate);
                }
            }

            picked.into_iter().collect()
        }
        _ => (0..modulus).collect(),
    }
}

fn build_identity_accumulator(
    polynomial_size: PolynomialSize,
    glwe_size: GlweSize,
    message_modulus: usize,
    ciphertext_modulus: CiphertextModulus<u64>,
    delta: u64,
) -> GlweCiphertextOwned<u64> {
    generate_programmable_bootstrap_glwe_lut(
        polynomial_size,
        glwe_size,
        message_modulus,
        ciphertext_modulus,
        delta,
        |x: u64| x,
    )
}

fn build_many_identity_accumulator(
    polynomial_size: PolynomialSize,
    glwe_size: GlweSize,
    total_plaintext_modulus: usize,
    ciphertext_modulus: CiphertextModulus<u64>,
    delta: u64,
    slot_count: usize,
    input_modulus: usize,
    input_offset: usize,
    used_slots: &[usize],
) -> (GlweCiphertextOwned<u64>, usize) {
    assert!(
        polynomial_size.0 % total_plaintext_modulus == 0,
        "polynomial size must be divisible by total plaintext modulus"
    );
    assert!(
        polynomial_size.0 % slot_count == 0,
        "polynomial size must be divisible by slot count"
    );

    let box_size = polynomial_size.0 / total_plaintext_modulus;
    let func_chunk_size = polynomial_size.0 / slot_count;
    let slot_capacity = func_chunk_size / box_size;
    assert!(
        input_offset + input_modulus <= slot_capacity,
        "input placement (offset {input_offset}, width {input_modulus}) exceeds slot capacity {slot_capacity}"
    );

    let mut accumulator_scalar = vec![0u64; polynomial_size.0];

    for &slot in used_slots {
        let start = slot * func_chunk_size;
        let end = start + func_chunk_size;
        let func_chunk = &mut accumulator_scalar[start..end];

        for (msg_value, box_) in func_chunk
            .chunks_exact_mut(box_size)
            .skip(input_offset)
            .take(input_modulus)
            .enumerate()
        {
            let encoded = (msg_value as u64).wrapping_mul(delta);
            box_.fill(encoded);
        }

        // Saturate any remaining boxes in an active slot to reduce edge effects when
        // experimenting with larger total plaintext spaces than strictly necessary.
        if slot_capacity > input_offset + input_modulus && input_modulus > 0 {
            let tail_value = ((input_modulus - 1) as u64).wrapping_mul(delta);
            for box_ in func_chunk
                .chunks_exact_mut(box_size)
                .skip(input_offset + input_modulus)
            {
                box_.fill(tail_value);
            }
        }
    }

    let half_box_size = box_size / 2;
    for value in accumulator_scalar[0..half_box_size].iter_mut() {
        *value = value.wrapping_neg();
    }
    accumulator_scalar.rotate_left(half_box_size);

    let accumulator_plaintext = PlaintextList::from_container(accumulator_scalar);

    (
        allocate_and_trivially_encrypt_new_glwe_ciphertext(
            glwe_size,
            &accumulator_plaintext,
            ciphertext_modulus,
        ),
        func_chunk_size,
    )
}

fn build_sdr_pbs_identity_box4_accumulator(
    polynomial_size: PolynomialSize,
    glwe_size: GlweSize,
    interval_count: usize,
    ciphertext_modulus: CiphertextModulus<u64>,
    delta: u64,
) -> GlweCiphertextOwned<u64> {
    let mut coeffs = vec![0u64; polynomial_size.0];

    for message in 0..interval_count {
        let even_code = (2 * message as u64).wrapping_mul(delta);
        let odd_code = (2 * message as u64 + 1).wrapping_mul(delta);
        let base = 4 * message;
        coeffs[base] = even_code;
        coeffs[base + 1] = odd_code;
        coeffs[base + 2] = even_code;
        coeffs[base + 3] = odd_code;
    }

    let accumulator_plaintext = PlaintextList::from_container(coeffs);
    allocate_and_trivially_encrypt_new_glwe_ciphertext(
        glwe_size,
        &accumulator_plaintext,
        ciphertext_modulus,
    )
}

fn torus_cyclic_distance(value: u64, target: u64, modulus: u64) -> u64 {
    let diff = value.abs_diff(target);
    diff.min(modulus - diff)
}

fn nearest_sdr_code(raw: u64, parity: u64, legal_code_modulus: u64, total_modulus: u64) -> (u64, u64) {
    let mut best_code = parity;
    let mut best_distance = u64::MAX;
    let mut code = parity;

    while code < legal_code_modulus {
        let distance = torus_cyclic_distance(raw, code, total_modulus);
        if distance < best_distance {
            best_code = code;
            best_distance = distance;
        }
        code += 2;
    }

    (best_code, best_distance)
}

fn apply_blind_rotate_with_mode(
    params: PbsParams,
    ms_mode: MsMode,
    lwe_input: LweCiphertextOwned<u64>,
    accumulator: &mut GlweCiphertextOwned<u64>,
    fourier_bsk: &FourierLweBootstrapKeyOwned,
) {
    let log_modulus = params.polynomial_size.to_blind_rotation_input_modulus_log();

    match ms_mode {
        MsMode::Standard => {
            let msed = lwe_ciphertext_modulus_switch(lwe_input, log_modulus);
            blind_rotate_assign(&msed, accumulator, fourier_bsk);
        }
        MsMode::CenteredMean => {
            let msed = lwe_ciphertext_centered_binary_modulus_switch(lwe_input, log_modulus);
            blind_rotate_assign(&msed, accumulator, fourier_bsk);
        }
    }
}

fn run_identity_diagnostic(
    params: PbsParams,
    small_lwe_sk: &LweSecretKeyOwned<u64>,
    big_lwe_sk: &LweSecretKeyOwned<u64>,
    fourier_bsk: &FourierLweBootstrapKeyOwned,
    encryption_generator: &mut EncryptionRandomGenerator<DefaultRandomGenerator>,
    diag_mode: DiagMode,
    ms_mode: MsMode,
    bits: usize,
    samples_per_message: usize,
    edge_window: usize,
    message_limit: Option<usize>,
) -> IdentityStats {
    let input_modulus = 1u64 << bits;
    let many_lut_used_slots = parse_env_usize_list("DIAG_MANY_USED_SLOTS", &[0, 1]);
    let many_lut_slot_count = parse_env_usize(
        "DIAG_MANY_SLOT_COUNT",
        many_lut_used_slots
            .iter()
            .copied()
            .max()
            .map(|max_slot| max_slot + 1)
            .unwrap_or(2),
    );
    let many_lut_total_factor = parse_env_usize("DIAG_MANY_TOTAL_FACTOR", 2);
    let many_lut_input_offset = parse_env_usize("DIAG_MANY_INPUT_OFFSET", 0);

    let (
        delta,
        accumulator,
        extract_positions,
        comparison_modulus,
        valid_output_upper_bound,
    ): (u64, GlweCiphertextOwned<u64>, Vec<usize>, u64, u64) = match diag_mode {
            DiagMode::StandardIdentity => {
                let delta = (1u64 << 63) / input_modulus;
                let accumulator = build_identity_accumulator(
                    params.polynomial_size,
                    params.glwe_dimension.to_glwe_size(),
                    input_modulus as usize,
                    params.ciphertext_modulus,
                    delta,
                );
                (delta, accumulator, vec![0], input_modulus, input_modulus)
            }
            DiagMode::ManyLutIdentityTwoOutputs => {
                let total_plaintext_modulus = input_modulus * many_lut_total_factor as u64;
                let delta = (1u64 << 63) / total_plaintext_modulus;
                let (accumulator, func_chunk_size) = build_many_identity_accumulator(
                    params.polynomial_size,
                    params.glwe_dimension.to_glwe_size(),
                    total_plaintext_modulus as usize,
                    params.ciphertext_modulus,
                    delta,
                    many_lut_slot_count,
                    input_modulus as usize,
                    many_lut_input_offset,
                    &many_lut_used_slots,
                );
                (
                    delta,
                    accumulator,
                    many_lut_used_slots
                        .iter()
                        .map(|slot| slot * func_chunk_size)
                        .collect(),
                    total_plaintext_modulus,
                    input_modulus,
                )
            }
            DiagMode::SdrPbsIdentityBox4 => {
                let log_modulus = params.polynomial_size.to_blind_rotation_input_modulus_log();
                let delta = 1u64 << (64 - log_modulus.0);
                let accumulator = build_sdr_pbs_identity_box4_accumulator(
                    params.polynomial_size,
                    params.glwe_dimension.to_glwe_size(),
                    input_modulus as usize,
                    params.ciphertext_modulus,
                    delta,
                );
                (delta, accumulator, Vec::new(), input_modulus, input_modulus)
            }
        };

    let mut stats = IdentityStats::default();
    let messages = sample_messages(input_modulus, edge_window, message_limit);

    for &message in &messages {
        for trial in 0..samples_per_message {
            let plaintext = match diag_mode {
                DiagMode::SdrPbsIdentityBox4 => Plaintext((4 * message).wrapping_mul(delta)),
                DiagMode::ManyLutIdentityTwoOutputs => {
                    Plaintext((message + many_lut_input_offset as u64).wrapping_mul(delta))
                }
                DiagMode::StandardIdentity => Plaintext(message * delta),
            };
            let lwe_input = allocate_and_encrypt_new_lwe_ciphertext(
                small_lwe_sk,
                plaintext,
                params.lwe_noise_distribution,
                params.ciphertext_modulus,
                encryption_generator,
            );

            if matches!(diag_mode, DiagMode::StandardIdentity) && matches!(ms_mode, MsMode::Standard)
            {
                let mut lwe_output = LweCiphertext::new(
                    0u64,
                    big_lwe_sk.lwe_dimension().to_lwe_size(),
                    params.ciphertext_modulus,
                );
                programmable_bootstrap_lwe_ciphertext(
                    &lwe_input,
                    &mut lwe_output,
                    &accumulator,
                    fourier_bsk,
                );

                let decrypted = decrypt_lwe_ciphertext(big_lwe_sk, &lwe_output).0;
                let raw = divide_round(decrypted, delta);
                let decoded = raw % comparison_modulus;
                stats.record(
                    message,
                    trial,
                    raw,
                    decoded,
                    comparison_modulus,
                    valid_output_upper_bound,
                );
                continue;
            }

            if matches!(diag_mode, DiagMode::SdrPbsIdentityBox4) {
                let log_modulus = params.polynomial_size.to_blind_rotation_input_modulus_log();
                let total_modulus = 1u64 << log_modulus.0;
                let legal_code_modulus = 2 * input_modulus;
                let input = match ms_mode {
                    MsMode::Standard => lwe_ciphertext_modulus_switch(
                        lwe_input,
                        log_modulus,
                    ),
                    MsMode::CenteredMean => lwe_ciphertext_centered_binary_modulus_switch(
                        lwe_input,
                        log_modulus,
                    ),
                };
                let mut local_accumulator = accumulator.clone();
                let (sample0, sample1) = blind_rotate_and_extract_two_coefficients(
                    &input,
                    &mut local_accumulator,
                    fourier_bsk,
                );

                let raw0 = divide_round(decrypt_lwe_ciphertext(big_lwe_sk, &sample0).0, delta)
                    % total_modulus;
                let raw1 = divide_round(decrypt_lwe_ciphertext(big_lwe_sk, &sample1).0, delta)
                    % total_modulus;

                let (raw0_even, raw0_even_cost) =
                    nearest_sdr_code(raw0, 0, legal_code_modulus, total_modulus);
                let (raw0_odd, raw0_odd_cost) =
                    nearest_sdr_code(raw0, 1, legal_code_modulus, total_modulus);
                let (raw1_even, raw1_even_cost) =
                    nearest_sdr_code(raw1, 0, legal_code_modulus, total_modulus);
                let (raw1_odd, raw1_odd_cost) =
                    nearest_sdr_code(raw1, 1, legal_code_modulus, total_modulus);

                let direct_cost = raw0_even_cost + raw1_odd_cost;
                let swapped_cost = raw0_odd_cost + raw1_even_cost;

                let (even_code, odd_code) = if direct_cost <= swapped_cost {
                    (raw0_even, raw1_odd)
                } else {
                    (raw1_even, raw0_odd)
                };

                stats.record(
                    message,
                    trial,
                    even_code / 2,
                    even_code / 2,
                    comparison_modulus,
                    valid_output_upper_bound,
                );
                stats.record(
                    message,
                    trial,
                    (odd_code - 1) / 2,
                    (odd_code - 1) / 2,
                    comparison_modulus,
                    valid_output_upper_bound,
                );
                continue;
            }

            let mut local_accumulator = accumulator.clone();
            apply_blind_rotate_with_mode(params, ms_mode, lwe_input, &mut local_accumulator, fourier_bsk);

            for &extract_position in &extract_positions {
                let mut lwe_output = LweCiphertext::new(
                    0u64,
                    big_lwe_sk.lwe_dimension().to_lwe_size(),
                    params.ciphertext_modulus,
                );
                extract_lwe_sample_from_glwe_ciphertext(
                    &local_accumulator,
                    &mut lwe_output,
                    MonomialDegree(extract_position),
                );

                let decrypted = decrypt_lwe_ciphertext(big_lwe_sk, &lwe_output).0;
                let raw = divide_round(decrypted, delta);
                let decoded = raw % comparison_modulus;
                stats.record(
                    message,
                    trial,
                    raw,
                    decoded,
                    comparison_modulus,
                    valid_output_upper_bound,
                );
            }
        }
    }

    stats
}

fn print_stats(bits: usize, stats: &IdentityStats) {
    let total = stats.total_trials as f64;
    let exact_rate = if total == 0.0 {
        0.0
    } else {
        stats.exact_trials as f64 / total * 100.0
    };
    let wrap_rate = if total == 0.0 {
        0.0
    } else {
        stats.padding_wrap_trials as f64 / total * 100.0
    };

    println!("\n=== Identity PBS diagnostic: {bits}-bit ===");
    println!("total trials            : {}", stats.total_trials);
    println!("exact decode rate       : {:.4}%", exact_rate);
    println!("padding-wrap rate       : {:.4}%", wrap_rate);
    println!("valid-range violations  : {}", stats.out_of_range_trials);
    println!("|error| > 1 codeword    : {}", stats.off_by_more_than_1);
    println!("|error| > 2 codewords   : {}", stats.off_by_more_than_2);
    println!("|error| > 4 codewords   : {}", stats.off_by_more_than_4);
    println!("|error| > 8 codewords   : {}", stats.off_by_more_than_8);
    println!("max |error|             : {}", stats.max_abs_error);

    if !stats.examples.is_empty() {
        println!("first failure examples  :");
        for example in &stats.examples {
            println!(
                "  msg={} trial={} raw={} decoded={} signed_error={}",
                example.message,
                example.trial,
                example.raw,
                example.decoded,
                example.signed_error
            );
        }
    }
}

fn main() {
    let lwe_dimension = parse_env_usize("DIAG_LWE_DIM", 742);
    let glwe_dimension = parse_env_usize("DIAG_GLWE_DIM", 1);
    let polynomial_size = parse_env_usize("DIAG_POLY_SIZE", 2048);
    let pbs_base_log = parse_env_usize("DIAG_PBS_BASE_LOG", 23);
    let pbs_level = parse_env_usize("DIAG_PBS_LEVEL", 1);
    let lwe_noise_std = parse_env_f64("DIAG_LWE_NOISE_STD", 0.000007069849454709433);
    let glwe_noise_std = parse_env_f64("DIAG_GLWE_NOISE_STD", 0.00000000000000029403601535432533);

    let params = PbsParams {
        small_lwe_dimension: LweDimension(lwe_dimension),
        glwe_dimension: GlweDimension(glwe_dimension),
        polynomial_size: PolynomialSize(polynomial_size),
        lwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
            lwe_noise_std,
        )),
        glwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
            glwe_noise_std,
        )),
        pbs_base_log: DecompositionBaseLog(pbs_base_log),
        pbs_level: DecompositionLevelCount(pbs_level),
        ciphertext_modulus: CiphertextModulus::new_native(),
    };

    let samples_per_message = parse_env_usize(
        "DIAG_SAMPLES_PER_MESSAGE",
        DEFAULT_SAMPLES_PER_MESSAGE,
    );
    let edge_window = parse_env_usize("DIAG_EDGE_WINDOW", DEFAULT_EDGE_WINDOW);
    let message_limit = std::env::var("DIAG_MESSAGE_LIMIT")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let bits = parse_bits();
    let use_seeded_bsk = parse_env_bool("DIAG_USE_SEEDED_BSK", true);
    let ms_mode = parse_ms_mode();
    let diag_mode = parse_diag_mode();
    let master_seed = parse_env_u128("DIAG_MASTER_SEED");

    println!("=== Identity PBS diagnostic ===");
    println!(
        "params: lwe_dim={}, glwe_dim={}, poly_size={}, pbs_base_log={}, pbs_level={}",
        params.small_lwe_dimension.0,
        params.glwe_dimension.0,
        params.polynomial_size.0,
        params.pbs_base_log.0,
        params.pbs_level.0
    );
    println!(
        "noise std: lwe={:.3e}, glwe={:.3e}",
        lwe_noise_std,
        glwe_noise_std
    );
    println!(
        "sampling: samples_per_message={}, edge_window={}, message_limit={:?}",
        samples_per_message,
        edge_window,
        message_limit
    );
    println!("bootstrap key path: {}", if use_seeded_bsk { "seeded->decompress" } else { "standard" });
    println!(
        "modulus switch mode: {}",
        match ms_mode {
            MsMode::Standard => "standard",
            MsMode::CenteredMean => "centered_mean",
        }
    );
    println!(
        "diagnostic mode: {}",
        match diag_mode {
            DiagMode::StandardIdentity => "standard_identity",
            DiagMode::ManyLutIdentityTwoOutputs => "many_lut_identity_2",
            DiagMode::SdrPbsIdentityBox4 => "sdr_pbs_identity_box4",
        }
    );
    println!(
        "master seed: {}",
        master_seed
            .map(|value| value.to_string())
            .unwrap_or_else(|| "system".to_string())
    );
    if matches!(diag_mode, DiagMode::ManyLutIdentityTwoOutputs) {
        println!(
            "many-lut layout: total_factor={}, slot_count={}, input_offset={}, used_slots={:?}",
            parse_env_usize("DIAG_MANY_TOTAL_FACTOR", 2),
            parse_env_usize("DIAG_MANY_SLOT_COUNT", 2),
            parse_env_usize("DIAG_MANY_INPUT_OFFSET", 0),
            parse_env_usize_list("DIAG_MANY_USED_SLOTS", &[0, 1])
        );
    }

    let (
        small_lwe_sk,
        big_lwe_sk,
        mut encryption_generator,
        standard_bsk,
    ): (
        LweSecretKeyOwned<u64>,
        LweSecretKeyOwned<u64>,
        EncryptionRandomGenerator<DefaultRandomGenerator>,
        LweBootstrapKeyOwned<u64>,
    ) = if let Some(seed_value) = master_seed {
        let mut seeder = DeterministicSeeder::<DefaultRandomGenerator>::new(Seed(seed_value));
        let mut secret_generator =
            SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());
        let mut encryption_generator =
            EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), &mut seeder);

        let small_lwe_sk =
            LweSecretKey::generate_new_binary(params.small_lwe_dimension, &mut secret_generator);
        let glwe_sk = GlweSecretKey::generate_new_binary(
            params.glwe_dimension,
            params.polynomial_size,
            &mut secret_generator,
        );
        let big_lwe_sk = glwe_sk.clone().into_lwe_secret_key();

        println!("generating bootstrap key...");
        let standard_bsk = if use_seeded_bsk {
            let seeded_bsk = par_allocate_and_generate_new_seeded_lwe_bootstrap_key(
                &small_lwe_sk,
                &glwe_sk,
                params.pbs_base_log,
                params.pbs_level,
                params.glwe_noise_distribution,
                params.ciphertext_modulus,
                &mut seeder,
            );

            seeded_bsk.decompress_into_lwe_bootstrap_key()
        } else {
            par_allocate_and_generate_new_lwe_bootstrap_key(
                &small_lwe_sk,
                &glwe_sk,
                params.pbs_base_log,
                params.pbs_level,
                params.glwe_noise_distribution,
                params.ciphertext_modulus,
                &mut encryption_generator,
            )
        };

        (small_lwe_sk, big_lwe_sk, encryption_generator, standard_bsk)
    } else {
        let mut boxed_seeder = new_seeder();
        let seeder = boxed_seeder.as_mut();
        let mut secret_generator =
            SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());
        let mut encryption_generator =
            EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);

        let small_lwe_sk =
            LweSecretKey::generate_new_binary(params.small_lwe_dimension, &mut secret_generator);
        let glwe_sk = GlweSecretKey::generate_new_binary(
            params.glwe_dimension,
            params.polynomial_size,
            &mut secret_generator,
        );
        let big_lwe_sk = glwe_sk.clone().into_lwe_secret_key();

        println!("generating bootstrap key...");
        let standard_bsk = if use_seeded_bsk {
            let seeded_bsk = par_allocate_and_generate_new_seeded_lwe_bootstrap_key(
                &small_lwe_sk,
                &glwe_sk,
                params.pbs_base_log,
                params.pbs_level,
                params.glwe_noise_distribution,
                params.ciphertext_modulus,
                seeder,
            );

            seeded_bsk.decompress_into_lwe_bootstrap_key()
        } else {
            par_allocate_and_generate_new_lwe_bootstrap_key(
                &small_lwe_sk,
                &glwe_sk,
                params.pbs_base_log,
                params.pbs_level,
                params.glwe_noise_distribution,
                params.ciphertext_modulus,
                &mut encryption_generator,
            )
        };

        (small_lwe_sk, big_lwe_sk, encryption_generator, standard_bsk)
    };
    let mut fourier_bsk = FourierLweBootstrapKey::new(
        standard_bsk.input_lwe_dimension(),
        standard_bsk.glwe_size(),
        standard_bsk.polynomial_size(),
        standard_bsk.decomposition_base_log(),
        standard_bsk.decomposition_level_count(),
    );
    convert_standard_lwe_bootstrap_key_to_fourier(&standard_bsk, &mut fourier_bsk);

    for bit_width in bits {
        let stats = run_identity_diagnostic(
            params,
            &small_lwe_sk,
            &big_lwe_sk,
            &fourier_bsk,
            &mut encryption_generator,
            diag_mode,
            ms_mode,
            bit_width,
            samples_per_message,
            edge_window,
            message_limit,
        );
        print_stats(bit_width, &stats);
    }
}

