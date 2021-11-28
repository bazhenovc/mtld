// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use basis_universal::sys::UastcPackFlags_PackUASTCLevelSlower;
use basis_universal::*;
use image::*;
use rayon::iter::*;
use std::fs::*;
use std::io::BufReader;
use std::path::*;

use crate::application_error::*;

pub fn compress_basisu(
    pack_cache_path: &Path,
    force_basisu: bool,
    basisu_single_threaded: bool,
    basisu_cache_path: &Path,
) -> Result<(), ApplicationError> {
    create_dir_all(basisu_cache_path)?;

    let directory_contents = read_dir(&pack_cache_path)?.filter_map(|f| f.ok()).map(|f| f.path());

    if basisu_single_threaded {
        let temp_file_path = basisu_cache_path.join("mtldbasisu.tmp");
        for material_path in directory_contents {
            compress_single_material(&temp_file_path, &material_path, force_basisu, basisu_cache_path)?;
        }
    } else {
        directory_contents
            .collect::<Vec<PathBuf>>()
            .par_iter()
            .map(|material_path| {
                let temp_file_path =
                    basisu_cache_path.join(format!("mtldbasisu{}.tmp", rayon::current_thread_index().unwrap_or(0)));
                compress_single_material(&temp_file_path, material_path, force_basisu, basisu_cache_path)
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;
    }

    Ok(())
}

fn compress_single_material(
    temp_file_path: &Path,
    material_path: &Path,
    force_basisu: bool,
    basisu_cache_path: &Path,
) -> Result<(), ApplicationError> {
    if let Some(material_dir_name) = material_path.file_stem() {
        let material_json = material_path.join("Material.json");
        if material_json.exists() {
            println!("BASISU {:?}", material_dir_name);

            let material: serde_json::Value = serde_json::from_str(&read_to_string(&material_json)?)?;

            let target_path = basisu_cache_path.join(material_dir_name);
            create_dir_all(&target_path)?;

            let target_material_json = target_path.join("Material.json");
            if force_basisu || !target_material_json.exists() {
                copy(&material_json, target_material_json)?;
            }

            let albedo_source_path = material_path.join("Albedo.png");
            if albedo_source_path.exists() {
                let albedo_target_path = target_path.join("Albedo.basisu");
                if force_basisu || !albedo_target_path.exists() {
                    let has_opacity = material.get("opacity").and_then(|f| f.as_bool()).unwrap_or_default();
                    let albedo_image = load(BufReader::new(File::open(&albedo_source_path)?), ImageFormat::Png)?;
                    if has_opacity {
                        if albedo_image.color() != ColorType::Rgba8 {
                            return Err(ApplicationError::InvalidImage(albedo_source_path));
                        }
                    } else if albedo_image.color() != ColorType::Rgb8 {
                        return Err(ApplicationError::InvalidImage(albedo_source_path));
                    }

                    let mut compressor_params = common_compressor_params();
                    compressor_params.set_color_space(ColorSpace::Srgb);
                    compressor_params.source_image_mut(0).init(
                        albedo_image.as_bytes(),
                        albedo_image.width(),
                        albedo_image.height(),
                        if has_opacity { 4 } else { 3 },
                    );

                    let mut compressor = Compressor::new(1);
                    unsafe {
                        compressor.init(&compressor_params);
                        compressor.process()?;
                    }

                    write(&temp_file_path, compressor.basis_file())?;
                    rename(&temp_file_path, albedo_target_path)?;
                }
            }

            let normal_source_path = material_path.join("Normal.png");
            if normal_source_path.exists() {
                let normal_target_path = target_path.join("Normal.basisu");
                if force_basisu || !normal_target_path.exists() {
                    let normal_image = load(BufReader::new(File::open(&normal_source_path)?), ImageFormat::Png)?;
                    if normal_image.color() != ColorType::Rgb8 {
                        return Err(ApplicationError::InvalidImage(normal_source_path));
                    }

                    let mut compressor_params = common_compressor_params();
                    compressor_params.set_color_space(ColorSpace::Linear);
                    compressor_params.tune_for_normal_maps();
                    compressor_params.source_image_mut(0).init(
                        normal_image.as_bytes(),
                        normal_image.width(),
                        normal_image.height(),
                        3,
                    );

                    let mut compressor = Compressor::new(1);
                    unsafe {
                        compressor.init(&compressor_params);
                        compressor.process()?;
                    }

                    write(&temp_file_path, compressor.basis_file())?;
                    rename(&temp_file_path, &normal_target_path)?;
                }
            }

            let material_pack_source_path = material_path.join("MetallicOcclusionDisplacementRoughness.png");
            if material_pack_source_path.exists() {
                let material_pack_target_path = target_path.join("MetallicOcclusionDisplacementRoughness.basisu");
                if force_basisu || !material_pack_target_path.exists() {
                    let material_pack_image = load(
                        BufReader::new(File::open(&material_pack_source_path)?),
                        ImageFormat::Png,
                    )?;
                    if material_pack_image.color() != ColorType::Rgba8 {
                        return Err(ApplicationError::InvalidImage(material_pack_source_path));
                    }

                    let mut compressor_params = common_compressor_params();
                    compressor_params.set_color_space(ColorSpace::Linear);
                    compressor_params.source_image_mut(0).init(
                        material_pack_image.as_bytes(),
                        material_pack_image.width(),
                        material_pack_image.height(),
                        4,
                    );

                    let mut compressor = Compressor::new(1);
                    unsafe {
                        compressor.init(&compressor_params);
                        compressor.process()?;
                    }

                    write(&temp_file_path, compressor.basis_file())?;
                    rename(&temp_file_path, &material_pack_target_path)?;
                }
            }
        }
    }
    Ok(())
}

fn common_compressor_params() -> CompressorParams {
    let mut compressor_params = CompressorParams::new();
    compressor_params.set_basis_format(BasisTextureFormat::UASTC4x4);
    compressor_params.set_uastc_quality_level(UastcPackFlags_PackUASTCLevelSlower);
    compressor_params.set_rdo_uastc(Some(0.5));
    compressor_params.set_generate_mipmaps(true);
    compressor_params
}
