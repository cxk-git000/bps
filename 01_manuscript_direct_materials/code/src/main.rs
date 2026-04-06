use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tfhe::core_crypto::commons::generators::DeterministicSeeder;
use tfhe::core_crypto::commons::math::random::Seed;
use tfhe::core_crypto::algorithms::glwe_sample_extraction::extract_lwe_sample_from_glwe_ciphertext;
use tfhe::core_crypto::algorithms::lwe_bootstrap_key_conversion::convert_standard_lwe_bootstrap_key_to_fourier;
use tfhe::core_crypto::algorithms::lwe_bootstrap_key_generation::par_allocate_and_generate_new_seeded_lwe_bootstrap_key;
use tfhe::core_crypto::algorithms::lwe_programmable_bootstrapping::{
    blind_rotate_and_extract_two_coefficients, blind_rotate_assign,
    generate_programmable_bootstrap_glwe_lut,
};
use tfhe::core_crypto::algorithms::misc::divide_round;
use tfhe::core_crypto::algorithms::modulus_switch::lwe_ciphertext_centered_binary_modulus_switch;
use tfhe::core_crypto::prelude::*;

const REL_ERROR_THRESHOLD: f64 = 0.01;
const DEFAULT_NUM_TESTS: usize = 10000;
const DEFAULT_ENCODING_CHECK_SAMPLES: usize = 200000;
const SIGNIFICANT_THRESHOLDS: [f64; 3] = [0.005, 0.01, 0.02];
const E2E_ERROR_THRESHOLD: f64 = 0.01;
const E2E_SCORE_WEIGHT_ACTIVATION: f64 = 0.65;
const E2E_SCORE_WEIGHT_DERIVATIVE: f64 = 0.35;
const E2E_UPDATE_BASELINE: f64 = 0.5;
const E2E_UPDATE_TARGET: f64 = 0.5;
const E2E_UPDATE_STEP_SIZE: f64 = 0.4;
const CSV_HEADER: &str = concat!(
    "pair,scheme,bits,points,significant_errors,sigerr_0p5,sigerr_1p0,sigerr_2p0,",
    "projection_events,invalid_outputs,",
    "mean_err1,std_err1,rmse_err1,median_err1,p90_err1,p95_err1,p99_err1,p999_err1,max_err1,",
    "mean_err2,std_err2,rmse_err2,median_err2,p90_err2,p95_err2,p99_err2,p999_err2,max_err2,",
    "avg_input_us,avg_core_us,avg_decode_us,avg_eval_us\n"
);
const CODEBOOK_CSV_HEADER: &str = concat!(
    "pair,scheme,bits,total_inputs,exact_recovery,mismatches,projection_events,invalid_outputs,",
    "joint_le1,joint_le2,code1_le1,code1_le2,code2_le1,code2_le2,",
    "mean_code_err1,mean_code_err2,max_code_err1,max_code_err2,",
    "avg_input_us,avg_core_us,avg_decode_us,avg_eval_us\n"
);
const E2E_CSV_HEADER: &str = concat!(
    "pair,scheme,bits,points,projection_events,invalid_outputs,",
    "score_significant_errors,score_mean,score_std,score_rmse,score_median,score_p90,score_p95,score_p99,score_p999,score_max,",
    "update_significant_errors,update_mean,update_std,update_rmse,update_median,update_p90,update_p95,update_p99,update_p999,update_max,",
    "avg_input_us,avg_core_us,avg_decode_us,avg_downstream_us,avg_eval_us,avg_e2e_us\n"
);
const RUNTIME_CSV_HEADER: &str =
    "scheme,selected,seed,bsk_generation_s,fourier_conversion_s\n";

#[derive(Clone, Copy)]
struct ValueEncoding {
    min: f64,
    max: f64,
}

impl ValueEncoding {
    fn range(self) -> f64 {
        self.max - self.min
    }
}

#[derive(Clone, Copy)]
struct FunctionPair {
    name: &'static str,
    test_x_min: f64,
    test_x_max: f64,
    output1: ValueEncoding,
    output2: ValueEncoding,
    compute_true: fn(f64) -> (f64, f64),
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
enum SchemeKind {
    Standard,
    SdrPbs,
    ManyLut,
}

impl SchemeKind {
    fn as_str(self) -> &'static str {
        match self {
            SchemeKind::Standard => "standard_pbs",
            SchemeKind::SdrPbs => "sdr_pbs",
            SchemeKind::ManyLut => "many_lut",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ManyLutLayout {
    total_factor: usize,
    slot_count: usize,
    input_offset: usize,
    used_slots: [usize; 2],
}

#[derive(Clone, Copy, Debug)]
struct InputGuardLayout {
    total_factor: usize,
    input_offset: usize,
}

#[derive(Clone, Copy, Debug)]
struct SchemeConfig {
    kind: SchemeKind,
    bits: usize,
    pbs: PbsParams,
    input_guard: Option<InputGuardLayout>,
    many_lut: Option<ManyLutLayout>,
}

struct Runtime {
    params: PbsParams,
    small_lwe_sk: LweSecretKeyOwned<u64>,
    big_lwe_sk: LweSecretKeyOwned<u64>,
    fourier_bsk: FourierLweBootstrapKeyOwned,
    encryption_generator: EncryptionRandomGenerator<DefaultRandomGenerator>,
    seed: Option<u128>,
    bsk_generation_time: f64,
    fourier_conversion_time: f64,
}

#[derive(Clone, Copy, Default)]
struct TimingBreakdown {
    input_us: f64,
    core_us: f64,
    decode_us: f64,
}

struct EvalStats {
    errors1: Vec<f64>,
    errors2: Vec<f64>,
    eval_times_us: Vec<f64>,
    input_times_us: Vec<f64>,
    core_times_us: Vec<f64>,
    decode_times_us: Vec<f64>,
    significant_errors: usize,
    threshold_counts: [usize; SIGNIFICANT_THRESHOLDS.len()],
    projection_events: usize,
    invalid_outputs: usize,
}

impl EvalStats {
    fn new(point_count: usize) -> Self {
        Self {
            errors1: Vec::with_capacity(point_count),
            errors2: Vec::with_capacity(point_count),
            eval_times_us: Vec::with_capacity(point_count),
            input_times_us: Vec::with_capacity(point_count),
            core_times_us: Vec::with_capacity(point_count),
            decode_times_us: Vec::with_capacity(point_count),
            significant_errors: 0,
            threshold_counts: [0; SIGNIFICANT_THRESHOLDS.len()],
            projection_events: 0,
            invalid_outputs: 0,
        }
    }

    fn record(
        &mut self,
        pair: &FunctionPair,
        reconstructed: (f64, f64),
        truth: (f64, f64),
        projection_events: usize,
        invalid_outputs: usize,
        timing: TimingBreakdown,
    ) {
        let err1 = (reconstructed.0 - truth.0).abs();
        let err2 = (reconstructed.1 - truth.1).abs();
        let thresh1 = pair.output1.range() * REL_ERROR_THRESHOLD;
        let thresh2 = pair.output2.range() * REL_ERROR_THRESHOLD;

        if err1 > thresh1 || err2 > thresh2 {
            self.significant_errors += 1;
        }

        for (index, threshold) in SIGNIFICANT_THRESHOLDS.iter().copied().enumerate() {
            let local_thresh1 = pair.output1.range() * threshold;
            let local_thresh2 = pair.output2.range() * threshold;
            if err1 > local_thresh1 || err2 > local_thresh2 {
                self.threshold_counts[index] += 1;
            }
        }

        self.projection_events += projection_events;
        self.invalid_outputs += invalid_outputs;
        self.errors1.push(err1);
        self.errors2.push(err2);
        self.input_times_us.push(timing.input_us);
        self.core_times_us.push(timing.core_us);
        self.decode_times_us.push(timing.decode_us);
        self.eval_times_us
            .push(timing.input_us + timing.core_us + timing.decode_us);
    }
}

struct SchemeReport {
    pair_name: &'static str,
    scheme_name: &'static str,
    bits: usize,
    point_count: usize,
    significant_errors: usize,
    threshold_counts: [usize; SIGNIFICANT_THRESHOLDS.len()],
    projection_events: usize,
    invalid_outputs: usize,
    avg_input_us: f64,
    avg_core_us: f64,
    avg_decode_us: f64,
    avg_eval_us: f64,
    stats1: ErrorSummary,
    stats2: ErrorSummary,
}

struct EndToEndReport {
    pair_name: &'static str,
    scheme_name: &'static str,
    bits: usize,
    point_count: usize,
    projection_events: usize,
    invalid_outputs: usize,
    score_significant_errors: usize,
    update_significant_errors: usize,
    score_stats: ErrorSummary,
    update_stats: ErrorSummary,
    avg_input_us: f64,
    avg_core_us: f64,
    avg_decode_us: f64,
    avg_downstream_us: f64,
    avg_eval_us: f64,
    avg_e2e_us: f64,
}

struct CodebookReport {
    pair_name: &'static str,
    scheme_name: &'static str,
    bits: usize,
    total_inputs: usize,
    exact_recovery: usize,
    mismatches: usize,
    projection_events: usize,
    invalid_outputs: usize,
    joint_le1: usize,
    joint_le2: usize,
    code1_le1: usize,
    code1_le2: usize,
    code2_le1: usize,
    code2_le2: usize,
    mean_code_err1: f64,
    mean_code_err2: f64,
    max_code_err1: u64,
    max_code_err2: u64,
    avg_input_us: f64,
    avg_core_us: f64,
    avg_decode_us: f64,
    avg_eval_us: f64,
}

struct EncodingCoverageReport {
    pair_name: &'static str,
    samples: usize,
    observed_min1: f64,
    observed_max1: f64,
    observed_min2: f64,
    observed_max2: f64,
    output1_clipped: bool,
    output2_clipped: bool,
}

impl EncodingCoverageReport {
    fn has_clipping(&self) -> bool {
        self.output1_clipped || self.output2_clipped
    }
}

struct EndToEndStats {
    score_errors: Vec<f64>,
    update_errors: Vec<f64>,
    input_times_us: Vec<f64>,
    core_times_us: Vec<f64>,
    decode_times_us: Vec<f64>,
    downstream_times_us: Vec<f64>,
    eval_times_us: Vec<f64>,
    e2e_times_us: Vec<f64>,
    projection_events: usize,
    invalid_outputs: usize,
    score_significant_errors: usize,
    update_significant_errors: usize,
}

impl EndToEndStats {
    fn new(point_count: usize) -> Self {
        Self {
            score_errors: Vec::with_capacity(point_count),
            update_errors: Vec::with_capacity(point_count),
            input_times_us: Vec::with_capacity(point_count),
            core_times_us: Vec::with_capacity(point_count),
            decode_times_us: Vec::with_capacity(point_count),
            downstream_times_us: Vec::with_capacity(point_count),
            eval_times_us: Vec::with_capacity(point_count),
            e2e_times_us: Vec::with_capacity(point_count),
            projection_events: 0,
            invalid_outputs: 0,
            score_significant_errors: 0,
            update_significant_errors: 0,
        }
    }

    fn record(
        &mut self,
        score_error: f64,
        update_error: f64,
        projection_events: usize,
        invalid_outputs: usize,
        timing: EndToEndTiming,
    ) {
        if score_error > E2E_ERROR_THRESHOLD {
            self.score_significant_errors += 1;
        }
        if update_error > E2E_ERROR_THRESHOLD {
            self.update_significant_errors += 1;
        }

        self.projection_events += projection_events;
        self.invalid_outputs += invalid_outputs;
        self.score_errors.push(score_error);
        self.update_errors.push(update_error);
        self.input_times_us.push(timing.input_us);
        self.core_times_us.push(timing.core_us);
        self.decode_times_us.push(timing.decode_us);
        self.downstream_times_us.push(timing.downstream_us);
        self.eval_times_us
            .push(timing.input_us + timing.core_us + timing.decode_us);
        self.e2e_times_us.push(
            timing.input_us + timing.core_us + timing.decode_us + timing.downstream_us,
        );
    }
}

#[derive(Clone, Copy)]
struct ErrorSummary {
    mean: f64,
    std: f64,
    rmse: f64,
    median: f64,
    p90: f64,
    p95: f64,
    p99: f64,
    p999: f64,
    max: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExperimentMode {
    Continuous,
    Codebook,
    EndToEnd,
}

#[derive(Clone, Copy, Default)]
struct SchemeSelection {
    standard: bool,
    sdr_pbs: bool,
    many_lut: bool,
}

#[derive(Clone, Copy)]
struct EndToEndTiming {
    input_us: f64,
    core_us: f64,
    decode_us: f64,
    downstream_us: f64,
}

#[derive(Clone, Copy)]
struct DownstreamOutputs {
    score: f64,
    update: f64,
}

fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

fn softplus(x: f64) -> f64 {
    (1.0 + x.exp()).ln()
}

fn swish(x: f64) -> f64 {
    x * sigmoid(x)
}

fn swish_deriv(x: f64) -> f64 {
    let s = sigmoid(x);
    s + x * s * (1.0 - s)
}

const ALPHA_GELU: f64 = 1.702;

fn gelu(x: f64) -> f64 {
    x * sigmoid(ALPHA_GELU * x)
}

fn gelu_deriv(x: f64) -> f64 {
    let s = sigmoid(ALPHA_GELU * x);
    s + ALPHA_GELU * x * s * (1.0 - s)
}

fn elu(x: f64) -> f64 {
    if x > 0.0 {
        x
    } else {
        x.exp() - 1.0
    }
}

fn elu_deriv(x: f64) -> f64 {
    if x > 0.0 {
        1.0
    } else {
        x.exp()
    }
}

fn mish(x: f64) -> f64 {
    x * softplus(x).tanh()
}

fn mish_deriv(x: f64) -> f64 {
    let sp = softplus(x);
    let th = sp.tanh();
    let sech2 = 1.0 / sp.cosh().powi(2);
    let s = sigmoid(x);
    sech2 * x * s + th
}

const FUNCTION_PAIRS: [FunctionPair; 7] = [
    FunctionPair {
        name: "tanh_sech2",
        test_x_min: -2.0,
        test_x_max: 2.0,
        output1: ValueEncoding { min: -1.0, max: 1.0 },
        output2: ValueEncoding { min: 0.0, max: 1.0 },
        compute_true: |x| (x.tanh(), 1.0 / x.cosh().powi(2)),
    },
    FunctionPair {
        name: "sigmoid_sigmoid_deriv",
        test_x_min: -6.0,
        test_x_max: 6.0,
        output1: ValueEncoding { min: 0.0, max: 1.0 },
        output2: ValueEncoding { min: 0.0, max: 0.25 },
        compute_true: |x| {
            let s = sigmoid(x);
            (s, s * (1.0 - s))
        },
    },
    FunctionPair {
        name: "softplus_sigmoid",
        test_x_min: -4.0,
        test_x_max: 4.0,
        output1: ValueEncoding {
            min: 0.018,
            max: 4.018,
        },
        output2: ValueEncoding { min: 0.0, max: 1.0 },
        compute_true: |x| (softplus(x), sigmoid(x)),
    },
    FunctionPair {
        name: "swish_swish_deriv",
        test_x_min: -4.0,
        test_x_max: 4.0,
        output1: ValueEncoding {
            min: -0.2785,
            max: 4.0,
        },
        output2: ValueEncoding {
            min: -0.10,
            max: 1.10,
        },
        compute_true: |x| (swish(x), swish_deriv(x)),
    },
    FunctionPair {
        name: "gelu_gelu_deriv",
        test_x_min: -4.0,
        test_x_max: 4.0,
        output1: ValueEncoding {
            min: -0.17,
            max: 4.0,
        },
        output2: ValueEncoding {
            min: -0.12,
            max: 1.12,
        },
        compute_true: |x| (gelu(x), gelu_deriv(x)),
    },
    FunctionPair {
        name: "elu_elu_deriv",
        test_x_min: -4.0,
        test_x_max: 3.0,
        output1: ValueEncoding { min: -1.0, max: 3.0 },
        output2: ValueEncoding { min: 0.0, max: 1.0 },
        compute_true: |x| (elu(x), elu_deriv(x)),
    },
    FunctionPair {
        name: "mish_mish_deriv",
        test_x_min: -4.0,
        test_x_max: 3.0,
        output1: ValueEncoding {
            min: -0.31,
            max: 3.0,
        },
        output2: ValueEncoding {
            min: -0.11,
            max: 1.26,
        },
        compute_true: |x| (mish(x), mish_deriv(x)),
    },
];

fn standard_scheme() -> SchemeConfig {
    SchemeConfig {
        kind: SchemeKind::Standard,
        bits: parse_env_usize("PAPER_STANDARD_BITS", 10),
        pbs: PbsParams {
            small_lwe_dimension: LweDimension(parse_env_usize("PAPER_STANDARD_LWE_DIM", 1012)),
            glwe_dimension: GlweDimension(1),
            polynomial_size: PolynomialSize(parse_env_usize("PAPER_STANDARD_POLY_SIZE", 8192)),
            lwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
                1.647968356631524e-07,
            )),
            glwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
                2.168404344971009e-19,
            )),
            pbs_base_log: DecompositionBaseLog(15),
            pbs_level: DecompositionLevelCount(2),
            ciphertext_modulus: CiphertextModulus::new_native(),
        },
        input_guard: Some(InputGuardLayout {
            total_factor: parse_env_usize("PAPER_STANDARD_INPUT_FACTOR", 2),
            input_offset: parse_env_usize("PAPER_STANDARD_INPUT_OFFSET", 512),
        }),
        many_lut: None,
    }
}

fn sdr_pbs_scheme() -> SchemeConfig {
    SchemeConfig {
        kind: SchemeKind::SdrPbs,
        bits: parse_env_usize("PAPER_SDR_BITS", 10),
        pbs: PbsParams {
            small_lwe_dimension: LweDimension(parse_env_usize("PAPER_SDR_LWE_DIM", 1012)),
            glwe_dimension: GlweDimension(1),
            polynomial_size: PolynomialSize(parse_env_usize("PAPER_SDR_POLY_SIZE", 8192)),
            lwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
                1.647968356631524e-07,
            )),
            glwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
                2.168404344971009e-19,
            )),
            pbs_base_log: DecompositionBaseLog(15),
            pbs_level: DecompositionLevelCount(2),
            ciphertext_modulus: CiphertextModulus::new_native(),
        },
        input_guard: Some(InputGuardLayout {
            total_factor: parse_env_usize("PAPER_SDR_INPUT_FACTOR", 2),
            input_offset: parse_env_usize("PAPER_SDR_INPUT_OFFSET", 512),
        }),
        many_lut: None,
    }
}

fn many_lut_scheme() -> SchemeConfig {
    SchemeConfig {
        kind: SchemeKind::ManyLut,
        bits: parse_env_usize("PAPER_MANY_BITS", 9),
        pbs: PbsParams {
            small_lwe_dimension: LweDimension(1012),
            glwe_dimension: GlweDimension(1),
            polynomial_size: PolynomialSize(8192),
            lwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
                1.647968356631524e-07,
            )),
            glwe_noise_distribution: DynamicDistribution::new_gaussian_from_std_dev(StandardDev(
                2.168404344971009e-19,
            )),
            pbs_base_log: DecompositionBaseLog(15),
            pbs_level: DecompositionLevelCount(2),
            ciphertext_modulus: CiphertextModulus::new_native(),
        },
        input_guard: None,
        many_lut: Some(ManyLutLayout {
            total_factor: parse_env_usize("PAPER_MANY_TOTAL_FACTOR", 4),
            slot_count: parse_env_usize("PAPER_MANY_SLOT_COUNT", 2),
            input_offset: parse_env_usize("PAPER_MANY_INPUT_OFFSET", 256),
            used_slots: [0, 1],
        }),
    }
}

fn parse_env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_env_u128(name: &str) -> Option<u128> {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<u128>().ok())
}

fn parse_experiment_mode() -> ExperimentMode {
    match env::var("PAPER_MODE")
        .unwrap_or_else(|_| "continuous".to_string())
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "codebook" => ExperimentMode::Codebook,
        "end_to_end" | "end-to-end" | "e2e" => ExperimentMode::EndToEnd,
        _ => ExperimentMode::Continuous,
    }
}

fn parse_scheme_selection() -> SchemeSelection {
    let value = env::var("PAPER_SCHEMES").unwrap_or_else(|_| "standard,sdr_pbs,many".to_string());
    if value.trim().is_empty() {
        return SchemeSelection {
            standard: true,
            sdr_pbs: true,
            many_lut: true,
        };
    }

    let mut selection = SchemeSelection::default();
    for token in value.split([',', ';', ' ']) {
        match token.trim().to_ascii_lowercase().as_str() {
            "" => {}
            "standard" | "std" | "standard_pbs" => selection.standard = true,
            "sdr" | "sdr_pbs" => selection.sdr_pbs = true,
            "many" | "many_lut" | "many-lut" => selection.many_lut = true,
            "all" => {
                selection.standard = true;
                selection.sdr_pbs = true;
                selection.many_lut = true;
            }
            _ => {}
        }
    }

    selection
}

fn threshold_label(threshold: f64) -> String {
    format!("{:.1}", threshold * 100.0).replace('.', "p")
}

fn parse_output_dir() -> Option<PathBuf> {
    env::var("PAPER_OUTPUT_DIR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(PathBuf::from)
}

fn pair_filters() -> Vec<String> {
    env::var("PAPER_PAIR_FILTER")
        .or_else(|_| env::var("SDR_PBS_PAIR_FILTER"))
        .ok()
        .map(|value| {
            value
                .split([',', ';'])
                .map(str::trim)
                .filter(|token| !token.is_empty())
                .map(|token| token.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn derive_seed(master_seed: Option<u128>, offset: u128) -> Option<u128> {
    master_seed.map(|seed| seed.wrapping_add(offset))
}

fn selected_schemes_label(selection: SchemeSelection) -> String {
    let mut labels = Vec::new();
    if selection.standard {
        labels.push("standard");
    }
    if selection.sdr_pbs {
        labels.push("sdr_pbs");
    }
    if selection.many_lut {
        labels.push("many_lut");
    }
    labels.join(",")
}

fn selected_pairs<'a>(filters: &[String]) -> Vec<&'a FunctionPair>
where
    'static: 'a,
{
    FUNCTION_PAIRS
        .iter()
        .filter(|pair| {
            filters.is_empty()
                || filters
                    .iter()
                    .any(|needle| pair.name.to_ascii_lowercase().contains(needle))
        })
        .collect()
}

fn levels(bits: usize) -> usize {
    1usize << bits
}

fn max_code(bits: usize) -> u64 {
    (levels(bits) - 1) as u64
}

fn x_for_index(index: usize, pair: &FunctionPair, level_count: usize) -> f64 {
    let t = (index as f64 + 0.5) / level_count as f64;
    pair.test_x_min + (pair.test_x_max - pair.test_x_min) * t
}

fn x_for_point(index: usize, point_count: usize, pair: &FunctionPair) -> f64 {
    let t = (index as f64 + 0.5) / point_count as f64;
    pair.test_x_min + (pair.test_x_max - pair.test_x_min) * t
}

fn encode_input_index(x: f64, pair: &FunctionPair, bits: usize) -> u64 {
    let level_count = levels(bits);
    let normalized =
        ((x - pair.test_x_min) / (pair.test_x_max - pair.test_x_min)).clamp(0.0, 1.0);
    let scaled = (normalized * level_count as f64).floor() as usize;
    scaled.min(level_count - 1) as u64
}

fn quantize_value(value: f64, bits: usize, encoding: ValueEncoding) -> u64 {
    let max_code = max_code(bits) as f64;
    let normalized = ((value - encoding.min) / encoding.range()).clamp(0.0, 1.0);
    (normalized * max_code).round() as u64
}

fn dequantize_value(code: u64, bits: usize, encoding: ValueEncoding) -> f64 {
    let max_code = max_code(bits) as f64;
    encoding.min + (code as f64 / max_code) * encoding.range()
}

fn normalize_value(value: f64, encoding: ValueEncoding) -> f64 {
    ((value - encoding.min) / encoding.range()).clamp(0.0, 1.0)
}

fn compute_downstream_outputs(pair: &FunctionPair, outputs: (f64, f64)) -> DownstreamOutputs {
    let activation = normalize_value(outputs.0, pair.output1);
    let derivative = normalize_value(outputs.1, pair.output2);
    let score =
        E2E_SCORE_WEIGHT_ACTIVATION * activation + E2E_SCORE_WEIGHT_DERIVATIVE * derivative;
    let update = E2E_UPDATE_BASELINE
        - E2E_UPDATE_STEP_SIZE * (activation - E2E_UPDATE_TARGET) * derivative;
    DownstreamOutputs { score, update }
}

fn scan_output_encoding(pair: &FunctionPair, sample_count: usize) -> EncodingCoverageReport {
    let effective_samples = sample_count.max(1);
    let mut observed_min1 = f64::INFINITY;
    let mut observed_max1 = f64::NEG_INFINITY;
    let mut observed_min2 = f64::INFINITY;
    let mut observed_max2 = f64::NEG_INFINITY;

    for index in 0..=effective_samples {
        let t = index as f64 / effective_samples as f64;
        let x = pair.test_x_min + (pair.test_x_max - pair.test_x_min) * t;
        let (value1, value2) = (pair.compute_true)(x);
        observed_min1 = observed_min1.min(value1);
        observed_max1 = observed_max1.max(value1);
        observed_min2 = observed_min2.min(value2);
        observed_max2 = observed_max2.max(value2);
    }

    EncodingCoverageReport {
        pair_name: pair.name,
        samples: effective_samples,
        observed_min1,
        observed_max1,
        observed_min2,
        observed_max2,
        output1_clipped: observed_min1 < pair.output1.min || observed_max1 > pair.output1.max,
        output2_clipped: observed_min2 < pair.output2.min || observed_max2 > pair.output2.max,
    }
}

fn format_encoding_coverage(report: &EncodingCoverageReport, pair: &FunctionPair) -> String {
    format!(
        concat!(
            "{} | samples={} | ",
            "output1 observed=[{:.12},{:.12}] declared=[{:.12},{:.12}] clipped={} | ",
            "output2 observed=[{:.12},{:.12}] declared=[{:.12},{:.12}] clipped={}"
        ),
        report.pair_name,
        report.samples,
        report.observed_min1,
        report.observed_max1,
        pair.output1.min,
        pair.output1.max,
        report.output1_clipped,
        report.observed_min2,
        report.observed_max2,
        pair.output2.min,
        pair.output2.max,
        report.output2_clipped,
    )
}

fn compute_summary(errors: &[f64]) -> ErrorSummary {
    if errors.is_empty() {
        return ErrorSummary {
            mean: 0.0,
            std: 0.0,
            rmse: 0.0,
            median: 0.0,
            p90: 0.0,
            p95: 0.0,
            p99: 0.0,
            p999: 0.0,
            max: 0.0,
        };
    }

    let n = errors.len();
    let mean = errors.iter().sum::<f64>() / n as f64;
    let variance = errors
        .iter()
        .map(|value| (value - mean).powi(2))
        .sum::<f64>()
        / n as f64;
    let rmse = (errors.iter().map(|value| value.powi(2)).sum::<f64>() / n as f64).sqrt();
    let mut sorted = errors.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let quantile = |percent: f64| -> f64 {
        let scaled = ((n - 1) as f64 * percent).round() as usize;
        let index = scaled.min(n - 1);
        sorted[index]
    };

    ErrorSummary {
        mean,
        std: variance.sqrt(),
        rmse,
        median: quantile(0.50),
        p90: quantile(0.90),
        p95: quantile(0.95),
        p99: quantile(0.99),
        p999: quantile(0.999),
        max: *sorted.last().unwrap(),
    }
}

fn torus_cyclic_distance(value: u64, target: u64, modulus: u64) -> u64 {
    let diff = value.abs_diff(target);
    diff.min(modulus - diff)
}

fn nearest_sdr_code(
    raw: u64,
    parity: u64,
    legal_code_modulus: u64,
    total_modulus: u64,
) -> (u64, u64) {
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

fn nearest_contiguous_code(
    raw: u64,
    valid_code_modulus: u64,
    total_modulus: u64,
) -> (u64, u64, bool) {
    if raw < valid_code_modulus {
        return (raw, 0, false);
    }

    let zero_distance = torus_cyclic_distance(raw, 0, total_modulus);
    let top_code = valid_code_modulus - 1;
    let top_distance = torus_cyclic_distance(raw, top_code, total_modulus);

    if zero_distance <= top_distance {
        (0, zero_distance, true)
    } else {
        (top_code, top_distance, true)
    }
}

fn standard_total_plaintext_modulus(config: SchemeConfig) -> u64 {
    let factor = config
        .input_guard
        .map(|guard| guard.total_factor)
        .unwrap_or(1);
    (levels(config.bits) * factor) as u64
}

fn decode_standard_output(raw: u64, bits: usize, total_plaintext_modulus: u64) -> (u64, bool, bool) {
    let rounded = raw % total_plaintext_modulus;
    let valid_code_modulus = levels(bits) as u64;
    let invalid_output = rounded >= valid_code_modulus;
    let (projected, _, projected_flag) =
        nearest_contiguous_code(rounded, valid_code_modulus, total_plaintext_modulus);
    (projected, projected_flag, invalid_output)
}

fn build_runtime(params: PbsParams, seed: Option<u128>) -> Runtime {
    if let Some(seed_value) = seed {
        let mut seeder = DeterministicSeeder::<DefaultRandomGenerator>::new(Seed(seed_value));
        let mut secret_generator =
            SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());
        let encryption_generator =
            EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), &mut seeder);

        let small_lwe_sk =
            LweSecretKey::generate_new_binary(params.small_lwe_dimension, &mut secret_generator);
        let glwe_sk = GlweSecretKey::generate_new_binary(
            params.glwe_dimension,
            params.polynomial_size,
            &mut secret_generator,
        );
        let big_lwe_sk = glwe_sk.clone().into_lwe_secret_key();

        let start_bsk_generation = Instant::now();
        let standard_bsk = par_allocate_and_generate_new_seeded_lwe_bootstrap_key(
            &small_lwe_sk,
            &glwe_sk,
            params.pbs_base_log,
            params.pbs_level,
            params.glwe_noise_distribution,
            params.ciphertext_modulus,
            &mut seeder,
        );
        let bsk_generation_time = start_bsk_generation.elapsed().as_secs_f64();

        let standard_bsk: LweBootstrapKeyOwned<u64> =
            standard_bsk.decompress_into_lwe_bootstrap_key();
        let mut fourier_bsk = FourierLweBootstrapKey::new(
            standard_bsk.input_lwe_dimension(),
            standard_bsk.glwe_size(),
            standard_bsk.polynomial_size(),
            standard_bsk.decomposition_base_log(),
            standard_bsk.decomposition_level_count(),
        );
        let start_fourier_conversion = Instant::now();
        convert_standard_lwe_bootstrap_key_to_fourier(&standard_bsk, &mut fourier_bsk);
        let fourier_conversion_time = start_fourier_conversion.elapsed().as_secs_f64();

        Runtime {
            params,
            small_lwe_sk,
            big_lwe_sk,
            fourier_bsk,
            encryption_generator,
            seed,
            bsk_generation_time,
            fourier_conversion_time,
        }
    } else {
        let mut boxed_seeder = new_seeder();
        let seeder = boxed_seeder.as_mut();
        let mut secret_generator =
            SecretRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed());
        let encryption_generator =
            EncryptionRandomGenerator::<DefaultRandomGenerator>::new(seeder.seed(), seeder);

        let small_lwe_sk =
            LweSecretKey::generate_new_binary(params.small_lwe_dimension, &mut secret_generator);
        let glwe_sk = GlweSecretKey::generate_new_binary(
            params.glwe_dimension,
            params.polynomial_size,
            &mut secret_generator,
        );
        let big_lwe_sk = glwe_sk.clone().into_lwe_secret_key();

        let start_bsk_generation = Instant::now();
        let standard_bsk = par_allocate_and_generate_new_seeded_lwe_bootstrap_key(
            &small_lwe_sk,
            &glwe_sk,
            params.pbs_base_log,
            params.pbs_level,
            params.glwe_noise_distribution,
            params.ciphertext_modulus,
            seeder,
        );
        let bsk_generation_time = start_bsk_generation.elapsed().as_secs_f64();

        let standard_bsk: LweBootstrapKeyOwned<u64> =
            standard_bsk.decompress_into_lwe_bootstrap_key();
        let mut fourier_bsk = FourierLweBootstrapKey::new(
            standard_bsk.input_lwe_dimension(),
            standard_bsk.glwe_size(),
            standard_bsk.polynomial_size(),
            standard_bsk.decomposition_base_log(),
            standard_bsk.decomposition_level_count(),
        );
        let start_fourier_conversion = Instant::now();
        convert_standard_lwe_bootstrap_key_to_fourier(&standard_bsk, &mut fourier_bsk);
        let fourier_conversion_time = start_fourier_conversion.elapsed().as_secs_f64();

        Runtime {
            params,
            small_lwe_sk,
            big_lwe_sk,
            fourier_bsk,
            encryption_generator,
            seed,
            bsk_generation_time,
            fourier_conversion_time,
        }
    }
}

fn build_standard_accumulator(
    pair: &FunctionPair,
    bits: usize,
    output_index: usize,
    params: PbsParams,
) -> (GlweCiphertextOwned<u64>, u64, u64) {
    let level_count = levels(bits);
    let delta = (1u64 << 63) / level_count as u64;
    let acc = generate_programmable_bootstrap_glwe_lut(
        params.polynomial_size,
        params.glwe_dimension.to_glwe_size(),
        level_count,
        params.ciphertext_modulus,
        delta,
        |input_code: u64| {
            let x = x_for_index(input_code as usize, pair, level_count);
            let (value1, value2) = (pair.compute_true)(x);
            match output_index {
                0 => quantize_value(value1, bits, pair.output1),
                1 => quantize_value(value2, bits, pair.output2),
                _ => unreachable!(),
            }
        },
    );
    (acc, delta, 0)
}

fn build_guarded_standard_accumulator(
    pair: &FunctionPair,
    bits: usize,
    output_index: usize,
    params: PbsParams,
    guard: InputGuardLayout,
) -> (GlweCiphertextOwned<u64>, u64, u64) {
    let level_count = levels(bits);
    let total_input_modulus = level_count * guard.total_factor;
    assert!(
        guard.input_offset + level_count <= total_input_modulus,
        "standard guard exceeds total input modulus"
    );

    let delta = (1u64 << 63) / total_input_modulus as u64;
    let box_size = params.polynomial_size.0 / total_input_modulus;
    let mut coeffs = vec![0u64; params.polynomial_size.0];

    for (input_slot, box_) in coeffs.chunks_exact_mut(box_size).enumerate() {
        let active_index = if input_slot < guard.input_offset {
            0
        } else if input_slot >= guard.input_offset + level_count {
            level_count - 1
        } else {
            input_slot - guard.input_offset
        };

        let x = x_for_index(active_index, pair, level_count);
        let (value1, value2) = (pair.compute_true)(x);
        let code = match output_index {
            0 => quantize_value(value1, bits, pair.output1),
            1 => quantize_value(value2, bits, pair.output2),
            _ => unreachable!(),
        };
        box_.fill(code.wrapping_mul(delta));
    }

    let half_box_size = box_size / 2;
    for coefficient in coeffs[0..half_box_size].iter_mut() {
        *coefficient = coefficient.wrapping_neg();
    }
    coeffs.rotate_left(half_box_size);

    let accumulator_plaintext = PlaintextList::from_container(coeffs);
    let accumulator = allocate_and_trivially_encrypt_new_glwe_ciphertext(
        params.glwe_dimension.to_glwe_size(),
        &accumulator_plaintext,
        params.ciphertext_modulus,
    );

    (accumulator, delta, guard.input_offset as u64)
}

fn build_sdr_pbs_accumulator(
    pair: &FunctionPair,
    bits: usize,
    params: PbsParams,
) -> (GlweCiphertextOwned<u64>, u64, u64) {
    let level_count = levels(bits);
    let expected_polynomial_size = 4 * level_count;
    assert_eq!(
        params.polynomial_size.0, expected_polynomial_size,
        "SDR-PBS box-4 layout needs polynomial_size = 4 * levels"
    );

    let blind_rotation_log = params.polynomial_size.to_blind_rotation_input_modulus_log();
    let delta = 1u64 << (64 - blind_rotation_log.0);
    let mut coeffs = vec![0u64; params.polynomial_size.0];

    for input_index in 0..level_count {
        let x = x_for_index(input_index, pair, level_count);
        let (value1, value2) = (pair.compute_true)(x);
        let even_code = 2 * quantize_value(value1, bits, pair.output1);
        let odd_code = 2 * quantize_value(value2, bits, pair.output2) + 1;
        let base = 4 * input_index;

        coeffs[base] = even_code.wrapping_mul(delta);
        coeffs[base + 1] = odd_code.wrapping_mul(delta);
        coeffs[base + 2] = even_code.wrapping_mul(delta);
        coeffs[base + 3] = odd_code.wrapping_mul(delta);
    }

    let accumulator_plaintext = PlaintextList::from_container(coeffs);
    let accumulator = allocate_and_trivially_encrypt_new_glwe_ciphertext(
        params.glwe_dimension.to_glwe_size(),
        &accumulator_plaintext,
        params.ciphertext_modulus,
    );

    (accumulator, delta, 0)
}

fn build_guarded_sdr_pbs_accumulator(
    pair: &FunctionPair,
    bits: usize,
    params: PbsParams,
    guard: InputGuardLayout,
) -> (GlweCiphertextOwned<u64>, u64, u64) {
    let level_count = levels(bits);
    let total_intervals = level_count * guard.total_factor;
    let expected_polynomial_size = 4 * total_intervals;
    assert_eq!(
        params.polynomial_size.0, expected_polynomial_size,
        "guarded SDR-PBS needs polynomial_size = 4 * total_intervals"
    );
    assert!(
        guard.input_offset + level_count <= total_intervals,
        "SDR-PBS guard exceeds total interval count"
    );

    let blind_rotation_log = params.polynomial_size.to_blind_rotation_input_modulus_log();
    let delta = 1u64 << (64 - blind_rotation_log.0);
    let mut coeffs = vec![0u64; params.polynomial_size.0];

    for interval in 0..total_intervals {
        let active_index = if interval < guard.input_offset {
            0
        } else if interval >= guard.input_offset + level_count {
            level_count - 1
        } else {
            interval - guard.input_offset
        };

        let x = x_for_index(active_index, pair, level_count);
        let (value1, value2) = (pair.compute_true)(x);
        let even_code = 2 * quantize_value(value1, bits, pair.output1);
        let odd_code = 2 * quantize_value(value2, bits, pair.output2) + 1;
        let base = 4 * interval;

        coeffs[base] = even_code.wrapping_mul(delta);
        coeffs[base + 1] = odd_code.wrapping_mul(delta);
        coeffs[base + 2] = even_code.wrapping_mul(delta);
        coeffs[base + 3] = odd_code.wrapping_mul(delta);
    }

    let accumulator_plaintext = PlaintextList::from_container(coeffs);
    let accumulator = allocate_and_trivially_encrypt_new_glwe_ciphertext(
        params.glwe_dimension.to_glwe_size(),
        &accumulator_plaintext,
        params.ciphertext_modulus,
    );

    (accumulator, delta, guard.input_offset as u64)
}

fn build_many_lut_accumulator(
    pair: &FunctionPair,
    bits: usize,
    params: PbsParams,
    layout: ManyLutLayout,
) -> (GlweCiphertextOwned<u64>, usize, u64, u64) {
    let level_count = levels(bits);
    let total_plaintext_modulus = level_count * layout.total_factor;
    let delta = (1u64 << 63) / total_plaintext_modulus as u64;
    let box_size = params.polynomial_size.0 / total_plaintext_modulus;
    let func_chunk_size = params.polynomial_size.0 / layout.slot_count;
    let slot_capacity = func_chunk_size / box_size;
    assert!(
        layout.input_offset + level_count <= slot_capacity,
        "many-lut active range exceeds slot capacity"
    );

    let mut coeffs = vec![0u64; params.polynomial_size.0];

    for (slot, output_index) in layout.used_slots.into_iter().zip([0usize, 1usize]) {
        let start = slot * func_chunk_size;
        let end = start + func_chunk_size;
        let chunk = &mut coeffs[start..end];

        let first_x = x_for_index(0, pair, level_count);
        let last_x = x_for_index(level_count - 1, pair, level_count);
        let first_code = {
            let (value1, value2) = (pair.compute_true)(first_x);
            match output_index {
                0 => quantize_value(value1, bits, pair.output1),
                1 => quantize_value(value2, bits, pair.output2),
                _ => unreachable!(),
            }
        }
        .wrapping_mul(delta);
        let last_code = {
            let (value1, value2) = (pair.compute_true)(last_x);
            match output_index {
                0 => quantize_value(value1, bits, pair.output1),
                1 => quantize_value(value2, bits, pair.output2),
                _ => unreachable!(),
            }
        }
        .wrapping_mul(delta);

        for guard_box in chunk.chunks_exact_mut(box_size).take(layout.input_offset) {
            guard_box.fill(first_code);
        }

        for (input_index, box_) in chunk
            .chunks_exact_mut(box_size)
            .skip(layout.input_offset)
            .take(level_count)
            .enumerate()
        {
            let x = x_for_index(input_index, pair, level_count);
            let (value1, value2) = (pair.compute_true)(x);
            let code = match output_index {
                0 => quantize_value(value1, bits, pair.output1),
                1 => quantize_value(value2, bits, pair.output2),
                _ => unreachable!(),
            };
            box_.fill(code.wrapping_mul(delta));
        }

        for guard_box in chunk
            .chunks_exact_mut(box_size)
            .skip(layout.input_offset + level_count)
        {
            guard_box.fill(last_code);
        }
    }

    let half_box_size = box_size / 2;
    for coefficient in coeffs[0..half_box_size].iter_mut() {
        *coefficient = coefficient.wrapping_neg();
    }
    coeffs.rotate_left(half_box_size);

    let accumulator_plaintext = PlaintextList::from_container(coeffs);
    let accumulator = allocate_and_trivially_encrypt_new_glwe_ciphertext(
        params.glwe_dimension.to_glwe_size(),
        &accumulator_plaintext,
        params.ciphertext_modulus,
    );

    (
        accumulator,
        func_chunk_size,
        delta,
        total_plaintext_modulus as u64,
    )
}

fn encrypt_input(runtime: &mut Runtime, plaintext: Plaintext<u64>) -> LweCiphertextOwned<u64> {
    allocate_and_encrypt_new_lwe_ciphertext(
        &runtime.small_lwe_sk,
        plaintext,
        runtime.params.lwe_noise_distribution,
        runtime.params.ciphertext_modulus,
        &mut runtime.encryption_generator,
    )
}

fn centered_blind_rotate(
    runtime: &Runtime,
    lwe_input: LweCiphertextOwned<u64>,
    accumulator: &GlweCiphertextOwned<u64>,
) -> GlweCiphertextOwned<u64> {
    let blind_rotation_log = runtime.params.polynomial_size.to_blind_rotation_input_modulus_log();
    let msed = lwe_ciphertext_centered_binary_modulus_switch(lwe_input, blind_rotation_log);
    let mut local_accumulator = accumulator.clone();
    blind_rotate_assign(&msed, &mut local_accumulator, &runtime.fourier_bsk);
    local_accumulator
}

fn centered_pbs_extract_zero(
    runtime: &Runtime,
    lwe_input: LweCiphertextOwned<u64>,
    accumulator: &GlweCiphertextOwned<u64>,
) -> LweCiphertextOwned<u64> {
    let rotated_accumulator = centered_blind_rotate(runtime, lwe_input, accumulator);
    let mut output = LweCiphertext::new(
        0u64,
        runtime.big_lwe_sk.lwe_dimension().to_lwe_size(),
        runtime.params.ciphertext_modulus,
    );
    extract_lwe_sample_from_glwe_ciphertext(&rotated_accumulator, &mut output, MonomialDegree(0));
    output
}

fn decode_sdr_pbs_outputs(
    runtime: &Runtime,
    sample0: &LweCiphertextOwned<u64>,
    sample1: &LweCiphertextOwned<u64>,
    bits: usize,
    delta: u64,
) -> ((u64, u64), usize, usize) {
    let blind_rotation_log = runtime.params.polynomial_size.to_blind_rotation_input_modulus_log();
    let total_modulus = 1u64 << blind_rotation_log.0;
    let legal_code_modulus = 2 * levels(bits) as u64;

    let raw0 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, sample0).0, delta)
        % total_modulus;
    let raw1 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, sample1).0, delta)
        % total_modulus;
    let invalid_outputs =
        usize::from(raw0 >= legal_code_modulus) + usize::from(raw1 >= legal_code_modulus);

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
    let projection_events = usize::from(direct_cost > 0 || swapped_cost > 0);

    let (f1_enc, f2_enc) = if direct_cost <= swapped_cost {
        (raw0_even, raw1_odd)
    } else {
        (raw1_even, raw0_odd)
    };

    (
        (f1_enc / 2, (f2_enc - 1) / 2),
        projection_events,
        invalid_outputs,
    )
}

fn evaluate_standard(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
) -> SchemeReport {
    let (acc1, delta, input_offset) = match config.input_guard {
        Some(guard) => build_guarded_standard_accumulator(pair, config.bits, 0, config.pbs, guard),
        None => build_standard_accumulator(pair, config.bits, 0, config.pbs),
    };
    let (acc2, _, _) = match config.input_guard {
        Some(guard) => build_guarded_standard_accumulator(pair, config.bits, 1, config.pbs, guard),
        None => build_standard_accumulator(pair, config.bits, 1, config.pbs),
    };
    let total_plaintext_modulus = standard_total_plaintext_modulus(config);
    let mut stats = EvalStats::new(point_count);

    for point in 0..point_count {
        let start_input = Instant::now();
        let x = x_for_point(point, point_count, pair);
        let truth = (pair.compute_true)(x);
        let input_code = encode_input_index(x, pair, config.bits);
        let plaintext = Plaintext((input_code + input_offset).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        let input_us = start_input.elapsed().as_secs_f64() * 1_000_000.0;

        let start_core = Instant::now();
        let out1 = centered_pbs_extract_zero(runtime, lwe_input.clone(), &acc1);
        let out2 = centered_pbs_extract_zero(runtime, lwe_input, &acc2);
        let core_us = start_core.elapsed().as_secs_f64() * 1_000_000.0;

        let start_decode = Instant::now();
        let raw1 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &out1).0, delta);
        let raw2 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &out2).0, delta);
        let (code1, projected1, invalid1) =
            decode_standard_output(raw1, config.bits, total_plaintext_modulus);
        let (code2, projected2, invalid2) =
            decode_standard_output(raw2, config.bits, total_plaintext_modulus);
        let reconstructed = (
            dequantize_value(code1, config.bits, pair.output1),
            dequantize_value(code2, config.bits, pair.output2),
        );
        let decode_us = start_decode.elapsed().as_secs_f64() * 1_000_000.0;

        stats.record(
            pair,
            reconstructed,
            truth,
            usize::from(projected1) + usize::from(projected2),
            usize::from(invalid1) + usize::from(invalid2),
            TimingBreakdown {
                input_us,
                core_us,
                decode_us,
            },
        );
    }

    finalize_report(pair, config, point_count, stats)
}

fn evaluate_sdr_pbs(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
) -> SchemeReport {
    let (accumulator, delta, input_offset) = match config.input_guard {
        Some(guard) => build_guarded_sdr_pbs_accumulator(pair, config.bits, config.pbs, guard),
        None => build_sdr_pbs_accumulator(pair, config.bits, config.pbs),
    };
    let blind_rotation_log = runtime.params.polynomial_size.to_blind_rotation_input_modulus_log();
    let mut stats = EvalStats::new(point_count);

    for point in 0..point_count {
        let start_input = Instant::now();
        let x = x_for_point(point, point_count, pair);
        let truth = (pair.compute_true)(x);
        let input_code = encode_input_index(x, pair, config.bits);
        let plaintext = Plaintext((4 * (input_code + input_offset)).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        let input_us = start_input.elapsed().as_secs_f64() * 1_000_000.0;

        let start_core = Instant::now();
        let msed = lwe_ciphertext_centered_binary_modulus_switch(lwe_input, blind_rotation_log);
        let mut local_accumulator = accumulator.clone();
        let (sample0, sample1) = blind_rotate_and_extract_two_coefficients(
            &msed,
            &mut local_accumulator,
            &runtime.fourier_bsk,
        );
        let core_us = start_core.elapsed().as_secs_f64() * 1_000_000.0;

        let start_decode = Instant::now();
        let ((code1, code2), projection_events, invalid_outputs) =
            decode_sdr_pbs_outputs(runtime, &sample0, &sample1, config.bits, delta);
        let reconstructed = (
            dequantize_value(code1, config.bits, pair.output1),
            dequantize_value(code2, config.bits, pair.output2),
        );
        let decode_us = start_decode.elapsed().as_secs_f64() * 1_000_000.0;

        stats.record(
            pair,
            reconstructed,
            truth,
            projection_events,
            invalid_outputs,
            TimingBreakdown {
                input_us,
                core_us,
                decode_us,
            },
        );
    }

    finalize_report(pair, config, point_count, stats)
}

fn evaluate_many_lut(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
) -> SchemeReport {
    let layout = config.many_lut.expect("many-lut config is required");
    let (accumulator, func_chunk_size, delta, total_plaintext_modulus) =
        build_many_lut_accumulator(pair, config.bits, config.pbs, layout);
    let mut stats = EvalStats::new(point_count);

    for point in 0..point_count {
        let start_input = Instant::now();
        let x = x_for_point(point, point_count, pair);
        let truth = (pair.compute_true)(x);
        let input_code = encode_input_index(x, pair, config.bits);
        let plaintext = Plaintext((input_code + layout.input_offset as u64).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        let input_us = start_input.elapsed().as_secs_f64() * 1_000_000.0;

        let start_core = Instant::now();
        let rotated_accumulator = centered_blind_rotate(runtime, lwe_input, &accumulator);
        let core_us = start_core.elapsed().as_secs_f64() * 1_000_000.0;

        let start_decode = Instant::now();
        let mut raw_codes = [0u64; 2];
        let mut projection_events = 0usize;
        let mut invalid_outputs = 0usize;

        for (index, slot) in layout.used_slots.iter().copied().enumerate() {
            let mut output = LweCiphertext::new(
                0u64,
                runtime.big_lwe_sk.lwe_dimension().to_lwe_size(),
                runtime.params.ciphertext_modulus,
            );
            extract_lwe_sample_from_glwe_ciphertext(
                &rotated_accumulator,
                &mut output,
                MonomialDegree(slot * func_chunk_size),
            );

            let rounded =
                divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &output).0, delta)
                    % total_plaintext_modulus;
            let (projected, _, projected_flag) = nearest_contiguous_code(
                rounded,
                levels(config.bits) as u64,
                total_plaintext_modulus,
            );
            raw_codes[index] = projected;
            projection_events += usize::from(projected_flag);
            invalid_outputs += usize::from(rounded >= levels(config.bits) as u64);
        }
        let decode_us = start_decode.elapsed().as_secs_f64() * 1_000_000.0;

        let reconstructed = (
            dequantize_value(raw_codes[0], config.bits, pair.output1),
            dequantize_value(raw_codes[1], config.bits, pair.output2),
        );
        stats.record(
            pair,
            reconstructed,
            truth,
            projection_events,
            invalid_outputs,
            TimingBreakdown {
                input_us,
                core_us,
                decode_us,
            },
        );
    }

    finalize_report(pair, config, point_count, stats)
}

fn evaluate_standard_end_to_end(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
) -> EndToEndReport {
    let (acc1, delta, input_offset) = match config.input_guard {
        Some(guard) => build_guarded_standard_accumulator(pair, config.bits, 0, config.pbs, guard),
        None => build_standard_accumulator(pair, config.bits, 0, config.pbs),
    };
    let (acc2, _, _) = match config.input_guard {
        Some(guard) => build_guarded_standard_accumulator(pair, config.bits, 1, config.pbs, guard),
        None => build_standard_accumulator(pair, config.bits, 1, config.pbs),
    };
    let total_plaintext_modulus = standard_total_plaintext_modulus(config);
    let mut stats = EndToEndStats::new(point_count);

    for point in 0..point_count {
        let start_input = Instant::now();
        let x = x_for_point(point, point_count, pair);
        let truth = (pair.compute_true)(x);
        let truth_tasks = compute_downstream_outputs(pair, truth);
        let input_code = encode_input_index(x, pair, config.bits);
        let plaintext = Plaintext((input_code + input_offset).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        let input_us = start_input.elapsed().as_secs_f64() * 1_000_000.0;

        let start_core = Instant::now();
        let out1 = centered_pbs_extract_zero(runtime, lwe_input.clone(), &acc1);
        let out2 = centered_pbs_extract_zero(runtime, lwe_input, &acc2);
        let core_us = start_core.elapsed().as_secs_f64() * 1_000_000.0;

        let start_decode = Instant::now();
        let raw1 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &out1).0, delta);
        let raw2 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &out2).0, delta);
        let (code1, projected1, invalid1) =
            decode_standard_output(raw1, config.bits, total_plaintext_modulus);
        let (code2, projected2, invalid2) =
            decode_standard_output(raw2, config.bits, total_plaintext_modulus);
        let reconstructed = (
            dequantize_value(code1, config.bits, pair.output1),
            dequantize_value(code2, config.bits, pair.output2),
        );
        let decode_us = start_decode.elapsed().as_secs_f64() * 1_000_000.0;

        let start_downstream = Instant::now();
        let reconstructed_tasks = compute_downstream_outputs(pair, reconstructed);
        let score_error = (reconstructed_tasks.score - truth_tasks.score).abs();
        let update_error = (reconstructed_tasks.update - truth_tasks.update).abs();
        let downstream_us = start_downstream.elapsed().as_secs_f64() * 1_000_000.0;

        stats.record(
            score_error,
            update_error,
            usize::from(projected1) + usize::from(projected2),
            usize::from(invalid1) + usize::from(invalid2),
            EndToEndTiming {
                input_us,
                core_us,
                decode_us,
                downstream_us,
            },
        );
    }

    finalize_end_to_end_report(pair, config, point_count, stats)
}

fn evaluate_sdr_pbs_end_to_end(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
) -> EndToEndReport {
    let (accumulator, delta, input_offset) = match config.input_guard {
        Some(guard) => build_guarded_sdr_pbs_accumulator(pair, config.bits, config.pbs, guard),
        None => build_sdr_pbs_accumulator(pair, config.bits, config.pbs),
    };
    let blind_rotation_log = runtime.params.polynomial_size.to_blind_rotation_input_modulus_log();
    let mut stats = EndToEndStats::new(point_count);

    for point in 0..point_count {
        let start_input = Instant::now();
        let x = x_for_point(point, point_count, pair);
        let truth = (pair.compute_true)(x);
        let truth_tasks = compute_downstream_outputs(pair, truth);
        let input_code = encode_input_index(x, pair, config.bits);
        let plaintext = Plaintext((4 * (input_code + input_offset)).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        let input_us = start_input.elapsed().as_secs_f64() * 1_000_000.0;

        let start_core = Instant::now();
        let msed = lwe_ciphertext_centered_binary_modulus_switch(lwe_input, blind_rotation_log);
        let mut local_accumulator = accumulator.clone();
        let (sample0, sample1) = blind_rotate_and_extract_two_coefficients(
            &msed,
            &mut local_accumulator,
            &runtime.fourier_bsk,
        );
        let core_us = start_core.elapsed().as_secs_f64() * 1_000_000.0;

        let start_decode = Instant::now();
        let ((code1, code2), projection_events, invalid_outputs) =
            decode_sdr_pbs_outputs(runtime, &sample0, &sample1, config.bits, delta);
        let reconstructed = (
            dequantize_value(code1, config.bits, pair.output1),
            dequantize_value(code2, config.bits, pair.output2),
        );
        let decode_us = start_decode.elapsed().as_secs_f64() * 1_000_000.0;

        let start_downstream = Instant::now();
        let reconstructed_tasks = compute_downstream_outputs(pair, reconstructed);
        let score_error = (reconstructed_tasks.score - truth_tasks.score).abs();
        let update_error = (reconstructed_tasks.update - truth_tasks.update).abs();
        let downstream_us = start_downstream.elapsed().as_secs_f64() * 1_000_000.0;

        stats.record(
            score_error,
            update_error,
            projection_events,
            invalid_outputs,
            EndToEndTiming {
                input_us,
                core_us,
                decode_us,
                downstream_us,
            },
        );
    }

    finalize_end_to_end_report(pair, config, point_count, stats)
}

fn evaluate_many_lut_end_to_end(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
) -> EndToEndReport {
    let layout = config.many_lut.expect("many-lut config is required");
    let (accumulator, func_chunk_size, delta, total_plaintext_modulus) =
        build_many_lut_accumulator(pair, config.bits, config.pbs, layout);
    let mut stats = EndToEndStats::new(point_count);

    for point in 0..point_count {
        let start_input = Instant::now();
        let x = x_for_point(point, point_count, pair);
        let truth = (pair.compute_true)(x);
        let truth_tasks = compute_downstream_outputs(pair, truth);
        let input_code = encode_input_index(x, pair, config.bits);
        let plaintext = Plaintext((input_code + layout.input_offset as u64).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        let input_us = start_input.elapsed().as_secs_f64() * 1_000_000.0;

        let start_core = Instant::now();
        let rotated_accumulator = centered_blind_rotate(runtime, lwe_input, &accumulator);
        let core_us = start_core.elapsed().as_secs_f64() * 1_000_000.0;

        let start_decode = Instant::now();
        let mut raw_codes = [0u64; 2];
        let mut projection_events = 0usize;
        let mut invalid_outputs = 0usize;

        for (index, slot) in layout.used_slots.iter().copied().enumerate() {
            let mut output = LweCiphertext::new(
                0u64,
                runtime.big_lwe_sk.lwe_dimension().to_lwe_size(),
                runtime.params.ciphertext_modulus,
            );
            extract_lwe_sample_from_glwe_ciphertext(
                &rotated_accumulator,
                &mut output,
                MonomialDegree(slot * func_chunk_size),
            );

            let rounded =
                divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &output).0, delta)
                    % total_plaintext_modulus;
            let (projected, _, projected_flag) = nearest_contiguous_code(
                rounded,
                levels(config.bits) as u64,
                total_plaintext_modulus,
            );
            raw_codes[index] = projected;
            projection_events += usize::from(projected_flag);
            invalid_outputs += usize::from(rounded >= levels(config.bits) as u64);
        }
        let reconstructed = (
            dequantize_value(raw_codes[0], config.bits, pair.output1),
            dequantize_value(raw_codes[1], config.bits, pair.output2),
        );
        let decode_us = start_decode.elapsed().as_secs_f64() * 1_000_000.0;

        let start_downstream = Instant::now();
        let reconstructed_tasks = compute_downstream_outputs(pair, reconstructed);
        let score_error = (reconstructed_tasks.score - truth_tasks.score).abs();
        let update_error = (reconstructed_tasks.update - truth_tasks.update).abs();
        let downstream_us = start_downstream.elapsed().as_secs_f64() * 1_000_000.0;

        stats.record(
            score_error,
            update_error,
            projection_events,
            invalid_outputs,
            EndToEndTiming {
                input_us,
                core_us,
                decode_us,
                downstream_us,
            },
        );
    }

    finalize_end_to_end_report(pair, config, point_count, stats)
}

fn finalize_codebook_report(
    pair_name: &'static str,
    scheme_name: &'static str,
    bits: usize,
    total_inputs: usize,
    exact_recovery: usize,
    projection_events: usize,
    invalid_outputs: usize,
    joint_le1: usize,
    joint_le2: usize,
    code1_le1: usize,
    code1_le2: usize,
    code2_le1: usize,
    code2_le2: usize,
    sum_code_err1: u64,
    sum_code_err2: u64,
    max_code_err1: u64,
    max_code_err2: u64,
    input_times_us: &[f64],
    core_times_us: &[f64],
    decode_times_us: &[f64],
) -> CodebookReport {
    let avg_input_us =
        input_times_us.iter().sum::<f64>() / input_times_us.len().max(1) as f64;
    let avg_core_us = core_times_us.iter().sum::<f64>() / core_times_us.len().max(1) as f64;
    let avg_decode_us =
        decode_times_us.iter().sum::<f64>() / decode_times_us.len().max(1) as f64;
    let mean_code_err1 = sum_code_err1 as f64 / total_inputs.max(1) as f64;
    let mean_code_err2 = sum_code_err2 as f64 / total_inputs.max(1) as f64;

    CodebookReport {
        pair_name,
        scheme_name,
        bits,
        total_inputs,
        exact_recovery,
        mismatches: total_inputs.saturating_sub(exact_recovery),
        projection_events,
        invalid_outputs,
        joint_le1,
        joint_le2,
        code1_le1,
        code1_le2,
        code2_le1,
        code2_le2,
        mean_code_err1,
        mean_code_err2,
        max_code_err1,
        max_code_err2,
        avg_input_us,
        avg_core_us,
        avg_decode_us,
        avg_eval_us: avg_input_us + avg_core_us + avg_decode_us,
    }
}

fn evaluate_standard_codebook(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
) -> CodebookReport {
    let (acc1, delta, input_offset) = match config.input_guard {
        Some(guard) => build_guarded_standard_accumulator(pair, config.bits, 0, config.pbs, guard),
        None => build_standard_accumulator(pair, config.bits, 0, config.pbs),
    };
    let (acc2, _, _) = match config.input_guard {
        Some(guard) => build_guarded_standard_accumulator(pair, config.bits, 1, config.pbs, guard),
        None => build_standard_accumulator(pair, config.bits, 1, config.pbs),
    };
    let total_inputs = levels(config.bits);
    let total_plaintext_modulus = standard_total_plaintext_modulus(config);
    let mut exact_recovery = 0usize;
    let mut projection_events = 0usize;
    let mut invalid_outputs = 0usize;
    let mut joint_le1 = 0usize;
    let mut joint_le2 = 0usize;
    let mut code1_le1 = 0usize;
    let mut code1_le2 = 0usize;
    let mut code2_le1 = 0usize;
    let mut code2_le2 = 0usize;
    let mut sum_code_err1 = 0u64;
    let mut sum_code_err2 = 0u64;
    let mut max_code_err1 = 0u64;
    let mut max_code_err2 = 0u64;
    let mut input_times_us = Vec::with_capacity(total_inputs);
    let mut core_times_us = Vec::with_capacity(total_inputs);
    let mut decode_times_us = Vec::with_capacity(total_inputs);

    for input_index in 0..total_inputs {
        let start_input = Instant::now();
        let x = x_for_index(input_index, pair, total_inputs);
        let truth = (pair.compute_true)(x);
        let expected_code1 = quantize_value(truth.0, config.bits, pair.output1);
        let expected_code2 = quantize_value(truth.1, config.bits, pair.output2);
        let plaintext = Plaintext(((input_index as u64) + input_offset).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        input_times_us.push(start_input.elapsed().as_secs_f64() * 1_000_000.0);

        let start_core = Instant::now();
        let out1 = centered_pbs_extract_zero(runtime, lwe_input.clone(), &acc1);
        let out2 = centered_pbs_extract_zero(runtime, lwe_input, &acc2);
        core_times_us.push(start_core.elapsed().as_secs_f64() * 1_000_000.0);

        let start_decode = Instant::now();
        let raw1 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &out1).0, delta);
        let raw2 = divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &out2).0, delta);
        let (code1, projected1, invalid1) =
            decode_standard_output(raw1, config.bits, total_plaintext_modulus);
        let (code2, projected2, invalid2) =
            decode_standard_output(raw2, config.bits, total_plaintext_modulus);
        decode_times_us.push(start_decode.elapsed().as_secs_f64() * 1_000_000.0);

        projection_events += usize::from(projected1) + usize::from(projected2);
        invalid_outputs += usize::from(invalid1) + usize::from(invalid2);
        let err1 = code1.abs_diff(expected_code1);
        let err2 = code2.abs_diff(expected_code2);
        if code1 == expected_code1 && code2 == expected_code2 {
            exact_recovery += 1;
        }
        if err1 <= 1 {
            code1_le1 += 1;
        }
        if err1 <= 2 {
            code1_le2 += 1;
        }
        if err2 <= 1 {
            code2_le1 += 1;
        }
        if err2 <= 2 {
            code2_le2 += 1;
        }
        if err1 <= 1 && err2 <= 1 {
            joint_le1 += 1;
        }
        if err1 <= 2 && err2 <= 2 {
            joint_le2 += 1;
        }
        sum_code_err1 += err1;
        sum_code_err2 += err2;
        max_code_err1 = max_code_err1.max(err1);
        max_code_err2 = max_code_err2.max(err2);
    }

    finalize_codebook_report(
        pair.name,
        config.kind.as_str(),
        config.bits,
        total_inputs,
        exact_recovery,
        projection_events,
        invalid_outputs,
        joint_le1,
        joint_le2,
        code1_le1,
        code1_le2,
        code2_le1,
        code2_le2,
        sum_code_err1,
        sum_code_err2,
        max_code_err1,
        max_code_err2,
        &input_times_us,
        &core_times_us,
        &decode_times_us,
    )
}

fn evaluate_sdr_pbs_codebook(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
) -> CodebookReport {
    let (accumulator, delta, input_offset) = match config.input_guard {
        Some(guard) => build_guarded_sdr_pbs_accumulator(pair, config.bits, config.pbs, guard),
        None => build_sdr_pbs_accumulator(pair, config.bits, config.pbs),
    };
    let total_inputs = levels(config.bits);
    let blind_rotation_log = runtime.params.polynomial_size.to_blind_rotation_input_modulus_log();
    let mut exact_recovery = 0usize;
    let mut projection_events = 0usize;
    let mut invalid_outputs = 0usize;
    let mut joint_le1 = 0usize;
    let mut joint_le2 = 0usize;
    let mut code1_le1 = 0usize;
    let mut code1_le2 = 0usize;
    let mut code2_le1 = 0usize;
    let mut code2_le2 = 0usize;
    let mut sum_code_err1 = 0u64;
    let mut sum_code_err2 = 0u64;
    let mut max_code_err1 = 0u64;
    let mut max_code_err2 = 0u64;
    let mut input_times_us = Vec::with_capacity(total_inputs);
    let mut core_times_us = Vec::with_capacity(total_inputs);
    let mut decode_times_us = Vec::with_capacity(total_inputs);

    for input_index in 0..total_inputs {
        let start_input = Instant::now();
        let x = x_for_index(input_index, pair, total_inputs);
        let truth = (pair.compute_true)(x);
        let expected_code1 = quantize_value(truth.0, config.bits, pair.output1);
        let expected_code2 = quantize_value(truth.1, config.bits, pair.output2);
        let plaintext =
            Plaintext((4 * ((input_index as u64) + input_offset)).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        input_times_us.push(start_input.elapsed().as_secs_f64() * 1_000_000.0);

        let start_core = Instant::now();
        let msed = lwe_ciphertext_centered_binary_modulus_switch(lwe_input, blind_rotation_log);
        let mut local_accumulator = accumulator.clone();
        let (sample0, sample1) = blind_rotate_and_extract_two_coefficients(
            &msed,
            &mut local_accumulator,
            &runtime.fourier_bsk,
        );
        core_times_us.push(start_core.elapsed().as_secs_f64() * 1_000_000.0);

        let start_decode = Instant::now();
        let ((code1, code2), local_projection_events, local_invalid_outputs) =
            decode_sdr_pbs_outputs(runtime, &sample0, &sample1, config.bits, delta);
        decode_times_us.push(start_decode.elapsed().as_secs_f64() * 1_000_000.0);

        projection_events += local_projection_events;
        invalid_outputs += local_invalid_outputs;
        let err1 = code1.abs_diff(expected_code1);
        let err2 = code2.abs_diff(expected_code2);
        if code1 == expected_code1 && code2 == expected_code2 {
            exact_recovery += 1;
        }
        if err1 <= 1 {
            code1_le1 += 1;
        }
        if err1 <= 2 {
            code1_le2 += 1;
        }
        if err2 <= 1 {
            code2_le1 += 1;
        }
        if err2 <= 2 {
            code2_le2 += 1;
        }
        if err1 <= 1 && err2 <= 1 {
            joint_le1 += 1;
        }
        if err1 <= 2 && err2 <= 2 {
            joint_le2 += 1;
        }
        sum_code_err1 += err1;
        sum_code_err2 += err2;
        max_code_err1 = max_code_err1.max(err1);
        max_code_err2 = max_code_err2.max(err2);
    }

    finalize_codebook_report(
        pair.name,
        config.kind.as_str(),
        config.bits,
        total_inputs,
        exact_recovery,
        projection_events,
        invalid_outputs,
        joint_le1,
        joint_le2,
        code1_le1,
        code1_le2,
        code2_le1,
        code2_le2,
        sum_code_err1,
        sum_code_err2,
        max_code_err1,
        max_code_err2,
        &input_times_us,
        &core_times_us,
        &decode_times_us,
    )
}

fn evaluate_many_lut_codebook(
    runtime: &mut Runtime,
    pair: &FunctionPair,
    config: SchemeConfig,
) -> CodebookReport {
    let layout = config.many_lut.expect("many-lut config is required");
    let (accumulator, func_chunk_size, delta, total_plaintext_modulus) =
        build_many_lut_accumulator(pair, config.bits, config.pbs, layout);
    let total_inputs = levels(config.bits);
    let mut exact_recovery = 0usize;
    let mut projection_events = 0usize;
    let mut invalid_outputs = 0usize;
    let mut joint_le1 = 0usize;
    let mut joint_le2 = 0usize;
    let mut code1_le1 = 0usize;
    let mut code1_le2 = 0usize;
    let mut code2_le1 = 0usize;
    let mut code2_le2 = 0usize;
    let mut sum_code_err1 = 0u64;
    let mut sum_code_err2 = 0u64;
    let mut max_code_err1 = 0u64;
    let mut max_code_err2 = 0u64;
    let mut input_times_us = Vec::with_capacity(total_inputs);
    let mut core_times_us = Vec::with_capacity(total_inputs);
    let mut decode_times_us = Vec::with_capacity(total_inputs);

    for input_index in 0..total_inputs {
        let start_input = Instant::now();
        let x = x_for_index(input_index, pair, total_inputs);
        let truth = (pair.compute_true)(x);
        let expected_code1 = quantize_value(truth.0, config.bits, pair.output1);
        let expected_code2 = quantize_value(truth.1, config.bits, pair.output2);
        let plaintext =
            Plaintext(((input_index as u64) + layout.input_offset as u64).wrapping_mul(delta));
        let lwe_input = encrypt_input(runtime, plaintext);
        input_times_us.push(start_input.elapsed().as_secs_f64() * 1_000_000.0);

        let start_core = Instant::now();
        let rotated_accumulator = centered_blind_rotate(runtime, lwe_input, &accumulator);
        core_times_us.push(start_core.elapsed().as_secs_f64() * 1_000_000.0);

        let start_decode = Instant::now();
        let mut raw_codes = [0u64; 2];
        let mut local_projection_events = 0usize;
        let mut local_invalid_outputs = 0usize;

        for (index, slot) in layout.used_slots.iter().copied().enumerate() {
            let mut output = LweCiphertext::new(
                0u64,
                runtime.big_lwe_sk.lwe_dimension().to_lwe_size(),
                runtime.params.ciphertext_modulus,
            );
            extract_lwe_sample_from_glwe_ciphertext(
                &rotated_accumulator,
                &mut output,
                MonomialDegree(slot * func_chunk_size),
            );

            let rounded =
                divide_round(decrypt_lwe_ciphertext(&runtime.big_lwe_sk, &output).0, delta)
                    % total_plaintext_modulus;
            let (projected, _, projected_flag) = nearest_contiguous_code(
                rounded,
                levels(config.bits) as u64,
                total_plaintext_modulus,
            );
            raw_codes[index] = projected;
            local_projection_events += usize::from(projected_flag);
            local_invalid_outputs += usize::from(rounded >= levels(config.bits) as u64);
        }
        decode_times_us.push(start_decode.elapsed().as_secs_f64() * 1_000_000.0);

        projection_events += local_projection_events;
        invalid_outputs += local_invalid_outputs;
        let err1 = raw_codes[0].abs_diff(expected_code1);
        let err2 = raw_codes[1].abs_diff(expected_code2);
        if raw_codes[0] == expected_code1 && raw_codes[1] == expected_code2 {
            exact_recovery += 1;
        }
        if err1 <= 1 {
            code1_le1 += 1;
        }
        if err1 <= 2 {
            code1_le2 += 1;
        }
        if err2 <= 1 {
            code2_le1 += 1;
        }
        if err2 <= 2 {
            code2_le2 += 1;
        }
        if err1 <= 1 && err2 <= 1 {
            joint_le1 += 1;
        }
        if err1 <= 2 && err2 <= 2 {
            joint_le2 += 1;
        }
        sum_code_err1 += err1;
        sum_code_err2 += err2;
        max_code_err1 = max_code_err1.max(err1);
        max_code_err2 = max_code_err2.max(err2);
    }

    finalize_codebook_report(
        pair.name,
        config.kind.as_str(),
        config.bits,
        total_inputs,
        exact_recovery,
        projection_events,
        invalid_outputs,
        joint_le1,
        joint_le2,
        code1_le1,
        code1_le2,
        code2_le1,
        code2_le2,
        sum_code_err1,
        sum_code_err2,
        max_code_err1,
        max_code_err2,
        &input_times_us,
        &core_times_us,
        &decode_times_us,
    )
}

fn finalize_report(
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
    stats: EvalStats,
) -> SchemeReport {
    let avg_input_us =
        stats.input_times_us.iter().sum::<f64>() / stats.input_times_us.len().max(1) as f64;
    let avg_core_us =
        stats.core_times_us.iter().sum::<f64>() / stats.core_times_us.len().max(1) as f64;
    let avg_decode_us =
        stats.decode_times_us.iter().sum::<f64>() / stats.decode_times_us.len().max(1) as f64;
    let avg_eval_us =
        stats.eval_times_us.iter().sum::<f64>() / stats.eval_times_us.len().max(1) as f64;

    SchemeReport {
        pair_name: pair.name,
        scheme_name: config.kind.as_str(),
        bits: config.bits,
        point_count,
        significant_errors: stats.significant_errors,
        threshold_counts: stats.threshold_counts,
        projection_events: stats.projection_events,
        invalid_outputs: stats.invalid_outputs,
        avg_input_us,
        avg_core_us,
        avg_decode_us,
        avg_eval_us,
        stats1: compute_summary(&stats.errors1),
        stats2: compute_summary(&stats.errors2),
    }
}

fn finalize_end_to_end_report(
    pair: &FunctionPair,
    config: SchemeConfig,
    point_count: usize,
    stats: EndToEndStats,
) -> EndToEndReport {
    let avg_input_us =
        stats.input_times_us.iter().sum::<f64>() / stats.input_times_us.len().max(1) as f64;
    let avg_core_us =
        stats.core_times_us.iter().sum::<f64>() / stats.core_times_us.len().max(1) as f64;
    let avg_decode_us =
        stats.decode_times_us.iter().sum::<f64>() / stats.decode_times_us.len().max(1) as f64;
    let avg_downstream_us = stats.downstream_times_us.iter().sum::<f64>()
        / stats.downstream_times_us.len().max(1) as f64;
    let avg_eval_us =
        stats.eval_times_us.iter().sum::<f64>() / stats.eval_times_us.len().max(1) as f64;
    let avg_e2e_us =
        stats.e2e_times_us.iter().sum::<f64>() / stats.e2e_times_us.len().max(1) as f64;

    EndToEndReport {
        pair_name: pair.name,
        scheme_name: config.kind.as_str(),
        bits: config.bits,
        point_count,
        projection_events: stats.projection_events,
        invalid_outputs: stats.invalid_outputs,
        score_significant_errors: stats.score_significant_errors,
        update_significant_errors: stats.update_significant_errors,
        score_stats: compute_summary(&stats.score_errors),
        update_stats: compute_summary(&stats.update_errors),
        avg_input_us,
        avg_core_us,
        avg_decode_us,
        avg_downstream_us,
        avg_eval_us,
        avg_e2e_us,
    }
}

fn print_runtime(label: &str, runtime: &Runtime) {
    println!(
        "runtime {:<18} | lwe_dim={} poly={} pbs=({},{}) | seed={} | bsk_gen={:.2}s fourier={:.2}s",
        label,
        runtime.params.small_lwe_dimension.0,
        runtime.params.polynomial_size.0,
        runtime.params.pbs_base_log.0,
        runtime.params.pbs_level.0,
        runtime
            .seed
            .map(|value| value.to_string())
            .unwrap_or_else(|| "system".to_string()),
        runtime.bsk_generation_time,
        runtime.fourier_conversion_time,
    );
}

fn print_report(report: &SchemeReport, pair: &FunctionPair) {
    println!(
        "{} | {} | bits={} | points={}",
        report.pair_name, report.scheme_name, report.bits, report.point_count
    );
    println!(
        "  significant errors > {:.1}% range: {}",
        REL_ERROR_THRESHOLD * 100.0,
        report.significant_errors
    );
    println!(
        "  projection events: {} | invalid outputs: {} | avg eval: {:.2} us",
        report.projection_events, report.invalid_outputs, report.avg_eval_us
    );
    println!(
        "  output1 [{:.4}, {:.4}] mean={:.3e} rmse={:.3e} p95={:.3e} max={:.3e}",
        pair.output1.min,
        pair.output1.max,
        report.stats1.mean,
        report.stats1.rmse,
        report.stats1.p95,
        report.stats1.max,
    );
    println!(
        "  output2 [{:.4}, {:.4}] mean={:.3e} rmse={:.3e} p95={:.3e} max={:.3e}",
        pair.output2.min,
        pair.output2.max,
        report.stats2.mean,
        report.stats2.rmse,
        report.stats2.p95,
        report.stats2.max,
    );
}

fn print_codebook_report(report: &CodebookReport) {
    println!(
        "{} | {} | bits={} | codebook inputs={} | exact={} joint<=1={} joint<=2={} mean_code_err=({:.3}, {:.3}) max_code_err=({}, {})",
        report.pair_name,
        report.scheme_name,
        report.bits,
        report.total_inputs,
        report.exact_recovery,
        report.joint_le1,
        report.joint_le2,
        report.mean_code_err1,
        report.mean_code_err2,
        report.max_code_err1,
        report.max_code_err2,
    );
}

fn print_end_to_end_report(report: &EndToEndReport) {
    println!(
        "{} | {} | bits={} | e2e points={}",
        report.pair_name, report.scheme_name, report.bits, report.point_count
    );
    println!(
        "  score sigerr > {:.1}%: {} | update sigerr > {:.1}%: {}",
        E2E_ERROR_THRESHOLD * 100.0,
        report.score_significant_errors,
        E2E_ERROR_THRESHOLD * 100.0,
        report.update_significant_errors,
    );
    println!(
        "  projection events: {} | invalid outputs: {} | avg eval: {:.2} us | avg downstream: {:.2} us | avg e2e: {:.2} us",
        report.projection_events,
        report.invalid_outputs,
        report.avg_eval_us,
        report.avg_downstream_us,
        report.avg_e2e_us,
    );
    println!(
        "  score err mean={:.3e} rmse={:.3e} p95={:.3e} max={:.3e}",
        report.score_stats.mean,
        report.score_stats.rmse,
        report.score_stats.p95,
        report.score_stats.max,
    );
    println!(
        "  update err mean={:.3e} rmse={:.3e} p95={:.3e} max={:.3e}",
        report.update_stats.mean,
        report.update_stats.rmse,
        report.update_stats.p95,
        report.update_stats.max,
    );
}

fn csv_row(report: &SchemeReport) -> String {
    let fields = [
        report.pair_name.to_string(),
        report.scheme_name.to_string(),
        report.bits.to_string(),
        report.point_count.to_string(),
        report.significant_errors.to_string(),
        report.threshold_counts[0].to_string(),
        report.threshold_counts[1].to_string(),
        report.threshold_counts[2].to_string(),
        report.projection_events.to_string(),
        report.invalid_outputs.to_string(),
        format!("{:.12e}", report.stats1.mean),
        format!("{:.12e}", report.stats1.std),
        format!("{:.12e}", report.stats1.rmse),
        format!("{:.12e}", report.stats1.median),
        format!("{:.12e}", report.stats1.p90),
        format!("{:.12e}", report.stats1.p95),
        format!("{:.12e}", report.stats1.p99),
        format!("{:.12e}", report.stats1.p999),
        format!("{:.12e}", report.stats1.max),
        format!("{:.12e}", report.stats2.mean),
        format!("{:.12e}", report.stats2.std),
        format!("{:.12e}", report.stats2.rmse),
        format!("{:.12e}", report.stats2.median),
        format!("{:.12e}", report.stats2.p90),
        format!("{:.12e}", report.stats2.p95),
        format!("{:.12e}", report.stats2.p99),
        format!("{:.12e}", report.stats2.p999),
        format!("{:.12e}", report.stats2.max),
        format!("{:.6}", report.avg_input_us),
        format!("{:.6}", report.avg_core_us),
        format!("{:.6}", report.avg_decode_us),
        format!("{:.6}", report.avg_eval_us),
    ];
    format!("{}\n", fields.join(","))
}

fn end_to_end_csv_row(report: &EndToEndReport) -> String {
    let fields = [
        report.pair_name.to_string(),
        report.scheme_name.to_string(),
        report.bits.to_string(),
        report.point_count.to_string(),
        report.projection_events.to_string(),
        report.invalid_outputs.to_string(),
        report.score_significant_errors.to_string(),
        format!("{:.12e}", report.score_stats.mean),
        format!("{:.12e}", report.score_stats.std),
        format!("{:.12e}", report.score_stats.rmse),
        format!("{:.12e}", report.score_stats.median),
        format!("{:.12e}", report.score_stats.p90),
        format!("{:.12e}", report.score_stats.p95),
        format!("{:.12e}", report.score_stats.p99),
        format!("{:.12e}", report.score_stats.p999),
        format!("{:.12e}", report.score_stats.max),
        report.update_significant_errors.to_string(),
        format!("{:.12e}", report.update_stats.mean),
        format!("{:.12e}", report.update_stats.std),
        format!("{:.12e}", report.update_stats.rmse),
        format!("{:.12e}", report.update_stats.median),
        format!("{:.12e}", report.update_stats.p90),
        format!("{:.12e}", report.update_stats.p95),
        format!("{:.12e}", report.update_stats.p99),
        format!("{:.12e}", report.update_stats.p999),
        format!("{:.12e}", report.update_stats.max),
        format!("{:.6}", report.avg_input_us),
        format!("{:.6}", report.avg_core_us),
        format!("{:.6}", report.avg_decode_us),
        format!("{:.6}", report.avg_downstream_us),
        format!("{:.6}", report.avg_eval_us),
        format!("{:.6}", report.avg_e2e_us),
    ];
    format!("{}\n", fields.join(","))
}

fn codebook_csv_row(report: &CodebookReport) -> String {
    let fields = [
        report.pair_name.to_string(),
        report.scheme_name.to_string(),
        report.bits.to_string(),
        report.total_inputs.to_string(),
        report.exact_recovery.to_string(),
        report.mismatches.to_string(),
        report.projection_events.to_string(),
        report.invalid_outputs.to_string(),
        report.joint_le1.to_string(),
        report.joint_le2.to_string(),
        report.code1_le1.to_string(),
        report.code1_le2.to_string(),
        report.code2_le1.to_string(),
        report.code2_le2.to_string(),
        format!("{:.6}", report.mean_code_err1),
        format!("{:.6}", report.mean_code_err2),
        report.max_code_err1.to_string(),
        report.max_code_err2.to_string(),
        format!("{:.6}", report.avg_input_us),
        format!("{:.6}", report.avg_core_us),
        format!("{:.6}", report.avg_decode_us),
        format!("{:.6}", report.avg_eval_us),
    ];
    format!("{}\n", fields.join(","))
}

fn write_summary_csv(path: &Path, reports: &[SchemeReport]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(CSV_HEADER.as_bytes())?;
    for report in reports {
        file.write_all(csv_row(report).as_bytes())?;
    }
    Ok(())
}

fn write_end_to_end_summary_csv(path: &Path, reports: &[EndToEndReport]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(E2E_CSV_HEADER.as_bytes())?;
    for report in reports {
        file.write_all(end_to_end_csv_row(report).as_bytes())?;
    }
    Ok(())
}

fn write_codebook_summary_csv(path: &Path, reports: &[CodebookReport]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(CODEBOOK_CSV_HEADER.as_bytes())?;
    for report in reports {
        file.write_all(codebook_csv_row(report).as_bytes())?;
    }
    Ok(())
}

fn write_runtime_csv(
    path: &Path,
    standard: Option<&Runtime>,
    sdr_pbs: Option<&Runtime>,
    many_lut: Option<&Runtime>,
) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(RUNTIME_CSV_HEADER.as_bytes())?;

    let rows = [
        ("standard_pbs", standard),
        ("sdr_pbs", sdr_pbs),
        ("many_lut", many_lut),
    ];

    for (scheme, runtime) in rows {
        let selected = runtime.is_some();
        let seed = runtime
            .and_then(|runtime| runtime.seed)
            .map(|value| value.to_string())
            .unwrap_or_else(|| "".to_string());
        let bsk_generation_s = runtime
            .map(|runtime| format!("{:.6}", runtime.bsk_generation_time))
            .unwrap_or_else(|| "".to_string());
        let fourier_conversion_s = runtime
            .map(|runtime| format!("{:.6}", runtime.fourier_conversion_time))
            .unwrap_or_else(|| "".to_string());

        let fields = [
            scheme.to_string(),
            selected.to_string(),
            seed,
            bsk_generation_s,
            fourier_conversion_s,
        ];
        file.write_all(format!("{}\n", fields.join(",")).as_bytes())?;
    }

    Ok(())
}

fn write_encoding_coverage(
    path: &Path,
    reports: &[EncodingCoverageReport],
    selected_pairs: &[&FunctionPair],
) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    for (report, pair) in reports.iter().zip(selected_pairs.iter().copied()) {
        writeln!(file, "{}", format_encoding_coverage(report, pair))?;
    }
    Ok(())
}

fn write_run_notes(
    path: &Path,
    point_count: usize,
    mode: ExperimentMode,
    selection: SchemeSelection,
    master_seed: Option<u128>,
    standard: SchemeConfig,
    sdr_pbs: SchemeConfig,
    many_lut: SchemeConfig,
) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "points={point_count}")?;
    writeln!(
        file,
        "mode={}",
        match mode {
            ExperimentMode::Continuous => "continuous",
            ExperimentMode::Codebook => "codebook",
            ExperimentMode::EndToEnd => "end_to_end",
        }
    )?;
    writeln!(file, "selected_schemes={}", selected_schemes_label(selection))?;
    writeln!(
        file,
        "master_seed={}",
        master_seed
            .map(|value| value.to_string())
            .unwrap_or_else(|| "system".to_string())
    )?;
    writeln!(file, "standard={standard:?}")?;
    writeln!(file, "sdr_pbs={sdr_pbs:?}")?;
    writeln!(file, "many_lut={many_lut:?}")?;
    if let Some(guard) = standard.input_guard {
        writeln!(file, "standard_guard_factor={}", guard.total_factor)?;
        writeln!(file, "standard_input_offset={}", guard.input_offset)?;
    }
    if let Some(guard) = sdr_pbs.input_guard {
        writeln!(file, "sdr_guard_factor={}", guard.total_factor)?;
        writeln!(file, "sdr_input_offset={}", guard.input_offset)?;
        writeln!(file, "sdr_box_layout=4")?;
        writeln!(
            file,
            "sdr_box_note=guard_factor and box_layout are distinct; polynomial_size=4*(levels*guard_factor)"
        )?;
    }
    if let Some(layout) = many_lut.many_lut {
        writeln!(file, "many_total_factor={}", layout.total_factor)?;
        writeln!(file, "many_input_offset={}", layout.input_offset)?;
        writeln!(file, "many_slot_count={}", layout.slot_count)?;
    }
    writeln!(
        file,
        "threshold={:.2}% of each output dynamic range",
        REL_ERROR_THRESHOLD * 100.0
    )?;
    let threshold_labels = SIGNIFICANT_THRESHOLDS
        .iter()
        .copied()
        .map(threshold_label)
        .collect::<Vec<_>>()
        .join(",");
    writeln!(file, "threshold_labels={threshold_labels}")?;
    if mode == ExperimentMode::EndToEnd {
        writeln!(file, "e2e_error_threshold={:.2}%", E2E_ERROR_THRESHOLD * 100.0)?;
        writeln!(
            file,
            "e2e_score=({:.2})*a_norm+({:.2})*d_norm",
            E2E_SCORE_WEIGHT_ACTIVATION,
            E2E_SCORE_WEIGHT_DERIVATIVE,
        )?;
        writeln!(
            file,
            "e2e_update={:.2}-{:.2}*(a_norm-{:.2})*d_norm",
            E2E_UPDATE_BASELINE,
            E2E_UPDATE_STEP_SIZE,
            E2E_UPDATE_TARGET,
        )?;
    }
    Ok(())
}

fn main() {
    let point_count = parse_env_usize(
        "PAPER_NUM_TESTS",
        parse_env_usize("SDR_PBS_NUM_TESTS", DEFAULT_NUM_TESTS),
    );
    let mode = parse_experiment_mode();
    let scheme_selection = parse_scheme_selection();
    let master_seed = parse_env_u128("PAPER_MASTER_SEED");
    let pair_filters = pair_filters();
    let selected_pairs = selected_pairs(&pair_filters);
    let output_dir = parse_output_dir();

    if selected_pairs.is_empty() {
        eprintln!("No function pairs matched the current filter.");
        return;
    }
    if !scheme_selection.standard && !scheme_selection.sdr_pbs && !scheme_selection.many_lut {
        eprintln!("No schemes matched PAPER_SCHEMES.");
        return;
    }

    let standard_config = standard_scheme();
    let sdr_pbs_config = sdr_pbs_scheme();
    let many_lut_config = many_lut_scheme();
    let encoding_check_samples =
        parse_env_usize("PAPER_ENCODING_CHECK_SAMPLES", DEFAULT_ENCODING_CHECK_SAMPLES);
    let encoding_reports = selected_pairs
        .iter()
        .map(|pair| scan_output_encoding(pair, encoding_check_samples))
        .collect::<Vec<_>>();

    if encoding_reports.iter().any(EncodingCoverageReport::has_clipping) {
        eprintln!(
            "warning: declared output encodings do not fully cover at least one selected pair over {} dense samples",
            encoding_check_samples
        );
        for (report, pair) in encoding_reports
            .iter()
            .zip(selected_pairs.iter().copied())
            .filter(|(report, _)| report.has_clipping())
        {
            eprintln!("  {}", format_encoding_coverage(report, pair));
        }
    } else {
        println!(
            "encoding coverage check passed for selected pairs over {} dense samples",
            encoding_check_samples
        );
    }

    println!("building runtimes...");
    let mut standard_runtime = if scheme_selection.standard {
        Some(build_runtime(
            standard_config.pbs,
            derive_seed(master_seed, 0x1000_u128),
        ))
    } else {
        None
    };
    let mut sdr_pbs_runtime = if scheme_selection.sdr_pbs {
        Some(build_runtime(
            sdr_pbs_config.pbs,
            derive_seed(master_seed, 0x2000_u128),
        ))
    } else {
        None
    };
    let mut many_lut_runtime = if scheme_selection.many_lut {
        Some(build_runtime(
            many_lut_config.pbs,
            derive_seed(master_seed, 0x3000_u128),
        ))
    } else {
        None
    };
    if let Some(runtime) = &standard_runtime {
        print_runtime("standard", runtime);
    }
    if let Some(runtime) = &sdr_pbs_runtime {
        print_runtime("sdr_pbs", runtime);
    }
    if let Some(runtime) = &many_lut_runtime {
        print_runtime("many-lut", runtime);
    }

    if let Some(dir) = &output_dir {
        if let Err(error) = fs::create_dir_all(dir) {
            eprintln!("failed to create output dir {}: {error}", dir.display());
        }
        let encoding_path = dir.join("encoding_coverage.txt");
        if let Err(error) = write_encoding_coverage(&encoding_path, &encoding_reports, &selected_pairs)
        {
            eprintln!("failed to write {}: {error}", encoding_path.display());
        }
    }

    let mut reports = Vec::new();
    let mut end_to_end_reports = Vec::new();
    let mut codebook_reports = Vec::new();

    for pair in selected_pairs {
        println!("\n=== {} ===", pair.name);

        match mode {
            ExperimentMode::Continuous => {
                if let Some(runtime) = &mut standard_runtime {
                    let report = evaluate_standard(runtime, pair, standard_config, point_count);
                    print_report(&report, pair);
                    reports.push(report);
                }
                if let Some(runtime) = &mut sdr_pbs_runtime {
                    let report = evaluate_sdr_pbs(runtime, pair, sdr_pbs_config, point_count);
                    print_report(&report, pair);
                    reports.push(report);
                }
                if let Some(runtime) = &mut many_lut_runtime {
                    let report = evaluate_many_lut(runtime, pair, many_lut_config, point_count);
                    print_report(&report, pair);
                    reports.push(report);
                }
            }
            ExperimentMode::Codebook => {
                if let Some(runtime) = &mut standard_runtime {
                    let report = evaluate_standard_codebook(runtime, pair, standard_config);
                    print_codebook_report(&report);
                    codebook_reports.push(report);
                }
                if let Some(runtime) = &mut sdr_pbs_runtime {
                    let report = evaluate_sdr_pbs_codebook(runtime, pair, sdr_pbs_config);
                    print_codebook_report(&report);
                    codebook_reports.push(report);
                }
                if let Some(runtime) = &mut many_lut_runtime {
                    let report = evaluate_many_lut_codebook(runtime, pair, many_lut_config);
                    print_codebook_report(&report);
                    codebook_reports.push(report);
                }
            }
            ExperimentMode::EndToEnd => {
                if let Some(runtime) = &mut standard_runtime {
                    let report =
                        evaluate_standard_end_to_end(runtime, pair, standard_config, point_count);
                    print_end_to_end_report(&report);
                    end_to_end_reports.push(report);
                }
                if let Some(runtime) = &mut sdr_pbs_runtime {
                    let report =
                        evaluate_sdr_pbs_end_to_end(runtime, pair, sdr_pbs_config, point_count);
                    print_end_to_end_report(&report);
                    end_to_end_reports.push(report);
                }
                if let Some(runtime) = &mut many_lut_runtime {
                    let report =
                        evaluate_many_lut_end_to_end(runtime, pair, many_lut_config, point_count);
                    print_end_to_end_report(&report);
                    end_to_end_reports.push(report);
                }
            }
        }
    }

    if let Some(dir) = output_dir {
        let notes_path = dir.join("run_notes.txt");
        let runtime_path = dir.join("runtime_breakdown.csv");

        match mode {
            ExperimentMode::Continuous => {
                let summary_path = dir.join("summary.csv");
                if let Err(error) = write_summary_csv(&summary_path, &reports) {
                    eprintln!("failed to write {}: {error}", summary_path.display());
                } else {
                    println!("\nsummary written to {}", summary_path.display());
                }
            }
            ExperimentMode::Codebook => {
                let summary_path = dir.join("codebook_summary.csv");
                if let Err(error) = write_codebook_summary_csv(&summary_path, &codebook_reports) {
                    eprintln!("failed to write {}: {error}", summary_path.display());
                } else {
                    println!("\ncodebook summary written to {}", summary_path.display());
                }
            }
            ExperimentMode::EndToEnd => {
                let summary_path = dir.join("end_to_end_summary.csv");
                if let Err(error) =
                    write_end_to_end_summary_csv(&summary_path, &end_to_end_reports)
                {
                    eprintln!("failed to write {}: {error}", summary_path.display());
                } else {
                    println!("\nend-to-end summary written to {}", summary_path.display());
                }
            }
        }

        if let Err(error) = write_runtime_csv(
            &runtime_path,
            standard_runtime.as_ref(),
            sdr_pbs_runtime.as_ref(),
            many_lut_runtime.as_ref(),
        ) {
            eprintln!("failed to write {}: {error}", runtime_path.display());
        }

        if let Err(error) = write_run_notes(
            &notes_path,
            point_count,
            mode,
            scheme_selection,
            master_seed,
            standard_config,
            sdr_pbs_config,
            many_lut_config,
        )
        {
            eprintln!("failed to write {}: {error}", notes_path.display());
        }
    }
}
