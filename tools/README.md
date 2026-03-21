# Shell Quest Tools

Developer utilities for Shell Quest content pipeline.

## 3D Asset Pipeline

### simplify_glb.py

Reduces polygon count of GLB 3D models using Blender's decimate modifier.

**Purpose**: Optimize large sculpted models for real-time terminal rendering while maintaining visual quality.

**Usage**:
```bash
blender --background --python tools/simplify_glb.py -- \
  <input.glb> <output.glb> [ratio]
```

**Arguments**:
- `input.glb` - Source GLB file (binary glTF 2.0)
- `output.glb` - Output simplified GLB file
- `ratio` - Optional decimate ratio (default: 0.3 = keep 30% of polygons)

**Example**:
```bash
# Reduce model to 15% polygons (aggressive optimization)
blender --background --python tools/simplify_glb.py -- \
  assets/sculpts/source.glb \
  assets/sculpts/optimized.glb \
  0.15

# Moderate reduction (30% - default)
blender --background --python tools/simplify_glb.py -- \
  assets/sculpts/source.glb \
  assets/sculpts/optimized.glb
```

**Technical Details**:
- Uses Blender's `DECIMATE` modifier with collapse + triangulate
- Preserves materials, normals, UVs, and vertex colors
- Exports as GLB (binary glTF) with full material properties
- Prints polygon count and file size reduction statistics

**Typical Results**:
- 0.30 ratio: ~70% file size reduction (balanced)
- 0.15 ratio: ~60-70% file size reduction (aggressive)
- Quality degrades gracefully - test visually in-game

**Dependencies**:
- Blender 5.0+ (tested with 5.0.1)
- Python 3.x (bundled with Blender)

**Real-World Example** (from commit 872bad0):
```bash
# Simplified difficulty portrait models
blender --background --python tools/simplify_glb.py -- \
  mods/shell-quest/assets/3d/sculpts/3-2.glb \
  mods/shell-quest/assets/3d/sculpts/3-2-simplified.glb \
  0.15

# Results:
# Input:  2,493,448 polygons → 43MB
# Output:   374,016 polygons → 30MB (15% faces, 70% file size)
```

**When to Use**:
- Models from sculpting tools (ZBrush, Blender Sculpt) with 1M+ polygons
- File sizes over 20MB (terminal renderer bottleneck)
- Before committing new 3D assets to repository

**When NOT to Use**:
- Low-poly models (<50k faces) - decimation overhead not worth it
- Animated meshes with morph targets (decimate breaks blendshapes)
- Models with critical edge flow (topology-dependent features)

---

## Other Tools

### devtool (Rust CLI)

Main content authoring CLI. See `README.md` and `devtool --help` for details.

### schema-gen (Rust)

Generates YAML schemas for all mods. Run via `./refresh-schemas.sh`.

### docs/build_api_docs.py

Generates HTML documentation from `concat-report.txt`. Run via `./gendocs`.
