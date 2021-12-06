// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

use reqwest::blocking::*;
use std::fs::*;
use std::io::{copy, BufWriter, Cursor};

use crate::application_error::*;

pub fn download_ambientcg(
    force_download: bool,
    download_cache_path: &std::path::Path,
    download_resolutions: &[String],
    download_extensions: &[String],
    download_types_request: &str,
    user_agent: &str,
) -> Result<(), ApplicationError> {
    let mut download_types = Vec::with_capacity(download_resolutions.len() * download_extensions.len());
    for resolution in download_resolutions {
        for extension in download_extensions {
            download_types.push(format!("{}-{}", resolution, extension));
        }
    }

    create_dir_all(&download_cache_path)?;
    let temp_file_path = download_cache_path.join("mtldownload.tmp");

    let metadata_path = download_cache_path.join(".mtld");
    create_dir_all(&metadata_path)?;

    let client = Client::builder().user_agent(user_agent).build()?;

    let mut request_offset = 0;
    loop {
        let request = client
            .get(format!(
                "https://ambientcg.com/api/v2/full_json?type={}&offset={}&sort=Latest&include=downloadData",
                download_types_request, request_offset,
            ))
            .send()?;

        println!("GET {} {}", request.url(), request.status());
        if request.status() != 200 {
            break;
        }

        let json_full = request.text()?;
        let metadata: serde_json::Value = serde_json::from_str(&json_full)?;

        let found_assets = metadata
            .as_object()
            .and_then(|f| f.get("foundAssets"))
            .and_then(|f| f.as_array())
            .ok_or(ApplicationError::InvalidMetadata)?;

        if found_assets.is_empty() {
            break;
        }

        request_offset += found_assets.len();

        for asset in found_assets {
            if let Some(asset_id) = asset.get("assetId").and_then(|f| f.as_str()) {
                if let Some(downloads) = asset
                    .get("downloadFolders")
                    .and_then(|f| f.get("/"))
                    .and_then(|f| f.get("downloadFiletypeCategories"))
                    .and_then(|f| f.get("zip"))
                    .and_then(|f| f.get("downloads"))
                    .and_then(|f| f.as_array())
                {
                    for download_type in &download_types {
                        if let Some(download) =
                            downloads
                                .iter()
                                .find(|download| match download.get("attribute").and_then(|f| f.as_str()) {
                                    Some(attribute) => attribute == download_type,
                                    None => false,
                                })
                        {
                            if let Some(download_link) = download.get("fullDownloadPath").and_then(|f| f.as_str()) {
                                let zip_path = download_cache_path.join(asset_id).with_extension("zip");
                                if force_download || !zip_path.exists() {
                                    match client.get(download_link).send() {
                                        Ok(download_data) => {
                                            println!("GET {}", download_link);
                                            use std::io::Write;

                                            let mut cursor = Cursor::new(download_data.bytes()?);
                                            let mut file = BufWriter::new(File::create(&temp_file_path)?);
                                            copy(&mut cursor, &mut file)?;
                                            file.flush()?;
                                            rename(&temp_file_path, &zip_path)?;
                                        }
                                        Err(e) => println!("ERR {} {:?}", asset_id, e),
                                    }
                                }

                                let json_path = metadata_path.join(asset_id).with_extension("json");
                                if !json_path.exists() {
                                    write(
                                        &temp_file_path,
                                        format!(
                                            concat!(
                                                "{{\n",
                                                " \"category\": \"{}\",\n",
                                                " \"type\": \"{}\",\n",
                                                " \"method\": \"{}\"\n",
                                                "}}"
                                            ),
                                            asset.get("category").and_then(|f| f.as_str()).unwrap_or("null"),
                                            asset.get("dataType").and_then(|f| f.as_str()).unwrap_or("null"),
                                            asset.get("creationMethod").and_then(|f| f.as_str()).unwrap_or("null")
                                        ),
                                    )?;
                                    rename(&temp_file_path, &json_path)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
