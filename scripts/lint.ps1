Param()
$ErrorActionPreference = 'Stop'

Push-Location "$PSScriptRoot\..\Pigeon"
try {
    cargo clippy --all-targets -- -D warnings
}
finally {
    Pop-Location
}


