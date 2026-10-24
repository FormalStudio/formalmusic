# Builds kopuzd at the Kopuz rev Cargo.toml pins into target/kopuzd and the nd CLI
# at the NativeDesktop rev Cargo.lock pins into target/nd-cli, then packs the app
# with kopuzd beside formalmusic.exe (crates/app/nativedesktop.toml lists it).
# Installing nd-cli from git keeps NativeDesktop's .cargo/config.toml (macbook build
# dir and a sh rustc wrapper) out of the build.
#
#   scripts/package-windows.ps1 [nd package args]
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$rev = (Select-String -Path "$root\Cargo.toml" -Pattern 'Kopuz-org/kopuz", rev = "([^"]+)"' |
    Select-Object -First 1).Matches.Groups[1].Value
if (-not $rev) { throw 'no Kopuz rev in Cargo.toml' }
cargo install --locked --git https://github.com/Kopuz-org/kopuz --rev $rev kopuz-kopuzd --root "$root\target\kopuzd"
if ($LASTEXITCODE) { throw "building kopuzd $rev failed" }
$nd = (Select-String -Path "$root\Cargo.lock" -Pattern 'FormalStudio/NativeDesktop\?branch=main#([0-9a-f]+)' |
    Select-Object -First 1).Matches.Groups[1].Value
if (-not $nd) { throw 'no NativeDesktop rev in Cargo.lock' }
cargo install --locked --git https://github.com/FormalStudio/NativeDesktop --rev $nd nd-cli --root "$root\target\nd-cli"
if ($LASTEXITCODE) { throw "building nd-cli $nd failed" }
& "$root\target\nd-cli\bin\nd.exe" package windows --dir "$root\crates\app" @args
if ($LASTEXITCODE) { throw 'nd package windows failed' }
