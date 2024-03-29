# Copyright (c) 2021 Kyrylo Bazhenov
#
# This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
# If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

import os, bpy, json, uuid
from pathlib import Path

MTLD_PACK_CACHE = '/path/to/.mtld-pack-cache/'

def create_image_node(material, image_path, srgb, location_y):
    image = bpy.data.images.load(filepath = str(image_path), check_existing = True)
    if not srgb:
        image.colorspace_settings.name = 'Non-Color'
    tex_image = material.node_tree.nodes.new('ShaderNodeTexImage')
    tex_image.location = [-1200.0, location_y]
    tex_image.image = image
    # tex_image.interpolation = 'Closest'
    tex_image.interpolation = 'Cubic'
    return tex_image

material_category_ids = None
if bpy.app.version >= (3, 0, 0):
    material_category_ids = dict()

for material_path in Path(MTLD_PACK_CACHE).iterdir():
    if not material_path.is_dir():
        continue

    material_json = None
    with open(material_path.joinpath('Material.json')) as f:
        material_json = json.load(f)

    material_exists = False
    material_name = material_json['name']
    material = None
    try:
        material = bpy.data.materials[material_name]
        material_exists = True
    except:
        material = bpy.data.materials.new(name = material_name)
        material.use_nodes = True
        material.use_fake_user = True

    if bpy.app.version >= (3, 0, 0):
        material_category = material_json['category']
        category_uuid = None
        try:
            category_uuid = material_category_ids[material_category]
        except:
            category_uuid = str(uuid.uuid4())
            material_category_ids[material_category] = category_uuid
        material.asset_mark()
        material.asset_data.catalog_id = category_uuid

    if material_exists:
        continue

    material.cycles.displacement_method = 'DISPLACEMENT'
    for node in material.node_tree.nodes:
        material.node_tree.nodes.remove(node)

    has_albedo = material_json['albedo']
    has_opacity = material_json['opacity']
    has_normal = material_json['normal']
    has_metalness = material_json['metalness']
    has_roughness = material_json['roughness']
    # has_ao = material_json['ao']
    has_displacement = material_json['displacement']

    invert_normal = False

    color_image_path = material_path.joinpath(f'{material_name}_A.png')
    material_pack_image_path = material_path.joinpath(f'{material_name}_MODR.png')
    normal_image_path = material_path.joinpath(f'{material_name}_N.png')

    material_output = material.node_tree.nodes.new('ShaderNodeOutputMaterial')
    material_output.location = [400.0, -100.0]

    principled_bsdf = material.node_tree.nodes.new('ShaderNodeBsdfPrincipled')
    principled_bsdf.location = [0.0, 400.0]
    material.node_tree.links.new(principled_bsdf.outputs['BSDF'], material_output.inputs['Surface'])

    image_node_location_y = 0.0

    if has_albedo or has_opacity:
        color_image = create_image_node(material, color_image_path, True, image_node_location_y)
        material.node_tree.links.new(color_image.outputs['Color'], principled_bsdf.inputs['Base Color'])
        if has_opacity:
            material.node_tree.links.new(color_image.outputs['Alpha'], principled_bsdf.inputs['Alpha'])
            material.blend_method = 'CLIP'
            material.shadow_method = 'CLIP'
            material.use_backface_culling = False
        else:
            material.use_backface_culling = True

    material_pack_image = None
    displacement = None
    if has_metalness or has_roughness or has_displacement:
        image_node_location_y -= 400.0
        material_pack_image = create_image_node(material, material_pack_image_path, False, image_node_location_y)

        separate_rgb = None
        if has_metalness or has_displacement:
            separate_rgb = material.node_tree.nodes.new('ShaderNodeSeparateRGB')
            separate_rgb.location = [-900.0, image_node_location_y]
            material.node_tree.links.new(material_pack_image.outputs['Color'], separate_rgb.inputs['Image'])

        if has_metalness:
            material.node_tree.links.new(separate_rgb.outputs['R'], principled_bsdf.inputs['Metallic'])

        if has_roughness:
            material.node_tree.links.new(material_pack_image.outputs['Alpha'], principled_bsdf.inputs['Roughness'])

        if has_displacement:
            image_node_location_y -= 400.0
            displacement = material.node_tree.nodes.new('ShaderNodeDisplacement')
            displacement.location = [0.0, image_node_location_y]
            displacement.inputs['Scale'].default_value = 0.05

            material.node_tree.links.new(separate_rgb.outputs['B'], displacement.inputs['Height'])
            material.node_tree.links.new(displacement.outputs['Displacement'], material_output.inputs['Displacement'])

    if has_normal:
        image_node_location_y -= 400.0
        normal_image = create_image_node(material, normal_image_path, False, image_node_location_y)

        if invert_normal:
            separate_rgb = material.node_tree.nodes.new('ShaderNodeSeparateRGB')
            separate_rgb.location = [-900.0, image_node_location_y]
            material.node_tree.links.new(normal_image.outputs['Color'], separate_rgb.inputs['Image'])

            invert = material.node_tree.nodes.new('ShaderNodeInvert')
            invert.location = [-700, image_node_location_y]
            material.node_tree.links.new(separate_rgb.outputs['G'], invert.inputs['Color'])

            combine_rgb = material.node_tree.nodes.new('ShaderNodeCombineRGB')
            combine_rgb.location = [-500, image_node_location_y]
            material.node_tree.links.new(separate_rgb.outputs['R'], combine_rgb.inputs['R'])
            material.node_tree.links.new(invert.outputs['Color'], combine_rgb.inputs['G'])
            material.node_tree.links.new(separate_rgb.outputs['B'], combine_rgb.inputs['B'])

            normal_map = material.node_tree.nodes.new('ShaderNodeNormalMap')
            normal_map.location = [-300, image_node_location_y]
            material.node_tree.links.new(combine_rgb.outputs['Image'], normal_map.inputs['Color'])
            material.node_tree.links.new(normal_map.outputs['Normal'], principled_bsdf.inputs['Normal'])
        else:
            normal_map = material.node_tree.nodes.new('ShaderNodeNormalMap')
            normal_map.location = [-300, image_node_location_y]
            material.node_tree.links.new(normal_image.outputs['Color'], normal_map.inputs['Color'])
            material.node_tree.links.new(normal_map.outputs['Normal'], principled_bsdf.inputs['Normal'])

        if displacement != None:
            material.node_tree.links.new(normal_map.outputs['Normal'], displacement.inputs['Normal'])

    if bpy.app.version >= (3, 0, 0):
        material.asset_generate_preview()

if bpy.app.version >= (3, 0, 0):
    with open(bpy.path.abspath('//blender_assets.cats.txt'), 'w') as f:
        f.write('# MTLD Asset Catalog Definition\n')
        f.write('VERSION 1\n')
        for name, uuid in material_category_ids.items():
            f.write(f'{uuid}:Materials/{name}:{name}\n')
