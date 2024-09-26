// Copyright (c) 2024 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::io::{Cursor, Write};

use serde_json::{Map, Value};
use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    models::{CaptureBase, Oca, OcaLayer},
    said::{verify_said, Said},
};

pub fn generate_zip(oca: Oca) -> Result<Vec<u8>, String> {
    let mut meta_map = Map::new();
    meta_map.insert(
        "root".to_string(),
        Value::String(oca.capture_base.digest.clone()),
    );
    let mut root_files = Map::new();
    let mut overlays = Map::new();
    let mut overlay_files = vec![];
    for (name, mut overlay) in oca.overlays {
        if overlay.update_digest().is_err() {
            return Err("Failed updating said".to_string());
        }
        let digest = overlay.digest().to_string();
        overlays.insert(name, Value::String(digest.clone()));
        overlay_files.push((
            format!("{digest}.json"),
            serde_json::to_string(&overlay)
                .map_err(|e| format!("{e}"))?
                .as_bytes()
                .to_vec(),
        ));
    }
    root_files.insert(oca.capture_base.digest.clone(), Value::Object(overlays));
    meta_map.insert("files".to_string(), Value::Object(root_files));
    let meta_file = serde_json::to_string(&Value::Object(meta_map)).map_err(|e| format!("{e}"))?;
    let mut archive_buffer = vec![];
    {
        let mut zip_archive = ZipWriter::new(Cursor::new(&mut archive_buffer));
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
        zip_archive
            .start_file("meta.json", options)
            .map_err(|e| format!("{e}"))?;
        zip_archive
            .write_all(meta_file.as_bytes())
            .map_err(|e| format!("{e}"))?;

        zip_archive
            .start_file(format!("{}.json", oca.capture_base.digest), options)
            .map_err(|e| format!("{e}"))?;
        zip_archive
            .write_all(
                serde_json::to_string(&oca.capture_base)
                    .map_err(|e| format!("{e}"))?
                    .as_bytes(),
            )
            .map_err(|e| format!("{e}"))?;
        for (name, file) in overlay_files {
            zip_archive
                .start_file(name, options)
                .map_err(|e| format!("{e}"))?;
            zip_archive.write_all(&file).map_err(|e| format!("{e}"))?;
        }
    }

    Ok(archive_buffer)
}

pub fn parse_zip(file: &[u8]) -> Result<Oca, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(file)).unwrap();
    let meta = archive.by_name("meta.json").unwrap();
    let meta: serde_json::Value = serde_json::from_reader(meta).unwrap();
    let root = meta.get("root").unwrap().as_str().unwrap();
    let capture_base = archive.by_name(&format!("{root}.json")).unwrap();
    assert!(verify_said(root, capture_base).unwrap());
    let capture_base = archive.by_name(&format!("{root}.json")).unwrap();
    let capture_base: CaptureBase =
        serde_json::from_reader(capture_base).map_err(|e| format!("{e}"))?;
    let mut overlays = vec![];
    for (key, value) in meta
        .get("files")
        .unwrap()
        .get(root)
        .unwrap()
        .as_object()
        .unwrap()
    {
        let Some(value) = value.as_str() else {
            println!("value not str");
            continue;
        };
        let Ok(layer_zip_file) = archive.by_name(&format!("{value}.json")) else {
            println!("Archive not found {key}");
            continue;
        };
        if !verify_said(value, layer_zip_file).unwrap_or(false) {
            let var_name = Err("SAID failed".to_string());
            return var_name;
        }
        let Ok(layer_zip_file) = archive.by_name(&format!("{value}.json")) else {
            println!("Archive not found {key}");
            continue;
        };

        let layer: OcaLayer =
            serde_json::from_reader(layer_zip_file).map_err(|e| format!("{e}"))?;
        overlays.push((key.to_string(), layer));
    }
    Ok(Oca {
        capture_base,
        overlays,
    })
}
