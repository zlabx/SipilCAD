//! The steel profile catalog, from SipilGen (Gunung Garuda and Perwaja Steel).
//!
//! `data.rs` is generated from SipilGen's data (`scripts/gen_steel_profiles.py`);
//! everything here is hand-written lookup on top of it.

use super::data::{MANUFACTURERS, PROFILES};

/// The kinds of section SipilGen lists.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    WideFlange,
    HBeam,
    IBeam,
    King,
    Queen,
    EqualAngle,
    UnequalAngle,
    Channel,
    TBeam,
    Honeycomb,
    CellForm,
}

impl Kind {
    /// SipilGen's key for the kind (`wide-flange`).
    pub fn key(self) -> &'static str {
        match self {
            Kind::WideFlange => "wide-flange",
            Kind::HBeam => "h-beam",
            Kind::IBeam => "i-beam",
            Kind::King => "king",
            Kind::Queen => "queen",
            Kind::EqualAngle => "equal-angle",
            Kind::UnequalAngle => "unequal-angle",
            Kind::Channel => "channel",
            Kind::TBeam => "t-beam",
            Kind::Honeycomb => "honeycomb",
            Kind::CellForm => "cell-form",
        }
    }

    /// The small picture of the kind shown in the popup.
    pub fn icon(self) -> &'static [u8] {
        match self {
            Kind::WideFlange => include_bytes!("../../../assets/icons/sipil/steel/wide-flange.svg"),
            Kind::HBeam => include_bytes!("../../../assets/icons/sipil/steel/h-beam.svg"),
            Kind::IBeam => include_bytes!("../../../assets/icons/sipil/steel/i-beam.svg"),
            Kind::King => include_bytes!("../../../assets/icons/sipil/steel/king.svg"),
            Kind::Queen => include_bytes!("../../../assets/icons/sipil/steel/queen.svg"),
            Kind::EqualAngle => include_bytes!("../../../assets/icons/sipil/steel/equal-angle.svg"),
            Kind::UnequalAngle => include_bytes!("../../../assets/icons/sipil/steel/unequal-angle.svg"),
            Kind::Channel => include_bytes!("../../../assets/icons/sipil/steel/channel.svg"),
            Kind::TBeam => include_bytes!("../../../assets/icons/sipil/steel/t-beam.svg"),
            Kind::Honeycomb => include_bytes!("../../../assets/icons/sipil/steel/honeycomb.svg"),
            Kind::CellForm => include_bytes!("../../../assets/icons/sipil/steel/cell-form.svg"),
        }
    }
}

/// One steel section. Sizes in millimetres (0 where a kind has none).
#[derive(Clone, Copy, Debug)]
pub struct Profile {
    /// Manufacturer id (`gunung-garuda`).
    pub mfr: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    /// Depth and width of the section.
    pub h: f64,
    pub b: f64,
    /// Web and flange thickness.
    pub tw: f64,
    pub tf: f64,
    /// Leg thickness of an angle.
    pub t: f64,
    /// Root fillet radius and toe radius.
    pub r1: f64,
    pub r2: f64,
    /// Centre of gravity in the section's own coordinates, where SipilGen gives one.
    pub centroid: Option<[f64; 2]>,
    pub area_cm2: f64,
    pub kg_m: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Manufacturer {
    pub id: &'static str,
    pub name: &'static str,
    pub country: &'static str,
    /// The kinds it makes, with the label its catalogue uses.
    pub filters: &'static [(Kind, &'static str)],
}

pub fn manufacturers() -> &'static [Manufacturer] {
    MANUFACTURERS
}

pub fn manufacturer(id: &str) -> Option<&'static Manufacturer> {
    MANUFACTURERS.iter().find(|m| m.id == id)
}

/// The profiles of one manufacturer and kind, in catalogue order.
pub fn profiles_of(mfr: &str, kind: Kind) -> Vec<&'static Profile> {
    PROFILES.iter().filter(|p| p.mfr == mfr && p.kind == kind).collect()
}

/// The key a drawing stores for a profile: `gunung-garuda/WF-200X100X5.5X8`.
pub fn key(p: &Profile) -> String {
    format!("{}/{}", p.mfr, p.name)
}

pub fn find_key(key: &str) -> Option<&'static Profile> {
    let (mfr, name) = key.split_once('/')?;
    PROFILES.iter().find(|p| p.mfr == mfr && p.name == name)
}

/// What a new column or beam starts with.
pub fn default_profile() -> &'static Profile {
    find_key("gunung-garuda/WF-200X100X5.5X8").unwrap_or(&PROFILES[0])
}

pub fn all() -> &'static [Profile] {
    PROFILES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_catalog_matches_sipilgen() {
        assert_eq!(all().len(), 272);
        assert_eq!(manufacturers().len(), 2);
        assert_eq!(manufacturer("gunung-garuda").unwrap().filters.len(), 10);
        assert_eq!(profiles_of("gunung-garuda", Kind::WideFlange).len(), 15);
        assert_eq!(profiles_of("perwaja-steel", Kind::HBeam).len(), 44);
        assert!(profiles_of("perwaja-steel", Kind::King).is_empty());
    }

    #[test]
    fn keys_are_unique_and_found_again() {
        let mut seen = std::collections::HashSet::new();
        for p in all() {
            assert!(seen.insert(key(p)), "{} repeats", key(p));
            assert_eq!(find_key(&key(p)).unwrap().name, p.name);
            assert!(p.h > 0.0 && p.b > 0.0 && p.area_cm2 > 0.0 && p.kg_m > 0.0, "{}", p.name);
        }
        assert!(find_key("nobody/WF-1").is_none() && find_key("no-slash").is_none());
        assert_eq!(default_profile().name, "WF-200X100X5.5X8");
    }

    #[test]
    fn every_kind_a_maker_lists_has_profiles_and_a_picture() {
        for m in manufacturers() {
            for (kind, label) in m.filters {
                assert!(!profiles_of(m.id, *kind).is_empty(), "{} {}", m.id, label);
                assert!(kind.icon().starts_with(b"<svg"), "{}", kind.key());
            }
        }
    }
}
