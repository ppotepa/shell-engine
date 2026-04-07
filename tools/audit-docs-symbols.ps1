# audit-docs-symbols.ps1 — Scan markdown docs for legacy/removed symbols.
#
# Usage:
#   .\tools\audit-docs-symbols.ps1
#   .\tools\audit-docs-symbols.ps1 "world.ship_set_turn" "TopDownShipController"
#   .\tools\audit-docs-symbols.ps1 --root docs "attach_ship_controller"
#
# Exit codes:
#   0 - no matches
#   1 - matches found
#   2 - invalid usage or missing dependency
param (
    [string]   $root    = "",
    [switch]   $h,
    [switch]   $help,
    [Parameter(ValueFromRemainingArguments)]
    [string[]] $patterns = @()
)

$ScriptDir  = $PSScriptRoot
$RepoRoot   = Split-Path $ScriptDir -Parent
$SearchRoot = if ($root) { $root } else { $RepoRoot }

$DefaultPatterns = @(
    "TopDownShipController"
    "attach_ship_controller"
    "world.ship_set_turn"
    "world.ship_set_thrust"
    "world.ship_heading"
    "world.ship_heading_vector"
    "world.ship_velocity"
)

function Show-Help {
    Write-Host @"
Scan markdown docs for legacy/removed symbols.

Usage:
  audit-docs-symbols.ps1 [--root <path>] [pattern...]

Options:
  --root <path>   Limit scan to this directory (default: repo root)
  -h, --help      Show this help

Behavior:
  - If patterns are provided, scans for those patterns.
  - If no patterns, scans for built-in legacy API patterns.
"@
}

if ($h -or $help) { Show-Help; exit 0 }

if (-not (Get-Command "rg" -ErrorAction SilentlyContinue)) {
    Write-Error "ERROR: ripgrep (rg) is required but not installed."
    exit 2
}

if (-not (Test-Path $SearchRoot)) {
    Write-Error "ERROR: search root does not exist: $SearchRoot"
    exit 2
}

$activePats = if ($patterns.Count -gt 0) { $patterns } else { $DefaultPatterns }

Write-Host "Scanning markdown files in: $SearchRoot"
Write-Host "Patterns: $($activePats -join ', ')"

$found = $false
foreach ($p in $activePats) {
    $results = rg -nF --glob "**/*.md" --glob "!target/**" --glob "!.git/**" $p $SearchRoot 2>&1
    if ($LASTEXITCODE -eq 0 -and $results) {
        $results | Write-Host
        $found = $true
    }
}

if ($found) {
    Write-Host ""
    Write-Host "Legacy symbol matches found."
    exit 1
}

Write-Host "No matches found."
exit 0
