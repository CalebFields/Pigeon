Param()
$ErrorActionPreference = 'Continue'

Push-Location "$PSScriptRoot\..\Pigeon"

try {
    # Capture all output without printing intermediate lines
    # Use verbose to ensure we see per-binary "Running` lines for labeling
    $lines = & cargo test --verbose 2>&1
    $exitCode = $LASTEXITCODE
}
finally {
    Pop-Location
}

# Aggregate totals from individual test binaries (including doc-tests)
$totalPassed = 0
$totalFailed = 0
$totalIgnored = 0
$totalMeasured = 0
$totalFiltered = 0
$anyFailedStatus = $false

$pattern = 'test result: (?<status>ok|FAILED)\. (?<passed>\d+) passed; (?<failed>\d+) failed; (?<ignored>\d+) ignored; (?<measured>\d+) measured; (?<filtered>\d+) filtered out;'
$currentLabel = $null
foreach ($line in $lines) {
    # Track label from Running lines
    if ($line -match 'Running `(?<cmd>[^`]+)`') {
        $cmd = $Matches['cmd']
        if ($cmd -match '\\deps\\(?<name>[^\\-]+)-') { $currentLabel = $Matches['name'] }
        elseif ($cmd -match 'rustdoc\.exe' -and $cmd -match '--crate-name\s+(?<cname>\S+)') { $currentLabel = "doc-tests $($Matches['cname'])" }
        else { $currentLabel = [System.IO.Path]::GetFileName(($cmd -replace '"','')) }
        continue
    }
    if ($line -match '^Doc-tests\s+(?<cname>\S+)') { $currentLabel = "doc-tests $($Matches['cname'])"; continue }

    # Emit labeled test summaries only
    if ($line -match $pattern) {
        $label = if ($currentLabel) { "[$currentLabel] " } else { "" }
        Write-Host ("{0}{1}" -f $label, $line)
        $totalPassed += [int]$Matches['passed']
        $totalFailed += [int]$Matches['failed']
        $totalIgnored += [int]$Matches['ignored']
        $totalMeasured += [int]$Matches['measured']
        $totalFiltered += [int]$Matches['filtered']
        if ($Matches['status'] -eq 'FAILED') { $anyFailedStatus = $true }
        $currentLabel = $null
    }
}

Write-Host "`n==================== Overall Test Results ===================="
Write-Host ("Passed:   {0}" -f $totalPassed)
Write-Host ("Failed:   {0}" -f $totalFailed)
Write-Host ("Ignored:  {0}" -f $totalIgnored)
Write-Host ("Measured: {0}" -f $totalMeasured)
Write-Host ("Filtered: {0}" -f $totalFiltered)
Write-Host "===============================================================`n"

if ($exitCode -ne 0 -or $totalFailed -gt 0 -or $anyFailedStatus) {
    exit 1
} else {
    exit 0
}


