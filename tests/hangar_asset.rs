use planet_hoppers::graphics::hangar::HangarStyle;
use serde_json::Value;
use std::{fs, path::PathBuf};

fn asset() -> (Value, Vec<u8>) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(root.join(HangarStyle::default().station_model)).unwrap();
    assert_eq!(&bytes[..4], b"glTF");
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 2);
    let len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let doc = serde_json::from_slice(&bytes[20..20 + len]).unwrap();
    assert_eq!(&bytes[24 + len..28 + len], b"BIN\0");
    (doc, bytes[28 + len..].to_vec())
}

#[test]
fn approved_environment_contains_only_self_contained_architecture() {
    let (doc, _) = asset();
    let nodes = doc["nodes"].as_array().unwrap();
    for node in nodes {
        let name = node["name"].as_str().unwrap();
        assert!(!name.contains("NO EXPORTAR"), "reference leaked: {name}");
        assert!(node["mesh"].is_u64());
        for key in [
            "camera",
            "extensions",
            "matrix",
            "rotation",
            "scale",
            "translation",
        ] {
            assert!(
                node[key].is_null(),
                "unbaked or non-architectural node: {name}"
            );
        }
    }
    for name in [
        "PRESIDIUM | Perfil portante",
        "CUBIERTA | Casco facetado Zakera",
        "SALIDA | Perfil escalonado",
    ] {
        assert!(
            nodes.iter().any(|node| node["name"] == name),
            "missing approved architecture: {name}"
        );
    }
    assert!(doc["cameras"].is_null());
    let images = doc["images"].as_array().unwrap();
    assert_eq!(images.len(), 1, "one shared architectural radiance atlas");
    for image in images {
        assert!(image["bufferView"].is_u64() && image["uri"].is_null());
        assert_eq!(image["mimeType"], "image/png");
    }
    for buffer in doc["buffers"].as_array().unwrap() {
        assert!(
            buffer["uri"].is_null(),
            "GLB must not depend on external files"
        );
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let original: Value = serde_json::from_slice(
        &fs::read(root.join("docs/references/hangar/reconstruction/source-inspection.json"))
            .unwrap(),
    )
    .unwrap();
    for material in doc["materials"].as_array().unwrap() {
        let name = material["name"].as_str().unwrap();
        let source = &original["materials"][name];
        assert!(
            !source.is_null(),
            "material is not from the approved station: {name}"
        );
        let close = |a: &Value, b: &Value| {
            assert!((a.as_f64().unwrap() - b.as_f64().unwrap()).abs() < 1e-6);
        };
        for i in 0..4 {
            close(
                &material["pbrMetallicRoughness"]["baseColorFactor"][i],
                &source["Base Color"][i],
            );
        }
        close(
            &material["pbrMetallicRoughness"]["metallicFactor"],
            &source["Metallic"],
        );
        close(
            &material["pbrMetallicRoughness"]["roughnessFactor"],
            &source["Roughness"],
        );
    }
}

#[test]
fn exported_vertices_preserve_game_units_and_the_rotating_ship_clearance() {
    let (doc, bin) = asset();
    let mut largest_x = 0.0f32;
    let mut highest = 0.0f32;
    for mesh in doc["meshes"].as_array().unwrap() {
        for primitive in mesh["primitives"].as_array().unwrap() {
            assert!(primitive["attributes"]["NORMAL"].is_u64());
            assert!(primitive["attributes"]["TEXCOORD_0"].is_u64());
            let a =
                &doc["accessors"][primitive["attributes"]["POSITION"].as_u64().unwrap() as usize];
            assert_eq!(a["componentType"], 5126);
            assert_eq!(a["type"], "VEC3");
            let count = a["count"].as_u64().unwrap() as usize;
            assert!(count <= 65_535, "avoid raylib's 16-bit index truncation");
            let view = &doc["bufferViews"][a["bufferView"].as_u64().unwrap() as usize];
            let offset = view["byteOffset"].as_u64().unwrap_or(0) as usize
                + a["byteOffset"].as_u64().unwrap_or(0) as usize;
            let stride = view["byteStride"].as_u64().unwrap_or(12) as usize;
            for i in 0..count {
                let at = offset + i * stride;
                let read = |k| f32::from_le_bytes(bin[at + k..at + k + 4].try_into().unwrap());
                let (x, y, z) = (read(0), read(4), read(8));
                assert!(x.is_finite() && y.is_finite() && z.is_finite());
                largest_x = largest_x.max(x.abs());
                highest = highest.max(y);
                assert!(
                    !(0.001..6.9).contains(&y) || x.hypot(z) >= 4.8,
                    "architecture vertex intersects platform/ship reserve: ({x}, {y}, {z})"
                );
            }
        }
    }
    assert!((26.0..27.0).contains(&largest_x));
    assert!((25.0..27.0).contains(&highest));
}
