param(
    [string]$CodeRoot = "",
    [string]$OutputRoot = "",
    [string]$TargetDir = "",
    [string]$TmpDir = "",
    [string]$MasterSeed = "10101"
)

$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false

if ([string]::IsNullOrWhiteSpace($CodeRoot)) {
    $CodeRoot = (
        Resolve-Path (Join-Path $PSScriptRoot "..\..\..\01_manuscript_direct_materials\code")
    ).Path
}
$SupportRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $SupportRoot "results\reproduced\01_identity_diagnostics\base_matrix"
}
if ([string]::IsNullOrWhiteSpace($TargetDir)) {
    $TargetDir = Join-Path $CodeRoot "target"
}
if ([string]::IsNullOrWhiteSpace($TmpDir)) {
    $TmpDir = Join-Path $SupportRoot "tmp"
}

function Invoke-DiagnoseRun {
    param(
        [hashtable]$EnvMap,
        [string]$OutFile
    )

    $oldValues = @{}
    foreach ($key in $EnvMap.Keys) {
        $oldValues[$key] = [System.Environment]::GetEnvironmentVariable($key, "Process")
        [System.Environment]::SetEnvironmentVariable($key, [string]$EnvMap[$key], "Process")
    }

    Push-Location $CodeRoot
    try {
        cargo run --quiet --bin diagnose 2>&1 | Out-File -FilePath $OutFile -Encoding utf8
        if ($LASTEXITCODE -ne 0) {
            throw "cargo run failed with exit code $LASTEXITCODE"
        }
    }
    finally {
        Pop-Location
        foreach ($key in $EnvMap.Keys) {
            [System.Environment]::SetEnvironmentVariable($key, $oldValues[$key], "Process")
        }
    }
}

New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
New-Item -ItemType Directory -Force -Path $TargetDir | Out-Null
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

$common = @{
    CARGO_TARGET_DIR = $TargetDir
    TMP = $TmpDir
    TEMP = $TmpDir
    DIAG_BITS = "10"
    DIAG_SAMPLES_PER_MESSAGE = "4"
    DIAG_MESSAGE_LIMIT = "128"
    DIAG_LWE_DIM = "930"
    DIAG_POLY_SIZE = "4096"
    DIAG_PBS_BASE_LOG = "15"
    DIAG_PBS_LEVEL = "2"
    DIAG_LWE_NOISE_STD = "6.782362904013915e-07"
    DIAG_GLWE_NOISE_STD = "2.168404344971009e-19"
}

if (-not [string]::IsNullOrWhiteSpace($MasterSeed)) {
    $common["DIAG_MASTER_SEED"] = $MasterSeed
}

$runs = @(
    @{
        Name = "standard_identity_centered"
        Env = @{
            DIAG_MODE = "standard_identity"
            DIAG_MS_MODE = "centered"
        }
    },
    @{
        Name = "standard_identity_standard"
        Env = @{
            DIAG_MODE = "standard_identity"
            DIAG_MS_MODE = "standard"
        }
    },
    @{
        Name = "many_lut_identity_standard"
        Env = @{
            DIAG_MODE = "many_lut"
            DIAG_MS_MODE = "standard"
        }
    },
    @{
        Name = "many_lut_identity_centered"
        Env = @{
            DIAG_MODE = "many_lut"
            DIAG_MS_MODE = "centered"
        }
    },
    @{
        Name = "sdr_pbs_identity_standard"
        Env = @{
            DIAG_MODE = "sdr_pbs"
            DIAG_MS_MODE = "standard"
        }
    },
    @{
        Name = "sdr_pbs_identity_centered"
        Env = @{
            DIAG_MODE = "sdr_pbs"
            DIAG_MS_MODE = "centered"
        }
    }
)

foreach ($run in $runs) {
    $outFile = Join-Path $OutputRoot ($run.Name + ".txt")
    $envMap = @{}

    foreach ($key in $common.Keys) {
        $envMap[$key] = $common[$key]
    }
    foreach ($key in $run.Env.Keys) {
        $envMap[$key] = $run.Env[$key]
    }

    Write-Host "Running $($run.Name) -> $outFile"
    Invoke-DiagnoseRun -EnvMap $envMap -OutFile $outFile
}
