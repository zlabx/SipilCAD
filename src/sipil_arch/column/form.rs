//! The column popup's form: the typed fields and how they become a [`ColumnDef`].
//! Pure (no iced), checked in the scratch crate.

use super::model::{ColumnDef, Fill};
use crate::sipil_arch::steel::pick::Pick;
use crate::sipil_wall::engine::{format_number, parse_number};

#[derive(Clone, Debug, PartialEq)]
pub struct Fields {
    pub pick: Pick,
    pub rot: String,
    pub mark: String,
    pub fill: Fill,
    pub label: bool,
    pub text_mm: String,
}

impl Fields {
    pub fn from_def(d: &ColumnDef) -> Fields {
        Fields {
            pick: Pick::from_key(&d.profile),
            rot: format_number(d.rot_deg),
            mark: d.mark.clone(),
            fill: d.fill,
            label: d.label,
            text_mm: format_number(d.text_mm),
        }
    }

    /// Parse and range-check the fields on top of `base` (which keeps the
    /// column's number, place and unit).
    pub fn to_def(&self, base: &ColumnDef) -> Result<ColumnDef, String> {
        let rot = parse_number(&self.rot).ok_or("Rotation: enter a number")?;
        let text = parse_number(&self.text_mm).ok_or("Text height: enter a number")?;
        if !(20.0..=2000.0).contains(&text) {
            return Err("Text height: must be between 20 and 2000 mm".into());
        }
        if self.mark.chars().count() > 40 {
            return Err("Mark: at most 40 characters".into());
        }
        Ok(ColumnDef {
            profile: self.pick.key.clone(),
            rot_deg: rot,
            mark: self.mark.trim().to_string(),
            fill: self.fill,
            label: self.label,
            text_mm: text,
            ..base.clone()
        }
        .sanitized())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sipil_arch::steel::{catalog, pick::PickMsg};

    #[test]
    fn fields_round_trip() {
        let mut d = ColumnDef::new(catalog::default_profile(), 1000.0);
        d.id = 3;
        d.at = [10.0, 20.0];
        d.mark = "K1".into();
        let f = Fields::from_def(&d);
        assert_eq!((f.rot.as_str(), f.text_mm.as_str()), ("0", "150"));
        assert_eq!(f.to_def(&d).unwrap(), d);
    }

    #[test]
    fn the_picked_profile_and_other_settings_land_in_the_definition() {
        let d = ColumnDef::new(catalog::default_profile(), 1000.0);
        let mut f = Fields::from_def(&d);
        f.pick.apply(&PickMsg::Kind(catalog::Kind::HBeam));
        f.rot = "-90".into();
        f.fill = Fill::None;
        f.label = false;
        let out = f.to_def(&d).unwrap();
        assert!(out.profile.contains("/H"), "{}", out.profile);
        assert_eq!((out.rot_deg, out.fill, out.label), (270.0, Fill::None, false));
    }

    #[test]
    fn bad_input_is_reported() {
        let d = ColumnDef::new(catalog::default_profile(), 1000.0);
        let mut f = Fields::from_def(&d);
        f.rot = "x".into();
        assert!(f.to_def(&d).unwrap_err().contains("Rotation"));
        f.rot = "45,5".into(); // decimal comma
        assert!((f.to_def(&d).unwrap().rot_deg - 45.5).abs() < 1e-9);
        f.text_mm = "5".into();
        assert!(f.to_def(&d).unwrap_err().contains("Text height"));
        f.text_mm = "150".into();
        f.mark = "x".repeat(41);
        assert!(f.to_def(&d).unwrap_err().contains("Mark"));
    }
}
