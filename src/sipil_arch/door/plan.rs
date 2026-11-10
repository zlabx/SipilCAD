//! A door as seen in the floor plan: frame jambs, leaf (closed, or swung
//! open with its arc) in the opening of a wall.
//!
//! Pure geometry in drawing units. `Placement` says where on the wall the
//! centre of the opening is and which way the wall runs.

use std::f64::consts::PI;

use super::catalog::DoorType;
use super::model::{DPrim, DRole, DShape, DoorDef};
use crate::sipil_wall::engine::{Shape, P};

/// Where a door sits: the centre of its opening on the wall axis, the
/// direction of the run, the wall's total thickness, drawing units per metre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Placement {
    pub c: P,
    pub ang: f64,
    pub tot: f64,
    pub upm: f64,
}

/// A point `s` along the run and `d` to its left, from the opening centre.
fn pt(at: &Placement, s: f64, d: f64) -> P {
    let (sin, cos) = at.ang.sin_cos();
    [at.c[0] + s * cos - d * sin, at.c[1] + s * sin + d * cos]
}

/// Half the opening in the wall: half the clear width plus one frame, drawing units.
pub fn opening_half(def: &DoorDef, ty: &DoorType, upm: f64) -> f64 {
    (def.width_m / 2.0 + ty.frame_m) * upm
}

fn poly(role: DRole, pts: Vec<P>, closed: bool) -> DPrim {
    DPrim {
        role,
        shape: DShape::Geo(Shape::Poly { pts, closed }),
    }
}

/// The plan symbol of `def` at `at`.
pub fn plan(def: &DoorDef, ty: &DoorType, at: &Placement) -> Vec<DPrim> {
    let def = def.sanitized();
    let k = at.upm;
    let (w, f, lt) = (def.width_m * k, ty.frame_m * k, ty.leaf_m * k);
    let half_depth = at.tot / 2.0;
    let mut out = Vec::new();

    // Frame jambs: a box on each side of the clear width, as deep as the wall.
    for sign in [-1.0, 1.0] {
        let (s0, s1) = (sign * w / 2.0, sign * (w / 2.0 + f));
        let corners = vec![
            pt(at, s0, -half_depth),
            pt(at, s1, -half_depth),
            pt(at, s1, half_depth),
            pt(at, s0, half_depth),
        ];
        out.push(poly(DRole::Frame, corners, true));
    }

    // Leaves: (hinge position along the run, direction toward the latch).
    let leaves: Vec<(f64, f64)> = if ty.leaves >= 2 {
        vec![(-w / 2.0, 1.0), (w / 2.0, -1.0)]
    } else if def.hinge_start {
        vec![(-w / 2.0, 1.0)]
    } else {
        vec![(w / 2.0, -1.0)]
    };
    let len = w / leaves.len() as f64;
    let angle = def.angle_deg.to_radians();
    let swung = def.open && angle > 0.017;
    let side = if def.swing_left { 1.0 } else { -1.0 };

    for (hinge, toward) in leaves {
        if !swung {
            let (s0, s1) = (hinge, hinge + toward * len);
            let corners = vec![
                pt(at, s0, -lt / 2.0),
                pt(at, s1, -lt / 2.0),
                pt(at, s1, lt / 2.0),
                pt(at, s0, lt / 2.0),
            ];
            out.push(poly(DRole::Leaf, corners, true));
            continue;
        }
        // Local direction of the open leaf and its perpendicular.
        let (o_s, o_d) = (toward * angle.cos(), side * angle.sin());
        let (p_s, p_d) = (-o_d, o_s);
        let h = (hinge, 0.0);
        let corner = |along: f64, across: f64| pt(at, h.0 + o_s * along + p_s * across, h.1 + o_d * along + p_d * across);
        let corners = vec![
            corner(0.0, lt / 2.0),
            corner(len, lt / 2.0),
            corner(len, -lt / 2.0),
            corner(0.0, -lt / 2.0),
        ];
        out.push(poly(DRole::Leaf, corners, true));
        // Swing arc from the closed leaf to the open one.
        let steps = ((angle / (PI / 2.0)) * 16.0).ceil().max(2.0) as usize;
        let arc: Vec<P> = (0..=steps)
            .map(|i| {
                let th = angle * i as f64 / steps as f64;
                pt(at, h.0 + len * toward * th.cos(), len * side * th.sin())
            })
            .collect();
        out.push(poly(DRole::Swing, arc, false));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sipil_arch::door::catalog;

    fn at() -> Placement {
        Placement { c: [0.0, 0.0], ang: 0.0, tot: 0.13, upm: 1.0 }
    }

    fn pts_of(p: &DPrim) -> &Vec<P> {
        match &p.shape {
            DShape::Geo(Shape::Poly { pts, .. }) => pts,
            _ => panic!("not a polyline"),
        }
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn closed_door_is_two_jambs_and_a_leaf_in_the_opening() {
        let ty = catalog::find("P2").unwrap();
        let mut d = DoorDef::of_type(ty);
        d.open = false;
        let prims = plan(&d, ty, &at());
        assert_eq!(prims.iter().filter(|p| p.role == DRole::Frame).count(), 2);
        let leaves: Vec<_> = prims.iter().filter(|p| p.role == DRole::Leaf).collect();
        assert_eq!(leaves.len(), 1);
        assert!(prims.iter().all(|p| p.role != DRole::Swing));
        let xs: Vec<f64> = pts_of(leaves[0]).iter().map(|p| p[0]).collect();
        // Hinge at the start: the leaf spans the clear width, -0.40 .. 0.40.
        assert!(close(xs.iter().cloned().fold(f64::MAX, f64::min), -0.40));
        assert!(close(xs.iter().cloned().fold(f64::MIN, f64::max), 0.40));
        // The jambs lie outside the clear width, 0.40 .. 0.45 each side.
        for j in prims.iter().filter(|p| p.role == DRole::Frame) {
            let lo = pts_of(j).iter().map(|p| p[0].abs()).fold(f64::MAX, f64::min);
            let hi = pts_of(j).iter().map(|p| p[0].abs()).fold(f64::MIN, f64::max);
            assert!(close(lo, 0.40) && close(hi, 0.45));
        }
    }

    #[test]
    fn open_door_swings_to_the_chosen_side_with_an_arc() {
        let ty = catalog::find("P2").unwrap();
        let d = DoorDef::of_type(ty); // open 90, hinge at the start, swings left
        let prims = plan(&d, ty, &at());
        let arc = prims.iter().find(|p| p.role == DRole::Swing).unwrap();
        let a = pts_of(arc);
        // From the closed tip (0.40, 0) to the open tip (-0.40, 0.80).
        assert!(close(a[0][0], 0.40) && close(a[0][1], 0.0));
        let last = a[a.len() - 1];
        assert!(close(last[0], -0.40) && close(last[1], 0.80));
        // Swing right instead: the arc goes below the wall.
        let mut r = d.clone();
        r.swing_left = false;
        let prims = plan(&r, ty, &at());
        let arc = prims.iter().find(|p| p.role == DRole::Swing).unwrap();
        assert!(pts_of(arc).iter().all(|p| p[1] <= 1e-9));
        // A smaller angle stops the arc short.
        let mut s = d.clone();
        s.angle_deg = 45.0;
        let prims = plan(&s, ty, &at());
        let arc = prims.iter().find(|p| p.role == DRole::Swing).unwrap();
        let last = pts_of(arc).last().copied().unwrap();
        assert!(close(last[1], 0.80 * 45f64.to_radians().sin()));
    }

    #[test]
    fn hinge_at_the_end_mirrors_the_door() {
        let ty = catalog::find("P1").unwrap();
        let mut d = DoorDef::of_type(ty);
        d.open = false;
        d.hinge_start = false;
        let prims = plan(&d, ty, &at());
        let leaf = prims.iter().find(|p| p.role == DRole::Leaf).unwrap();
        assert!(pts_of(leaf).iter().all(|p| p[0] >= -0.40 - 1e-9 && p[0] <= 0.40 + 1e-9));
    }

    #[test]
    fn double_door_has_two_leaves_and_two_arcs() {
        let ty = catalog::find("P3").unwrap();
        let d = DoorDef::of_type(ty);
        let prims = plan(&d, ty, &at());
        assert_eq!(prims.iter().filter(|p| p.role == DRole::Leaf).count(), 2);
        assert_eq!(prims.iter().filter(|p| p.role == DRole::Swing).count(), 2);
    }

    #[test]
    fn the_symbol_follows_the_wall_direction_and_unit() {
        let ty = catalog::find("P2").unwrap();
        let mut d = DoorDef::of_type(ty);
        d.open = false;
        let p = Placement { c: [100.0, 50.0], ang: PI / 2.0, tot: 130.0, upm: 1000.0 };
        let prims = plan(&d, ty, &p);
        let leaf = prims.iter().find(|p| p.role == DRole::Leaf).unwrap();
        // Wall runs north: the leaf spans y = 50 - 400 .. 50 + 400 at x ~ 100.
        let ys: Vec<f64> = pts_of(leaf).iter().map(|q| q[1]).collect();
        assert!(close(ys.iter().cloned().fold(f64::MAX, f64::min), -350.0));
        assert!(close(ys.iter().cloned().fold(f64::MIN, f64::max), 450.0));
        assert!(pts_of(leaf).iter().all(|q| (q[0] - 100.0).abs() <= 20.0 + 1e-9));
        assert!(close(opening_half(&d, ty, 1000.0), 450.0));
    }
}
