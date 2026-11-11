//! The drawing of a grid as primitives: a plan (lines and bubbles), or a
//! vertical view (an elevation or a section: the lines of one axis standing up,
//! the levels across them). Pure geometry: no host and no `codec`, so it is
//! tested alone.
//!
//! Everything is built in millimetres of model space at a drawing scale
//! (`scale` = 100 for 1:100) and converted to the drawing's units at the end
//! (`upm` model units per metre). Bubble and text sizes are given on the sheet
//! in paper millimetres and multiplied by the scale, so a bubble prints the
//! same size at any scale.
//!
//! Orientation, as a person looking at the building sees it (north is +Y, east
//! +X): a South elevation has west on the left (X grows to the right); North
//! the reverse; East has north on the right (Y grows to the right); West the
//! reverse. A section along a numbered line (`1-1`, a Y line) looks north and
//! shows the X lines left to right; along a lettered line (`A-A`, an X line) it
//! looks west and shows the Y lines left to right.

use super::bubble::{bubble_row, P};
use super::model::Grid;
use super::spec::{Axis, Bubble, Row};
use super::view::{Side, ViewSpec};

/// Bubble radius on the sheet, paper mm.
pub const BUBBLE_R_PAPER_MM: f64 = 4.0;
/// Level tag text height on the sheet, paper mm.
pub const TEXT_PAPER_MM: f64 = 2.5;
/// Line end to bubble edge, paper mm.
const STUB_PAPER_MM: f64 = 2.0;
/// Least free space between two bubbles, paper mm.
const PAD_PAPER_MM: f64 = 1.0;
/// How far a vertical view reaches above the top level when it has no storey to
/// copy, mm.
const DEFAULT_STOREY_MM: f64 = 3000.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GLayer {
    Grid,
    Level,
}

impl GLayer {
    pub fn name(self) -> &'static str {
        match self {
            GLayer::Grid => "GRID",
            GLayer::Level => "LEVEL",
        }
    }
}

/// Layers the grid uses, with their colour (ACI).
pub const LAYERS: [(&str, i16); 2] = [("GRID", 1), ("LEVEL", 5)];

/// Text placement: centred on the point, or starting at it (left), both
/// vertically centred.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Center,
    Left,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GShape {
    Line {
        a: P,
        b: P,
    },
    Circle {
        c: P,
        r: f64,
    },
    Text {
        at: P,
        height: f64,
        text: String,
        anchor: Anchor,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct GPrim {
    pub layer: GLayer,
    pub shape: GShape,
}

fn line(layer: GLayer, a: P, b: P) -> GPrim {
    GPrim {
        layer,
        shape: GShape::Line { a, b },
    }
}

/// Text height that fits `label` in a bubble of radius `r`.
fn label_height(label: &str, r: f64) -> f64 {
    match label.chars().count() {
        0 | 1 => r,
        2 => r * 0.8,
        _ => r * 0.6,
    }
}

/// Bubbles for one side of a set of lines: the circles, their names and the
/// leaders from the line ends.
fn bubbles(out: &mut Vec<GPrim>, labels: &[&str], ends: &[P], dir: P, along: P, scale: f64) {
    let r = BUBBLE_R_PAPER_MM * scale;
    let placed = bubble_row(
        ends,
        dir,
        along,
        r,
        STUB_PAPER_MM * scale,
        r,
        PAD_PAPER_MM * scale,
    );
    for (label, p) in labels.iter().zip(placed) {
        for w in p.path.windows(2) {
            out.push(line(GLayer::Grid, w[0], w[1]));
        }
        out.push(GPrim {
            layer: GLayer::Grid,
            shape: GShape::Circle { c: p.center, r },
        });
        out.push(GPrim {
            layer: GLayer::Grid,
            shape: GShape::Text {
                at: p.center,
                height: label_height(label, r),
                text: (*label).to_string(),
                anchor: Anchor::Center,
            },
        });
    }
}

/// Plan: the X lines (vertical) and the Y lines (horizontal) with bubbles.
fn plan(g: &Grid, scale: f64) -> Vec<GPrim> {
    let (x0, x1) = g.x_span();
    let (y0, y1) = g.y_span();
    let e = g.extend_mm;
    let mut out = Vec::new();
    let (xs, xe) = g.x_bubble.sides();
    let (ys, ye) = g.y_bubble.sides();

    let x_labels: Vec<&str> = g.x.iter().map(|r| r.id.as_str()).collect();
    let bottoms: Vec<P> = g.x.iter().map(|r| [r.pos, y0 - e]).collect();
    let tops: Vec<P> = g.x.iter().map(|r| [r.pos, y1 + e]).collect();
    for (b, t) in bottoms.iter().zip(&tops) {
        out.push(line(GLayer::Grid, *b, *t));
    }
    if xs {
        bubbles(
            &mut out,
            &x_labels,
            &bottoms,
            [0.0, -1.0],
            [1.0, 0.0],
            scale,
        );
    }
    if xe {
        bubbles(&mut out, &x_labels, &tops, [0.0, 1.0], [1.0, 0.0], scale);
    }

    let y_labels: Vec<&str> = g.y.iter().map(|r| r.id.as_str()).collect();
    let lefts: Vec<P> = g.y.iter().map(|r| [x0 - e, r.pos]).collect();
    let rights: Vec<P> = g.y.iter().map(|r| [x1 + e, r.pos]).collect();
    for (l, r) in lefts.iter().zip(&rights) {
        out.push(line(GLayer::Grid, *l, *r));
    }
    if ys {
        bubbles(&mut out, &y_labels, &lefts, [-1.0, 0.0], [0.0, 1.0], scale);
    }
    if ye {
        bubbles(&mut out, &y_labels, &rights, [1.0, 0.0], [0.0, 1.0], scale);
    }
    out
}

/// `+3.50` / `-0.50`: an elevation in metres with its sign.
pub fn elevation_text(mm: f64) -> String {
    format!("{:+.2}", mm / 1000.0)
}

/// A vertical view: grid lines standing at `lines` (label, u in mm), levels
/// across them, bubbles at the foot (or head, or both) of the lines.
fn vertical(
    lines: &[(String, f64)],
    levels: &[Row],
    bubble: Bubble,
    extend: f64,
    scale: f64,
) -> Vec<GPrim> {
    let mut lines: Vec<(String, f64)> = lines.to_vec();
    lines.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let (u0, u1) = if lines.is_empty() {
        (0.0, super::model::FALLBACK_MM)
    } else {
        (lines[0].1, lines[lines.len() - 1].1)
    };
    let z_lo = levels.first().map_or(0.0, |l| l.pos);
    let z_top_level = levels.last().map_or(0.0, |l| l.pos);
    let storey = if levels.len() >= 2 {
        z_top_level - levels[levels.len() - 2].pos
    } else {
        DEFAULT_STOREY_MM
    };
    let z_hi = z_top_level + storey;
    let (foot, head) = bubble.sides();
    let mut out = Vec::new();

    let labels: Vec<&str> = lines.iter().map(|l| l.0.as_str()).collect();
    let feet: Vec<P> = lines.iter().map(|l| [l.1, z_lo - extend]).collect();
    let heads: Vec<P> = lines.iter().map(|l| [l.1, z_hi + extend]).collect();
    for (f, h) in feet.iter().zip(&heads) {
        out.push(line(GLayer::Grid, *f, *h));
    }
    if foot {
        bubbles(&mut out, &labels, &feet, [0.0, -1.0], [1.0, 0.0], scale);
    }
    if head {
        bubbles(&mut out, &labels, &heads, [0.0, 1.0], [1.0, 0.0], scale);
    }

    let text_h = TEXT_PAPER_MM * scale;
    for lv in levels {
        out.push(line(
            GLayer::Level,
            [u0 - extend, lv.pos],
            [u1 + extend, lv.pos],
        ));
        out.push(GPrim {
            layer: GLayer::Level,
            shape: GShape::Text {
                at: [u1 + extend + text_h, lv.pos],
                height: text_h,
                text: format!("{}  {}", lv.id, elevation_text(lv.pos)),
                anchor: Anchor::Left,
            },
        });
    }
    out
}

fn mirrored(rows: &[Row]) -> Vec<(String, f64)> {
    rows.iter().map(|r| (r.id.clone(), -r.pos)).collect()
}

fn straight(rows: &[Row]) -> Vec<(String, f64)> {
    rows.iter().map(|r| (r.id.clone(), r.pos)).collect()
}

/// The primitives of `view` of the grid, in model millimetres, at drawing scale
/// `scale` (1:`scale`).
pub fn build_mm(g: &Grid, view: &ViewSpec, scale: f64) -> Vec<GPrim> {
    let levels = g.levels();
    match view {
        ViewSpec::Plan { .. } => plan(g, scale),
        ViewSpec::Elevation(side) => {
            let (lines, bubble) = match side {
                Side::South => (straight(&g.x), g.x_bubble),
                Side::North => (mirrored(&g.x), g.x_bubble),
                Side::East => (straight(&g.y), g.y_bubble),
                Side::West => (mirrored(&g.y), g.y_bubble),
            };
            vertical(&lines, &levels, bubble, g.extend_mm, scale)
        }
        ViewSpec::Section { along, .. } => {
            // Along a numbered (Y) line the section looks north and shows the X lines.
            let (lines, bubble) = if *along == Axis::Y {
                (straight(&g.x), g.x_bubble)
            } else {
                (straight(&g.y), g.y_bubble)
            };
            vertical(&lines, &levels, bubble, g.extend_mm, scale)
        }
    }
}

fn scale_point(p: P, k: f64) -> P {
    [p[0] * k, p[1] * k]
}

/// [`build_mm`] converted to a drawing in which one metre is `upm` units.
pub fn build(g: &Grid, view: &ViewSpec, upm: f64, scale: f64) -> Vec<GPrim> {
    let k = upm / 1000.0;
    build_mm(g, view, scale)
        .into_iter()
        .map(|p| GPrim {
            layer: p.layer,
            shape: match p.shape {
                GShape::Line { a, b } => GShape::Line {
                    a: scale_point(a, k),
                    b: scale_point(b, k),
                },
                GShape::Circle { c, r } => GShape::Circle {
                    c: scale_point(c, k),
                    r: r * k,
                },
                GShape::Text {
                    at,
                    height,
                    text,
                    anchor,
                } => GShape::Text {
                    at: scale_point(at, k),
                    height: height * k,
                    text,
                    anchor,
                },
            },
        })
        .collect()
}

/// The box around the primitives, `(min, max)`. Text is measured as 0.7 of its
/// height per character.
pub fn extents(prims: &[GPrim]) -> Option<(P, P)> {
    let mut lo = [f64::INFINITY; 2];
    let mut hi = [f64::NEG_INFINITY; 2];
    let mut grow = |p: P| {
        for i in 0..2 {
            lo[i] = lo[i].min(p[i]);
            hi[i] = hi[i].max(p[i]);
        }
    };
    for p in prims {
        match &p.shape {
            GShape::Line { a, b } => {
                grow(*a);
                grow(*b);
            }
            GShape::Circle { c, r } => {
                grow([c[0] - r, c[1] - r]);
                grow([c[0] + r, c[1] + r]);
            }
            GShape::Text {
                at,
                height,
                text,
                anchor,
            } => {
                let w = 0.7 * height * text.chars().count() as f64;
                let (x0, x1) = match anchor {
                    Anchor::Center => (at[0] - w / 2.0, at[0] + w / 2.0),
                    Anchor::Left => (at[0], at[0] + w),
                };
                grow([x0, at[1] - height / 2.0]);
                grow([x1, at[1] + height / 2.0]);
            }
        }
    }
    (lo[0].is_finite() && hi[0].is_finite()).then_some((lo, hi))
}

/// The drawing scales offered, 1:n.
pub const SCALES: [u32; 12] = [10, 20, 25, 50, 75, 100, 125, 150, 200, 250, 500, 1000];

/// The smallest standard scale at which the view, bubbles and tags included,
/// fits a `w` x `h` paper-mm area. The largest scale if none does.
pub fn pick_scale(g: &Grid, view: &ViewSpec, w: f64, h: f64) -> u32 {
    for s in SCALES {
        let prims = build_mm(g, view, s as f64);
        if let Some((lo, hi)) = extents(&prims) {
            if (hi[0] - lo[0]) / s as f64 <= w && (hi[1] - lo[1]) / s as f64 <= h {
                return s;
            }
        }
    }
    SCALES[SCALES.len() - 1]
}
