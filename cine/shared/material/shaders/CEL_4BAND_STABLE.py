import bpy

# ============================================================
# SIMPLE STABLE 4-BAND CEL SHADER
# - działa w EEVEE
# - reaguje na światło sceny
# - bez kolorowania światłem na końcu
# - wersja diagnostyczna / stabilna
# ============================================================

MATERIAL_NAME = "CEL_4BAND_STABLE"
TARGET_OBJECT_NAME = ""   # np. "Cube", albo zostaw ""

BASE_COLOR = (0.85, 0.85, 0.85, 1.0)

BLACK_COLOR = (0.0, 0.0, 0.0, 1.0)
DARK_COLOR  = (0.18, 0.18, 0.18, 1.0)
MID_TINT    = (1.0, 1.0, 1.0, 1.0)
PEAK_COLOR  = (1.0, 1.0, 1.0, 1.0)

# Progi:
# bw < T1         -> black
# T1..T2          -> dark
# T2..T3          -> mid
# > T3            -> peak
T1 = 0.08
T2 = 0.35
T3 = 0.78

DARK_BLEND = 0.85
PEAK_BLEND = 0.65

REPLACE_ACTIVE_SLOT = True
REUSE_EXISTING_IMAGE = True


# ============================================================
# HELPERS
# ============================================================

def set_eevee():
    scene = bpy.context.scene
    for engine in ("BLENDER_EEVEE_NEXT", "BLENDER_EEVEE"):
        try:
            scene.render.engine = engine
            return engine
        except Exception:
            pass
    raise RuntimeError("Nie udało się ustawić EEVEE.")

def is_mesh(obj):
    return obj is not None and obj.type == 'MESH' and hasattr(obj.data, "materials")

def get_target():
    if TARGET_OBJECT_NAME:
        obj = bpy.data.objects.get(TARGET_OBJECT_NAME)
        if is_mesh(obj):
            return obj
        raise RuntimeError(f"Obiekt '{TARGET_OBJECT_NAME}' nie istnieje albo nie jest MESH.")

    obj = bpy.context.active_object
    if is_mesh(obj):
        return obj

    for o in bpy.context.selected_objects:
        if is_mesh(o):
            return o

    for o in bpy.context.scene.objects:
        if is_mesh(o):
            return o

    raise RuntimeError("Nie znaleziono obiektu MESH. Zaznacz np. Cube.")

def find_first_image_from_material(mat):
    if not mat or not mat.use_nodes or not mat.node_tree:
        return None
    for node in mat.node_tree.nodes:
        if node.type == 'TEX_IMAGE' and getattr(node, "image", None) is not None:
            return node.image
    return None

def clear_nodes(nt):
    for n in list(nt.nodes):
        nt.nodes.remove(n)

def new_node(nt, node_type, name, x, y):
    n = nt.nodes.new(node_type)
    n.name = name
    n.label = name
    n.location = (x, y)
    return n

def link(nt, a, b):
    nt.links.new(a, b)

def set_rgb(node, rgba):
    node.outputs[0].default_value = rgba

def set_val(node, value):
    node.outputs[0].default_value = value

def make_math(nt, name, op, x, y):
    n = new_node(nt, "ShaderNodeMath", name, x, y)
    n.operation = op
    return n

def make_mix(nt, name, blend_type, x, y, fac=1.0):
    n = new_node(nt, "ShaderNodeMixRGB", name, x, y)
    n.blend_type = blend_type
    n.inputs[0].default_value = fac
    return n

def assign_material(obj, mat):
    if REPLACE_ACTIVE_SLOT and len(obj.material_slots) > 0:
        obj.material_slots[obj.active_material_index].material = mat
    else:
        obj.data.materials.append(mat)
        obj.active_material = mat


# ============================================================
# BUILD
# ============================================================

def build_material(reference_material):
    existing_image = None
    if REUSE_EXISTING_IMAGE:
        existing_image = find_first_image_from_material(reference_material)

    mat = bpy.data.materials.get(MATERIAL_NAME)
    if mat is None:
        mat = bpy.data.materials.new(MATERIAL_NAME)

    mat.use_nodes = True
    nt = mat.node_tree
    clear_nodes(nt)

    # --------------------------------------------------------
    # Inputs
    # --------------------------------------------------------
    n_base = new_node(nt, "ShaderNodeRGB", "Base Color", -1700, 500)
    set_rgb(n_base, BASE_COLOR)

    n_black = new_node(nt, "ShaderNodeRGB", "Black Color", -1700, 260)
    set_rgb(n_black, BLACK_COLOR)

    n_dark = new_node(nt, "ShaderNodeRGB", "Dark Color", -1700, 60)
    set_rgb(n_dark, DARK_COLOR)

    n_mid_tint = new_node(nt, "ShaderNodeRGB", "Mid Tint", -1700, -140)
    set_rgb(n_mid_tint, MID_TINT)

    n_peak = new_node(nt, "ShaderNodeRGB", "Peak Color", -1700, -340)
    set_rgb(n_peak, PEAK_COLOR)

    n_t1 = new_node(nt, "ShaderNodeValue", "T1", -1700, -600)
    set_val(n_t1, T1)

    n_t2 = new_node(nt, "ShaderNodeValue", "T2", -1700, -740)
    set_val(n_t2, T2)

    n_t3 = new_node(nt, "ShaderNodeValue", "T3", -1700, -880)
    set_val(n_t3, T3)

    n_dark_blend = new_node(nt, "ShaderNodeValue", "Dark Blend", -1700, -1020)
    set_val(n_dark_blend, DARK_BLEND)

    n_peak_blend = new_node(nt, "ShaderNodeValue", "Peak Blend", -1700, -1160)
    set_val(n_peak_blend, PEAK_BLEND)

    # --------------------------------------------------------
    # Base source
    # --------------------------------------------------------
    if existing_image is not None:
        n_img = new_node(nt, "ShaderNodeTexImage", "Base Image", -1700, 760)
        n_img.image = existing_image

        n_img_mul = make_mix(nt, "Image x Base", "MULTIPLY", -1400, 650, fac=1.0)
        link(nt, n_img.outputs["Color"], n_img_mul.inputs[1])
        link(nt, n_base.outputs["Color"], n_img_mul.inputs[2])

        base_source = n_img_mul.outputs["Color"]
    else:
        base_source = n_base.outputs["Color"]

    # --------------------------------------------------------
    # Lighting
    # --------------------------------------------------------
    n_diffuse = new_node(nt, "ShaderNodeBsdfDiffuse", "Diffuse", -1100, 500)
    n_diffuse.inputs["Roughness"].default_value = 0.0
    link(nt, base_source, n_diffuse.inputs["Color"])

    n_s2r = new_node(nt, "ShaderNodeShaderToRGB", "Shader To RGB", -860, 500)
    link(nt, n_diffuse.outputs["BSDF"], n_s2r.inputs["Shader"])

    n_bw = new_node(nt, "ShaderNodeRGBToBW", "RGB To BW", -620, 500)
    link(nt, n_s2r.outputs["Color"], n_bw.inputs["Color"])

    # --------------------------------------------------------
    # Masks
    # --------------------------------------------------------
    black_mask = make_math(nt, "Black Mask", "LESS_THAN", -360, 700)
    link(nt, n_bw.outputs["Val"], black_mask.inputs[0])
    link(nt, n_t1.outputs["Value"], black_mask.inputs[1])

    gt_t1 = make_math(nt, "BW > T1", "GREATER_THAN", -360, 520)
    link(nt, n_bw.outputs["Val"], gt_t1.inputs[0])
    link(nt, n_t1.outputs["Value"], gt_t1.inputs[1])

    lt_t2 = make_math(nt, "BW < T2", "LESS_THAN", -360, 360)
    link(nt, n_bw.outputs["Val"], lt_t2.inputs[0])
    link(nt, n_t2.outputs["Value"], lt_t2.inputs[1])

    dark_mask = make_math(nt, "Dark Mask", "MULTIPLY", -120, 440)
    link(nt, gt_t1.outputs["Value"], dark_mask.inputs[0])
    link(nt, lt_t2.outputs["Value"], dark_mask.inputs[1])

    gt_t2 = make_math(nt, "BW > T2", "GREATER_THAN", -360, 120)
    link(nt, n_bw.outputs["Val"], gt_t2.inputs[0])
    link(nt, n_t2.outputs["Value"], gt_t2.inputs[1])

    lt_t3 = make_math(nt, "BW < T3", "LESS_THAN", -360, -40)
    link(nt, n_bw.outputs["Val"], lt_t3.inputs[0])
    link(nt, n_t3.outputs["Value"], lt_t3.inputs[1])

    mid_mask = make_math(nt, "Mid Mask", "MULTIPLY", -120, 40)
    link(nt, gt_t2.outputs["Value"], mid_mask.inputs[0])
    link(nt, lt_t3.outputs["Value"], mid_mask.inputs[1])

    peak_mask = make_math(nt, "Peak Mask", "GREATER_THAN", -360, -260)
    link(nt, n_bw.outputs["Val"], peak_mask.inputs[0])
    link(nt, n_t3.outputs["Value"], peak_mask.inputs[1])

    # --------------------------------------------------------
    # Bands
    # --------------------------------------------------------
    black_band = n_black.outputs["Color"]

    dark_band = make_mix(nt, "Dark Band", "MIX", -1100, 160)
    link(nt, n_dark_blend.outputs["Value"], dark_band.inputs[0])
    link(nt, base_source, dark_band.inputs[1])
    link(nt, n_dark.outputs["Color"], dark_band.inputs[2])

    mid_band = make_mix(nt, "Mid Band", "MULTIPLY", -1100, -40, fac=1.0)
    link(nt, base_source, mid_band.inputs[1])
    link(nt, n_mid_tint.outputs["Color"], mid_band.inputs[2])

    peak_band = make_mix(nt, "Peak Band", "MIX", -1100, -240)
    link(nt, n_peak_blend.outputs["Value"], peak_band.inputs[0])
    link(nt, base_source, peak_band.inputs[1])
    link(nt, n_peak.outputs["Color"], peak_band.inputs[2])

    # --------------------------------------------------------
    # Apply masks
    # --------------------------------------------------------
    black_apply = make_mix(nt, "Black * Mask", "MULTIPLY", 120, 700, fac=1.0)
    link(nt, black_band, black_apply.inputs[1])
    link(nt, black_mask.outputs["Value"], black_apply.inputs[2])

    dark_apply = make_mix(nt, "Dark * Mask", "MULTIPLY", 120, 440, fac=1.0)
    link(nt, dark_band.outputs["Color"], dark_apply.inputs[1])
    link(nt, dark_mask.outputs["Value"], dark_apply.inputs[2])

    mid_apply = make_mix(nt, "Mid * Mask", "MULTIPLY", 120, 140, fac=1.0)
    link(nt, mid_band.outputs["Color"], mid_apply.inputs[1])
    link(nt, mid_mask.outputs["Value"], mid_apply.inputs[2])

    peak_apply = make_mix(nt, "Peak * Mask", "MULTIPLY", 120, -160, fac=1.0)
    link(nt, peak_band.outputs["Color"], peak_apply.inputs[1])
    link(nt, peak_mask.outputs["Value"], peak_apply.inputs[2])

    # --------------------------------------------------------
    # Sum
    # --------------------------------------------------------
    add1 = make_mix(nt, "Add1", "ADD", 400, 520, fac=1.0)
    link(nt, black_apply.outputs["Color"], add1.inputs[1])
    link(nt, dark_apply.outputs["Color"], add1.inputs[2])

    add2 = make_mix(nt, "Add2", "ADD", 640, 240, fac=1.0)
    link(nt, add1.outputs["Color"], add2.inputs[1])
    link(nt, mid_apply.outputs["Color"], add2.inputs[2])

    add3 = make_mix(nt, "Add3", "ADD", 880, -40, fac=1.0)
    link(nt, add2.outputs["Color"], add3.inputs[1])
    link(nt, peak_apply.outputs["Color"], add3.inputs[2])

    # --------------------------------------------------------
    # Output
    # --------------------------------------------------------
    n_emission = new_node(nt, "ShaderNodeEmission", "Emission", 1120, -40)
    n_emission.inputs["Strength"].default_value = 1.0
    link(nt, add3.outputs["Color"], n_emission.inputs["Color"])

    n_out = new_node(nt, "ShaderNodeOutputMaterial", "Output", 1340, -40)
    link(nt, n_emission.outputs["Emission"], n_out.inputs["Surface"])

    return mat


# ============================================================
# MAIN
# ============================================================

engine = set_eevee()
obj = get_target()
mat = build_material(obj.active_material)
assign_material(obj, mat)

print("====================================")
print("Gotowe")
print("Engine:", engine)
print("Object:", obj.name)
print("Material:", mat.name)
print("====================================")
