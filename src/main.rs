// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use structopt::*;

use std::path::*;
use std::str::FromStr;

mod application_error;
mod basisu;
mod download;
mod pack;
mod unity;

use crate::application_error::*;

#[derive(Debug, StructOpt)]
struct CommandLineOptions {
    #[structopt(long = "download", help = "Downloads .zip files")]
    download: bool,

    #[structopt(
        long = "force-download",
        help = "Forces download even when .zip files exist in the cache"
    )]
    force_download: bool,

    #[structopt(
        long = "download-cache",
        help = "Folder where downloaded .zip files will be stored",
        default_value = ".mtld-download-cache",
        parse(from_os_str)
    )]
    download_cache_path: PathBuf,

    #[structopt(
        long = "download-resolutions",
        help = "Download using the first resolution from this list, try next one if failed",
        default_value = "4K,3K,2K,1K"
    )]
    download_resolutions: ArgumentVec,

    #[structopt(
        long = "download-extensions",
        help = "Download using the first extension form this list, try next one if failed",
        default_value = "JPG,PNG"
    )]
    download_extensions: ArgumentVec,

    #[structopt(
        long = "download-type",
        help = "Specifies types of assets to be downloaded",
        default_value = "PhotoTexturePBR,DecalPBR,AtlasPBR"
    )]
    download_type: String,

    #[structopt(
        long = "user-agent",
        help = "Override default User-Agent header when making HTTP requests",
        default_value = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/44.0.2403.157 Safari/537.36"
    )]
    user_agent: String,

    #[structopt(long = "pack", help = "Packs downloaded files")]
    pack: bool,

    #[structopt(long = "force-pack", help = "Force packing even when files exist")]
    force_pack: bool,

    #[structopt(long = "pack-single-threaded", help = "Don't use multi threading for packing")]
    pack_single_threaded: bool,

    #[structopt(
        long = "pack-cache-path",
        help = "Folder where packed files will be stored",
        default_value = ".mtld-pack-cache",
        parse(from_os_str)
    )]
    pack_cache_path: PathBuf,

    #[structopt(
        long = "pack-normal-map-type",
        help = "Normal map type to use for packing",
        default_value = "OpenGL"
    )]
    pack_normal_map_type: crate::pack::NormalMapType,

    #[structopt(long = "pack-target-width", help = "Packed image width", default_value = "1024")]
    pack_target_width: u32,

    #[structopt(long = "pack-target-height", help = "Packed image height", default_value = "1024")]
    pack_target_height: u32,

    #[structopt(long = "basisu", help = "Compresses packed files with Basis Universal")]
    basisu: bool,

    #[structopt(
        long = "force-basisu",
        help = "Force compressing with Basis Universal even when files exist"
    )]
    force_basisu: bool,

    #[structopt(
        long = "basisu-single-threaded",
        help = "Dont't use multithreading for compressing with Basis Universal"
    )]
    basisu_single_threaded: bool,

    #[structopt(
        long = "basisu-cache-path",
        help = "Folder where compressed Basis Universal files will be stored",
        default_value = ".mtld-basisu-cache",
        parse(from_os_str)
    )]
    basisu_cache_path: PathBuf,

    #[structopt(long = "unity", help = "Generates Unity3D meta files")]
    unity: bool,

    #[structopt(
        long = "force-unity",
        help = "Force generate Unity3D meta files even when files exist"
    )]
    force_unity: bool,

    #[structopt(
        long = "unity-cache-path",
        help = "Folder where Unity3D meta files will be stored",
        default_value = ".mtld-unity-cache",
        parse(from_os_str)
    )]
    unity_cache_path: PathBuf,

    #[structopt(
        long = "unity-texture-template",
        help = "Template file to generate Unity3D texture meta files",
        default_value = "templates/unity_texture.template",
        parse(from_os_str)
    )]
    unity_texture_template: PathBuf,

    #[structopt(
        long = "unity-material-template",
        help = "Template file to generate Unity3D material meta files",
        default_value = "templates/unity_material.template",
        parse(from_os_str)
    )]
    unity_material_template: PathBuf,
}

#[derive(Debug, PartialEq)]
struct ArgumentVec(Vec<String>);

impl FromStr for ArgumentVec {
    type Err = ApplicationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.split(',').map(|x| x.trim().to_owned()).collect()))
    }
}

fn main() -> Result<(), ApplicationError> {
    let command_line = CommandLineOptions::from_args();

    if command_line.download || command_line.force_download {
        download::download_ambientcg(
            command_line.force_download,
            &command_line.download_cache_path,
            &command_line.download_resolutions.0,
            &command_line.download_extensions.0,
            &command_line.download_type,
            &command_line.user_agent,
        )?;
    }

    if command_line.pack || command_line.force_pack {
        pack::pack(
            &command_line.download_cache_path,
            command_line.force_pack,
            command_line.pack_single_threaded,
            &command_line.pack_cache_path,
            command_line.pack_normal_map_type,
            command_line.pack_target_width,
            command_line.pack_target_height,
        )?;
    }

    if command_line.basisu || command_line.force_basisu {
        basisu::compress_basisu(
            &command_line.pack_cache_path,
            command_line.force_basisu,
            command_line.basisu_single_threaded,
            &command_line.basisu_cache_path,
        )?;
    }

    if command_line.unity || command_line.force_unity {
        unity::generate_unity(
            &command_line.pack_cache_path,
            command_line.unity,
            &command_line.unity_cache_path,
            &command_line.unity_texture_template,
            &command_line.unity_material_template,
        )?;
    }

    Ok(())
}
