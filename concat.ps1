<#
.SYNOPSIS
    PowerShell equivalent of the 'concat' bash script.

.DESCRIPTION
    Concatenates tracked source-code files with metadata:
    file path, git status, line count, mtime, last commit date, and content.
    At the end, appends a list of ignored non-source files (paths only).

.PARAMETER Output
    Path to the output file. Defaults to concat-report.txt.

.PARAMETER Stdout
    Write to stdout instead of a file.

.PARAMETER IncludeGenerated
    Include generated files (reserved; currently has no effect).

.PARAMETER ExcludeFrom
    Path to an ignore-rules file. Patterns are repo-relative.

.PARAMETER RustOnly
    Include tracked *.rs files only. Default output: concat.rs.txt.

.PARAMETER MdOnly
    Include tracked *.md files only. Default output: concat.md.txt.

.EXAMPLE
    .\concat.ps1
    .\concat.ps1 -RustOnly
    .\concat.ps1 -MdOnly
    .\concat.ps1 -Output out.txt -ExcludeFrom .concatignore-rs
    .\concat.ps1 -Stdout | clip
#>

[CmdletBinding()]
param(
    [Alias('o')]
    [string]$Output = '',

    [switch]$Stdout,
    [switch]$IncludeGenerated,
    [string]$ExcludeFrom = '',
    [Alias('rs')]
    [switch]$RustOnly,
    [switch]$MdOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RootDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $RootDir

# ── Validate mode combination ──────────────────────────────────────────────────
if ($RustOnly -and $MdOnly) {
    Write-Error '[concat] choose either -RustOnly or -MdOnly'
    exit 2
}

# ── Determine output file ──────────────────────────────────────────────────────
$OutputFileExplicit = $Output -ne ''
if (-not $OutputFileExplicit) {
    if ($RustOnly) { $Output = 'concat.rs.txt' }
    elseif ($MdOnly) { $Output = 'concat.md.txt' }
    else { $Output = 'concat-report.txt' }
}

# ── Verify we're inside a git repo ────────────────────────────────────────────
$null = git rev-parse --is-inside-work-tree 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Error '[concat] this script must run inside a git repository'
    exit 1
}

# ── Exclude pattern loading ────────────────────────────────────────────────────
function Resolve-ExcludeFile([string]$Candidate) {
    if (-not $Candidate) { return $null }
    if (Test-Path $Candidate) { return (Resolve-Path $Candidate).Path }
    $abs = Join-Path $RootDir $Candidate
    if (Test-Path $abs) { return (Resolve-Path $abs).Path }
    return $null
}

$ExcludePatterns = @()
$ExcludeFileSource = 'none'
$ExcludedByRules = 0

function Import-ExcludePatterns([string]$FilePath) {
    $script:ExcludePatterns = Get-Content $FilePath |
        ForEach-Object {
            $line = $_ -replace '#.*', ''  # strip comments
            $line = $line.Trim()
            if ($line) { $line }
        }
}

function Test-PathExcluded([string]$RepoPath) {
    foreach ($pattern in $script:ExcludePatterns) {
        $norm = $pattern.TrimStart('/')

        if ($norm.EndsWith('/')) {
            # Directory prefix match
            if ($RepoPath -like "$norm*") { return $true }
        }
        else {
            # Exact or sub-path glob match
            if ($RepoPath -like $norm -or $RepoPath -like "$norm/*") { return $true }
        }
    }
    return $false
}

# Resolve and load exclude file
if ($ExcludeFrom) {
    $resolved = Resolve-ExcludeFile $ExcludeFrom
    if (-not $resolved) {
        Write-Error "[concat] exclude file not found: $ExcludeFrom"
        exit 2
    }
    $ExcludeFrom = $resolved
    $ExcludeFileSource = $ExcludeFrom
}
elseif ($RustOnly) {
    $resolved = Resolve-ExcludeFile '.concatignore-rs'
    if ($resolved) { $ExcludeFrom = $resolved; $ExcludeFileSource = $resolved }
}
elseif (-not $MdOnly) {
    $resolved = Resolve-ExcludeFile '.concatignore'
    if ($resolved) { $ExcludeFrom = $resolved; $ExcludeFileSource = $resolved }
}

if ($ExcludeFrom) {
    Import-ExcludePatterns $ExcludeFrom
}

# ── Collect files via git ls-files ────────────────────────────────────────────
$SkippedGenerated = 0

$AllFiles = if ($RustOnly) {
    git ls-files '*.rs' | Sort-Object
}
elseif ($MdOnly) {
    git ls-files '*.md' | Sort-Object | Where-Object { Test-Path $_ }
}
else {
    git ls-files '*.rs' '*.rhai' '*.sh' '*.py' 'mods/**/*.yml' 'mods/**/*.yaml' | Sort-Object
}

$Files = [System.Collections.Generic.List[string]]::new()
foreach ($f in $AllFiles) {
    if (-not (Test-Path $f)) { continue }
    if (Test-PathExcluded $f) { $ExcludedByRules++; continue }
    $Files.Add($f)
}

# ── Ignored non-source files (yml/yaml/blend not already included) ─────────────
$IncludedSet = [System.Collections.Generic.HashSet[string]]::new($Files)
$IgnoredNonSource = git ls-files '*.yml' '*.yaml' '*.blend' '*.blend1' |
    Sort-Object |
    Where-Object { (Test-Path $_) -and (-not $IncludedSet.Contains($_)) }

if ($Files.Count -eq 0) {
    $msg = if ($RustOnly) { 'no tracked Rust files found after filtering' }
           elseif ($MdOnly) { 'no tracked Markdown files found after filtering' }
           else { 'no tracked source files found (.rs/.rhai/.sh/.py + mods/**/*.yml|yaml)' }
    Write-Error "[concat] $msg"
    exit 1
}

# ── Build report ──────────────────────────────────────────────────────────────
function Build-Report {
    $sb = [System.Text.StringBuilder]::new()

    $generatedAt  = (Get-Date).ToString('yyyy-MM-dd HH:mm:ss zzz')
    $totalLines   = ($Files | ForEach-Object { (Get-Content $_ -Raw -ErrorAction SilentlyContinue) -split "`n" | Measure-Object | Select-Object -ExpandProperty Count }) |
                    Measure-Object -Sum | Select-Object -ExpandProperty Sum

    $null = $sb.AppendLine('# concat report')
    $null = $sb.AppendLine("generated_at: $generatedAt")
    $null = $sb.AppendLine("repo: $RootDir")
    $null = $sb.AppendLine("files: $($Files.Count)")
    $null = $sb.AppendLine("lines_total: $totalLines")
    $null = $sb.AppendLine("include_generated: $(if ($IncludeGenerated) { 1 } else { 0 })")
    $null = $sb.AppendLine("rust_only: $(if ($RustOnly) { 1 } else { 0 })")
    $null = $sb.AppendLine("md_only: $(if ($MdOnly) { 1 } else { 0 })")
    $null = $sb.AppendLine("exclude_file: $ExcludeFileSource")
    $null = $sb.AppendLine("exclude_rules: $($ExcludePatterns.Count)")
    $null = $sb.AppendLine("excluded_by_rules: $ExcludedByRules")
    $null = $sb.AppendLine("skipped_generated_files: $SkippedGenerated")
    $null = $sb.AppendLine("ignored_non_source_files: $(@($IgnoredNonSource).Count)")
    $null = $sb.AppendLine('')

    foreach ($f in $Files) {
        $statusRaw = git status --porcelain -- $f 2>$null | Select-Object -First 1
        $status    = if ($statusRaw) { $statusRaw.Substring(0, [Math]::Min(2, $statusRaw.Length)) } else { 'clean' }

        $content    = Get-Content $f -Raw -ErrorAction SilentlyContinue
        if ($null -eq $content) { $content = '' }
        $lineCount  = ($content -split "`n").Count
        $mtime      = (Get-Item $f).LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss zzz')
        $lastCommit = git log -1 --date=iso-strict --format='%cd' -- $f 2>$null
        if (-not $lastCommit) { $lastCommit = 'n/a' }

        $null = $sb.AppendLine('=' * 80)
        $null = $sb.AppendLine("file: $f")
        $null = $sb.AppendLine("status: $status")
        $null = $sb.AppendLine("lines: $lineCount")
        $null = $sb.AppendLine("mtime: $mtime")
        $null = $sb.AppendLine("last_commit_date: $lastCommit")
        $null = $sb.AppendLine('--- BEGIN CONTENT ---')
        $null = $sb.Append($content)
        # Ensure trailing newline before END marker
        if ($content.Length -gt 0 -and -not $content.EndsWith("`n")) {
            $null = $sb.AppendLine('')
        }
        $null = $sb.AppendLine('--- END CONTENT ---')
        $null = $sb.AppendLine('')
    }

    $null = $sb.AppendLine('=' * 80)
    $null = $sb.AppendLine('ignored_non_source_files:')
    if (@($IgnoredNonSource).Count -eq 0) {
        $null = $sb.AppendLine('(none)')
    }
    else {
        foreach ($ip in $IgnoredNonSource) {
            $null = $sb.AppendLine($ip)
        }
    }

    return $sb.ToString()
}

$report = Build-Report

if ($Stdout) {
    Write-Output $report
}
else {
    [System.IO.File]::WriteAllText((Join-Path $RootDir $Output), $report, [System.Text.Encoding]::UTF8)
    Write-Host "[concat] wrote report: $Output"
}
