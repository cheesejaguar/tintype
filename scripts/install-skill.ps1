param(
    [ValidateSet('claude-code', 'codex', 'both')]
    [string]$Agent = 'both'
)

& {
    $ErrorActionPreference = 'Stop'
    $ProgressPreference = 'SilentlyContinue'

    function Install-Skill($ConfigDir, $Download) {
        $directory = Join-Path $ConfigDir 'skills/tintype'
        $dest = Join-Path $directory 'SKILL.md'
        [void][IO.Directory]::CreateDirectory($directory)
        $staged = Join-Path $directory ('.SKILL.md.' + [IO.Path]::GetRandomFileName())
        try {
            [IO.File]::Copy($Download, $staged)
            if ([IO.File]::Exists($dest)) {
                [IO.File]::Replace($staged, $dest, [NullString]::Value)
            } else {
                [IO.File]::Move($staged, $dest)
            }
            Write-Output "Installed $dest"
        } finally {
            [IO.File]::Delete($staged)
        }
    }

    $source = 'https://raw.githubusercontent.com/cheesejaguar/tintype/main/skills/tintype/SKILL.md'
    $download = [IO.Path]::GetTempFileName()
    try {
        # Windows PowerShell may use older TLS defaults.
        $protocol = [Net.ServicePointManager]::SecurityProtocol
        try {
            [Net.ServicePointManager]::SecurityProtocol = $protocol -bor [Net.SecurityProtocolType]::Tls12
            Invoke-WebRequest -Uri $source -UseBasicParsing -TimeoutSec 30 -OutFile $download
        } finally {
            [Net.ServicePointManager]::SecurityProtocol = $protocol
        }
        if ((Get-Item -LiteralPath $download).Length -eq 0) { throw 'The downloaded Tintype skill is empty.' }

        if ($Agent -ne 'codex') {
            $claudeConfig = if ($env:CLAUDE_CONFIG_DIR) { $env:CLAUDE_CONFIG_DIR } else { Join-Path $HOME '.claude' }
            Install-Skill $claudeConfig $download
        }
        if ($Agent -ne 'claude-code') {
            Install-Skill (Join-Path $HOME '.agents') $download
        }
        Write-Output 'Start a new agent session to use Tintype.'
    } finally {
        [IO.File]::Delete($download)
    }
}
