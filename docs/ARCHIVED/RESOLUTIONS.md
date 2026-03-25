# Terminal Resolution & Halfblock Rendering

This document explains how resolutions work in Shell Quest, how halfblock rendering scales content, and which terminal sizes produce clean output.

---

## 🎬 Overview: Halfblock Rendering

Shell Quest uses **halfblock rendering** to achieve high vertical resolution on terminals.

### The Basic Model

**Traditional terminal rendering:**
- Each character cell = 1 pixel
- 120 character columns × 30 rows = 120 × 30 pixel canvas
- Limited vertical resolution (max ~50-60 pixels in practice)

**Halfblock rendering (our approach):**
- Each character cell can display **2 vertical pixels** (upper/lower blocks: ▀ ▄)
- Render at 2× height internally, then pack pairs into halfblock characters
- 120 character columns × 30 rows = 120 × 60 virtual pixels

### Resolution Mapping Formula

```
Terminal Size (cells):        W × H
                              ↓
Rendered as Virtual Buffer:  W × (H × 2) pixels
                              ↓
Output back to Terminal:      W × H cells (using halfblock chars)
```

**Examples:**

| Terminal Size | Virtual Buffer | Packing Method |
|---|---|---|
| 120 × 30 | 120 × 60 | Pairs of vertical pixels → halfblocks |
| 160 × 50 | 160 × 100 | Pairs of vertical pixels → halfblocks |
| 200 × 100 | 200 × 200 | Pairs of vertical pixels → halfblocks |
| 300 × 100 | 300 × 200 | Pairs of vertical pixels → halfblocks |
| 379 × 109 | 379 × 218 | Pairs of vertical pixels → halfblocks |

---

## 📊 Recommended Terminal Resolutions

These resolutions are known to render cleanly without flickering or performance degradation.

### Tier 1: Universally Safe (All Terminals)

**Best compatibility across all terminals, even older/constrained ones:**

| Resolution | Virtual Buffer | Cells | Notes |
|---|---|---|---|
| 120 × 30 | 120 × 60 | Standard | Oldest terminal emulators support this |
| 160 × 40 | 160 × 80 | Common | Good compromise for small screens |
| 200 × 50 | 200 × 100 | Wide | Supports ultrawide terminals |

**Aspect ratios:** 4:1 (W:H aspect on virtual buffer is ~2:1)

### Tier 2: Modern Terminals (Recommended Default)

**Balances visual quality with broad terminal support:**

| Resolution | Virtual Buffer | Cells | Notes |
|---|---|---|---|
| 160 × 50 | 160 × 100 | Balanced | Best for most modern terminals |
| 200 × 65 | 200 × 130 | Detailed | Good visual quality, smooth rendering |
| 240 × 75 | 240 × 150 | Large | Supports bigger screens |
| 320 × 90 | 320 × 180 | Extra Large | Maximum recommended size for 99% of terminals |

**Aspect ratios:** 3:1 to 3.5:1 (W:H aspect on virtual buffer)

### Tier 3: High-Resolution Terminals (Requires --unconstrained)

**For modern, powerful terminal emulators (Tabby, iTerm2, modern VSCode terminal):**

| Resolution | Virtual Buffer | Cells | Notes |
|---|---|---|---|
| 300 × 100 | 300 × 200 | Tabby native | May flicker without `--unconstrained` |
| 379 × 109 | 379 × 218 | Konsole max | Sharp rendering if terminal font supports it |
| 400 × 120 | 400 × 240 | Extra detail | Performance impact; verify on target hardware |

**⚠️ Known Issues:**
- Tabby full-screen without `--unconstrained` → stuttering/flickering
- Performance degrades beyond 320 × 180 on slower systems
- Font rendering quality depends on terminal font size/DPI

---

## 🧮 Virtual Size Calculation

### For Game Designers: Choosing a Scene Virtual Size

Future mod.yaml constraints:

```yaml
terminal:
  virtual-size: max-available
  virtual-size-max: "320x180"        # Never render larger than this
  allow-native-resolution: true      # Users can override with --unconstrained
```

The engine will:

### Terminal Compatibility Check

Given your desired **virtual size**, what terminal is required?

```
Virtual Size: W × H pixels
Terminal Columns Needed: W
Terminal Rows Needed: H / 2
```

**Example:** You want virtual 320 × 180
- Terminal needs: 320 columns × 90 rows
- Konsole max: ~379 × 109 ✓ Fits!
- Small laptop terminal: ~120 × 30 ✗ Too small

---

## ✨ What Makes a Resolution "Clean"?

### Good (Clean) Resolutions

✅ **Divisible by 2 in height** (since halfblock packs pairs)
- 60, 100, 130, 150, 180, 200, 218 all divide evenly by 2
- No rounding artifacts, no lost pixels

✅ **Common aspect ratios** (~3:1 W:H on virtual buffer)
- 320 × 100 (3.2:1) — clean
- 300 × 90 (3.3:1) — clean  
- 240 × 70 (3.4:1) — clean

✅ **Font rendering aligns**
- Most terminal fonts are designed for even-height renders
- Even pixel counts allow proper font anti-aliasing

### Poor (Artifacts) Resolutions

❌ **Odd heights** (cannot divide evenly by 2)
- 379 × 109 (height is odd) → bottom pixel might be lost or duplicated
- 300 × 99 → may cause misalignment
- ✓ But 379 × 218 rendering maps to 379 × 109 terminal = works fine!

❌ **Extremely high resolutions**
- Beyond 400 × 120 (virtual: 400 × 240)
- Performance degrades (fullscreen render every frame)
- Flickering on slower systems

❌ **Mismatched aspect ratios**
- Very wide (W much larger than H) → text appears stretched
- Very tall (H much larger than W) → text appears compressed

---

## 🚀 Shell Quest Configuration: How It Works

### Default Behavior (mod.yaml)

```yaml
terminal:
  virtual-size: max-available
```

The engine will:
1. Detect terminal size at startup
2. Use that size directly (fill available space)
3. Respect `min-width` / `min-height` constraints from mod.yaml

### Scene Overrides (Per-Scene)

```yaml
scenes/my-scene/scene.yml:
  virtual-size-override: "200x120"
  # Actual size rendered = 200 × 120 pixels
  # Terminal requires 200 columns × 60 rows
```
1. Detect terminal size
2. Cap to `virtual-size-max` **unless** `--unconstrained` flag is provided
3. Allow power users to opt-in to full native resolution

### User Control (CLI)

```bash
# Respects mod.yaml constraints (default)
cargo run -p app

# Power user: ignore constraints, use full terminal size
cargo run -p app -- --unconstrained
```

---

## 📈 Performance & Rendering Quality

### CPU Performance by Resolution

| Resolution | Virtual Buffer | Estimated CPU | Notes |
|---|---|---|---|
| 120 × 60 | 7.2K pixels | Very Low | Instant, even on slow systems |
| 160 × 100 | 16K pixels | Low | No perceivable lag |
| 240 × 150 | 36K pixels | Medium | Optimizations help, smooth |
| 320 × 180 | 57.6K pixels | Medium-High | With `--opt-comp` required |
| 379 × 218 | 82.6K pixels | High | May flicker on slower systems |
| 400 × 240 | 96K pixels | Very High | Verify performance first |

### Optimization Flags Needed

- **Up to 240 × 150:** Works without flags
- **240 × 150 to 320 × 180:** Use `cargo run -p app -- --opt-comp`
- **Beyond 320 × 180:** Use `--opt-comp --opt-present` (and test thoroughly)

---

## 🎯 Recommendations by Use Case

### For Maximum Compatibility (No Size Constraints)
- **Tier 1:** Use 160 × 50 (virtual 160 × 100) as default
- **Fallback:** Support 120 × 30 for very old terminals
- **Upscale:** Allow 240 × 75 on modern terminals

### For Best Visual Quality (Recommended)
- **Target:** 320 × 90 (virtual 320 × 180)
- **Constraint:** Set `virtual-size-max: "320x180"` in mod.yaml
- **Allow override:** Enable `allow-native-resolution: true` for Tabby users

### For Maximum Resolution (Experimental)
- **Requires:** `--unconstrained` flag + optimization flags
- **Test first:** Verify on target hardware
- **Known issue:** Tabby full-screen may flicker without optimization tuning

---

## 🔧 Troubleshooting

### Flickering in Tabby
- **Cause:** Halfblock packing not synchronized with presenter
- **Solution:** Use `--unconstrained` flag only if needed; otherwise stick to 320 × 180 constraint

### Text Blurry or Pixelated
- **Cause:** Terminal font size doesn't match render resolution
- **Solution:** Adjust terminal font size (usually increase by 1–2 points)

### Performance Degradation at High Resolution
- **Cause:** CPU rendering all pixels, no dirty-region optimization
- **Solution:** Use `--opt-comp` flag; verify resolution ≤ 320 × 180

### Some Pixels Missing (Odd Heights)
- **Cause:** Height not divisible by 2 in halfblock packing
- **Solution:** Use even heights (100, 120, 150, 180, 200, etc.)

---

## 📚 Reference: All Tested Resolutions

| Terminal | Virtual | Status | Notes |
|---|---|---|---|
| Konsole 120×30 | 120×60 | ✅ | Baseline |
| GNOME Terminal 200×50 | 200×100 | ✅ | Standard |
| Tabby 300×100 | 300×200 | ✅ | Needs `--unconstrained` to avoid flicker |
| iTerm2 200×60 | 200×120 | ✅ | No issues |
| VSCode Terminal 160×50 | 160×100 | ✅ | Default recommendation |
| Windows Terminal 240×65 | 240×130 | ✅ | Good balance |
| Konsole max 379×109 | 379×218 | ✅ | Highest tested; can flicker |

---

## 🧠 Architecture Notes

### Why Halfblock Works
- Unicode halfblock characters (▀ ▄) use standard terminal font rendering
- No custom fonts required
- Width stays same (terminal character width), height doubles (pair upper/lower blocks)

### Why Odd Heights Cause Issues
- Halfblock packing iterates pairs: pixel 0-1 → char 0, pixel 2-3 → char 1, etc.
- Odd height means last pixel pair is incomplete
- Some systems handle this gracefully, others duplicate/drop the pixel

### Why Performance Degrades at High Resolution
- Each frame renders full virtual buffer (unless dirty region optimization active)
- 82.6K pixels (379 × 218) = 14× more work than 120 × 60
- With `--opt-comp`, only dirty regions re-render (still high overhead for full-screen changes)

---

## 🚀 Future Work

1. **Adaptive resolution:** Auto-scale based on terminal capabilities
2. **Visual tests:** Automated regression tests at each tier
3. **DPI awareness:** Better handling of HiDPI terminals
4. **Font metrics:** Detect font size and adjust rendering accordingly
