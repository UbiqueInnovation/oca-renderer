// Copyright (c) 2024 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::impl_said;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Oca {
    pub(crate) capture_base: CaptureBase,
    pub(crate) overlays: Vec<(String, OcaLayer)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaptureBase {
    r#type: String,
    pub(crate) digest: String,
    pub(crate) classification: Option<String>,
    pub(crate) attributes: BTreeMap<String, String>,
    pub(crate) flagged_attributes: Vec<String>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CharacterEncoding {
    capture_base: String,
    digest: String,
    r#type: String,
    default_character_encoding: Encoding,
    attribute_character_encoding: BTreeMap<String, Encoding>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Encoding {
    #[serde(rename = "utf-8")]
    Utf8,
    #[serde(rename = "base64")]
    Base64,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Conformance {
    capture_base: String,
    digest: String,
    r#type: String,
    attribute_conformance: BTreeMap<String, ConformancePolicy>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConformancePolicy {
    M,
    O,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Label {
    capture_base: String,
    digest: String,
    r#type: String,
    language: String,
    attribute_labels: BTreeMap<String, String>,
    attribute_categories: Vec<String>,
    category_labels: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Format {
    capture_base: String,
    digest: String,
    r#type: String,
    attribute_formats: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Style {
    capture_base: String,
    digest: String,
    r#type: String,
    style_json: StyleJson,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StyleJson {
    pub(crate) title: String,
    pub(crate) subtitle: String,
    pub(crate) card_color: u64,
    pub(crate) text_color: String,
    pub(crate) background_card: Option<String>,
    pub(crate) ordered_properties: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AttributeMapping {
    capture_base: String,
    digest: String,
    r#type: String,
    attribute_mapping: BTreeMap<String, String>,
}

impl AttributeMapping {
    pub fn map_json(&self, data: &Value) -> Value {
        let mut new_object = serde_json::Map::new();
        let old_object_selector = &mut jsonpath_lib::selector(data);
        for (key, mapped_key) in &self.attribute_mapping {
            let Ok(val) = old_object_selector(&format!("$.{mapped_key}")) else {
                println!("selector failed {mapped_key}");
                continue;
            };
            let Some(first_val) = val.first() else {
                println!("nothing found at {mapped_key}");
                continue;
            };
            if key != "$" {
                new_object.insert(key.to_string(), (*first_val).clone());
                continue;
            }
            // for now only remap objects
            if let Value::Object(inner) = first_val {
                for (inner_key, inner_val) in inner {
                    new_object.insert(inner_key.to_string(), inner_val.clone());
                }
            }
        }
        Value::Object(new_object)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum OcaLayer {
    CharacterEncoding(CharacterEncoding),
    Label(Label),
    Conformance(Conformance),
    Format(Format),
    Style(Style),
    AttributeMapping(AttributeMapping),
    Other(Value),
}

impl Conformance {
    pub fn new(
        capture_base: &str,
        attribute_conformance: BTreeMap<String, ConformancePolicy>,
    ) -> Self {
        Self {
            capture_base: capture_base.to_string(),
            r#type: "spec/overlays/conformance/1.0".to_string(),
            digest: "".to_string(),
            attribute_conformance,
        }
    }
}
impl CaptureBase {
    pub fn new(attributes: BTreeMap<String, String>, flagged_attributes: Vec<String>) -> Self {
        Self {
            r#type: "spec/capture_base/1.0".into(),
            digest: "".into(),
            classification: Some("GICS:45102010".into()),
            attributes,
            flagged_attributes,
        }
    }
}

impl OcaLayer {
    pub fn new_label_layer(
        capture_base: &str,
        language: &str,
        attribute_labels: BTreeMap<String, String>,
        attribute_categories: Vec<String>,
        category_labels: BTreeMap<String, String>,
    ) -> OcaLayer {
        OcaLayer::Label(Label {
            capture_base: capture_base.to_string(),
            digest: "".into(),
            r#type: "spec/overlays/label/1.0".into(),
            language: language.into(),
            attribute_labels,
            attribute_categories,
            category_labels,
        })
    }
    pub fn new_style_layer(capture_base: &str, style_json: StyleJson) -> Self {
        Self::Style(Style {
            capture_base: capture_base.to_string(),
            digest: "".into(),
            r#type: "spec/overlays/style/1.0".into(),
            style_json,
        })
    }
    pub fn new_format_layer(
        capture_base: &str,
        attribute_formats: BTreeMap<String, String>,
    ) -> Self {
        Self::Format(Format {
            capture_base: capture_base.to_string(),
            digest: "".into(),
            r#type: "spec/overlays/format/1.0".into(),
            attribute_formats,
        })
    }
    pub fn new_character_encoding(
        capture_base: &str,
        attribute_character_encoding: BTreeMap<String, Encoding>,
    ) -> Self {
        Self::CharacterEncoding(CharacterEncoding {
            capture_base: capture_base.into(),
            digest: "".into(),
            r#type: "spec/overlays/character_encoding/1.0".into(),
            default_character_encoding: Encoding::Utf8,
            attribute_character_encoding,
        })
    }
    pub fn new_attribute_mapping_layer(
        capture_base_digest: &str,
        attribute_mapping: BTreeMap<String, String>,
    ) -> Self {
        Self::AttributeMapping(AttributeMapping {
            capture_base: capture_base_digest.into(),
            digest: "".to_string(),
            r#type: "spec/overlays/attribute_mapping/1.0".to_string(),
            attribute_mapping,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StyleJsonFile {
    pub(crate) attributes: BTreeMap<String, Attribute>,
    pub(crate) style: StyleJson,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub(crate) display_name: String,
    pub(crate) field_type: AttributeFieldType,
}
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "UPPERCASE")]
pub enum AttributeFieldType {
    String,
    Date,
    DateTime,
    Time,
    DateOfBirth,
    Image,
    Boolean,
    Number,
    #[serde(other)]
    Other,
}

impl_said!(CharacterEncoding);
impl_said!(Label);
impl_said!(Conformance);
impl_said!(Format);
impl_said!(Style);
impl_said!(AttributeMapping);

#[cfg(test)]
mod test {
    use serde_json::Value;

    use super::AttributeMapping;

    #[test]
    fn test_mapping() {
        let layer = "{\n  \"capture_base\": \"EfJV8Si2m5KIEX-1VeLa0xsF2iuKn8e6VMLaNf36FErA\",\n  \"digest\": \"Epy2Tl3gXrFO9_7NkmJcEjryuIccs0SFi3WoUIUfh8Y8\",\n  \"type\": \"spec/overlays/attribute_mapping/1.0\",\n  \"attribute_mapping\": {\n    \"$\": \"vc\" }\n}";
        let mapping_layer: AttributeMapping = serde_json::from_str(layer).unwrap();
        let value : Value = serde_json::from_str("{\"vc\" : { \"givenName\" : \"Manfred\", \"surname\" : \"Mustermann\", \"dateOfBirth\" : \"20001010\" }}").unwrap();
        let new_val = mapping_layer.map_json(&value);
        println!("{new_val}");
    }
}
