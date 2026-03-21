#!/usr/bin/env python3
"""
Convert GLB to OBJ using Blender
"""
import bpy
import sys
import os

def convert_glb_to_obj(input_glb, output_obj):
    """Convert GLB to OBJ format"""
    # Clear scene
    bpy.ops.wm.read_homefile(use_empty=True)
    
    # Import GLB
    print(f"Importing {input_glb}...")
    bpy.ops.import_scene.gltf(filepath=input_glb)
    
    # Get all mesh objects
    mesh_objects = [obj for obj in bpy.context.scene.objects if obj.type == 'MESH']
    total_faces = sum(len(obj.data.polygons) for obj in mesh_objects)
    print(f"Loaded {len(mesh_objects)} mesh(es) with {total_faces:,} faces")
    
    # Export as OBJ
    print(f"Exporting to {output_obj}...")
    bpy.ops.wm.obj_export(
        filepath=output_obj,
        export_selected_objects=False,
        export_materials=True,
        export_normals=True,
        export_uv=True,
        export_triangulated_mesh=True
    )
    
    input_size = os.path.getsize(input_glb)
    output_size = os.path.getsize(output_obj)
    print(f"File size: {input_size/1024/1024:.1f}MB → {output_size/1024/1024:.1f}MB")
    
    return True

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: blender --background --python glb_to_obj.py -- <input.glb> <output.obj>")
        sys.exit(1)
    
    input_file = sys.argv[-2]
    output_file = sys.argv[-1]
    
    print(f"=== Converting GLB to OBJ ===")
    print(f"Input:  {input_file}")
    print(f"Output: {output_file}")
    print()
    
    success = convert_glb_to_obj(input_file, output_file)
    sys.exit(0 if success else 1)
