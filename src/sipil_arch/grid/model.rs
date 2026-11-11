//! A grid and the tag its entities carry. Pure (`codec`-free): the adapter maps
//! the tag to XDATA.
//!
//! The source of truth of a grid is its text (see `spec`): coordinates and
//! labels of the X lines, the Y lines and the levels. That is what the popup
//! edits, what a project keeps and what the first entity of the grid remembers,
//! so a drawing can be read back and edited exactly as it was typed.

use super::spec::{self, Axis, Bubble, Row};
use crate::sipil_wall::engine::XVal;

pub const XDATA_APP: &str = "SIPILGRID";
pub const XDATA_VERSION: i16 = 1;

/// An XDATA string holds at most 255 bytes; the grid texts stay well inside.
pub const MAX_TEXT: usize = 240;

/// How long a line is when the other axis has no lines to measure against, mm.
pub const FALLBACK_MM: f64 = 6000.0;

/// How far the lines run beyond the outer lines of the other axis, mm.
pub const DEFAULT_EXTEND_MM: f64 = 1000.0;

#[derive(Clone, Debug, PartialEq)]
pub struct GridDef {
    pub x_coords: String,
    pub x_labels: String,
    pub x_bubble: Bubble,
    pub y_coords: String,
    pub y_labels: String,
    pub y_bubble: Bubble,
    pub z_coords: String,
    pub z_labels: String,
    pub extend_mm: f64,
}

impl Default for GridDef {
    /// The house grid: A-C by 1-4, two levels.
    fn default() -> Self {
        GridDef {
            x_coords: "0.00 2*3000.00".into(),
            x_labels: "A B C".into(),
            x_bubble: Bubble::Start,
            y_coords: "0.00 3*3000.00".into(),
            y_labels: "1 2 3 4".into(),
            y_bubble: Bubble::Start,
            z_coords: "0.00 3000.00".into(),
            z_labels: "Level 1, Level 2".into(),
            extend_mm: DEFAULT_EXTEND_MM,
        }
    }
}

/// A grid with its texts read: the lines and levels in millimetres.
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub x: Vec<Row>,
    pub y: Vec<Row>,
    pub z: Vec<Row>,
    pub x_bubble: Bubble,
    pub y_bubble: Bubble,
    pub extend_mm: f64,
}

impl GridDef {
    /// The whole definition as one line of text (a custom property of a project).
    pub fn to_text(&self) -> String {
        crate::sipil_arch::fields::join(&[
            self.x_coords.clone(),
            self.x_labels.clone(),
            self.x_bubble.code().to_string(),
            self.y_coords.clone(),
            self.y_labels.clone(),
            self.y_bubble.code().to_string(),
            self.z_coords.clone(),
            self.z_labels.clone(),
            self.extend_mm.to_string(),
        ])
    }

    /// The definition [`GridDef::to_text`] wrote, or `None` for anything else.
    pub fn from_text(text: &str) -> Option<GridDef> {
        let f = crate::sipil_arch::fields::split(text);
        let [xc, xl, xb, yc, yl, yb, zc, zl, ext] = <[String; 9]>::try_from(f).ok()?;
        Some(GridDef {
            x_coords: xc,
            x_labels: xl,
            x_bubble: Bubble::from_code(xb.parse().ok()?),
            y_coords: yc,
            y_labels: yl,
            y_bubble: Bubble::from_code(yb.parse().ok()?),
            z_coords: zc,
            z_labels: zl,
            extend_mm: ext.parse().ok()?,
        })
    }

    /// Read the texts, or say what is wrong with them.
    pub fn resolve(&self) -> Result<Grid, String> {
        for (name, t) in [
            ("X coordinates", &self.x_coords),
            ("X labels", &self.x_labels),
            ("Y coordinates", &self.y_coords),
            ("Y labels", &self.y_labels),
            ("Level elevations", &self.z_coords),
            ("Level names", &self.z_labels),
        ] {
            if t.len() > MAX_TEXT {
                return Err(format!("{name}: at most {MAX_TEXT} characters."));
            }
        }
        if !self.extend_mm.is_finite() || self.extend_mm < 0.0 || self.extend_mm > 20000.0 {
            return Err("The extension must be between 0 and 20000 mm.".to_string());
        }
        let x = spec::rows(&self.x_coords, &self.x_labels, Axis::X)
            .map_err(|e| format!("X lines: {e}"))?;
        let y = spec::rows(&self.y_coords, &self.y_labels, Axis::Y)
            .map_err(|e| format!("Y lines: {e}"))?;
        let z = spec::rows(&self.z_coords, &self.z_labels, Axis::Z)
            .map_err(|e| format!("Levels: {e}"))?;
        if x.is_empty() && y.is_empty() {
            return Err("The grid needs at least one X or Y line.".to_string());
        }
        Ok(Grid {
            x,
            y,
            z,
            x_bubble: self.x_bubble,
            y_bubble: self.y_bubble,
            extend_mm: self.extend_mm,
        })
    }
}

fn span(rows: &[Row]) -> (f64, f64) {
    let lo = rows.iter().map(|r| r.pos).fold(f64::INFINITY, f64::min);
    let hi = rows.iter().map(|r| r.pos).fold(f64::NEG_INFINITY, f64::max);
    if rows.is_empty() {
        (0.0, FALLBACK_MM)
    } else {
        (lo, hi)
    }
}

impl Grid {
    /// The extent of the X lines (their positions), or `0..FALLBACK_MM` with none.
    pub fn x_span(&self) -> (f64, f64) {
        span(&self.x)
    }

    pub fn y_span(&self) -> (f64, f64) {
        span(&self.y)
    }

    /// The levels; a grid without any has the one ground level.
    pub fn levels(&self) -> Vec<Row> {
        if self.z.is_empty() {
            vec![Row {
                id: "Level 1".to_string(),
                pos: 0.0,
            }]
        } else {
            self.z.clone()
        }
    }
}

/// What a grid entity remembers: which view it belongs to, and, on the anchor
/// (the first entity) only, the whole definition.
#[derive(Clone, Debug, PartialEq)]
pub struct GTag {
    /// A `ViewSpec` text (`plan:arch:0`, `elev:north`...).
    pub view: String,
    /// The drawing scale (the `n` of 1:n) the bubbles and tags were sized for.
    pub scale: f64,
    pub def: Option<GridDef>,
}

impl GTag {
    pub fn encode(&self) -> Vec<XVal> {
        let mut v = vec![
            XVal::I16(XDATA_VERSION),
            XVal::Str(self.view.clone()),
            XVal::Real(self.scale),
            XVal::I16(i16::from(self.def.is_some())),
        ];
        if let Some(d) = &self.def {
            v.extend([
                XVal::Str(d.x_coords.clone()),
                XVal::Str(d.x_labels.clone()),
                XVal::I16(d.x_bubble.code()),
                XVal::Str(d.y_coords.clone()),
                XVal::Str(d.y_labels.clone()),
                XVal::I16(d.y_bubble.code()),
                XVal::Str(d.z_coords.clone()),
                XVal::Str(d.z_labels.clone()),
                XVal::Real(d.extend_mm),
            ]);
        }
        v
    }

    pub fn decode(vals: &[XVal]) -> Option<GTag> {
        let mut it = vals.iter();
        let Some(XVal::I16(XDATA_VERSION)) = it.next() else {
            return None;
        };
        let Some(XVal::Str(view)) = it.next() else {
            return None;
        };
        let Some(XVal::Real(scale)) = it.next() else {
            return None;
        };
        let Some(XVal::I16(has_def)) = it.next() else {
            return None;
        };
        if *has_def == 0 {
            return it.next().is_none().then(|| GTag {
                view: view.clone(),
                scale: *scale,
                def: None,
            });
        }
        let mut s = || match it.next() {
            Some(XVal::Str(s)) => Some(s.clone()),
            _ => None,
        };
        let x_coords = s()?;
        let x_labels = s()?;
        let xb = match it.next() {
            Some(XVal::I16(b)) => *b,
            _ => return None,
        };
        let mut s = || match it.next() {
            Some(XVal::Str(s)) => Some(s.clone()),
            _ => None,
        };
        let y_coords = s()?;
        let y_labels = s()?;
        let yb = match it.next() {
            Some(XVal::I16(b)) => *b,
            _ => return None,
        };
        let mut s = || match it.next() {
            Some(XVal::Str(s)) => Some(s.clone()),
            _ => None,
        };
        let z_coords = s()?;
        let z_labels = s()?;
        let Some(XVal::Real(extend_mm)) = it.next() else {
            return None;
        };
        it.next().is_none().then(|| GTag {
            view: view.clone(),
            scale: *scale,
            def: Some(GridDef {
                x_coords,
                x_labels,
                x_bubble: Bubble::from_code(xb),
                y_coords,
                y_labels,
                y_bubble: Bubble::from_code(yb),
                z_coords,
                z_labels,
                extend_mm: *extend_mm,
            }),
        })
    }
}
