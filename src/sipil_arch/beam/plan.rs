//! A beam as seen in the floor plan: its axis, the flange edges to scale (or
//! just the axis), and a label along it.

use super::model::{BeamDef, Style, LAYERS};
use crate::sipil_arch::steel::catalog;
use crate::sipil_arch::steel::draw::{text_width, Prim, SShape};
use crate::sipil_wall::engine::Shape;

const OUTLINE: &str = LAYERS[0].0;
const AXIS: &str = LAYERS[1].0;
const TEXT: &str = LAYERS[2].0;

/// Width of the beam in the plan, drawing units: the flange width of the profile.
pub fn width(def: &BeamDef) -> f64 {
    let p = catalog::find_key(&def.profile).unwrap_or_else(catalog::default_profile);
    p.b * def.upm / 1000.0
}

/// The primitives of `def`, the axis line first (it carries the tag).
pub fn prims(def: &BeamDef) -> Vec<Prim> {
    let def = def.sanitized();
    let p = catalog::find_key(&def.profile).unwrap_or_else(catalog::default_profile);
    let k = def.upm / 1000.0;
    let len = def.length();
    let mut out = vec![Prim { layer: AXIS, shape: SShape::Geo(Shape::Line { a: def.a, b: def.b }) }];
    if len < 1e-9 {
        return out;
    }
    let ang = (def.b[1] - def.a[1]).atan2(def.b[0] - def.a[0]);
    let (u, n) = ([ang.cos(), ang.sin()], [-ang.sin(), ang.cos()]);
    let half = p.b * k / 2.0;
    if def.style == Style::Outline {
        let corner = |at: [f64; 2], s: f64| [at[0] + n[0] * half * s, at[1] + n[1] * half * s];
        out.push(Prim {
            layer: OUTLINE,
            shape: SShape::Geo(Shape::Poly { pts: vec![corner(def.a, -1.0), corner(def.b, -1.0), corner(def.b, 1.0), corner(def.a, 1.0)], closed: true }),
        });
    }
    let text = format!("{} {}", def.mark, p.name).trim().to_string();
    if def.label && !text.is_empty() {
        let height = def.text_mm * k;
        // Keep the text reading left to right and upright.
        let flip = ang.abs() > std::f64::consts::FRAC_PI_2 + 1e-9;
        let (tu, tn, rot) = if flip { ([-u[0], -u[1]], [-n[0], -n[1]], ang + std::f64::consts::PI) } else { (u, n, ang) };
        let off = (if def.style == Style::Outline { half } else { 0.0 }) + 0.5 * height;
        let mid = [(def.a[0] + def.b[0]) / 2.0, (def.a[1] + def.b[1]) / 2.0];
        let w = text_width(&text, height);
        let at = [mid[0] + tn[0] * off - tu[0] * w / 2.0, mid[1] + tn[1] * off - tu[1] * w / 2.0];
        out.push(Prim { layer: TEXT, shape: SShape::Text { at, height, text, rot } });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn def(a: [f64; 2], b: [f64; 2]) -> BeamDef {
        let mut d = BeamDef::new(catalog::find_key("gunung-garuda/WF-200X100X5.5X8").unwrap(), 1000.0);
        d.a = a;
        d.b = b;
        d
    }

    #[test]
    fn an_outline_beam_is_axis_plus_a_flange_wide_rectangle() {
        let p = prims(&def([0.0, 0.0], [4000.0, 0.0]));
        assert_eq!(p[0].layer, "BEAM-AXIS");
        let rect = match &p[1].shape {
            SShape::Geo(Shape::Poly { pts, closed: true }) => pts,
            _ => panic!("no outline"),
        };
        let ys: Vec<f64> = rect.iter().map(|q| q[1]).collect();
        assert!(ys.iter().cloned().fold(f64::MAX, f64::min) + 50.0 < 1e-9, "flange 100 wide: -50 .. 50: {ys:?}");
        assert!((ys.iter().cloned().fold(f64::MIN, f64::max) - 50.0).abs() < 1e-9);
        assert!((width(&def([0.0, 0.0], [1.0, 0.0])) - 100.0).abs() < 1e-9);
    }

    #[test]
    fn a_centerline_beam_has_no_outline() {
        let mut d = def([0.0, 0.0], [4000.0, 0.0]);
        d.style = Style::Centerline;
        let p = prims(&d);
        assert!(p.iter().all(|q| q.layer != "BEAM-OUTLINE"));
        assert!(p.iter().any(|q| q.layer == "BEAM-TEXT"));
        d.label = false;
        assert_eq!(prims(&d).len(), 1);
    }

    #[test]
    fn the_label_reads_upright_along_the_beam() {
        let mid_of = |d: &BeamDef| match &prims(d).last().unwrap().shape {
            SShape::Text { at, rot, height, text } => (*at, *rot, *height, text.clone()),
            _ => panic!(),
        };
        // Going right: text on the left side (up), rotation 0, centred on x = 2000.
        let (at, rot, h, text) = mid_of(&def([0.0, 0.0], [4000.0, 0.0]));
        assert!(rot.abs() < 1e-9 && at[1] > 50.0, "{at:?}");
        let w = text_width(&text, h);
        assert!((at[0] + w / 2.0 - 2000.0).abs() < 1e-6);
        // Going left: the text is turned over so it still reads left to right, above the beam.
        let (at, rot, _, _) = mid_of(&def([4000.0, 0.0], [0.0, 0.0]));
        assert!(rot.abs() < 1e-9 || (rot - 2.0 * std::f64::consts::PI).abs() < 1e-9 || (rot.abs() - std::f64::consts::TAU).abs() < 1e-9, "{rot}");
        assert!(at[1] > 50.0, "{at:?}");
        // Straight up: reads bottom to top.
        let (_, rot, _, _) = mid_of(&def([0.0, 0.0], [0.0, 4000.0]));
        assert!((rot - std::f64::consts::FRAC_PI_2).abs() < 1e-9);
    }

    #[test]
    fn a_zero_length_beam_draws_nothing_but_the_axis() {
        assert_eq!(prims(&def([5.0, 5.0], [5.0, 5.0])).len(), 1);
    }
}
