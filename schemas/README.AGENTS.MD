# YAML Schema System — Generation & Validation

## Schema Generation

### refresh-schemas.sh

```bash
./refresh-schemas.sh
```

Regenerates all schemas in `schemas/` directory:

```
schemas/
├── scene.yml.schema.json       Generated from Scene model
├── mod.yaml.schema.json        Generated from ModManifest model
├── behavior.yml.schema.json    Generated from Behavior definition
├── effect.yml.schema.json      Generated from Effect definition
└── object.yml.schema.json      Generated from Object model
```

Equivalent to: `cargo run -p schema-gen -- --all-mods`

### Drift Checking

```bash
cargo run -p schema-gen -- --all-mods --check
```

Compares generated schemas against on-disk versions:
- Returns exit code 0 if identical
- Returns exit code 1 if any schema diverged
- Useful for CI validation before merge

Detects:
- Model field additions/removals
- Type changes
- New enum variants
- Requirement changes

## Key Schemas

### scene.yml.schema.json

Defines scene structure:

```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string" },
    "title": { "type": "string" },
    "virtual_size_override": { "type": "array", "items": { "type": "integer" } },
    "stages": { "type": "array", "items": { "$ref": "#/definitions/Stage" } },
    "layers": { "type": "array", "items": { "$ref": "#/definitions/Layer" } },
    "objects": { "type": "array", "items": { "$ref": "#/definitions/Object" } },
    "next": { "type": ["string", "null"] }
  },
  "required": ["id", "stages", "layers"]
}
```

Enforces:
- Scene ID format
- Required stages (on_enter, on_idle, on_leave)
- Layer structure (sprites, effects)
- Timeline validity (no negative durations)

### mod.yaml.schema.json

Defines mod manifest:

```json
{
  "type": "object",
  "properties": {
    "name": { "type": "string" },
    "version": { "type": "string" },
    "description": { "type": "string" },
    "author": { "type": "string" },
    "scenes": { "type": "array", "items": { "type": "string" } }
  },
  "required": ["name", "version"]
}
```

### behavior.yml.schema.json

Defines custom behavior:

```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string" },
    "steps": { "type": "array", "items": { "type": "object" } },
    "params": { "type": "object" }
  },
  "required": ["id", "steps"]
}
```

### effect.yml.schema.json

Defines custom effect:

```json
{
  "type": "object",
  "properties": {
    "id": { "type": "string" },
    "type": { "enum": ["builtin", "custom"] },
    "params": { "type": "object" }
  },
  "required": ["id", "type"]
}
```

## Validation Rules

### Timeline Validation

Compile-time warnings (debug builds only):
- Sprite `appear_at_ms >= on_enter.duration` → "sprite will never be visible"
- Sprite `disappear_at_ms <= appear_at_ms` → "sprite always hidden"
- Effect duration exceeds stage duration → "effect timing conflict"

### Reference Validation

- Scene `next: <ID>` must exist in mod
- Sprite references layer must exist
- Effect references must be registered
- Behavior parameter types must match schema

### Schema Validation

- YAML files must conform to `.schema.json`
- Catches typos (missing required fields)
- Validates enum values
- Type checking (string vs number, etc.)

## Custom Extensions

Mods can define custom schemas:

```
mods/shell-quest/
├── schemas/
│   ├── custom-behavior.schema.json
│   └── custom-effect.schema.json
└── behaviors/
    └── custom-behavior.yml
```

Engine merges mod schemas with core schemas during compilation.

## CI Integration

```yaml
# .github/workflows/validate.yml
- name: Check schema drift
  run: cargo run -p schema-gen -- --all-mods --check

- name: Lint all YAML
  run: python3 tools/yaml-lint.py schemas/ mods/
```

Returns exit code 1 if:
- Generated schemas diverge from on-disk
- YAML files don't conform to schema
- References are broken

## Troubleshooting

### Schema diverged error

```
error: Generated schema differs from on-disk version
File: schemas/scene.yml.schema.json
Difference: field 'new_property' added
```

Solution:
```bash
./refresh-schemas.sh
git add schemas/
git commit -m "schema: regenerate after model changes"
```

### YAML validation error

```
error: field 'sprite_type' does not match schema
Expected: one of ["cell", "halfblock", "obj", "gif"]
Got: "sprite"
```

Solution: Check YAML file for typo, fix, re-validate.

### Circular reference in schema

Rare, indicates model cycle. Check:
- Scene references Object
- Object references Scene
- Limit recursion depth in schema

## Schema Format

All schemas use JSON Schema Draft 7:
- `$ref` for references
- `definitions` for shared types
- `enum` for fixed value sets
- `oneOf` for union types
- `required` array for mandatory fields

Compatible with:
- VS Code YAML extension
- IntelliJ IDEA
- JSON Schema validators
- Online schema editors
