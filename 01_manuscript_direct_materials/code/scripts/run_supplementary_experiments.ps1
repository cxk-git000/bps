param(
    [string]$CodeRoot = "",
    [string]$ExePath = "",
    [string]$OutputRoot = "",
    [string]$TmpDir = "",
    [ValidateSet("all", "short", "tail100k", "endtoend")]
    [string]$Section = "all"
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($CodeRoot)) {
    $CodeRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
}
$RepoRoot = (Resolve-Path (Join-Path $CodeRoot "..")).Path

if ([string]::IsNullOrWhiteSpace($ExePath)) {
    $ExePath = Join-Path $CodeRoot "target\release\Re-test.exe"
}
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $RepoRoot "results\regenerated"
}
if ([string]::IsNullOrWhiteSpace($TmpDir)) {
    $TmpDir = Join-Path $RepoRoot "tmp"
}

$managedEnvKeys = @(
    "PAPER_OUTPUT_DIR",
    "PAPER_NUM_TESTS",
    "SDR_PBS_NUM_TESTS",
    "PAPER_MASTER_SEED",
    "PAPER_SCHEMES",
    "PAPER_PAIR_FILTER",
    "SDR_PBS_PAIR_FILTER",
    "PAPER_MODE",
    "PAPER_ENCODING_CHECK_SAMPLES",
    "PAPER_STANDARD_BITS",
    "PAPER_STANDARD_POLY_SIZE",
    "PAPER_STANDARD_LWE_DIM",
    "PAPER_STANDARD_INPUT_FACTOR",
    "PAPER_STANDARD_INPUT_OFFSET",
    "PAPER_SDR_BITS",
    "PAPER_SDR_POLY_SIZE",
    "PAPER_SDR_LWE_DIM",
    "PAPER_SDR_INPUT_FACTOR",
    "PAPER_SDR_INPUT_OFFSET",
    "PAPER_MANY_BITS",
    "PAPER_MANY_TOTAL_FACTOR",
    "PAPER_MANY_SLOT_COUNT",
    "PAPER_MANY_INPUT_OFFSET"
)

function Ensure-ReleaseBuild {
    if (Test-Path $ExePath) {
        return
    }

    Push-Location $CodeRoot
    try {
        cargo build --release --bins
        if ($LASTEXITCODE -ne 0) {
            throw "cargo build failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
    }
}

function Get-GuardPolySize {
    param(
        [int]$Bits,
        [int]$Factor
    )

    return [int]([math]::Pow(2, $Bits + 2) * $Factor)
}

function Invoke-Experiment {
    param(
        [string]$Name,
        [hashtable]$EnvMap
    )

    $runDir = Join-Path $OutputRoot $Name
    $logPath = Join-Path $runDir "run.log"
    New-Item -ItemType Directory -Force -Path $runDir | Out-Null

    $oldValues = @{}
    foreach ($key in $managedEnvKeys) {
        $oldValues[$key] = [System.Environment]::GetEnvironmentVariable($key, "Process")
    }

    try {
        [System.Environment]::SetEnvironmentVariable("TMP", $TmpDir, "Process")
        [System.Environment]::SetEnvironmentVariable("TEMP", $TmpDir, "Process")

        foreach ($key in $managedEnvKeys) {
            [System.Environment]::SetEnvironmentVariable($key, $null, "Process")
        }

        [System.Environment]::SetEnvironmentVariable("PAPER_OUTPUT_DIR", $runDir, "Process")
        foreach ($entry in $EnvMap.GetEnumerator()) {
            [System.Environment]::SetEnvironmentVariable(
                $entry.Key,
                [string]$entry.Value,
                "Process"
            )
        }

        "[$(Get-Date -Format s)] START $Name" | Tee-Object -FilePath $logPath -Append | Out-Null
        $oldErrorActionPreference = $ErrorActionPreference
        $commandExitCode = 0
        try {
            $ErrorActionPreference = "Continue"
            & $ExePath 2>&1 | Tee-Object -FilePath $logPath -Append
            $commandExitCode = $LASTEXITCODE
        }
        finally {
            $ErrorActionPreference = $oldErrorActionPreference
        }
        if ($commandExitCode -ne 0) {
            throw "run $Name failed with exit code $commandExitCode"
        }
        "[$(Get-Date -Format s)] END   $Name" | Tee-Object -FilePath $logPath -Append | Out-Null
    }
    finally {
        foreach ($key in $managedEnvKeys) {
            [System.Environment]::SetEnvironmentVariable($key, $oldValues[$key], "Process")
        }
    }
}

New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
Ensure-ReleaseBuild

if ($Section -in @("all", "short")) {
    $seedRuns = @(10101, 20202, 30303)
    foreach ($seed in $seedRuns) {
        Invoke-Experiment -Name "05_repeatability_multiseed_1000\allpairs_1000_seed_$seed" -EnvMap @{
            PAPER_NUM_TESTS = 1000
            PAPER_MASTER_SEED = $seed
            PAPER_SCHEMES = "standard,sdr_pbs,many"
        }
    }

    $guardPairs = @("softplus_sigmoid", "tanh_sech2", "sigmoid_sigmoid_deriv")
    $guardConfigs = @(
        @{ Name = "factor1_offset0"; Factor = 1; Offset = 0 },
        @{ Name = "factor2_offset0"; Factor = 2; Offset = 0 },
        @{ Name = "factor2_offset256"; Factor = 2; Offset = 256 },
        @{ Name = "factor2_offset512"; Factor = 2; Offset = 512 }
    )

    foreach ($pair in $guardPairs) {
        foreach ($cfg in $guardConfigs) {
            $bits = 10
            $poly = Get-GuardPolySize -Bits $bits -Factor $cfg.Factor
            Invoke-Experiment -Name "06_guardband_ablation\$pair\$($cfg.Name)" -EnvMap @{
                PAPER_NUM_TESTS = 1000
                PAPER_MASTER_SEED = 40404
                PAPER_SCHEMES = "standard,sdr_pbs"
                PAPER_PAIR_FILTER = $pair
                PAPER_STANDARD_BITS = $bits
                PAPER_STANDARD_POLY_SIZE = $poly
                PAPER_STANDARD_LWE_DIM = 1012
                PAPER_STANDARD_INPUT_FACTOR = $cfg.Factor
                PAPER_STANDARD_INPUT_OFFSET = $cfg.Offset
                PAPER_SDR_BITS = $bits
                PAPER_SDR_POLY_SIZE = $poly
                PAPER_SDR_LWE_DIM = 1012
                PAPER_SDR_INPUT_FACTOR = $cfg.Factor
                PAPER_SDR_INPUT_OFFSET = $cfg.Offset
            }
        }
    }

    Invoke-Experiment -Name "07_codebook_recovery_validation\allpairs" -EnvMap @{
        PAPER_MODE = "codebook"
        PAPER_MASTER_SEED = 50505
        PAPER_SCHEMES = "standard,sdr_pbs,many"
    }

    $bitPairs = @("tanh_sech2", "softplus_sigmoid", "gelu_gelu_deriv")
    foreach ($pair in $bitPairs) {
        foreach ($bits in @(8, 9, 10, 11)) {
            $guardPoly = Get-GuardPolySize -Bits $bits -Factor 2
            $offset = [int][math]::Pow(2, $bits - 1)
            $envMap = @{
                PAPER_NUM_TESTS = 1000
                PAPER_MASTER_SEED = (60000 + $bits)
                PAPER_PAIR_FILTER = $pair
                PAPER_STANDARD_BITS = $bits
                PAPER_STANDARD_POLY_SIZE = $guardPoly
                PAPER_STANDARD_LWE_DIM = 1012
                PAPER_STANDARD_INPUT_FACTOR = 2
                PAPER_STANDARD_INPUT_OFFSET = $offset
                PAPER_SDR_BITS = $bits
                PAPER_SDR_POLY_SIZE = $guardPoly
                PAPER_SDR_LWE_DIM = 1012
                PAPER_SDR_INPUT_FACTOR = 2
                PAPER_SDR_INPUT_OFFSET = $offset
            }

            if ($bits -le 10) {
                $manyOffset = [int][math]::Pow(2, $bits - 1)
                $envMap["PAPER_SCHEMES"] = "standard,sdr_pbs,many"
                $envMap["PAPER_MANY_BITS"] = $bits
                $envMap["PAPER_MANY_TOTAL_FACTOR"] = 4
                $envMap["PAPER_MANY_SLOT_COUNT"] = 2
                $envMap["PAPER_MANY_INPUT_OFFSET"] = $manyOffset
            }
            else {
                $envMap["PAPER_SCHEMES"] = "standard,sdr_pbs"
            }

            Invoke-Experiment -Name "08_bitwidth_sensitivity\$pair\bits_$bits" -EnvMap $envMap
        }
    }
}

if ($Section -in @("all", "tail100k")) {
    foreach ($pair in @("softplus_sigmoid", "sigmoid_sigmoid_deriv", "gelu_gelu_deriv")) {
        Invoke-Experiment -Name "09_tail_stability_100k\$pair" -EnvMap @{
            PAPER_NUM_TESTS = 100000
            PAPER_MASTER_SEED = 70707
            PAPER_SCHEMES = "standard,sdr_pbs"
            PAPER_PAIR_FILTER = $pair
        }
    }
}

if ($Section -in @("all", "endtoend")) {
    Invoke-Experiment -Name "12_end_to_end_micro_pipeline\representative_pairs_1000" -EnvMap @{
        PAPER_MODE = "end_to_end"
        PAPER_NUM_TESTS = 1000
        PAPER_MASTER_SEED = 80808
        PAPER_SCHEMES = "standard,sdr_pbs,many"
        PAPER_PAIR_FILTER = "softplus_sigmoid,sigmoid_sigmoid_deriv,gelu_gelu_deriv"
    }
}
