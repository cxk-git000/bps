param(
    [string]$CodeRoot = "",
    [string]$ExePath = "",
    [string]$OutputRoot = "",
    [string]$TmpDir = "",
    [int]$Points = 10000,
    [string]$MasterSeed = "10101"
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
    $OutputRoot = Join-Path $RepoRoot "results\regenerated\04_main_guarded_all_pairs_10000"
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

New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
Ensure-ReleaseBuild

$oldValues = @{}
foreach ($key in $managedEnvKeys) {
    $oldValues[$key] = [System.Environment]::GetEnvironmentVariable($key, "Process")
}

$logPath = Join-Path $OutputRoot "run.log"
try {
    [System.Environment]::SetEnvironmentVariable("TMP", $TmpDir, "Process")
    [System.Environment]::SetEnvironmentVariable("TEMP", $TmpDir, "Process")

    foreach ($key in $managedEnvKeys) {
        [System.Environment]::SetEnvironmentVariable($key, $null, "Process")
    }

    [System.Environment]::SetEnvironmentVariable("PAPER_OUTPUT_DIR", $OutputRoot, "Process")
    [System.Environment]::SetEnvironmentVariable("PAPER_NUM_TESTS", [string]$Points, "Process")
    [System.Environment]::SetEnvironmentVariable("PAPER_MASTER_SEED", $MasterSeed, "Process")
    [System.Environment]::SetEnvironmentVariable("PAPER_SCHEMES", "standard,sdr_pbs,many", "Process")

    "[$(Get-Date -Format s)] START 04_main_guarded_all_pairs_10000" | Tee-Object -FilePath $logPath -Append | Out-Null
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
        throw "main canonical run failed with exit code $commandExitCode"
    }
    "[$(Get-Date -Format s)] END   04_main_guarded_all_pairs_10000" | Tee-Object -FilePath $logPath -Append | Out-Null
}
finally {
    foreach ($key in $managedEnvKeys) {
        [System.Environment]::SetEnvironmentVariable($key, $oldValues[$key], "Process")
    }
}
