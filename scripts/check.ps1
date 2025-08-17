Param()
$ErrorActionPreference = 'Stop'

Push-Location "$PSScriptRoot\..\Pigeon"
try {
    Write-Host "[fmt] cargo fmt --check"
    cargo fmt --all -- --check

    Write-Host "[clippy] cargo clippy -D warnings"
    cargo clippy --all-targets -- -D warnings

    Write-Host "[test] cargo test --verbose"
    cargo test --verbose
}
finally {
    Pop-Location
}


