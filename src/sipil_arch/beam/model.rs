//! What a beam is: its definition and the tag its entities carry.
//!
//! Pure: no iced, no host types. A beam runs between two points in drawing
//! units; the width of its outline comes from the profile (millimetres).

use crate::sipil_arch::steel::catalog;
use crate::sipil_wall::engine::{XVal, P};

pub const XDATA_APP: &str = "SIPILBEAM";
const XDATA_VERSION: i16 = 1;

/// Layer name and ACI colour of each part: outline, axis, label.
pub const LAYERS: [(&str, i16); 3] = [("BEAM-OUTLINE", 6), ("BEAM-AXIS", 1), ("BEAM-TEXT", 2)];

/// How the beam is drawn in the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// Both flange edges to scale, with the axis.
    Outline,
    /// The axis only (a single line).
    Centerline,
}

impl Style {
    fn code(self) -> i16 {
        match self {
            Style::Outline => 0,
            Style::Centerline => 1,
        }
    }

    fn from_code(c: i16) -> Option<Style> {
        match c {
            0 => Some(Style::Outline),
            1 => Some(Style::Centerline),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BeamDef {
    pub id: u32,
    /// Profile key (`gunung-garuda/WF-300X150X6.5X9`).
    pub profile: String,
    /// Start and end of the axis, drawing units.
    pub a: P,
    pub b: P,
    /// Mark shown before the profile name (`B1`); may be empty.
    pub mark: String,
    pub style: Style,
    pub label: bool,
    /// Text height of the label, millimetres.
    pub text_mm: f64,
    /// Drawing units per metre.
    pub upm: f64,
}

impl BeamDef {
    pub fn new(profile: &catalog::Profile, upm: f64) -> BeamDef {
        BeamDef {
            id: 0,
            profile: catalog::key(profile),
            a: [0.0, 0.0],
            b: [0.0, 0.0],
            mark: String::new(),
            style: Style::Outline,
            label: true,
            text_mm: 150.0,
            upm,
        }
    }

    pub fn sanitized(&self) -> BeamDef {
        let fix = |v: f64, min: f64, max: f64| if v.is_finite() { v.clamp(min, max) } else { min };
        BeamDef {
            text_mm: fix(self.text_mm, 20.0, 2000.0),
            upm: if self.upm.is_finite() && self.upm > 0.0 { self.upm } else { 1000.0 },
            mark: self.mark.trim().to_string(),
            ..self.clone()
        }
    }

    pub fn length(&self) -> f64 {
        (self.b[0] - self.a[0]).hypot(self.b[1] - self.a[1])
    }
}

/// What every beam entity remembers: the number, and for the anchor (the axis
/// line) the definition, so a MOVE of the beam is noticed from the line's start.
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub id: u32,
    pub def: Option<BeamDef>,
}

impl Tag {
    pub fn encode(&self) -> Vec<XVal> {
        let mut v = vec![XVal::I16(XDATA_VERSION), XVal::I32(self.id as i32), XVal::I16(i16::from(self.def.is_some()))];
        if let Some(d) = &self.def {
            v.extend([
                XVal::Str(d.profile.clone()),
                XVal::Pt(d.a),
                XVal::Pt(d.b),
                XVal::Str(d.mark.clone()),
                XVal::I16(d.style.code()),
                XVal::I16(i16::from(d.label)),
                XVal::Real(d.text_mm),
                XVal::Real(d.upm),
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
            return Some(Tag { id, def: None });
        }
        let def = BeamDef {
            id,
            profile: string(it.next())?,
            a: pt(it.next())?,
            b: pt(it.next())?,
            mark: string(it.next())?,
            style: Style::from_code(int16(it.next())?)?,
            label: int16(it.next())? == 1,
            text_mm: real(it.next())?,
            upm: real(it.next())?,
        };
        Some(Tag { id, def: Some(def) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tags_round_trip() {
        let mut d = BeamDef::new(catalog::default_profile(), 1000.0);
        d.id = 2;
        d.a = [0.0, 1.5];
        d.b = [4000.0, 1.5];
        d.mark = "B1".into();
        d.style = Style::Centerline;
        for tag in [Tag { id: 2, def: Some(d.clone()) }, Tag { id: 2, def: None }] {
            assert_eq!(Tag::decode(&tag.encode()), Some(tag));
        }
        assert_eq!(Tag::decode(&[]), None);
        assert_eq!(Tag::decode(&[XVal::I16(3), XVal::I32(1), XVal::I16(0)]), None);
        let mut cut = Tag { id: 1, def: Some(d.clone()) }.encode();
        cut.truncate(cut.len() - 1);
        assert_eq!(Tag::decode(&cut), None);
        assert!((d.length() - 4000.0).abs() < 1e-9);
    }
}
