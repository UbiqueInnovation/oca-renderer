// Copyright (c) 2024 Ubique Innovation AG <https://www.ubique.ch>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use oca_render::{oca::parse_zip, oca_from_style, typst_renderer::TypstWorld};
use ureq::json;

fn main() {
    let oca = include_bytes!("../oca-typst/style_test.oca");
    let oca = parse_zip(oca).unwrap();
    let world = TypstWorld::new(
        "./temp".to_string(),
        json!({
            "vc": {
            "givenName" : "Mustermann",
            "surname" : "Manfred",
            "dateOfBirth" : "20001010"
            }
        }),
        // oca_from_style("https://dev-ssi-schema-creator-ws.ubique.ch/v1/schema/Basis%20ID/0.0.1")
        //     .unwrap(),
        oca,
    );
    let png = world.compile_png(0, 8.0).unwrap();

    std::fs::write("test.png", png).unwrap();
}
