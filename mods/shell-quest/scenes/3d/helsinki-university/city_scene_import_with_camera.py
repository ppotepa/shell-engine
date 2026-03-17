import bpy
import os
from mathutils import Vector

base_dir = os.path.dirname(bpy.data.filepath) if bpy.data.filepath else os.path.dirname(__file__)
obj_path = os.path.join(base_dir, "city_scene_horizontal_front_yup.obj")

# Clear scene
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)

# Import OBJ
bpy.ops.wm.obj_import(filepath=obj_path)

# Create camera
cam_data = bpy.data.cameras.new(name="MainBuildingCamera")
cam = bpy.data.objects.new("MainBuildingCamera", cam_data)
bpy.context.collection.objects.link(cam)

cam.location = (0.0, 16.0, 190.0)
cam.data.lens_unit = 'FOV'
cam.data.angle = 35.0 * 3.141592653589793 / 180.0

target = Vector((0.0, 12.0, -12.85))
direction = target - cam.location
cam.rotation_mode = 'QUATERNION'
cam.rotation_quaternion = direction.to_track_quat('-Z', 'Y')

bpy.context.scene.camera = cam
