// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use image::{imageops::*, *};
use itertools::izip;
use rayon::iter::*;
use std::fs::*;
use std::io::{copy, BufReader, Read, Seek};
use std::path::*;
use std::str::FromStr;
use zip::read::*;

use crate::application_error::*;

#[derive(Debug, Clone, Copy)]
pub enum NormalMapType {
    OpenGL,
    Direct3D,
}

impl FromStr for NormalMapType {
    type Err = ApplicationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OpenGL" => Ok(Self::OpenGL),
            "Direct3D" => Ok(Self::Direct3D),
            _ => Err(ApplicationError::InvalidParameter(s.to_string())),
        }
    }
}

pub fn pack(
    download_cache_path: &Path,
    force_pack: bool,
    pack_single_threaded: bool,
    pack_cache_path: &Path,
    pack_normal_map_type: NormalMapType,
    pack_target_width: u32,
    pack_target_height: u32,
) -> Result<(), ApplicationError> {
    create_dir_all(&pack_cache_path)?;

    let directory_contents = read_dir(&download_cache_path)?.filter_map(|f| f.ok()).map(|f| f.path());

    if pack_single_threaded {
        let temp_file_path = pack_cache_path.join("mtldpack.tmp");
        for zip_path in directory_contents {
            pack_single_image(
                &temp_file_path,
                &zip_path,
                force_pack,
                pack_cache_path,
                pack_normal_map_type,
                pack_target_width,
                pack_target_height,
            )?;
        }
    } else {
        directory_contents
            .collect::<Vec<PathBuf>>()
            .par_iter()
            .map(|zip_path| {
                let temp_file_path =
                    pack_cache_path.join(format!("mtldpack{}.tmp", rayon::current_thread_index().unwrap_or(0)));
                pack_single_image(
                    &temp_file_path,
                    zip_path,
                    force_pack,
                    pack_cache_path,
                    pack_normal_map_type,
                    pack_target_width,
                    pack_target_height,
                )
            })
            .collect::<Result<Vec<_>, ApplicationError>>()?;
    }
    Ok(())
}

fn pack_single_image(
    temp_file_path: &Path,
    zip_path: &Path,
    force_pack: bool,
    pack_cache_path: &Path,
    pack_normal_map_type: NormalMapType,
    pack_target_width: u32,
    pack_target_height: u32,
) -> Result<(), ApplicationError> {
    if let Some(zip_name) = zip_path.file_stem() {
        println!("PACK {:?}", zip_name);

        let mut zip_archive = ZipArchive::new(BufReader::new(File::open(zip_path)?))?;
        let mut albedo_image = None;
        let mut opacity_image = None;
        let mut normal_image = None;
        let mut metalness_image = None;
        let mut roughness_image = None;
        let mut ao_image = None;
        let mut displacement_image = None;

        for image_index in 0..zip_archive.len() {
            let file = zip_archive.by_index_raw(image_index)?;
            if let Some(image_type) = file.name().split('.').next() {
                if image_type.ends_with("_Color") {
                    albedo_image = Some(image_index);
                } else if image_type.ends_with("_Opacity") {
                    opacity_image = Some(image_index);
                } else if image_type.ends_with("_Metalness") {
                    metalness_image = Some(image_index);
                } else if image_type.ends_with("_Roughness") {
                    roughness_image = Some(image_index);
                } else if image_type.ends_with("_AmbientOcclusion") {
                    ao_image = Some(image_index);
                } else if image_type.ends_with("_Displacement") {
                    displacement_image = Some(image_index);
                } else {
                    match pack_normal_map_type {
                        NormalMapType::OpenGL => {
                            if image_type.ends_with("_NormalGL") {
                                normal_image = Some(image_index);
                            }
                        }

                        NormalMapType::Direct3D => {
                            if image_type.ends_with("_NormalDX") {
                                normal_image = Some(image_index);
                            }
                        }
                    }
                }
            }
        }

        let target_path = pack_cache_path.join(zip_name);
        create_dir_all(&target_path)?;

        let material_json_path = target_path.join("Material.json");
        if force_pack || !material_json_path.exists() {
            write(
                &material_json_path,
                &format!(
                    concat!(
                        "{{\n",
                        " \"name\": {:?},\n",
                        " \"albedo\": {},\n",
                        " \"opacity\": {},\n",
                        " \"normal\": {},\n",
                        " \"metalness\": {},\n",
                        " \"roughness\": {},\n",
                        " \"ao\": {},\n",
                        " \"displacement\": {}\n",
                        "}}",
                    ),
                    zip_name,
                    albedo_image.is_some(),
                    opacity_image.is_some(),
                    normal_image.is_some(),
                    metalness_image.is_some(),
                    roughness_image.is_some(),
                    ao_image.is_some(),
                    displacement_image.is_some(),
                ),
            )?;
        }

        let albedo_image_path = target_path.join("Albedo.png");
        if force_pack || !albedo_image_path.exists() {
            if let Some(albedo_image) = albedo_image {
                let albedo_image = decompress_image(&mut zip_archive, albedo_image)?
                    .resize_exact(pack_target_width, pack_target_height, FilterType::Lanczos3)
                    .into_rgb8();
                if let Some(opacity_image) = opacity_image {
                    let opacity_image = decompress_image(&mut zip_archive, opacity_image)?
                        .resize_exact(pack_target_width, pack_target_height, FilterType::Lanczos3)
                        .into_luma8();

                    let mut target_albedo_image = RgbaImage::new(pack_target_width, pack_target_height);

                    for (target, color, opacity) in izip!(
                        target_albedo_image.pixels_mut(),
                        albedo_image.pixels(),
                        opacity_image.pixels()
                    ) {
                        target[0] = color[0];
                        target[1] = color[1];
                        target[2] = color[2];
                        target[3] = opacity[0];
                    }

                    target_albedo_image.save_with_format(&temp_file_path, ImageFormat::Png)?;
                } else {
                    albedo_image.save_with_format(&temp_file_path, ImageFormat::Png)?;
                }

                rename(&temp_file_path, &albedo_image_path)?;
            }
        }

        let normal_image_path = target_path.join("Normal.png");
        if force_pack || !normal_image_path.exists() {
            if let Some(normal_image) = normal_image {
                let normal_image = decompress_image(&mut zip_archive, normal_image)?
                    .resize_exact(pack_target_width, pack_target_height, FilterType::Lanczos3)
                    .into_rgb8();
                normal_image.save_with_format(&temp_file_path, ImageFormat::Png)?;
                rename(&temp_file_path, &normal_image_path)?;
            }
        }

        if roughness_image.is_some() || metalness_image.is_some() || displacement_image.is_some() || ao_image.is_some()
        {
            let material_pack_image_path = target_path.join("MetallicOcclusionDisplacementRoughness.png");
            if force_pack || !material_pack_image_path.exists() {
                let mut material_pack_image = RgbaImage::new(pack_target_width, pack_target_height);

                if let Some(metalness_image) = metalness_image {
                    let metalness_image = decompress_image(&mut zip_archive, metalness_image)?
                        .resize_exact(pack_target_width, pack_target_height, FilterType::Lanczos3)
                        .into_luma8();

                    for (target, metalness) in
                        itertools::izip!(material_pack_image.pixels_mut(), metalness_image.pixels())
                    {
                        target[0] = metalness[0];
                    }
                }

                if let Some(ao_image) = ao_image {
                    let ao_image = decompress_image(&mut zip_archive, ao_image)?
                        .resize_exact(pack_target_width, pack_target_height, FilterType::Lanczos3)
                        .into_luma8();

                    for (target, ao) in izip!(material_pack_image.pixels_mut(), ao_image.pixels()) {
                        target[1] = ao[0];
                    }
                }

                if let Some(displacement_image) = displacement_image {
                    let displacement_image = decompress_image(&mut zip_archive, displacement_image)?
                        .resize_exact(pack_target_width, pack_target_height, FilterType::Lanczos3)
                        .into_luma8();

                    for (target, displacement) in
                        itertools::izip!(material_pack_image.pixels_mut(), displacement_image.pixels())
                    {
                        target[2] = displacement[0];
                    }
                }

                if let Some(roughness_image) = roughness_image {
                    let roughness_image = decompress_image(&mut zip_archive, roughness_image)?
                        .resize_exact(pack_target_width, pack_target_height, FilterType::Lanczos3)
                        .into_luma8();

                    for (target, roughness) in
                        itertools::izip!(material_pack_image.pixels_mut(), roughness_image.pixels())
                    {
                        target[3] = roughness[0];
                    }
                }

                material_pack_image.save_with_format(&temp_file_path, ImageFormat::Png)?;
                rename(&temp_file_path, &material_pack_image_path)?;
            }
        }
    }

    Ok(())
}

fn decompress_image<R: Read + Seek>(
    zip_archive: &mut ZipArchive<R>,
    image_index: usize,
) -> Result<DynamicImage, ApplicationError> {
    let mut image_file = zip_archive.by_index(image_index)?;

    let mut image_data = Vec::with_capacity(image_file.size() as _);
    copy(&mut image_file, &mut image_data)?;

    Ok(load_from_memory(&image_data)?)
}
