# Tintype installer for Windows (x86-64 and 32-bit x86).
#
#   irm https://cheesejaguar.github.io/tintype/install.ps1 | iex
#
# Executable only, without the model weights:
#
#   & ([scriptblock]::Create((irm https://cheesejaguar.github.io/tintype/install.ps1))) -NoModels
#
# TINTYPE_VERSION=v0.1.2 pins a release. TINTYPE_INSTALL_DIR sets the
# installation directory (default %LOCALAPPDATA%\Programs\tintype).
# TINTYPE_NO_MODELS=1 is the same as -NoModels. TINTYPE_NO_MODIFY_PATH=1
# leaves the user PATH alone. TINTYPE_PLATFORM=windows-x86 installs the
# 32-bit build on 64-bit Windows.
param(
    [switch]$NoModels,
    [switch]$Help
)

function Install-Tintype {
    param([bool]$NoModels, [bool]$Help)

    # Scoped to this function, so `irm | iex` leaves the caller's session alone.
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'
    Set-StrictMode -Version 3

    $repo = 'https://github.com/cheesejaguar/tintype'
    $api = 'https://api.github.com/repos/cheesejaguar/tintype/releases/latest'

    if ($Help) {
        Write-Output 'Usage: install.ps1 [-NoModels]'
        Write-Output 'TINTYPE_VERSION=v0.1.2 pins a release.'
        Write-Output 'TINTYPE_INSTALL_DIR sets the installation directory (default %LOCALAPPDATA%\Programs\tintype).'
        Write-Output 'TINTYPE_NO_MODIFY_PATH=1 leaves the user PATH unchanged.'
        Write-Output 'TINTYPE_PLATFORM=windows-x86 installs the 32-bit build on 64-bit Windows.'
        return
    }
    if ($env:TINTYPE_NO_MODELS -eq '1') { $NoModels = $true }
    $models = if ($NoModels) { 'code-only' } else { 'with-models' }

    # Older Windows PowerShell builds may not offer TLS 1.2 by default.
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12
    } catch { }

    $platform = $env:TINTYPE_PLATFORM
    if (-not $platform) {
        # A 32-bit PowerShell on 64-bit Windows reports the real OS in PROCESSOR_ARCHITEW6432.
        $arch = $env:PROCESSOR_ARCHITEW6432
        if (-not $arch) { $arch = $env:PROCESSOR_ARCHITECTURE }
        switch ($arch) {
            'AMD64' { $platform = 'windows-x86_64' }
            'x86' { $platform = 'windows-x86' }
            default { throw "Tintype: supported Windows targets are x86-64 and x86, not $arch" }
        }
    }
    if ($platform -notin @('windows-x86_64', 'windows-x86')) {
        throw "Tintype: TINTYPE_PLATFORM must be windows-x86_64 or windows-x86"
    }

    $version = $env:TINTYPE_VERSION
    if (-not $version) {
        try {
            $version = (Invoke-RestMethod -Uri $api -UseBasicParsing).tag_name
        } catch {
            throw "Tintype: could not resolve the latest release ($($_.Exception.Message))"
        }
        if (-not $version) { throw "Tintype: no stable release is available at $repo/releases" }
    }
    if ($version -notmatch '^v[0-9][a-zA-Z0-9._-]*$') {
        throw 'Tintype: TINTYPE_VERSION must be a release tag such as v0.1.2'
    }
    $source = "$repo/releases/download/$version"

    $dest = $env:TINTYPE_INSTALL_DIR
    if (-not $dest) {
        if (-not $env:LOCALAPPDATA) { throw 'Tintype: LOCALAPPDATA is not set; set TINTYPE_INSTALL_DIR' }
        $dest = Join-Path $env:LOCALAPPDATA 'Programs\tintype'
    }
    if ($dest -notmatch '^([a-zA-Z]:[\\/]|\\\\)') {
        throw 'Tintype: TINTYPE_INSTALL_DIR must be an absolute path'
    }
    $dest = [IO.Path]::GetFullPath($dest).TrimEnd('\')
    $marker = '.tintype-install'
    if ((Test-Path -LiteralPath $dest) -and -not (Test-Path -LiteralPath (Join-Path $dest $marker))) {
        throw "Tintype: $dest already exists and was not created by this installer. Choose another TINTYPE_INSTALL_DIR."
    }

    $parent = Split-Path -Parent $dest
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
    $suffix = [Guid]::NewGuid().ToString('N').Substring(0, 8)
    $lock = "$dest.lock"
    try {
        New-Item -ItemType Directory -Path $lock -ErrorAction Stop | Out-Null
    } catch {
        throw "Tintype: another installation may be running ($lock)"
    }
    $stage = "$dest.install-$suffix"
    $previous = "$dest.previous-$suffix"
    try {
        New-Item -ItemType Directory -Path $stage | Out-Null
        $payload = Join-Path $stage 'payload'
        New-Item -ItemType Directory -Path $payload | Out-Null

        $sums = Join-Path $stage 'SHA256SUMS'
        Invoke-WebRequest -Uri "$source/SHA256SUMS" -OutFile $sums -UseBasicParsing
        $checksums = @{}
        foreach ($line in Get-Content -LiteralPath $sums) {
            if ($line -match '^([0-9a-fA-F]{64}) [ *]?(\S+)$') { $checksums[$Matches[2]] = $Matches[1] }
        }
        $verify = {
            param([string]$archive)
            if (-not $checksums.ContainsKey($archive)) { throw "Tintype: missing checksum for $archive" }
            $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath (Join-Path $stage $archive)).Hash
            if ($actual -ne $checksums[$archive]) { throw "Tintype: checksum mismatch for $archive" }
        }

        $bundle = "tintype-$platform.zip"
        Invoke-WebRequest -Uri "$source/$bundle" -OutFile (Join-Path $stage $bundle) -UseBasicParsing
        & $verify $bundle
        Expand-Archive -LiteralPath (Join-Path $stage $bundle) -DestinationPath $payload
        $exe = Join-Path $payload 'tintype.exe'

        if (-not $NoModels) {
            $bundle = 'tintype-models.tar.gz'
            Invoke-WebRequest -Uri "$source/$bundle" -OutFile (Join-Path $stage $bundle) -UseBasicParsing
            & $verify $bundle
            # Windows 10 1803 and later ship bsdtar. Use it by path: Git's GNU tar
            # reads C:\ paths as remote hosts.
            $tar = Join-Path $env:SystemRoot 'System32\tar.exe'
            if (-not (Test-Path -LiteralPath $tar)) {
                throw 'Tintype: extracting the models needs tar.exe (Windows 10 1803 or later). Use -NoModels to install the executable alone.'
            }
            & $tar -xzf (Join-Path $stage $bundle) -C $payload
            if ($LASTEXITCODE -ne 0) { throw "Tintype: could not extract $bundle" }
            & $exe verify-models --root $payload | Out-Null
            if ($LASTEXITCODE -ne 0) { throw 'Tintype: the downloaded models failed verification' }
        }
        & $exe --version
        if ($LASTEXITCODE -ne 0) { throw 'Tintype: the downloaded executable did not run' }
        Set-Content -LiteralPath (Join-Path $payload $marker) -Value "$version $platform $models" -Encoding ASCII

        # Put the new version in place, keeping the old one until the swap succeeds.
        if (Test-Path -LiteralPath $dest) {
            try {
                Move-Item -LiteralPath $dest -Destination $previous
            } catch {
                throw "Tintype: could not replace $dest. Close any running tintype processes and try again."
            }
        }
        try {
            Move-Item -LiteralPath $payload -Destination $dest
        } catch {
            if (Test-Path -LiteralPath $previous) { Move-Item -LiteralPath $previous -Destination $dest }
            throw
        }
        if (Test-Path -LiteralPath $previous) {
            Remove-Item -LiteralPath $previous -Recurse -Force -ErrorAction SilentlyContinue
        }
    } finally {
        Remove-Item -LiteralPath $stage -Recurse -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $lock -Force -ErrorAction SilentlyContinue
    }

    Write-Output ''
    Write-Output "Installed $(Join-Path $dest 'tintype.exe')"
    $onPath = ($env:Path -split ';') -contains $dest
    if ($env:TINTYPE_NO_MODIFY_PATH -eq '1') {
        if (-not $onPath) { Write-Output "Add $dest to your PATH or run the executable at that path." }
    } else {
        # Edit the raw registry value so %VARIABLE% entries in the user PATH stay unexpanded.
        $key = Get-Item -LiteralPath 'HKCU:\Environment'
        $userPath = [string]$key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
        if (($userPath -split ';') -notcontains $dest) {
            $newPath = if ($userPath) { "$($userPath.TrimEnd(';'));$dest" } else { $dest }
            Set-ItemProperty -LiteralPath 'HKCU:\Environment' -Name Path -Value $newPath -Type ExpandString
            # Setting any user variable through .NET tells open programs to reload the environment.
            [Environment]::SetEnvironmentVariable('TINTYPE_INSTALLER_REFRESH', '1', 'User')
            [Environment]::SetEnvironmentVariable('TINTYPE_INSTALLER_REFRESH', $null, 'User')
            Write-Output "Added $dest to your user PATH. New terminals will find tintype."
        }
        if (-not $onPath) { $env:Path = "$env:Path;$dest" }
    }
    if (-not $NoModels) {
        Write-Output 'Run tintype demo or tintype detect photo.jpg.'
        Write-Output 'Bundled model weights are CC BY-NC-SA 4.0.'
    }
}

Install-Tintype -NoModels $NoModels.IsPresent -Help $Help.IsPresent
