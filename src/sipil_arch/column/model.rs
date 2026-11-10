//! What a column is: its definition and the tag its entities carry.
//!
//! Pure: no iced, no host types. Sizes of the section come from the profile
//! (millimetres); the position and everything drawn are in drawing units.

use crate::sipil_arch::steel::catalog;
use crate::sipil_wall::engine::{XVal, P};

pub const XDATA_APP: &str = "SIPILCOLUMN";
const XDATA_VERSION: i16 = 1;

/// Layer name and ACI colour of each part: section outline, fill, label.
pub const LAYERS: [(&str, i16); 3] = [("COLUMN-SECTION", 5), ("COLUMN-HATCH", 8), ("COLUMN-TEXT", 2)];

/// How the section is filled in the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    None,
    Solid,
    /// Steel hatch pattern (ANSI32).
    Hatch,
}

impl Fill {
    fn code(self) -> i16 {
        match self {
            Fill::None => 0,
            Fill::Solid => 1,
            Fill::Hatch => 2,
        }
    }

    fn from_code(c: i16) -> Option<Fill> {
        match c {
            0 => Some(Fill::None),
            1 => Some(Fill::Solid),
            2 => Some(Fill::Hatch),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColumnDef {
    /// Column number, unique in the drawing.
    pub id: u32,
    /// Profile key (`gunung-garuda/WF-200X100X5.5X8`).
    pub profile: String,
    /// Centre of gravity of the section, drawing units.
    pub at: P,
    /// Rotation of the section, degrees counter-clockwise.
    pub rot_deg: f64,
    /// Mark shown before the profile name (`K1`); may be empty.
    pub mark: String,
    pub fill: Fill,
    pub label: bool,
    /// Text height of the label, millimetres on paper-scale 1:1 of the model.
    pub text_mm: f64,
    /// Drawing units per metre, to turn the section's millimetres into units.
    pub upm: f64,
}

impl ColumnDef {
    pub fn new(profile: &catalog::Profile, upm: f64) -> ColumnDef {
        ColumnDef {
            id: 0,
            profile: catalog::key(profile),
            at: [0.0, 0.0],
            rot_deg: 0.0,
            mark: String::new(),
            fill: Fill::Solid,
            label: true,
            text_mm: 150.0,
            upm,
        }
    }

    pub fn sanitized(&self) -> ColumnDef {
        let fix = |v: f64, min: f64, max: f64| if v.is_finite() { v.clamp(min, max) } else { min };
        ColumnDef {
            rot_deg: if self.rot_deg.is_finite() { self.rot_deg.rem_euclid(360.0) } else { 0.0 },
            text_mm: fix(self.text_mm, 20.0, 2000.0),
            upm: if self.upm.is_finite() && self.upm > 0.0 { self.upm } else { 1000.0 },
            mark: self.mark.trim().to_string(),
            ..self.clone()
        }
    }
}

/// What every column entity remembers: the number, and for the anchor (the
/// section outline) the definition and where the outline started, so a MOVE of
/// the column is noticed.
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub id: u32,
    pub def: Option<ColumnDef>,
    pub origin: Option<P>,
}

impl Tag {
    pub fn encode(&self) -> Vec<XVal> {
        let mut v = vec![XVal::I16(XDATA_VERSION), XVal::I32(self.id as i32), XVal::I16(i16::from(self.def.is_some()))];
        if let Some(d) = &self.def {
            v.extend([
                XVal::Str(d.profile.clone()),
                XVal::Pt(d.at),
                XVal::Real(d.rot_deg),
                XVal::Str(d.mark.clone()),
                XVal::I16(d.fill.code()),
                XVal::I16(i16::from(d.label)),
                XVal::Real(d.text_mm),
                XVal::Real(d.upm),
                XVal::Pt(self.origin.unwrap_or(d.at)),
            ]);
        }
        v
    }

    pub fn decode(vals: &[XVal]) -> Option<Tag> {
        let mut it = vals.iter();
        let int16 = |v: Option<&XVal>| match v? {
            XVal::I16(x) => Some(*x),
            _ => None,
        };
        let real = |v: Option<&XVal>| match v? {
            XVal::Real(x) => Some(*x),
            _ => None,
        };
        let string = |v: Option<&XVal>| match v? {
            XVal::Str(x) => Some(x.clone()),
            _ => None,
        };
        let pt = |v: Option<&XVal>| match v? {
            XVal::Pt(x) => Some(*x),
            _ => None,
        };
        if int16(it.next())? != XDATA_VERSION {
            return None;
        }
        let id = match it.next()? {
            XVal::I32(x) => u32::try_from(*x).ok()?,
            _ => return None,
        };
        if int16(it.next())? != 1 {
            return Some(Tag { id, def: None, origin: None });
        }
        let def = ColumnDef {
            id,
            profile: string(it.next())?,
            at: pt(it.next())?,
            rot_deg: real(it.next())?,
            mark: string(it.next())?,
            fill: Fill::from_code(int16(it.next())?)?,
            label: int16(it.next())? == 1,
            text_mm: real(it.next())?,
            upm: real(it.next())?,
        };
        let origin = pt(it.next())?;
        Some(Tag { id, def: Some(def), origin: Some(origin) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_round_trip() {
        let mut d = ColumnDef::new(catalog::default_profile(), 1000.0);
        d.id = 4;
        d.at = [1500.5, -20.0];
        d.rot_deg = 90.0;
        d.mark = "K1".into();
        d.fill = Fill::Hatch;
        for tag in [Tag { id: 4, def: Some(d.clone()), origin: Some([1.0, 2.0]) }, Tag { id: 4, def: None, origin: None }] {
            assert_eq!(Tag::decode(&tag.encode()), Some(tag));
        }
        assert_eq!(Tag::decode(&[]), None);
        assert_eq!(Tag::decode(&[XVal::I16(7), XVal::I32(1), XVal::I16(0)]), None);
        let mut cut = Tag { id: 1, def: Some(d), origin: None }.encode();
        cut.truncate(cut.len() - 2);
        assert_eq!(Tag::decode(&cut), None);
    }

    #[test]
    fn sanitizing_clamps() {
        let mut d = ColumnDef::new(catalog::default_profile(), 1000.0);
        d.rot_deg = -90.0;
        d.text_mm = f64::NAN;
        d.upm = 0.0;
        d.mark = "  K2 ".into();
        let s = d.sanitized();
        assert_eq!((s.rot_deg, s.text_mm, s.upm, s.mark.as_str()), (270.0, 20.0, 1000.0, "K2"));
    }
}
