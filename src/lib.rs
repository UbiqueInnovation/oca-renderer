use std::collections::BTreeMap;

use models::{AttributeFieldType, CaptureBase, Encoding, Oca, OcaLayer, StyleJsonFile};

pub mod models;
pub mod oca;
pub mod said;
#[cfg(feature = "typst-plugin")]
pub mod typst;
#[cfg(feature = "typst-renderer")]
pub mod typst_renderer;

#[cfg(feature = "ureq")]
pub fn oca_from_style(url: &str) -> Result<Oca, String> {
    let style = ureq::get(url).call().map_err(|e| format!("{e}"))?;
    let style_json = style
        .into_json::<StyleJsonFile>()
        .map_err(|e| format!("{e}"))?;
    oca_from_style_json(style_json)
}

pub fn oca_from_style_json(style_json: StyleJsonFile) -> Result<Oca, String> {
    let mut attributes = BTreeMap::<String, String>::new();
    let mut vc_attributes = style_json.attributes.into_iter().collect::<Vec<(_, _)>>();
    vc_attributes.sort_by(|a, b| {
        style_json
            .style
            .ordered_properties
            .iter()
            .position(|c| c == &a.0)
            .cmp(
                &style_json
                    .style
                    .ordered_properties
                    .iter()
                    .position(|c| c == &b.0),
            )
    });
    let mut attr_desc = BTreeMap::<String, String>::new();
    let mut format_desc = BTreeMap::<String, String>::new();
    let mut attr_enc = BTreeMap::<String, Encoding>::new();

    for (key, value) in &vc_attributes {
        let ty: String = match value.field_type {
            AttributeFieldType::Date
            | AttributeFieldType::DateOfBirth
            | AttributeFieldType::DateTime
            | AttributeFieldType::Time => "DateTime".into(),
            _ => "Text".into(),
        };
        let format: Option<String> = match value.field_type {
            AttributeFieldType::Date | AttributeFieldType::DateOfBirth => Some("%Y%m%d".into()),
            AttributeFieldType::DateTime => Some("%Y%m%d%H%M%S".into()),
            AttributeFieldType::Time => Some("%H%M%S".into()),
            AttributeFieldType::Image => Some("image/png".into()),
            _ => None,
        };
        if let Some(fmt) = format {
            format_desc.insert(key.into(), fmt);
            if let AttributeFieldType::Image = value.field_type {
                attr_enc.insert(key.into(), Encoding::Base64);
            }
        }
        attributes.insert(key.into(), ty);
        attr_desc.insert(key.into(), value.display_name.to_string());
    }
    let mut capture_base = CaptureBase::new(attributes, vec![]);
    capture_base.update_digest().unwrap();
    let capture_base_digest = capture_base.digest.clone();
    let label_layer = OcaLayer::new_label_layer(
        &capture_base_digest,
        "en",
        attr_desc,
        vec![],
        BTreeMap::new(),
    );
    let style_layer = OcaLayer::new_style_layer(&capture_base_digest, style_json.style);

    let format_layer = OcaLayer::new_format_layer(&capture_base_digest, format_desc);
    let encoding_layer = OcaLayer::new_character_encoding(&capture_base_digest, attr_enc);
    Ok(Oca {
        capture_base,
        overlays: vec![
            ("label (en)".into(), label_layer),
            ("style".into(), style_layer),
            ("format".into(), format_layer),
            ("encoding".into(), encoding_layer),
        ],
    })
}

#[cfg(test)]
mod tests {
    use models::{Conformance, StyleJson};
    use oca::{generate_zip, parse_zip};
    use said::{verify_said_from_str, Said};

    use super::*;

    // #[test]
    // fn it_works() {
    //     let test_file = include_bytes!("../test.oca.zip");
    //     let result = parse_zip(test_file).unwrap();
    //     println!("{}", serde_json::to_string(&result).unwrap());
    // }

    // #[test]
    // fn generate_zip_test() {
    //     let test_file = include_bytes!("../test.oca.zip");
    //     let result = parse_zip(test_file).unwrap();
    //     let zip = generate_zip(result).unwrap();
    //     std::fs::write("new.zip", zip).unwrap();
    //     // assert_eq!(&test_file[..], &zip[..])
    // }
    #[test]
    fn get_oca_from_style_json() {
        let url = "https://dev-ssi-schema-creator-ws.ubique.ch/v1/schema/Basis%20ID/0.0.1";
        let oca = oca_from_style(url).unwrap();
        let zip = generate_zip(oca).unwrap();
        std::fs::write("basis_id.oca", zip).unwrap();
    }
    #[test]
    fn generate_custom() {
        let mut attributes = BTreeMap::<String, String>::new();
        attributes.insert("givenName".to_string(), "Text".into());
        attributes.insert("surname".to_string(), "Text".into());
        attributes.insert("dateOfBirth".to_string(), "DateTime".into());
        let mut capture_base = CaptureBase::new(attributes.clone(), vec!["dateOfBirth".into()]);

        capture_base.update_digest().unwrap();
        let capture_base_digest = capture_base.digest.clone();
        let mut mapping = BTreeMap::<String, String>::new();
        mapping.insert("givenName".into(), "vc.givenName".into());
        mapping.insert("surname".into(), "vc.surname".into());
        mapping.insert("dateOfBirth".into(), "vc.dateOfBirth".into());

        let attr_layer = OcaLayer::new_attribute_mapping_layer(&capture_base_digest, mapping);

        let style_json = StyleJson {
            title: "Test - {{ surname }}".into(),
            subtitle: "Outdoor Guide".into(),
            card_color: 4288983525,
            text_color: "light".into(),
            background_card: None,
            ordered_properties: vec!["givenName".into(), "surname".into(), "dateOfBirth".into()],
        };

        let style_layer = OcaLayer::new_style_layer(&capture_base_digest, style_json);

        let mut attr_labels = BTreeMap::<String, String>::new();
        attr_labels.insert("givenName".into(), "Nachname".into());
        attr_labels.insert("surname".into(), "Vorname".into());
        attr_labels.insert("dateOfBirth".into(), "Geburtsdatum".into());

        let label_layer = OcaLayer::new_label_layer(
            &capture_base_digest,
            "de",
            attr_labels,
            vec![],
            BTreeMap::new(),
        );
        let mut attr_format = BTreeMap::<String, String>::new();
        attr_format.insert("dateOfBirth".into(), "%Y%m%d".into());

        let format_layer = OcaLayer::new_format_layer(&capture_base_digest, attr_format);

        let oca = Oca {
            capture_base,
            overlays: vec![
                ("attribute mapping".into(), attr_layer),
                ("layer (en)".into(), label_layer),
                ("format".into(), format_layer),
                ("style".into(), style_layer),
            ],
        };
        let zip = generate_zip(oca).unwrap();
        std::fs::write("style_test.oca", zip).unwrap();
    }

    #[test]
    fn calculate_said() {
        let mut layer = OcaLayer::Conformance(Conformance::new(
            "Ezk3JiB2xru1K-cd-iW4kScdrpP7bKYizk-mrvFHoZLY",
            BTreeMap::new(),
        ));
        layer.update_digest().unwrap();
        println!("{}", layer.digest());
        let layer_str = serde_json::to_string(&layer).unwrap();

        assert!(verify_said_from_str(layer.digest(), &layer_str).unwrap());
    }
}
