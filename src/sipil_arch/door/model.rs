//! What a door is: its definition, the primitives it is drawn from, and the
//! tag its entities carry (stored as XDATA by the adapter).
//!
//! Pure: no iced, no `codec`, no host types. Positions along a wall are kept
//! in metres from the start of a run of the wall's axis, so a door follows
//! its wall when the wall is rebuilt in another unit or place.

use crate::sipil_wall::engine::{Shape, XVal, P};

/// Application name of the XDATA record (a registered APPID).
pub const XDATA_APP: &str = "SIPILDOOR";
const XDATA_VERSION: i16 = 1;

/// Which kind of part a primitive is; each has its own layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DRole {
    /// Frame (kusen) in the plan.
    Frame,
    /// Leaf (daun pintu) in the plan.
    Leaf,
    /// Swing arc of an open leaf.
    Swing,
    /// Lines of the detail sheet.
    Detail,
    /// Hatching in the detail sheet (wall, lintel).
    DetailHatch,
    /// Text of the detail sheet.
    Text,
    /// Dimension lines of the detail sheet.
    Dim,
}

/// Layer name and ACI colour of each role, in `DRole` order.
pub const LAYERS: [(&str, i16); 7] = [
    ("DOOR-FRAME", 30),
    ("DOOR-LEAF", 4),
    ("DOOR-SWING", 8),
    ("DOOR-DETAIL", 7),
    ("DOOR-DETAIL-HATCH", 8),
    ("DOOR-TEXT", 2),
    ("DOOR-DIM", 3),
];

impl DRole {
    pub fn layer(self) -> &'static str {
        LAYERS[self.code() as usize].0
    }

    pub fn code(self) -> i16 {
        match self {
            DRole::Frame => 0,
            DRole::Leaf => 1,
            DRole::Swing => 2,
            DRole::Detail => 3,
            DRole::DetailHatch => 4,
            DRole::Text => 5,
            DRole::Dim => 6,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DShape {
    /// A line, outline or hatch, the same shapes the wall uses.
    Geo(Shape),
    /// One line of text, left-aligned at `at` (baseline), `rot` radians.
    Text {
        at: P,
        height: f64,
        text: String,
        rot: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DPrim {
    pub role: DRole,
    pub shape: DShape,
}

/// A door in a wall.
#[derive(Clone, Debug, PartialEq)]
pub struct DoorDef {
    /// Door number, unique in the drawing.
    pub id: u32,
    /// The wall it is in (the wall's id).
    pub wall: u32,
    /// Run of the wall's axis (see `engine::axis`).
    pub seg: u32,
    /// Centre of the opening along that run from its start, metres.
    pub pos_m: f64,
    /// Type mark (`P2`).
    pub code: String,
    /// Drawn open (leaf swung) or closed.
    pub open: bool,
    /// Opening angle in degrees when open.
    pub angle_deg: f64,
    /// Clear width between the frames, metres.
    pub width_m: f64,
    /// Clear height, metres.
    pub height_m: f64,
    /// Hinge at the start of the run (otherwise at its end). Double doors
    /// are hinged at both ends.
    pub hinge_start: bool,
    /// Swings to the left of the run's direction (otherwise to the right).
    pub swing_left: bool,
}

impl DoorDef {
    /// A door of `ty` with the type's default size, not yet in a wall.
    pub fn of_type(ty: &super::catalog::DoorType) -> DoorDef {
        DoorDef {
            id: 0,
            wall: 0,
            seg: 0,
            pos_m: 0.0,
            code: ty.code.to_string(),
            open: true,
            angle_deg: 90.0,
            width_m: ty.width_m,
            height_m: ty.height_m,
            hinge_start: true,
            swing_left: true,
        }
    }

    /// Clamp to values the geometry can use.
    pub fn sanitized(&self) -> DoorDef {
        let fix = |v: f64, min: f64, max: f64| if v.is_finite() { v.clamp(min, max) } else { min };
        DoorDef {
            pos_m: fix(self.pos_m, 0.0, 1.0e6),
            angle_deg: fix(self.angle_deg, 0.0, 180.0),
            width_m: fix(self.width_m, 0.4, 5.0),
            height_m: fix(self.height_m, 1.5, 4.0),
            ..self.clone()
        }
    }
}

/// What every door entity remembers: the door's number, and for the anchor
/// (the first entity of the door) the definition.
#[derive(Clone, Debug, PartialEq)]
pub enum DTag {
    Door { id: u32, def: Option<DoorDef> },
    /// A detail sheet entity, by door type; the anchor knows where the sheet
    /// starts (its lower-left corner).
    Sheet { code: String, origin: Option<P> },
}

impl DTag {
    pub fn encode(&self) -> Vec<XVal> {
        match self {
            DTag::Door { id, def } => {
                let mut v = vec![
                    XVal::I16(XDATA_VERSION),
                    XVal::I16(0),
                    XVal::I32(*id as i32),
                    XVal::I16(i16::from(def.is_some())),
                ];
                if let Some(d) = def {
                    v.extend([
                        XVal::I32(d.wall as i32),
                        XVal::I32(d.seg as i32),
                        XVal::Real(d.pos_m),
                        XVal::Str(d.code.clone()),
                        XVal::I16(i16::from(d.open)),
                        XVal::Real(d.angle_deg),
                        XVal::Real(d.width_m),
                        XVal::Real(d.height_m),
                        XVal::I16(i16::from(d.hinge_start)),
                        XVal::I16(i16::from(d.swing_left)),
                    ]);
                }
                v
            }
            DTag::Sheet { code, origin } => {
                let mut v = vec![
                    XVal::I16(XDATA_VERSION),
                    XVal::I16(1),
                    XVal::Str(code.clone()),
                    XVal::I16(i16::from(origin.is_some())),
                ];
                if let Some(o) = origin {
                    v.push(XVal::Pt(*o));
                }
                v
            }
        }
    }

    /// Read a tag back; `None` for anything that is not a door tag.
    pub fn decode(vals: &[XVal]) -> Option<DTag> {
        type It<'a> = std::slice::Iter<'a, XVal>;
        fn i16_of(it: &mut It) -> Option<i16> {
            match it.next()? {
                XVal::I16(v) => Some(*v),
                _ => None,
            }
        }
        fn i32_of(it: &mut It) -> Option<i32> {
            match it.next()? {
                XVal::I32(v) => Some(*v),
                _ => None,
            }
        }
        fn real_of(it: &mut It) -> Option<f64> {
            match it.next()? {
                XVal::Real(v) => Some(*v),
                _ => None,
            }
        }
        fn str_of(it: &mut It) -> Option<String> {
            match it.next()? {
                XVal::Str(v) => Some(v.clone()),
                _ => None,
            }
        }
        let mut it = vals.iter();
        if i16_of(&mut it)? != XDATA_VERSION {
            return None;
        }
        match i16_of(&mut it)? {
            0 => {
                let id = u32::try_from(i32_of(&mut it)?).ok()?;
                let def = if i16_of(&mut it)? == 1 {
                    Some(DoorDef {
                        id,
                        wall: u32::try_from(i32_of(&mut it)?).ok()?,
                        seg: u32::try_from(i32_of(&mut it)?).ok()?,
                        pos_m: real_of(&mut it)?,
                        code: str_of(&mut it)?,
                        open: i16_of(&mut it)? == 1,
                        angle_deg: real_of(&mut it)?,
                        width_m: real_of(&mut it)?,
                        height_m: real_of(&mut it)?,
                        hinge_start: i16_of(&mut it)? == 1,
                        swing_left: i16_of(&mut it)? == 1,
                    })
                } else {
                    None
                };
                Some(DTag::Door { id, def })
            }
            1 => {
                let code = str_of(&mut it)?;
                let origin = if i16_of(&mut it)? == 1 {
                    match it.next()? {
                        XVal::Pt(p) => Some(*p),
                        _ => return None,
                    }
                } else {
                    None
                };
                Some(DTag::Sheet { code, origin })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sipil_arch::door::catalog;

    #[test]
    fn tags_round_trip() {
        let mut d = DoorDef::of_type(catalog::find("P2").unwrap());
        d.id = 7;
        d.wall = 3;
        d.seg = 2;
        d.pos_m = 1.75;
        d.open = false;
        d.hinge_start = false;
        for tag in [
            DTag::Door { id: 7, def: Some(d.clone()) },
            DTag::Door { id: 7, def: None },
            DTag::Sheet { code: "P2".into(), origin: Some([1.5, -2.5]) },
            DTag::Sheet { code: "P2".into(), origin: None },
        ] {
            assert_eq!(DTag::decode(&tag.encode()), Some(tag));
        }
        // Foreign, wrong-version or truncated data is not a tag.
        assert_eq!(DTag::decode(&[]), None);
        assert_eq!(DTag::decode(&[XVal::I16(9), XVal::I16(0)]), None);
        let mut cut = DTag::Door { id: 1, def: Some(d) }.encode();
        cut.truncate(cut.len() - 1);
        assert_eq!(DTag::decode(&cut), None);
    }

    #[test]
    fn sanitizing_clamps() {
        let mut d = DoorDef::of_type(catalog::fallback());
        d.width_m = f64::NAN;
        d.height_m = 99.0;
        d.angle_deg = -5.0;
        let s = d.sanitized();
        assert_eq!((s.width_m, s.height_m, s.angle_deg), (0.4, 4.0, 0.0));
    }

    #[test]
    fn every_role_has_a_layer() {
        let roles = [DRole::Frame, DRole::Leaf, DRole::Swing, DRole::Detail, DRole::DetailHatch, DRole::Text, DRole::Dim];
        for (i, r) in roles.iter().enumerate() {
            assert_eq!(r.code() as usize, i);
            assert!(r.layer().starts_with("DOOR-"));
        }
    }
}
