//! A column as seen in the floor plan: the section outline of its profile,
//! a fill, and a label.

use super::model::{ColumnDef, Fill, LAYERS};
use crate::sipil_arch::steel::draw::{place, text_width, Prim, SShape};
use crate::sipil_arch::steel::{catalog, shape};
use crate::sipil_wall::engine::Shape;

const SECTION: &str = LAYERS[0].0;
const HATCH: &str = LAYERS[1].0;
const TEXT: &str = LAYERS[2].0;

/// The thinnest part of the section, millimetres: what the hatch spacing follows.
fn thinnest(p: &catalog::Profile) -> f64 {
    [p.tw, p.tf, p.t].into_iter().filter(|v| *v > 0.0).fold(f64::MAX, f64::min).min(p.h.min(p.b))
}

/// The primitives of `def`, the section outline first (it carries the tag).
pub fn prims(def: &ColumnDef) -> Vec<Prim> {
    let def = def.sanitized();
    let p = catalog::find_key(&def.profile).unwrap_or_else(catalog::default_profile);
    let k = def.upm / 1000.0;
    let pts = place(&shape::outline(p), def.at, def.rot_deg.to_radians(), k);
    let mut out = vec![Prim { layer: SECTION, shape: SShape::Geo(Shape::Poly { pts: pts.clone(), closed: true }) }];
    match def.fill {
        Fill::None => {}
        Fill::Solid => out.push(Prim {
            layer: HATCH,
            shape: SShape::Geo(Shape::Hatch { name: "SOLID", solid: true, scale: 1.0, boundary: pts.clone() }),
        }),
        Fill::Hatch => out.push(Prim {
            layer: HATCH,
            shape: SShape::Geo(Shape::Hatch { name: "ANSI32", solid: false, scale: (thinnest(p) * k / 3.0) / 3.175, boundary: pts.clone() }),
        }),
    }
    let text = format!("{} {}", def.mark, p.name).trim().to_string();
    if def.label && !text.is_empty() {
        let height = def.text_mm * k;
        let max_x = pts.iter().map(|q| q[0]).fold(f64::MIN, f64::max);
        out.push(Prim {
            layer: TEXT,
            shape: SShape::Text { at: [max_x + 0.6 * height, def.at[1] - height / 2.0], height, text, rot: 0.0 },
        });
    }
    out
}

/// Width of the label, drawing units (for tests and previews).
pub fn label_width(def: &ColumnDef) -> f64 {
    prims(def)
        .iter()
        .find_map(|p| match &p.shape {
            SShape::Text { text, height, .. } => Some(text_width(text, *height)),
            _ => None,
        })
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(key: &str) -> ColumnDef {
        let mut d = ColumnDef::new(catalog::find_key(key).unwrap(), 1000.0);
        d.at = [5000.0, 3000.0];
        d
    }

    fn poly(p: &Prim) -> &Vec<[f64; 2]> {
        match &p.shape {
            SShape::Geo(Shape::Poly { pts, .. }) => pts,
            _ => panic!("not an outline"),
        }
    }

    #[test]
    fn a_column_is_its_section_centred_on_the_point() {
        let d = def("gunung-garuda/WF-200X100X5.5X8");
        let prims = prims(&d);
        let outline = poly(&prims[0]);
        let (area, c) = shape::area_centroid(outline);
        assert!((c[0] - 5000.0).abs() < 0.5 && (c[1] - 3000.0).abs() < 0.5, "{c:?}");
        // 27.16 cm2 in mm units (1 unit = 1 mm).
        assert!((area / 100.0 - 27.16).abs() < 0.2, "{area}");
        assert_eq!(prims[0].layer, "COLUMN-SECTION");
        assert!(prims.iter().any(|p| p.layer == "COLUMN-HATCH"));
        assert!(prims.iter().any(|p| matches!(&p.shape, SShape::Text { text, .. } if text == "WF-200X100X5.5X8")));
    }

    #[test]
    fn rotation_turns_the_section_and_the_unit_scales_it() {
        let mut d = def("gunung-garuda/WF-200X100X5.5X8");
        let (w, h) = shape::extent(poly(&prims(&d)[0]));
        assert!((w - 100.0).abs() < 1e-6 && (h - 200.0).abs() < 1e-6);
        d.rot_deg = 90.0;
        let (w, h) = shape::extent(poly(&prims(&d)[0]));
        assert!((w - 200.0).abs() < 1e-6 && (h - 100.0).abs() < 1e-6);
        // A drawing in metres: the same section is 0.1 x 0.2 units.
        d.rot_deg = 0.0;
        d.upm = 1.0;
        d.at = [5.0, 3.0];
        let (w, h) = shape::extent(poly(&prims(&d)[0]));
        assert!((w - 0.1).abs() < 1e-9 && (h - 0.2).abs() < 1e-9);
    }

    #[test]
    fn fill_and_label_follow_the_settings() {
        let mut d = def("gunung-garuda/WF-200X100X5.5X8");
        d.fill = Fill::None;
        d.label = false;
        assert_eq!(prims(&d).len(), 1);
        d.fill = Fill::Hatch;
        d.label = true;
        d.mark = "K1".into();
        let p = prims(&d);
        assert!(matches!(&p[1].shape, SShape::Geo(Shape::Hatch { name: "ANSI32", solid: false, .. })));
        assert!(matches!(&p[2].shape, SShape::Text { text, .. } if text == "K1 WF-200X100X5.5X8"));
        assert!(label_width(&d) > 0.0);
        // The label sits to the right of the section, on its centre line.
        if let SShape::Text { at, .. } = &p[2].shape {
            assert!(at[0] > 5000.0 + 50.0 && (at[1] - 3000.0).abs() < 150.0);
        }
    }

    #[test]
    fn a_missing_profile_falls_back_instead_of_failing() {
        let mut d = def("gunung-garuda/WF-200X100X5.5X8");
        d.profile = "gone/X".into();
        assert!(!prims(&d).is_empty());
    }
}
