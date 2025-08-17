Param()
$ErrorActionPreference = 'Stop'

Push-Location "$PSScriptRoot\..\Pigeon"
try {
    cargo fmt --all
}
finally {
    Pop-Location
}


