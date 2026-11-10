//! The beam popup's form: the typed fields and how they become a [`BeamDef`].
//! Pure (no iced), checked in the scratch crate.

use super::model::{BeamDef, Style};
use crate::sipil_arch::steel::pick::Pick;
use crate::sipil_wall::engine::{format_number, parse_number};

#[derive(Clone, Debug, PartialEq)]
pub struct Fields {
    pub pick: Pick,
    pub style: Style,
    pub mark: String,
    pub label: bool,
    pub text_mm: String,
}

impl Fields {
    pub fn from_def(d: &BeamDef) -> Fields {
        Fields {
            pick: Pick::from_key(&d.profile),
            style: d.style,
            mark: d.mark.clone(),
            label: d.label,
            text_mm: format_number(d.text_mm),
        }
    }

    /// Parse and range-check the fields on top of `base` (which keeps the
    /// beam's number, ends and unit).
    pub fn to_def(&self, base: &BeamDef) -> Result<BeamDef, String> {
        let text = parse_number(&self.text_mm).ok_or("Text height: enter a number")?;
        if !(20.0..=2000.0).contains(&text) {
            return Err("Text height: must be between 20 and 2000 mm".into());
        }
        if self.mark.chars().count() > 40 {
            return Err("Mark: at most 40 characters".into());
        }
        Ok(BeamDef {
            profile: self.pick.key.clone(),
            style: self.style,
            mark: self.mark.trim().to_string(),
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
    fn fields_round_trip_and_take_the_picked_profile() {
        let mut d = BeamDef::new(catalog::default_profile(), 1000.0);
        d.id = 2;
        d.b = [3000.0, 0.0];
        d.mark = "B1".into();
        let mut f = Fields::from_def(&d);
        assert_eq!(f.to_def(&d).unwrap(), d);
        f.pick.apply(&PickMsg::Kind(catalog::Kind::Channel));
        f.style = Style::Centerline;
        let out = f.to_def(&d).unwrap();
        assert!(out.profile.contains("UNP"), "{}", out.profile);
        assert_eq!((out.style, out.id, out.b), (Style::Centerline, 2, [3000.0, 0.0]));
    }

    #[test]
    fn bad_input_is_reported() {
        let d = BeamDef::new(catalog::default_profile(), 1000.0);
        let mut f = Fields::from_def(&d);
        f.text_mm = "abc".into();
        assert!(f.to_def(&d).unwrap_err().contains("Text height"));
        f.text_mm = "5000".into();
        assert!(f.to_def(&d).is_err());
        f.text_mm = "150".into();
        f.mark = "x".repeat(41);
        assert!(f.to_def(&d).unwrap_err().contains("Mark"));
    }
}
