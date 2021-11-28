// Copyright (c) 2021 Kyrylo Bazhenov
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[derive(Debug)]
pub enum ApplicationError {
    InvalidParameter(String),
    InvalidMetadata,
    InvalidImage(std::path::PathBuf),
    MetadataParse(serde_json::Error),
    Network(reqwest::Error),
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Image(image::ImageError),
    BasisUniversal(basis_universal::CompressorErrorCode),
}

impl std::fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<serde_json::Error> for ApplicationError {
    fn from(err: serde_json::Error) -> Self {
        Self::MetadataParse(err)
    }
}

impl From<reqwest::Error> for ApplicationError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err)
    }
}

impl From<std::io::Error> for ApplicationError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<zip::result::ZipError> for ApplicationError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::Zip(err)
    }
}

impl From<image::ImageError> for ApplicationError {
    fn from(err: image::ImageError) -> Self {
        Self::Image(err)
    }
}

impl From<basis_universal::CompressorErrorCode> for ApplicationError {
    fn from(err: basis_universal::CompressorErrorCode) -> Self {
        Self::BasisUniversal(err)
    }
}
