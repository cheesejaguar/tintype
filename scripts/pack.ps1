# Run from the repository root after
# cargo build --release --locked --bin tintype --target <triple>.
# Usage: scripts/pack.ps1 platform output-dir [binary]
param(
    [Parameter(Mandatory = $true)][string]$Platform,
    [Parameter(Mandatory = $true)][string]$OutDir,
    [string]$Binary = 'target\release\tintype.exe'
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 3

if ($Platform -notin @('windows-x86_64', 'windows-x86')) {
    throw "Unsupported platform: $Platform"
}
if (-not (Test-Path -LiteralPath $Binary -PathType Leaf)) {
    throw "Binary not found: $Binary"
}

New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
$stage = Join-Path ([IO.Path]::GetTempPath()) ("tintype-pack-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $stage | Out-Null
try {
    Copy-Item -LiteralPath $Binary -Destination (Join-Path $stage 'tintype.exe')
    foreach ($file in 'LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE.md') {
        Copy-Item -LiteralPath $file -Destination $stage
    }
    $archive = Join-Path (Resolve-Path -LiteralPath $OutDir) "tintype-$Platform.zip"
    if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive }
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $archive -CompressionLevel Optimal
} finally {
    Remove-Item -LiteralPath $stage -Recurse -Force
}
