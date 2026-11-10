//! Primitives and entities shared by the steel components (columns, beams).
//!
//! Pure: `codec` types only. Each component says which layer a primitive is
//! on and which XDATA record its entities carry.

use codec::entities::Text;
use codec::types::Vector3;
use codec::xdata::ExtendedDataRecord;
use codec::EntityType;

use crate::sipil_wall::engine::{Shape, P};
use crate::sipil_wall::entities::{shape_entity, PatternFn};

#[derive(Clone, Debug, PartialEq)]
pub enum SShape {
    /// A line, outline or hatch, the same shapes the wall uses.
    Geo(Shape),
    /// One line of text, left-aligned at `at` (baseline), `rot` radians.
    Text { at: P, height: f64, text: String, rot: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prim {
    pub layer: &'static str,
    pub shape: SShape,
}

/// Width of a text line, estimated (CAD text has no metrics here).
pub fn text_width(s: &str, height: f64) -> f64 {
    0.6 * height * s.chars().count() as f64
}

/// A millimetre outline placed in the drawing: rotated by `rot` radians about
/// its origin, scaled by `k` (drawing units per millimetre), moved to `at`.
pub fn place(poly: &[P], at: P, rot: f64, k: f64) -> Vec<P> {
    let (s, c) = rot.sin_cos();
    poly.iter().map(|q| [at[0] + (q[0] * c - q[1] * s) * k, at[1] + (q[0] * s + q[1] * c) * k]).collect()
}

/// The entity of a primitive on its layer; `record` is the component's tag.
pub fn entity(p: &Prim, pattern: PatternFn, record: Option<ExtendedDataRecord>) -> EntityType {
    let mut e = match &p.shape {
        SShape::Geo(s) => shape_entity(s, pattern),
        SShape::Text { at, height, text, rot } => EntityType::Text(
            Text::with_value(text.clone(), Vector3::new(at[0], at[1], 0.0))
                .with_height(*height)
                .with_rotation(*rot),
        ),
    };
    let c = e.common_mut();
    c.layer = p.layer.to_owned();
    if let Some(r) = record {
        c.extended_data.upsert_record(r);
    }
    e
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn place_rotates_scales_and_moves() {
        let out = place(&[[1000.0, 0.0], [0.0, 500.0]], [10.0, 20.0], std::f64::consts::FRAC_PI_2, 0.001);
        assert!((out[0][0] - 10.0).abs() < 1e-9 && (out[0][1] - 21.0).abs() < 1e-9);
        assert!((out[1][0] - 9.5).abs() < 1e-9 && (out[1][1] - 20.0).abs() < 1e-9);
    }

    #[test]
    fn entities_get_their_layer_and_tag() {
        let p = Prim { layer: "COLUMN-TEXT", shape: SShape::Text { at: [1.0, 2.0], height: 0.2, text: "K1".into(), rot: 0.0 } };
        let none = |_: &str, _: f64| None;
        let e = entity(&p, &none, Some(ExtendedDataRecord::new("SIPILCOLUMN")));
        assert_eq!(e.common().layer, "COLUMN-TEXT");
        assert!(e.common().extended_data.get_record("SIPILCOLUMN").is_some());
        assert!(text_width("K1", 10.0) > 0.0);
    }
}
