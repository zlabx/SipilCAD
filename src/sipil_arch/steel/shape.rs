//! The outline of a steel section, in millimetres with the centre of gravity
//! at the origin, x to the right and y up.
//!
//! A port of SipilGen's `drawSection` (assets/tb.js): the same outlines, with
//! the fillets and toes rounded the same way, so the CAD section is the one
//! the SipilGen page draws. Checked against the catalogue's area and centre of
//! gravity for every profile.

use super::catalog::{Kind, Profile};

pub type P = [f64; 2];

/// Slope of the inner face of a channel flange (SipilGen's `CHANNEL_TAPER`).
const CHANNEL_TAPER: f64 = 0.08;

/// Segments per quarter turn of a rounded corner.
const ARC_SEGMENTS: usize = 6;

/// Round the corners of a polygon: each vertex carries the radius of the arc
/// that is tangent to both its edges (0 keeps it sharp). Works for convex and
/// concave corners alike, so it also draws the root fillets.
fn round(pts: &[(P, f64)]) -> Vec<P> {
    let n = pts.len();
    let mut out = Vec::new();
    for i in 0..n {
        let (v, r) = pts[i];
        let a = pts[(i + n - 1) % n].0;
        let b = pts[(i + 1) % n].0;
        let (ua, ub) = (unit(sub(a, v)), unit(sub(b, v)));
        let cos = (ua[0] * ub[0] + ua[1] * ub[1]).clamp(-1.0, 1.0);
        let phi = cos.acos();
        if r <= 1e-9 || phi < 1e-6 || (std::f64::consts::PI - phi).abs() < 1e-6 {
            out.push(v);
            continue;
        }
        // Tangent length; shrink the radius if the edges are too short for it.
        let la = dist(a, v).min(dist(b, v)) * 0.999;
        let mut d = r / (phi / 2.0).tan();
        let mut r = r;
        if d > la {
            d = la;
            r = d * (phi / 2.0).tan();
        }
        let (t1, t2) = ([v[0] + ua[0] * d, v[1] + ua[1] * d], [v[0] + ub[0] * d, v[1] + ub[1] * d]);
        let bis = unit([ua[0] + ub[0], ua[1] + ub[1]]);
        let off = r / (phi / 2.0).sin();
        let c = [v[0] + bis[0] * off, v[1] + bis[1] * off];
        let a1 = (t1[1] - c[1]).atan2(t1[0] - c[0]);
        let a2 = (t2[1] - c[1]).atan2(t2[0] - c[0]);
        let mut delta = a2 - a1;
        while delta > std::f64::consts::PI {
            delta -= std::f64::consts::TAU;
        }
        while delta < -std::f64::consts::PI {
            delta += std::f64::consts::TAU;
        }
        let steps = ((delta.abs() / (std::f64::consts::PI / 2.0)) * ARC_SEGMENTS as f64).ceil().max(1.0) as usize;
        for k in 0..=steps {
            let th = a1 + delta * k as f64 / steps as f64;
            out.push([c[0] + r * th.cos(), c[1] + r * th.sin()]);
        }
    }
    out
}

fn sub(a: P, b: P) -> P {
    [a[0] - b[0], a[1] - b[1]]
}

fn dist(a: P, b: P) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn unit(v: P) -> P {
    let l = v[0].hypot(v[1]);
    if l < 1e-12 {
        [0.0, 0.0]
    } else {
        [v[0] / l, v[1] / l]
    }
}

fn pts(list: &[(f64, f64, f64)]) -> Vec<(P, f64)> {
    list.iter().map(|&(x, y, r)| ([x, y], r)).collect()
}

/// I, H and wide-flange sections (also the section through the solid web of a
/// castellated beam): origin at the centre.
fn i_section(p: &Profile) -> Vec<P> {
    let (hw, hh, tw, tf, r) = (p.b / 2.0, p.h / 2.0, p.tw / 2.0, p.tf, p.r1);
    let yi = hh - tf;
    round(&pts(&[
        (-hw, hh, 0.0),
        (hw, hh, 0.0),
        (hw, yi, 0.0),
        (tw, yi, r),
        (tw, -yi, r),
        (hw, -yi, 0.0),
        (hw, -hh, 0.0),
        (-hw, -hh, 0.0),
        (-hw, -yi, 0.0),
        (-tw, -yi, r),
        (-tw, yi, r),
        (-hw, yi, 0.0),
    ]))
}

/// King (arms both sides) and queen (one side) cross: an H section with T
/// sections welded to the web. Origin at the centre of the H.
fn cross(p: &Profile, both: bool) -> Vec<P> {
    let (b, hh, a, tf, r) = (p.b / 2.0, p.h / 2.0, p.tw / 2.0, p.tf, p.r1);
    let ii = hh - tf;
    let arm = p.h / 2.0 + p.tw / 2.0; // end of the T stem
    let xi = arm - tf; // inner face of the T flange
    let mut v = vec![
        (-b, hh, 0.0),
        (b, hh, 0.0),
        (b, ii, 0.0),
        (a, ii, r),
        (a, a, 0.0),
        (xi, a, r),
        (xi, b, 0.0),
        (arm, b, 0.0),
        (arm, -b, 0.0),
        (xi, -b, 0.0),
        (xi, -a, r),
        (a, -a, 0.0),
        (a, -ii, r),
        (b, -ii, 0.0),
        (b, -hh, 0.0),
        (-b, -hh, 0.0),
        (-b, -ii, 0.0),
        (-a, -ii, r),
    ];
    if both {
        v.extend([
            (-a, -a, 0.0),
            (-xi, -a, r),
            (-xi, -b, 0.0),
            (-arm, -b, 0.0),
            (-arm, b, 0.0),
            (-xi, b, 0.0),
            (-xi, a, r),
            (-a, a, 0.0),
        ]);
    }
    v.extend([(-a, ii, r), (-b, ii, 0.0)]);
    // The vertices at (a, a) and (-a, -a) etc. are corners of the web where
    // the arm joins; they are sharp in SipilGen as well.
    round(&pts(&v))
}

/// Angle: origin at the bottom-left corner of the outer faces, then moved.
fn angle(p: &Profile) -> Vec<P> {
    let (h, b, t, r1, r2) = (p.h, p.b, p.t, p.r1, p.r2);
    round(&pts(&[(0.0, 0.0, 0.0), (b, 0.0, 0.0), (b, t, r2), (t, t, r1), (t, h, r2), (0.0, h, 0.0)]))
}

/// Channel (UNP / C), web on the left and flanges to the right, inner flange
/// faces sloped.
fn channel(p: &Profile) -> Vec<P> {
    let (h, b, cw, cf, r1, r2) = (p.h, p.b, p.tw, p.tf, p.r1, p.r2);
    let cc = cf + CHANNEL_TAPER * (cw + b) / 2.0;
    let inner = |x: f64| cc - CHANNEL_TAPER * x;
    round(&pts(&[
        (0.0, 0.0, 0.0),
        (b, 0.0, 0.0),
        (b, inner(b), r2),
        (cw, inner(cw), r1),
        (cw, h - inner(cw), r1),
        (b, h - inner(b), r2),
        (b, h, 0.0),
        (0.0, h, 0.0),
    ]))
}

/// T section: flange on top, stem down, origin at the bottom-left of the box.
fn tee(p: &Profile) -> Vec<P> {
    let (h, b, sw, sf, r) = (p.h, p.b, p.tw, p.tf, p.r1);
    let (xl, xr) = (b / 2.0 - sw / 2.0, b / 2.0 + sw / 2.0);
    round(&pts(&[
        (xl, 0.0, 0.0),
        (xr, 0.0, 0.0),
        (xr, h - sf, r),
        (b, h - sf, 0.0),
        (b, h, 0.0),
        (0.0, h, 0.0),
        (0.0, h - sf, 0.0),
        (xl, h - sf, r),
    ]))
}

/// The outline of `p`, one closed polygon (the last point joins the first),
/// centred on its centre of gravity.
pub fn outline(p: &Profile) -> Vec<P> {
    let (poly, origin) = match p.kind {
        Kind::WideFlange | Kind::HBeam | Kind::IBeam | Kind::Honeycomb | Kind::CellForm => (i_section(p), [0.0, 0.0]),
        Kind::King => (cross(p, true), p.centroid.unwrap_or([0.0, 0.0])),
        Kind::Queen => (cross(p, false), p.centroid.unwrap_or([0.0, 0.0])),
        Kind::EqualAngle | Kind::UnequalAngle => (angle(p), p.centroid.unwrap_or([p.b / 2.0, p.h / 2.0])),
        Kind::Channel => (channel(p), p.centroid.unwrap_or([p.b / 2.0, p.h / 2.0])),
        Kind::TBeam => (tee(p), p.centroid.unwrap_or([p.b / 2.0, p.h / 2.0])),
    };
    poly.into_iter().map(|q| [q[0] - origin[0], q[1] - origin[1]]).collect()
}

/// Area (mm²) and centroid of a polygon.
pub fn area_centroid(poly: &[P]) -> (f64, P) {
    let (mut a2, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let cross = p[0] * q[1] - q[0] * p[1];
        a2 += cross;
        cx += (p[0] + q[0]) * cross;
        cy += (p[1] + q[1]) * cross;
    }
    let a = a2 / 2.0;
    if a.abs() < 1e-9 {
        return (0.0, [0.0, 0.0]);
    }
    (a.abs(), [cx / (6.0 * a), cy / (6.0 * a)])
}

/// Width and height of the outline's box, millimetres.
pub fn extent(poly: &[P]) -> (f64, f64) {
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for q in poly {
        x0 = x0.min(q[0]);
        x1 = x1.max(q[0]);
        y0 = y0.min(q[1]);
        y1 = y1.max(q[1]);
    }
    (x1 - x0, y1 - y0)
}

/// A small SVG of the section for the popup, `size` pixels square.
pub fn svg(p: &Profile, size: f64) -> String {
    let poly = outline(p);
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    for q in &poly {
        x0 = x0.min(q[0]);
        x1 = x1.max(q[0]);
        y0 = y0.min(q[1]);
        y1 = y1.max(q[1]);
    }
    let pad = 6.0;
    let s = (size - 2.0 * pad) / (x1 - x0).max(y1 - y0).max(1e-9);
    let (ox, oy) = (pad + (size - 2.0 * pad - (x1 - x0) * s) / 2.0, pad + (size - 2.0 * pad - (y1 - y0) * s) / 2.0);
    let d: Vec<String> = poly
        .iter()
        .enumerate()
        .map(|(i, q)| format!("{}{:.2} {:.2}", if i == 0 { "M" } else { "L" }, ox + (q[0] - x0) * s, oy + (y1 - q[1]) * s))
        .collect();
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {size} {size}\" width=\"{size}\" height=\"{size}\">\
         <path d=\"{}Z\" fill=\"#6DB7ED\" fill-opacity=\"0.35\" stroke=\"#6DB7ED\" stroke-width=\"1.6\" stroke-linejoin=\"round\"/></svg>",
        d.join("")
    )
}

#[cfg(test)]
mod tests {
    use super::super::catalog::{all, profiles_of};
    use super::*;

    #[test]
    fn every_outline_has_the_catalogue_area_and_centre_of_gravity() {
        let mut worst = (0.0f64, "");
        for p in all() {
            let poly = outline(p);
            let (area_mm2, c) = area_centroid(&poly);
            let rel = (area_mm2 / 100.0 - p.area_cm2).abs() / p.area_cm2;
            // Castellated beams are drawn through the solid web (the catalogue's gross area).
            let tol = if matches!(p.kind, Kind::Honeycomb | Kind::CellForm) { 0.07 } else { 0.02 };
            assert!(rel <= tol, "{}: area {:.2} cm2 vs {:.2} ({:.1}%)", p.name, area_mm2 / 100.0, p.area_cm2, rel * 100.0);
            if rel > worst.0 {
                worst = (rel, p.name);
            }
            // The centre of gravity is where the catalogue says (the outline is centred on it).
            if p.centroid.is_some() {
                let tol = 0.5 + 0.001 * p.h.max(p.b);
                assert!(c[0].abs() <= tol && c[1].abs() <= tol, "{}: centroid {:?}", p.name, c);
            }
        }
        eprintln!("worst area difference: {:.2}% ({})", worst.0 * 100.0, worst.1);
    }

    #[test]
    fn boxes_have_the_catalogue_depth_and_width() {
        for p in all() {
            let (w, h) = extent(&outline(p));
            let (ew, eh) = match p.kind {
                Kind::King => (p.h + p.tw, p.b.max(p.h)),
                Kind::Queen => (p.h / 2.0 + p.tw / 2.0 + p.b / 2.0, p.b.max(p.h)),
                _ => (p.b, p.h),
            };
            assert!((w - ew).abs() < 0.5 && (h - eh).abs() < 0.5, "{}: {w:.1}x{h:.1} vs {ew:.1}x{eh:.1}", p.name);
        }
    }

    #[test]
    fn rounding_a_corner_stays_tangent_and_inside_the_edges() {
        // A right angle with radius 2: the arc starts and ends 2 from the corner.
        let poly = round(&pts(&[(0.0, 0.0, 2.0), (10.0, 0.0, 0.0), (10.0, 10.0, 0.0), (0.0, 10.0, 0.0)]));
        assert!(poly.iter().any(|q| (q[0] - 2.0).abs() < 1e-9 && q[1].abs() < 1e-9));
        assert!(poly.iter().any(|q| q[0].abs() < 1e-9 && (q[1] - 2.0).abs() < 1e-9));
        let (a, _) = area_centroid(&poly);
        let expect = 100.0 - (4.0 - std::f64::consts::PI); // the corner loses r²(1 - π/4)·... = 4 - π
        assert!((a - expect).abs() < 0.05, "{a} vs {expect}");
    }

    #[test]
    fn symmetric_sections_are_centred_and_angles_are_not_mirrored() {
        let wf = &profiles_of("gunung-garuda", Kind::WideFlange)[0];
        let (_, c) = area_centroid(&outline(wf));
        assert!(c[0].abs() < 0.01 && c[1].abs() < 0.01, "{c:?}");
        // L90X75X9: vertical leg 90 tall, horizontal leg 75 wide, centroid inside the corner.
        let l = profiles_of("gunung-garuda", Kind::UnequalAngle).into_iter().find(|p| p.name == "L90X75X9").unwrap();
        let poly = outline(l);
        let (w, h) = extent(&poly);
        assert!((w - 75.0).abs() < 1e-6 && (h - 90.0).abs() < 1e-6);
        let min_x = poly.iter().map(|q| q[0]).fold(f64::MAX, f64::min);
        assert!((min_x + 20.04).abs() < 1e-6, "left edge sits 20.04 left of the centroid: {min_x}");
    }

    #[test]
    fn the_preview_svg_is_well_formed() {
        let p = &all()[0];
        let s = svg(p, 120.0);
        assert!(s.starts_with("<svg") && s.ends_with("</svg>") && s.contains("<path d=\"M"));
        assert_eq!(s.matches('<').count(), s.matches('>').count());
    }
}
