# Exercise the Windows installer offline using the exact archives built by CI.
# Usage: scripts/test-install.ps1 artifact-dir platform
param(
    [Parameter(Mandatory = $true)][string]$Assets,
    [Parameter(Mandatory = $true)][ValidateSet('windows-x86_64', 'windows-x86')][string]$Platform
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 3

$script:assets = (Resolve-Path -LiteralPath $Assets).Path
$installer = (Resolve-Path -LiteralPath 'scripts/install.ps1').Path
$cargoVersion = (Select-String -LiteralPath 'Cargo.toml' -Pattern '^version = "([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
$script:version = "v$cargoVersion"
$script:corrupt = $false

# Functions take precedence over cmdlets, so the installer reaches these mocks.
function Invoke-RestMethod {
    param([string]$Uri, [switch]$UseBasicParsing)
    if ($Uri -ne 'https://api.github.com/repos/cheesejaguar/tintype/releases/latest') { throw "unexpected API request: $Uri" }
    [pscustomobject]@{ tag_name = $script:version }
}
function Invoke-WebRequest {
    param([string]$Uri, [string]$OutFile, [switch]$UseBasicParsing)
    $prefix = "https://github.com/cheesejaguar/tintype/releases/download/$script:version/"
    if (-not $Uri.StartsWith($prefix)) { throw "unexpected download: $Uri" }
    $file = $Uri.Substring($prefix.Length)
    Copy-Item -LiteralPath (Join-Path $script:assets $file) -Destination $OutFile
    if ($script:corrupt -and $file -eq 'tintype-models.tar.gz') {
        Add-Content -LiteralPath $OutFile -Value 'bad bytes'
    }
}

function Assert([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw "Assertion failed: $Message" }
}

$stage = Join-Path ([IO.Path]::GetTempPath()) ("tintype-install-test-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $stage | Out-Null
$saved = @{}
foreach ($name in 'TINTYPE_MODEL_DIR', 'TINTYPE_VERSION', 'TINTYPE_INSTALL_DIR', 'TINTYPE_PLATFORM', 'TINTYPE_NO_MODELS', 'TINTYPE_NO_MODIFY_PATH') {
    $saved[$name] = [Environment]::GetEnvironmentVariable($name, 'Process')
    # PowerShell passes $null to .NET as an empty string, which leaves the
    # variable defined, so remove it through the Env: drive instead.
    Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
}
try {
    $dest = Join-Path $stage 'install dir with spaces'
    $env:TINTYPE_INSTALL_DIR = $dest
    $env:TINTYPE_PLATFORM = $Platform
    $env:TINTYPE_NO_MODIFY_PATH = '1'
    $exe = Join-Path $dest 'tintype.exe'

    # The documented one-liner: irm ... | iex
    Invoke-Expression (Get-Content -Raw -LiteralPath $installer)
    Assert (Test-Path -LiteralPath $exe) 'tintype.exe was installed'
    $result = @(& $exe demo --device cpu --json)
    Assert ($LASTEXITCODE -eq 0) 'demo succeeded'
    Assert ($result.Count -eq 1) 'demo printed one JSON line'
    $detection = $result[0] | ConvertFrom-Json
    Assert ($detection.verdict -in @('ai-generated', 'likely-real')) "demo verdict was $($detection.verdict)"
    $marker = Get-Content -Raw -LiteralPath (Join-Path $dest '.tintype-install')
    Assert ($marker.Trim() -eq "$script:version $Platform with-models") "install marker was $marker"
    $arch = if ($Platform -eq 'windows-x86') { 0x14c } else { 0x8664 }
    $bytes = [IO.File]::ReadAllBytes($exe)
    $pe = [BitConverter]::ToInt32($bytes, 0x3c)
    Assert ([BitConverter]::ToUInt16($bytes, $pe + 4) -eq $arch) "tintype.exe targets $Platform"

    # A corrupt models archive is rejected and leaves the existing install alone.
    $script:corrupt = $true
    $failed = $false
    try {
        Invoke-Expression (Get-Content -Raw -LiteralPath $installer)
    } catch {
        $failed = $true
        Assert ($_.Exception.Message -match 'checksum mismatch') "corrupt install failed with: $($_.Exception.Message)"
    }
    $script:corrupt = $false
    Assert $failed 'corrupt model archive was rejected'
    Assert ((Get-Content -Raw -LiteralPath (Join-Path $dest '.tintype-install')) -eq $marker) 'previous install kept'
    Assert (-not (Get-ChildItem -LiteralPath $stage -Filter '*.install-*')) 'staging directory removed'
    Assert (-not (Test-Path -LiteralPath "$dest.lock")) 'lock removed'

    # Code-only install with a pinned version replaces the previous one.
    $env:TINTYPE_VERSION = $script:version
    & ([scriptblock]::Create((Get-Content -Raw -LiteralPath $installer))) -NoModels
    & $exe --version
    Assert ($LASTEXITCODE -eq 0) 'code-only executable runs'
    Assert (-not (Test-Path -LiteralPath (Join-Path $dest 'models'))) 'code-only install has no models'
    Assert (-not (Get-ChildItem -LiteralPath $stage -Filter '*.previous-*')) 'previous version removed'

    # The installer refuses to replace a directory it did not create.
    $foreign = Join-Path $stage 'foreign'
    New-Item -ItemType Directory -Path $foreign | Out-Null
    $env:TINTYPE_INSTALL_DIR = $foreign
    $failed = $false
    try {
        Invoke-Expression (Get-Content -Raw -LiteralPath $installer)
    } catch {
        $failed = $true
    }
    Assert $failed 'foreign directory was refused'

    # User PATH registration, on a throwaway install directory.
    $env:TINTYPE_INSTALL_DIR = Join-Path $stage 'path test'
    Remove-Item Env:TINTYPE_NO_MODIFY_PATH
    $before = (Get-Item -LiteralPath 'HKCU:\Environment').GetValue('Path', '', 'DoNotExpandEnvironmentNames')
    try {
        & ([scriptblock]::Create((Get-Content -Raw -LiteralPath $installer))) -NoModels
        $after = (Get-Item -LiteralPath 'HKCU:\Environment').GetValue('Path', '', 'DoNotExpandEnvironmentNames')
        Assert (($after -split ';') -contains $env:TINTYPE_INSTALL_DIR) 'install directory added to the user PATH'
        Assert ((Get-Command tintype).Source -eq (Join-Path $env:TINTYPE_INSTALL_DIR 'tintype.exe')) 'tintype resolves on PATH'
    } finally {
        if ($before) {
            Set-ItemProperty -LiteralPath 'HKCU:\Environment' -Name Path -Value $before -Type ExpandString
        } else {
            Remove-ItemProperty -LiteralPath 'HKCU:\Environment' -Name Path -ErrorAction SilentlyContinue
        }
    }
    Write-Output "Windows installer test passed for $Platform"
} finally {
    foreach ($name in $saved.Keys) {
        if ($null -eq $saved[$name]) {
            Remove-Item -LiteralPath "Env:$name" -ErrorAction SilentlyContinue
        } else {
            Set-Item -LiteralPath "Env:$name" -Value $saved[$name]
        }
    }
    Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction SilentlyContinue
}
