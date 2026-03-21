#!/usr/bin/env python3
"""
Simplify GLB models using Blender's decimate modifier
Target: reduce file size to ~13MB (similar to existing models)
"""
import bpy
import sys
import os

def simplify_model(input_path, output_path, target_ratio=0.3):
    """
    Load GLB, apply decimate modifier, export simplified version
    
    Args:
        input_path: Path to input GLB file
        output_path: Path to output GLB file
        target_ratio: Decimate ratio (0.3 = keep 30% of polygons)
    """
    # Clear existing scene
    bpy.ops.wm.read_homefile(use_empty=True)
    
    # Import GLB
    print(f"Importing {input_path}...")
    bpy.ops.import_scene.gltf(filepath=input_path)
    
    # Get all mesh objects
    mesh_objects = [obj for obj in bpy.context.scene.objects if obj.type == 'MESH']
    
    if not mesh_objects:
        print("ERROR: No mesh objects found!")
        return False
    
    total_before = sum(len(obj.data.polygons) for obj in mesh_objects)
    print(f"Found {len(mesh_objects)} mesh(es) with {total_before:,} polygons total")
    
    # Apply decimate to each mesh
    for obj in mesh_objects:
        bpy.context.view_layer.objects.active = obj
        obj.select_set(True)
        
        # Add decimate modifier
        modifier = obj.modifiers.new(name="Decimate", type='DECIMATE')
        modifier.ratio = target_ratio
        modifier.use_collapse_triangulate = True
        
        # Apply modifier
        bpy.ops.object.modifier_apply(modifier="Decimate")
        
        obj.select_set(False)
    
    total_after = sum(len(obj.data.polygons) for obj in mesh_objects)
    print(f"After decimation: {total_after:,} polygons ({total_after/total_before*100:.1f}%)")
    
    # Export simplified GLB
    print(f"Exporting to {output_path}...")
    bpy.ops.export_scene.gltf(
        filepath=output_path,
        export_format='GLB',
        export_texcoords=True,
        export_normals=True,
        export_materials='EXPORT',
        use_selection=False
    )
    
    # Check file sizes
    input_size = os.path.getsize(input_path)
    output_size = os.path.getsize(output_path)
    print(f"File size: {input_size/1024/1024:.1f}MB → {output_size/1024/1024:.1f}MB ({output_size/input_size*100:.1f}%)")
    
    return True

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: blender --background --python simplify_glb.py -- <input.glb> <output.glb> [ratio]")
        sys.exit(1)
    
    # Arguments after '--' are in sys.argv
    input_file = sys.argv[-3] if len(sys.argv) >= 3 else sys.argv[1]
    output_file = sys.argv[-2] if len(sys.argv) >= 3 else sys.argv[2]
    ratio = float(sys.argv[-1]) if len(sys.argv) >= 4 and sys.argv[-1].replace('.','').isdigit() else 0.3
    
    print(f"=== Simplifying GLB Model ===")
    print(f"Input:  {input_file}")
    print(f"Output: {output_file}")
    print(f"Ratio:  {ratio}")
    print()
    
    success = simplify_model(input_file, output_file, ratio)
    sys.exit(0 if success else 1)
