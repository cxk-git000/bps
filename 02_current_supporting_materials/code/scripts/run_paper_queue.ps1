param(
    [string]$CodeRoot = "",
    [string]$ExePath = "",
    [string]$OutputRoot = "",
    [string]$TmpDir = "",
    [string]$MasterSeed = ""
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($CodeRoot)) {
    $CodeRoot = (
        Resolve-Path (Join-Path $PSScriptRoot "..\..\..\01_manuscript_direct_materials\code")
    ).Path
}
$SupportRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path

if ([string]::IsNullOrWhiteSpace($ExePath)) {
    $ExePath = Join-Path $CodeRoot "target\release\Re-test.exe"
}
if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $SupportRoot "results\reproduced\paper_queue_batch"
}
if ([string]::IsNullOrWhiteSpace($TmpDir)) {
    $TmpDir = Join-Path $SupportRoot "tmp"
}

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

function Invoke-PaperRun {
    param(
        [string]$Name,
        [int]$Points,
        [string]$PairFilter = ""
    )

    $runDir = Join-Path $OutputRoot $Name
    $logPath = Join-Path $runDir "run.log"
    New-Item -ItemType Directory -Force -Path $runDir | Out-Null

    $oldNumTests = [System.Environment]::GetEnvironmentVariable("PAPER_NUM_TESTS", "Process")
    $oldPairFilter = [System.Environment]::GetEnvironmentVariable("PAPER_PAIR_FILTER", "Process")
    $oldOutputDir = [System.Environment]::GetEnvironmentVariable("PAPER_OUTPUT_DIR", "Process")
    $oldMasterSeed = [System.Environment]::GetEnvironmentVariable("PAPER_MASTER_SEED", "Process")

    try {
        [System.Environment]::SetEnvironmentVariable("PAPER_NUM_TESTS", [string]$Points, "Process")
        [System.Environment]::SetEnvironmentVariable("PAPER_OUTPUT_DIR", $runDir, "Process")

        if ([string]::IsNullOrWhiteSpace($PairFilter)) {
            [System.Environment]::SetEnvironmentVariable("PAPER_PAIR_FILTER", $null, "Process")
        }
        else {
            [System.Environment]::SetEnvironmentVariable("PAPER_PAIR_FILTER", $PairFilter, "Process")
        }

        if ([string]::IsNullOrWhiteSpace($MasterSeed)) {
            [System.Environment]::SetEnvironmentVariable("PAPER_MASTER_SEED", $null, "Process")
        }
        else {
            [System.Environment]::SetEnvironmentVariable("PAPER_MASTER_SEED", $MasterSeed, "Process")
        }

        "[$(Get-Date -Format s)] START $Name" | Tee-Object -FilePath $logPath -Append | Out-Null
        & $ExePath 2>&1 | Tee-Object -FilePath $logPath -Append
        if ($LASTEXITCODE -ne 0) {
            throw "run $Name failed with exit code $LASTEXITCODE"
        }
        "[$(Get-Date -Format s)] END   $Name" | Tee-Object -FilePath $logPath -Append | Out-Null
    }
    finally {
        [System.Environment]::SetEnvironmentVariable("PAPER_NUM_TESTS", $oldNumTests, "Process")
        [System.Environment]::SetEnvironmentVariable("PAPER_PAIR_FILTER", $oldPairFilter, "Process")
        [System.Environment]::SetEnvironmentVariable("PAPER_OUTPUT_DIR", $oldOutputDir, "Process")
        [System.Environment]::SetEnvironmentVariable("PAPER_MASTER_SEED", $oldMasterSeed, "Process")
    }
}

New-Item -ItemType Directory -Force -Path $OutputRoot | Out-Null
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null
[System.Environment]::SetEnvironmentVariable("TMP", $TmpDir, "Process")
[System.Environment]::SetEnvironmentVariable("TEMP", $TmpDir, "Process")
Ensure-ReleaseBuild

$runs = @(
    @{ Name = "allpairs_1000"; Points = 1000; PairFilter = "" },
    @{ Name = "tanh_10000"; Points = 10000; PairFilter = "tanh" },
    @{ Name = "sigmoid_10000"; Points = 10000; PairFilter = "sigmoid" },
    @{ Name = "gelu_10000"; Points = 10000; PairFilter = "gelu" },
    @{ Name = "tanh_100000"; Points = 100000; PairFilter = "tanh" }
)

foreach ($run in $runs) {
    Invoke-PaperRun -Name $run.Name -Points $run.Points -PairFilter $run.PairFilter
}
