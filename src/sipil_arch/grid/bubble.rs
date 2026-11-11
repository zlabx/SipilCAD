//! Keeps grid bubbles from lying on top of each other. Port of SipilBIM's
//! `bubblelayout.py`. Pure: `std` only.
//!
//! The bubbles of the lines of a grid stand in a row beyond the ends of the
//! lines. When two lines are closer than a bubble is wide, the bubbles would
//! overlap and the names could not be read. The way a drawing office solves it:
//! the bubbles stay in the row but move apart along it, as little as can be,
//! and each moved one is joined to the end of its line by a leader with an
//! elbow. [`spread`] is the 1-D problem; [`bubble_row`] is the drawing of it.
//! Both work in any unit (millimetres of model space here).

pub type P = [f64; 2];

/// A bubble moved less than this fraction of its radius is not moved.
const STILL: f64 = 0.05;

/// Positions as close as possible to `centers` (least squares) with at least
/// `gap` between neighbours, in the same order. Equal centres keep the order
/// they were given in. `gap` of 0 or less changes nothing.
pub fn spread(centers: &[f64], gap: f64) -> Vec<f64> {
    let n = centers.len();
    if n == 0 {
        return Vec::new();
    }
    if gap <= 0.0 {
        return centers.to_vec();
    }
    let mut order: Vec<usize> = (0..n).collect();
    order.sort_by(|&a, &b| {
        centers[a]
            .partial_cmp(&centers[b])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    // A cluster is its members (indices, in order) and the sum of
    // (wanted - rank * gap) over them: its start is that sum / member count.
    let mut clusters: Vec<(Vec<usize>, f64)> = Vec::new();
    for i in order {
        clusters.push((vec![i], centers[i]));
        while clusters.len() > 1 {
            let (m1, t1) = {
                let p = &clusters[clusters.len() - 2];
                (p.0.len() as f64, p.1)
            };
            let (m2, t2) = {
                let c = &clusters[clusters.len() - 1];
                (c.0.len() as f64, c.1)
            };
            if t1 / m1 + m1 * gap <= t2 / m2 + 1e-12 {
                break;
            }
            let (cur_members, _) = clusters.pop().unwrap_or_default();
            let (mut members, _) = clusters.pop().unwrap_or_default();
            members.extend(cur_members);
            clusters.push((members, t1 + t2 - m2 * m1 * gap));
        }
    }
    let mut out = vec![0.0; n];
    for (members, total) in clusters {
        let start = total / members.len() as f64;
        for (rank, i) in members.into_iter().enumerate() {
            out[i] = start + rank as f64 * gap;
        }
    }
    out
}

/// Where one bubble is and the leader that joins it to its line.
#[derive(Clone, Debug, PartialEq)]
pub struct Placed {
    pub center: P,
    /// From the end of the line to the edge of the bubble (2 points, or 3 with an elbow).
    pub path: Vec<P>,
    /// How far it was moved along the row.
    pub shift: f64,
}

/// The bubbles of ONE side (the bottom ends of the X lines, say), in the order
/// of `ends`, the end of each line on that side.
///
/// `out` is the unit vector pointing away from the lines, `along` the unit
/// vector the row runs in. `r` is a bubble's radius, `stub` the line's end to a
/// bubble's edge when it is not moved, `jog` the extra run a moved bubble's
/// elbow needs, `pad` the least free space between two bubbles. All bubbles of a
/// row stand on one line, as near as can be: `jog` farther out only when some
/// bubble had to move.
pub fn bubble_row(
    ends: &[P],
    out: P,
    along: P,
    r: f64,
    stub: f64,
    jog: f64,
    pad: f64,
) -> Vec<Placed> {
    if ends.is_empty() {
        return Vec::new();
    }
    let wanted: Vec<f64> = ends
        .iter()
        .map(|e| e[0] * along[0] + e[1] * along[1])
        .collect();
    let placed = spread(&wanted, 2.0 * r + pad);
    let shifts: Vec<f64> = placed.iter().zip(&wanted).map(|(p, w)| p - w).collect();
    let moved = shifts.iter().any(|s| s.abs() > STILL * r);
    let reach = stub + if moved { jog } else { 0.0 };
    ends.iter()
        .zip(shifts)
        .map(|(e, s)| {
            let s = if s.abs() <= STILL * r { 0.0 } else { s };
            let edge = [
                e[0] + out[0] * reach + along[0] * s,
                e[1] + out[1] * reach + along[1] * s,
            ];
            let center = [edge[0] + out[0] * r, edge[1] + out[1] * r];
            let path = if s == 0.0 {
                vec![*e, edge]
            } else {
                vec![*e, [e[0] + out[0] * stub, e[1] + out[1] * stub], edge]
            };
            Placed {
                center,
                path,
                shift: s,
            }
        })
        .collect()
}
