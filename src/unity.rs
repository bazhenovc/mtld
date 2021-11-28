// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::fs::*;
use std::path::*;
use uuid::*;

use crate::application_error::*;

pub fn generate_unity(
    pack_cache_path: &Path,
    force_generate_unity: bool,
    unity_cache_path: &Path,
    unity_texture_template: &Path,
    unity_material_template: &Path,
) -> Result<(), ApplicationError> {
    create_dir_all(unity_cache_path)?;

    let texture_template = read_to_string(unity_texture_template)?;
    let material_template = read_to_string(unity_material_template)?;

    for dir in read_dir(pack_cache_path)? {
        let dir = dir?;
        let material_path = dir.path();

        if let Some(material_dir_name) = material_path.file_stem() {
            let material_json = material_path.join("Material.json");
            if material_json.exists() {
                let material: serde_json::Value = serde_json::from_str(&read_to_string(&material_json)?)?;

                if let Some(material_name) = material.get("name").and_then(|f| f.as_str()) {
                    if material_name == material_dir_name {
                        println!("UNITY {}", material_name);

                        let target_path = unity_cache_path.join(material_name);
                        create_dir_all(&target_path)?;

                        let has_albedo = material.get("albedo").and_then(|f| f.as_bool()).unwrap_or_default();
                        let has_opacity = material.get("opacity").and_then(|f| f.as_bool()).unwrap_or_default();
                        let has_normal = material.get("normal").and_then(|f| f.as_bool()).unwrap_or_default();
                        let has_metalness = material.get("metalness").and_then(|f| f.as_bool()).unwrap_or_default();
                        let has_roughness = material.get("roughness").and_then(|f| f.as_bool()).unwrap_or_default();
                        let has_ao = material.get("ao").and_then(|f| f.as_bool()).unwrap_or_default();
                        let has_displacement = material
                            .get("displacement")
                            .and_then(|f| f.as_bool())
                            .unwrap_or_default();

                        let has_material_pack = has_metalness || has_roughness || has_ao || has_displacement;

                        let material_file_path = target_path.join(format!("{}.mat", material_name));
                        if force_generate_unity || !material_file_path.exists() {
                            let albedo_uuid = Uuid::new_v4().to_string().replace("-", "");
                            if has_albedo {
                                let albedo_path = target_path.join("Albedo.png.meta");
                                write(
                                    &albedo_path,
                                    texture_template
                                        .replace("$$TEXTURE_GUID$$", &albedo_uuid)
                                        .replace("$$TEXTURE_SRGB$$", "1")
                                        .replace("$$TEXTURE_ALPHA$$", if has_opacity { "1" } else { "0" })
                                        .replace("$$TEXTURE_OPACITY$$", if has_opacity { "1" } else { "0" })
                                        .replace("$$TEXTURE_TYPE$$", "0"),
                                )?;
                            }

                            let normal_uuid = Uuid::new_v4().to_string().replace("-", "");
                            if has_normal {
                                let normal_path = target_path.join("Normal.png.meta");
                                write(
                                    &normal_path,
                                    texture_template
                                        .replace("$$TEXTURE_GUID$$", &normal_uuid)
                                        .replace("$$TEXTURE_SRGB$$", "0")
                                        .replace("$$TEXTURE_ALPHA$$", "0")
                                        .replace("$$TEXTURE_OPACITY$$", "0")
                                        .replace("$$TEXTURE_TYPE$$", "1"),
                                )?;
                            }

                            let material_pack_uuid = Uuid::new_v4().to_string().replace("-", "");
                            if has_material_pack {
                                let material_pack_path =
                                    target_path.join("MetallicOcclusionDisplacementRoughness.png.meta");
                                write(
                                    &material_pack_path,
                                    texture_template
                                        .replace("$$TEXTURE_GUID$$", &material_pack_uuid)
                                        .replace("$$TEXTURE_SRGB$$", "0")
                                        .replace("$$TEXTURE_ALPHA$$", "1")
                                        .replace("$$TEXTURE_OPACITY$$", "0")
                                        .replace("$$TEXTURE_TYPE$$", "0"),
                                )?;
                            }

                            let mut keywords = String::new();
                            if has_opacity {
                                keywords.push_str(" _ALPHATEST_ON");
                            }
                            if has_normal {
                                keywords.push_str(" _NORMALMAP");
                            }
                            if has_roughness || has_metalness {
                                keywords.push_str(" _METALLICSPECGLOSSMAP");
                            }
                            if has_ao {
                                keywords.push_str(" _OCCLUSIONMAP");
                            }

                            write(
                                &material_file_path,
                                material_template
                                    .replace("$$MATERIAL_NAME$$", material_name)
                                    .replace(
                                        "$$SHADER_KEYWORDS$$",
                                        if keywords.is_empty() { "" } else { &keywords[1..] },
                                    )
                                    .replace(
                                        "$$RENDER_TYPE$$",
                                        if has_opacity { "TransparentCutout" } else { "Opaque" },
                                    )
                                    .replace("$$COLOR_TEXTURE$$", &format_filename(has_albedo, &albedo_uuid))
                                    .replace("$$NORMAL_TEXTURE$$", &format_filename(has_normal, &normal_uuid))
                                    .replace(
                                        "$$METALLIC_GLOSS_TEXTURE$$",
                                        &format_filename(has_roughness || has_metalness, &material_pack_uuid),
                                    )
                                    .replace("$$AO_TEXTURE$$", &format_filename(has_ao, &material_pack_uuid))
                                    .replace("$$ALPHA_CLIP$$", if has_opacity { "1" } else { "0" }),
                            )?;
                        }
                    } else {
                        println!("WARN: {:?} != {}", material_dir_name, material_name);
                    }
                }
            }
        }
    }
    Ok(())
}

fn format_filename(exists: bool, uuid: &str) -> String {
    if exists {
        format!("{{fileID: 2800000, guid: {}, type: 3}}", uuid)
    } else {
        "{fileID: 0}".to_string()
    }
}
