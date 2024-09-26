// Copyright (c) 2024 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use base64::Engine;
use chrono::{NaiveDate, NaiveDateTime};
use wasm_minimal_protocol::{initiate_protocol, wasm_func};

use crate::{models::AttributeMapping, oca::parse_zip};

initiate_protocol!();

#[wasm_func]
pub fn format_date(date: &[u8], fmt_string: &[u8]) -> Result<Vec<u8>, String> {
    let date_string = std::str::from_utf8(date).map_err(|e| format!("{e}"))?;
    let fmt_string = std::str::from_utf8(fmt_string).map_err(|e| format!("{e}"))?;

    if let Ok(d) = NaiveDateTime::parse_from_str(date_string, fmt_string) {
        return Ok(d.to_string().as_bytes().to_vec());
    } else if let Ok(d) = NaiveDate::parse_from_str(date_string, fmt_string) {
        return Ok(d.to_string().as_bytes().to_vec());
    } else {
        return Err("Failed to parse".into());
    }
}

#[wasm_func]
pub fn get_oca(file: &[u8]) -> Result<Vec<u8>, String> {
    let oca = parse_zip(file)?;
    Ok(serde_json::to_string(&oca).unwrap().as_bytes().to_vec())
}
#[wasm_func]
pub fn render(text: &[u8], data: &[u8]) -> Result<Vec<u8>, String> {
    let data: serde_json::Value = serde_json::from_slice(data).map_err(|e| format!("{e}"))?;
    let text = std::str::from_utf8(text).map_err(|e| format!("{e}"))?;
    let template = mustache::compile_str(text).map_err(|e| format!("{e}"))?;
    let rendered_string = template
        .render_to_string(&data)
        .map_err(|e| format!("{e}"))?;
    Ok(rendered_string.as_bytes().to_vec())
}

#[wasm_func]
pub fn decode64(text: &[u8]) -> Result<Vec<u8>, String> {
    base64::prelude::BASE64_STANDARD
        .decode(text)
        .or_else(|_| base64::prelude::BASE64_STANDARD_NO_PAD.decode(text))
        .or_else(|_| base64::prelude::BASE64_URL_SAFE.decode(text))
        .or_else(|_| base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(text))
        .map_err(|e| format!("{e}"))
}
#[wasm_func]
pub fn remap_json(json: &[u8], mapping_layer: &[u8]) -> Result<Vec<u8>, String> {
    let val: serde_json::Value = serde_json::from_slice(json).map_err(|e| format!("{e}"))?;
    let mapping_layer: AttributeMapping =
        serde_json::from_slice(mapping_layer).map_err(|e| format!("{e}"))?;
    let result = mapping_layer.map_json(&val);
    Ok(serde_json::to_string(&result).unwrap().into_bytes())
}
