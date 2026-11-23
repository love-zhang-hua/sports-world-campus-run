param([string]$Cargo = 'cargo', [string]$MakeNSIS = 'makensis')
$ErrorActionPreference = 'Stop'
$projectRoot = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $projectRoot
try {
    & $Cargo build --release --locked --bin sportssimulation
    if ($LASTEXITCODE -ne 0) { throw 'Rust build failed' }
    $metadata = (& $Cargo metadata --format-version 1 --no-deps | ConvertFrom-Json)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot locate Cargo output' }
    New-Item -ItemType Directory -Path 'dist' -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $metadata.target_directory 'release/sportssimulation.exe') -Destination 'dist/SportsRoute.exe'
    & $MakeNSIS /NOCD /INPUTCHARSET UTF8 /OUTPUTCHARSET UTF8 'scripts/windows/installer.nsi'
    if ($LASTEXITCODE -ne 0) { throw 'Installer build failed' }
    Get-FileHash -LiteralPath 'dist/SportsRoute-1.7.1-Setup.exe' -Algorithm SHA256
} finally {
    Pop-Location
}
