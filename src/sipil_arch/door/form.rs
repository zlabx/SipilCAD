//! The door popup's form: the typed fields and how they become a [`DoorDef`].
//!
//! Pure (no iced), so the checks run in the scratch crate. Sizes are typed in
//! centimetres, stored in metres.

use super::catalog;
use super::model::DoorDef;
use crate::sipil_wall::engine::{format_number, parse_number};

#[derive(Clone, Debug, PartialEq)]
pub struct Fields {
    pub code: String,
    pub open: bool,
    pub angle: String,
    pub width_cm: String,
    pub height_cm: String,
    pub hinge_start: bool,
    pub swing_left: bool,
}

impl Fields {
    pub fn from_def(d: &DoorDef) -> Fields {
        Fields {
            code: d.code.clone(),
            open: d.open,
            angle: format_number(d.angle_deg),
            width_cm: format_number(d.width_m * 100.0),
            height_cm: format_number(d.height_m * 100.0),
            hinge_start: d.hinge_start,
            swing_left: d.swing_left,
        }
    }

    /// Choose a type: the size fields go back to that type's default size.
    pub fn set_type(&mut self, code: &str) {
        if let Some(t) = catalog::find(code) {
            self.code = t.code.to_string();
            self.width_cm = format_number(t.width_m * 100.0);
            self.height_cm = format_number(t.height_m * 100.0);
        }
    }

    /// Parse and range-check the fields on top of `base` (which keeps the
    /// door's number, wall and position).
    pub fn to_def(&self, base: &DoorDef) -> Result<DoorDef, String> {
        fn num(label: &str, text: &str, min: f64, max: f64) -> Result<f64, String> {
            let v = parse_number(text).ok_or_else(|| format!("{label}: enter a number"))?;
            if v < min || v > max {
                return Err(format!("{label}: must be between {} and {}", format_number(min), format_number(max)));
            }
            Ok(v)
        }
        let ty = catalog::find(&self.code).ok_or("Choose a door type")?;
        let width = num("Clear width (cm)", &self.width_cm, 40.0, 500.0)?;
        let height = num("Clear height (cm)", &self.height_cm, 150.0, 400.0)?;
        let angle = if self.open { num("Opening angle (degrees)", &self.angle, 1.0, 180.0)? } else { parse_number(&self.angle).unwrap_or(90.0).clamp(1.0, 180.0) };
        Ok(DoorDef {
            code: ty.code.to_string(),
            open: self.open,
            angle_deg: angle,
            width_m: width / 100.0,
            height_m: height / 100.0,
            hinge_start: self.hinge_start,
            swing_left: self.swing_left,
            ..base.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip_in_centimetres() {
        let d = DoorDef::of_type(catalog::find("P2").unwrap());
        let f = Fields::from_def(&d);
        assert_eq!((f.width_cm.as_str(), f.height_cm.as_str(), f.angle.as_str()), ("80", "210", "90"));
        assert_eq!(f.to_def(&d).unwrap(), d);
    }

    #[test]
    fn a_new_door_starts_open() {
        assert!(DoorDef::of_type(catalog::fallback()).open);
    }

    #[test]
    fn choosing_a_type_resets_the_size_only() {
        let mut f = Fields::from_def(&DoorDef::of_type(catalog::find("P2").unwrap()));
        f.open = false;
        f.hinge_start = false;
        f.width_cm = "95".into();
        f.set_type("P3");
        assert_eq!((f.code.as_str(), f.width_cm.as_str(), f.height_cm.as_str()), ("P3", "160", "210"));
        assert!(!f.open && !f.hinge_start, "state and hinge are kept");
        f.set_type("nope");
        assert_eq!(f.code, "P3");
    }

    #[test]
    fn bad_input_is_reported() {
        let d = DoorDef::of_type(catalog::fallback());
        let mut f = Fields::from_def(&d);
        f.width_cm = "abc".into();
        assert!(f.to_def(&d).unwrap_err().contains("Clear width"));
        f.width_cm = "20".into();
        assert!(f.to_def(&d).unwrap_err().contains("between"));
        f.width_cm = "85,5".into(); // decimal comma
        assert!((f.to_def(&d).unwrap().width_m - 0.855).abs() < 1e-12);
        f.height_cm = "500".into();
        assert!(f.to_def(&d).unwrap_err().contains("Clear height"));
        f.height_cm = "210".into();
        f.angle = "0".into();
        assert!(f.to_def(&d).unwrap_err().contains("angle"));
        f.open = false; // the angle does not matter for a closed door
        assert!(f.to_def(&d).is_ok());
        f.code = "P9".into();
        assert!(f.to_def(&d).is_err());
    }

    #[test]
    fn the_door_keeps_its_place_in_the_wall() {
        let mut base = DoorDef::of_type(catalog::fallback());
        base.id = 4;
        base.wall = 2;
        base.seg = 1;
        base.pos_m = 3.25;
        let mut f = Fields::from_def(&base);
        f.set_type("P2");
        let d = f.to_def(&base).unwrap();
        assert_eq!((d.id, d.wall, d.seg, d.pos_m, d.code.as_str()), (4, 2, 1, 3.25, "P2"));
    }
}
