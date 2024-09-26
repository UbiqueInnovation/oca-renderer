// Copyright (c) 2024 Ubique Innovation AG <https://www.ubique.ch>
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::Read;

use base64::Engine;
use zip::read::ZipFile;

use crate::models::{CaptureBase, OcaLayer};

pub trait Said {
    fn set_digest(&mut self, digest: &str);
    fn digest(&self) -> &str;
}

impl Said for OcaLayer {
    fn set_digest(&mut self, digest: &str) {
        match self {
            OcaLayer::CharacterEncoding(character_encoding) => {
                character_encoding.set_digest(digest);
            }
            OcaLayer::Label(label) => label.set_digest(digest),
            OcaLayer::Conformance(conformance) => conformance.set_digest(digest),
            OcaLayer::Format(format) => format.set_digest(digest),
            OcaLayer::Style(style) => style.set_digest(digest),
            OcaLayer::AttributeMapping(attribute_mapping) => attribute_mapping.set_digest(digest),
            OcaLayer::Other(..) => {}
        }
    }

    fn digest(&self) -> &str {
        match self {
            OcaLayer::CharacterEncoding(character_encoding) => character_encoding.digest(),
            OcaLayer::Label(label) => label.digest(),
            OcaLayer::Conformance(conformance) => conformance.digest(),
            OcaLayer::Format(format) => format.digest(),
            OcaLayer::Style(style) => style.digest(),
            OcaLayer::AttributeMapping(attribute_mapping) => attribute_mapping.digest(),
            OcaLayer::Other(value) => value.get("digest").unwrap().as_str().unwrap(),
        }
    }
}

impl OcaLayer {
    pub fn update_digest(&mut self) -> Result<(), String> {
        self.set_digest(&(0..44).map(|_| '#').collect::<String>());
        let result = serde_json::to_string(&self).map_err(|e| format!("{e}"))?;
        let result = calculate_said(&result);
        self.set_digest(&result);
        Ok(())
    }
}
impl CaptureBase {
    pub fn update_digest(&mut self) -> Result<(), String> {
        self.digest = (0..44).map(|_| '#').collect::<String>();
        let result = serde_json::to_string(&self).map_err(|e| format!("{e}"))?;
        let result = calculate_said(&result);
        self.digest = result;
        Ok(())
    }
}

pub fn verify_said(said: &str, mut file: ZipFile) -> Result<bool, String> {
    let mut json_representation = String::new();
    file.read_to_string(&mut json_representation)
        .map_err(|e| format!("{e}"))?;
    verify_said_from_str(said, &json_representation)
}
pub fn verify_said_from_str(said: &str, hash_input: &str) -> Result<bool, String> {
    let hash_input = hash_input.replace(said, &(0..said.len()).map(|_| '#').collect::<String>());
    let hash_input = hash_input.as_bytes();
    let digest_result = blake3::hash(hash_input);
    let said_calculated = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(digest_result.as_bytes());
    let said_calculated = format!("E{said_calculated}");
    Ok(said_calculated == said)
}
pub fn calculate_said(json: &str) -> String {
    let hash_input = json.trim().as_bytes();
    let digest_result = blake3::hash(hash_input);
    let said_calculated = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(digest_result.as_bytes());
    let said_calculated = format!("E{said_calculated}");
    said_calculated
}

#[macro_export]
macro_rules! impl_said {
    ($what:ident) => {
        impl $crate::said::Said for $what {
            fn set_digest(&mut self, digest: &str) {
                self.digest = digest.to_string()
            }
            fn digest(&self) -> &str {
                &self.digest
            }
        }
    };
}