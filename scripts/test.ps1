Param()
$ErrorActionPreference = 'Stop'

Push-Location "$PSScriptRoot\..\Pigeon"
try {
    cargo test --verbose
}
finally {
    Pop-Location
}


