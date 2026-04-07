# analyze.ps1 — Shell Quest project analysis tool
#
# Produces a complete dependency + reference graph and reports dead code,
# orphan crates, unreferenced mod assets, and scene navigation maps.
#
# Usage:
#   .\analyze.ps1                         # full analysis (no slow cargo check)
#   .\analyze.ps1 --dead-code             # include cargo dead-code pass (slow)
#   .\analyze.ps1 --mod mods/asteroids    # scope asset analysis to one mod
#   .\analyze.ps1 --dot                   # emit crate graph as .dot / scene .dot
#   .\analyze.ps1 --out reports/analysis  # write report to file (default: stdout)
#
# Outputs:
#   - Console report with sections
#   - reports/crate-graph.dot   (if --dot)
#   - reports/scene-graph.dot   (if --dot)
#   - reports/analysis.txt      (if --out)
#
param (
    [switch]   $DeadCode,
    [Alias("dead-code")]
    [switch]   $DeadCodeAlias,
    [switch]   $Dot,
    [string]   $Mod      = "",
    [string]   $Out      = "",
    [switch]   $Help
)

# Support --dead-code alias
if ($DeadCodeAlias) { $DeadCode = $true }

$ErrorActionPreference = "Stop"
$RepoRoot = $PSScriptRoot

# ─────────────────────────────────────────────────────────────────────────────
# Helpers
# ─────────────────────────────────────────────────────────────────────────────

$Lines = [System.Collections.Generic.List[string]]::new()

function Out-Line {
    param([string]$text = "")
    $Lines.Add($text)
    Write-Host $text
}

function Out-Header {
    param([string]$title)
    $bar = "═" * 72
    Out-Line ""
    Out-Line "╔$bar╗"
    Out-Line ("║  {0,-70}║" -f $title)
    Out-Line "╚$bar╝"
}

function Out-Section {
    param([string]$title)
    Out-Line ""
    Out-Line "── $title $("─" * ([Math]::Max(0, 68 - $title.Length)))"
}

if ($Help) {
    Write-Host @"
analyze.ps1 — Shell Quest project analysis

Options:
  --dead-code     Include cargo dead-code pass (slow, ~1-2 min)
  --dot           Emit Graphviz .dot files to reports/
  --mod <path>    Scope asset analysis to a single mod directory
  --out <path>    Write report to file (default: stdout only)
  --help          Show this help

Outputs:
  - Workspace crate dependency graph (text + optional .dot)
  - Orphan / unreferenced crates
  - Scene navigation graph (text + optional .dot)
  - Dead mod assets (scripts, objects, emitters, layers)
  - Dead code warnings from rustc (if --dead-code)
"@
    exit 0
}

# ─────────────────────────────────────────────────────────────────────────────
# 1. Workspace crate graph via cargo metadata
# ─────────────────────────────────────────────────────────────────────────────

Out-Header "SHELL QUEST PROJECT ANALYSIS"
Out-Line   "  Repo   : $RepoRoot"
Out-Line   "  Date   : $(Get-Date -Format 'yyyy-MM-dd HH:mm')"
if ($Mod)      { Out-Line "  Mod    : $Mod" }
if ($DeadCode) { Out-Line "  Mode   : full (dead-code enabled)" }

Out-Section "Loading workspace metadata..."

Push-Location $RepoRoot
$rawMeta = cargo metadata --format-version 1 --no-deps 2>&1
Pop-Location

if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo metadata failed. Is Rust/Cargo installed?"
    exit 1
}

$meta = $rawMeta | ConvertFrom-Json
$allPkgs   = $meta.packages
$crateNames = $allPkgs | Select-Object -ExpandProperty name | Sort-Object

Out-Line "  Workspace crates: $($crateNames.Count)"

# ─────────────────────────────────────────────────────────────────────────────
# 2. Build dependency map (workspace-internal only)
# ─────────────────────────────────────────────────────────────────────────────

$crateSet  = [System.Collections.Generic.HashSet[string]]::new()
foreach ($n in $crateNames) { [void]$crateSet.Add($n) }

# $deps[$crate] = list of workspace crates it depends on
$deps     = @{}
# $rdeps[$crate] = list of workspace crates that depend on it (reverse)
$rdeps    = @{}
foreach ($n in $crateNames) { $deps[$n] = @(); $rdeps[$n] = @() }

foreach ($pkg in $allPkgs) {
    foreach ($dep in $pkg.dependencies) {
        if ($crateSet.Contains($dep.name)) {
            $deps[$pkg.name]  += $dep.name
            $rdeps[$dep.name] += $pkg.name
        }
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 3. Crate dependency report
# ─────────────────────────────────────────────────────────────────────────────

Out-Section "Crate Dependency Graph"

$entryPoints = @("app", "editor", "launcher", "schema-gen", "devtool",
                 "sound-server", "ttf-rasterizer", "rust-os")

foreach ($ep in ($entryPoints | Sort-Object)) {
    if (-not $crateSet.Contains($ep)) { continue }
    $pkg = $allPkgs | Where-Object { $_.name -eq $ep }
    Out-Line "  [entry] $ep"
    foreach ($d in ($deps[$ep] | Sort-Object)) {
        Out-Line "    └─ $d"
    }
}

Out-Section "Crate Dependency Counts"
Out-Line ("  {0,-35} {1,6}  {2,6}" -f "Crate", "Deps", "Used-by")
Out-Line ("  {0,-35} {1,6}  {2,6}" -f "─────────────────────────────────", "──────", "──────")
foreach ($n in $crateNames) {
    $d = $deps[$n].Count
    $r = $rdeps[$n].Count
    $marker = if ($r -eq 0 -and $entryPoints -notcontains $n) { "  ← UNREFERENCED" } else { "" }
    Out-Line ("  {0,-35} {1,6}  {2,6}{3}" -f $n, $d, $r, $marker)
}

# ─────────────────────────────────────────────────────────────────────────────
# 4. Orphan crate detection
# ─────────────────────────────────────────────────────────────────────────────

Out-Section "Orphan / Unreferenced Crates"

$orphans = $crateNames | Where-Object {
    $rdeps[$_].Count -eq 0 -and $entryPoints -notcontains $_
}

if ($orphans) {
    Out-Line "  WARNING: The following workspace crates have no dependents and are"
    Out-Line "  not declared as known entry points. They may be dead weight:"
    foreach ($o in $orphans) {
        Out-Line "    ✗  $o"
    }
} else {
    Out-Line "  ✓  All workspace crates are either entry points or referenced."
}

# ─────────────────────────────────────────────────────────────────────────────
# 5. DOT graph output (crates)
# ─────────────────────────────────────────────────────────────────────────────

if ($Dot) {
    $dotDir = Join-Path $RepoRoot "reports"
    if (-not (Test-Path $dotDir)) { New-Item -ItemType Directory $dotDir | Out-Null }
    $dotPath = Join-Path $dotDir "crate-graph.dot"

    $dotLines = @("digraph crate_graph {")
    $dotLines += '  rankdir=LR;'
    $dotLines += '  node [shape=box fontname="Consolas" fontsize=10];'
    foreach ($ep in $entryPoints) {
        if ($crateSet.Contains($ep)) {
            $dotLines += "  `"$ep`" [style=filled fillcolor=lightblue];"
        }
    }
    foreach ($o in $orphans) {
        $dotLines += "  `"$o`" [style=filled fillcolor=lightyellow];"
    }
    foreach ($n in $crateNames) {
        foreach ($d in $deps[$n]) {
            $dotLines += "  `"$n`" -> `"$d`";"
        }
    }
    $dotLines += "}"
    $dotLines | Set-Content $dotPath
    Out-Line ""
    Out-Line "  Crate graph written: $dotPath"
    Out-Line "  Render with: dot -Tsvg reports/crate-graph.dot -o reports/crate-graph.svg"
}

# ─────────────────────────────────────────────────────────────────────────────
# 6. Mod asset analysis
# ─────────────────────────────────────────────────────────────────────────────

$modsRoot = Join-Path $RepoRoot "mods"
$modDirs  = if ($Mod) {
    @(Resolve-Path (Join-Path $RepoRoot $Mod))
} else {
    Get-ChildItem $modsRoot -Directory | Select-Object -ExpandProperty FullName
}

foreach ($modDir in $modDirs) {
    $modName = Split-Path $modDir -Leaf
    Out-Header "MOD ANALYSIS: $modName"

    # ── Collect all files by type ────────────────────────────────────────────
    $allRhai    = Get-ChildItem $modDir -Filter "*.rhai" -Recurse -ErrorAction SilentlyContinue |
                  Where-Object { $_.FullName -notmatch "\\target\\" }
    $allYml     = Get-ChildItem $modDir -Filter "*.yml"  -Recurse -ErrorAction SilentlyContinue |
                  Where-Object { $_.FullName -notmatch "\\target\\" }
    $allYaml    = Get-ChildItem $modDir -Filter "*.yaml" -Recurse -ErrorAction SilentlyContinue |
                  Where-Object { $_.FullName -notmatch "\\target\\" }
    $allObjects = Get-ChildItem (Join-Path $modDir "objects") -Filter "*.yml" -ErrorAction SilentlyContinue
    $allImages  = Get-ChildItem $modDir -Recurse -ErrorAction SilentlyContinue |
                  Where-Object { $_.Extension -match "\.(png|gif|jpg|jpeg)$" -and $_.FullName -notmatch "\\target\\" }

    Out-Section "File Inventory"
    Out-Line "  Rhai scripts  : $($allRhai.Count)"
    Out-Line "  YAML/YML files: $($allYml.Count + $allYaml.Count)"
    Out-Line "  Object defs   : $($allObjects.Count)"
    Out-Line "  Image assets  : $($allImages.Count)"

    # ── Build reference set from all text files ──────────────────────────────
    $allTextFiles = @($allRhai) + @($allYml) + @($allYaml)

    $referencedRhai    = @{}
    $referencedObjects = @{}
    $referencedImages  = @{}
    $referencedEmitters= @{}

    foreach ($f in $allTextFiles) {
        $content = Get-Content $f.FullName -Raw -ErrorAction SilentlyContinue
        if (-not $content) { continue }

        # Rhai script references (YAML behavior: and Rhai import "x")
        foreach ($m in [regex]::Matches($content, '(?:script|src):\s*[''"]?([^''">\s]+\.rhai)')) {
            $referencedRhai[$m.Groups[1].Value] = $f.Name
            # Also store just the filename without path prefix (e.g. ./mainmenu.rhai -> mainmenu.rhai)
            $referencedRhai[[System.IO.Path]::GetFileName($m.Groups[1].Value)] = $f.Name
        }
        foreach ($m in [regex]::Matches($content, 'import\s+"([^"]+)"')) {
            $referencedRhai[($m.Groups[1].Value + ".rhai")] = $f.Name
            $referencedRhai[$m.Groups[1].Value]             = $f.Name  # bare name
        }
        # behavior: inline path patterns
        foreach ($m in [regex]::Matches($content, '[\w/\-]+\.rhai')) {
            $referencedRhai[$m.Value] = $f.Name
            $referencedRhai[[System.IO.Path]::GetFileName($m.Value)] = $f.Name
        }

        # Object/sprite_template references
        foreach ($m in [regex]::Matches($content, 'sprite_template:\s*[''"]?([^\s''"]+)')) {
            $referencedObjects[$m.Groups[1].Value] = $f.Name
        }
        foreach ($m in [regex]::Matches($content, 'object:\s*[''"]?([^\s''"]+)')) {
            $referencedObjects[$m.Groups[1].Value] = $f.Name
        }

        # Emitter references — both literal world.emit("name") and string literals
        # that appear anywhere in the file (catches variable-assigned emitter names)
        foreach ($m in [regex]::Matches($content, 'world\.emit\s*\(\s*"([^"]+)"')) {
            $referencedEmitters[$m.Groups[1].Value] = $f.Name
        }
        # Also capture any quoted string that looks like an emitter name (dot-separated, no spaces)
        # This catches: let emitter = "ship.main"; or emitters[idx] pattern
        foreach ($m in [regex]::Matches($content, '"([a-zA-Z][a-zA-Z0-9_]*\.[a-zA-Z][a-zA-Z0-9_.-]*)"')) {
            $referencedEmitters[$m.Groups[1].Value] = $f.Name
        }

        # Image references
        foreach ($m in [regex]::Matches($content, '[\w/\-]+\.(png|gif|jpg|jpeg)')) {
            $referencedImages[$m.Value] = $f.Name
        }
    }

    # ── Dead Rhai scripts ────────────────────────────────────────────────────
    Out-Section "Dead Rhai Scripts (unreferenced)"
    $deadRhai = $allRhai | Where-Object {
        $name    = $_.Name
        $bareName= [System.IO.Path]::GetFileNameWithoutExtension($_.Name)
        -not ($referencedRhai.ContainsKey($name) -or $referencedRhai.ContainsKey($bareName))
    }
    if ($deadRhai) {
        foreach ($f in $deadRhai) {
            $rel = $f.FullName.Replace($modDir, "").TrimStart('\','/')
            Out-Line "  ✗  $rel"
        }
    } else {
        Out-Line "  ✓  All Rhai scripts are referenced."
    }

    # ── Dead object definitions ──────────────────────────────────────────────
    Out-Section "Dead Object Definitions (unreferenced)"
    if ($allObjects) {
        $deadObjs = $allObjects | Where-Object {
            $bareName = [System.IO.Path]::GetFileNameWithoutExtension($_.Name)
            -not $referencedObjects.ContainsKey($bareName)
        }
        if ($deadObjs) {
            foreach ($f in $deadObjs) {
                Out-Line "  ✗  objects/$($f.Name)"
            }
        } else {
            Out-Line "  ✓  All object definitions are referenced."
        }
    } else {
        Out-Line "  (no objects/ directory)"
    }

    # ── Dead image assets ────────────────────────────────────────────────────
    Out-Section "Dead Image Assets (unreferenced)"
    $deadImages = $allImages | Where-Object {
        $name     = $_.Name
        $nameNoExt= [System.IO.Path]::GetFileNameWithoutExtension($_.Name)
        -not ($referencedImages.ContainsKey($name) -or $referencedImages.ContainsKey($nameNoExt))
    }
    if ($deadImages) {
        foreach ($f in $deadImages) {
            $rel = $f.FullName.Replace($modDir, "").TrimStart('\','/')
            Out-Line "  ✗  $rel"
        }
    } elseif ($allImages.Count -eq 0) {
        Out-Line "  (no image assets)"
    } else {
        Out-Line "  ✓  All image assets are referenced."
    }

    # ── Emitter reference check ──────────────────────────────────────────────
    Out-Section "Emitter Cross-Reference"
    $emitterFile = Get-ChildItem $modDir -Filter "emitters.yaml" -Recurse -ErrorAction SilentlyContinue |
                   Select-Object -First 1
    if ($emitterFile) {
        $emitterContent = Get-Content $emitterFile.FullName -Raw
        $definedEmitters = [regex]::Matches($emitterContent, '(?m)^  ([a-zA-Z][a-zA-Z0-9._-]+):') |
                           ForEach-Object { $_.Groups[1].Value }
        foreach ($e in $definedEmitters) {
            $used = $referencedEmitters.ContainsKey($e)
            $mark = if ($used) { "  ✓" } else { "  ✗  UNUSED" }
            Out-Line "$mark  $e"
        }
    } else {
        Out-Line "  (no emitters.yaml found)"
    }

    # ── Scene navigation graph ───────────────────────────────────────────────
    Out-Section "Scene Navigation Graph"
    $sceneJumps = @{}

    foreach ($f in $allRhai) {
        $content = Get-Content $f.FullName -Raw -ErrorAction SilentlyContinue
        if (-not $content) { continue }
        foreach ($m in [regex]::Matches($content, 'game\.jump\s*\(\s*"([^"]+)"')) {
            $target = $m.Groups[1].Value
            $src    = $f.Name
            if (-not $sceneJumps.ContainsKey($src)) { $sceneJumps[$src] = @() }
            if ($sceneJumps[$src] -notcontains $target) {
                $sceneJumps[$src] += $target
            }
        }
    }

    if ($sceneJumps.Count -gt 0) {
        foreach ($src in ($sceneJumps.Keys | Sort-Object)) {
            foreach ($tgt in $sceneJumps[$src]) {
                Out-Line "  $src  →  $tgt"
            }
        }
    } else {
        Out-Line "  (no game.jump() calls found)"
    }

    # ── Scene DOT graph ──────────────────────────────────────────────────────
    if ($Dot -and $sceneJumps.Count -gt 0) {
        $dotDir = Join-Path $RepoRoot "reports"
        if (-not (Test-Path $dotDir)) { New-Item -ItemType Directory $dotDir | Out-Null }
        $dotPath = Join-Path $dotDir "$($modName)-scene-graph.dot"
        $dotLines = @("digraph scene_graph {")
        $dotLines += '  rankdir=LR;'
        $dotLines += '  node [shape=ellipse fontname="Consolas" fontsize=10];'
        foreach ($src in $sceneJumps.Keys) {
            foreach ($tgt in $sceneJumps[$src]) {
                $dotLines += "  `"$src`" -> `"$tgt`";"
            }
        }
        $dotLines += "}"
        $dotLines | Set-Content $dotPath
        Out-Line ""
        Out-Line "  Scene graph written: $dotPath"
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 7. Dead code via cargo check (opt-in, slow)
# ─────────────────────────────────────────────────────────────────────────────

if ($DeadCode) {
    Out-Header "DEAD CODE ANALYSIS (cargo check)"
    Out-Line "  Running cargo check with dead_code warnings enabled..."
    Out-Line "  (This may take 1-2 minutes on a cold build)"
    Out-Line ""

    Push-Location $RepoRoot
    $env:RUSTFLAGS = "-W dead-code -W unused-imports -W unused-variables"
    $checkOut = cargo check --workspace --message-format short 2>&1
    Remove-Item Env:\RUSTFLAGS -ErrorAction SilentlyContinue
    Pop-Location

    $warnings = $checkOut | Where-Object { $_ -match "warning:" -and $_ -match "never used|dead_code|unused" }
    if ($warnings) {
        $warnings | ForEach-Object { Out-Line "  $_" }
        Out-Line ""
        Out-Line "  Total dead-code warnings: $($warnings.Count)"
    } else {
        Out-Line "  ✓  No dead-code warnings found."
    }
}

# ─────────────────────────────────────────────────────────────────────────────
# 8. Summary
# ─────────────────────────────────────────────────────────────────────────────

Out-Header "SUMMARY"
Out-Line "  Workspace crates  : $($crateNames.Count)"
Out-Line "  Orphan crates     : $($orphans.Count)"
Out-Line "  Mods analysed     : $($modDirs.Count)"
if ($DeadCode) {
    Out-Line "  Dead-code pass    : enabled"
}
if ($Dot) {
    Out-Line "  DOT files         : reports/"
}
Out-Line ""
Out-Line "  Re-run with --dead-code for full rustc dead-code scan."
Out-Line "  Re-run with --dot to emit Graphviz .dot files."
Out-Line ""

# ─────────────────────────────────────────────────────────────────────────────
# 9. Write to file (optional)
# ─────────────────────────────────────────────────────────────────────────────

if ($Out) {
    $outPath = if ([System.IO.Path]::IsPathRooted($Out)) { $Out } else { Join-Path $RepoRoot $Out }
    $outDir  = Split-Path $outPath -Parent
    if (-not (Test-Path $outDir)) { New-Item -ItemType Directory $outDir | Out-Null }
    $Lines | Set-Content $outPath
    Write-Host "Report written: $outPath"
}
