//! The profile picker's state: maker, kind and profile, with the rules that
//! keep the three consistent. Pure (no iced); the widget is in `ui.rs`.

use super::catalog::{self, Kind, Profile};

/// What the person has chosen so far.
#[derive(Clone, Debug, PartialEq)]
pub struct Pick {
    pub mfr: &'static str,
    pub kind: Kind,
    /// Profile key (`gunung-garuda/WF-200X100X5.5X8`).
    pub key: String,
}

#[derive(Clone, Debug)]
pub enum PickMsg {
    Mfr(&'static str),
    Kind(Kind),
    /// By profile name, within the chosen maker.
    Profile(&'static str),
}

impl Pick {
    /// The picker positioned on a profile; an unknown key gives the default one.
    pub fn from_key(key: &str) -> Pick {
        let p = catalog::find_key(key).unwrap_or_else(catalog::default_profile);
        Pick { mfr: p.mfr, kind: p.kind, key: catalog::key(p) }
    }

    pub fn profile(&self) -> &'static Profile {
        catalog::find_key(&self.key).unwrap_or_else(catalog::default_profile)
    }

    /// The profiles of the chosen maker and kind.
    pub fn profiles(&self) -> Vec<&'static Profile> {
        catalog::profiles_of(self.mfr, self.kind)
    }

    /// The kinds the chosen maker makes.
    pub fn kinds(&self) -> Vec<(Kind, &'static str)> {
        catalog::manufacturer(self.mfr).map(|m| m.filters.to_vec()).unwrap_or_default()
    }

    fn select_first(&mut self) {
        if let Some(p) = self.profiles().first() {
            self.key = catalog::key(p);
        }
    }

    pub fn apply(&mut self, msg: &PickMsg) {
        match msg {
            PickMsg::Mfr(id) => {
                let Some(m) = catalog::manufacturer(id) else { return };
                self.mfr = m.id;
                // Keep the kind when the new maker has it, else its first.
                if !m.filters.iter().any(|(k, _)| *k == self.kind) {
                    if let Some((k, _)) = m.filters.first() {
                        self.kind = *k;
                    }
                }
                self.select_first();
            }
            PickMsg::Kind(k) => {
                if self.kinds().iter().any(|(kk, _)| kk == k) {
                    self.kind = *k;
                    self.select_first();
                }
            }
            PickMsg::Profile(name) => {
                if let Some(p) = self.profiles().into_iter().find(|p| p.name == *name) {
                    self.key = catalog::key(p);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_picker_starts_on_a_known_profile() {
        let p = Pick::from_key("perwaja-steel/I-200X100X26");
        assert_eq!((p.mfr, p.kind), ("perwaja-steel", Kind::IBeam));
        assert_eq!(p.profile().name, "I-200X100X26");
        assert!(p.profiles().len() == 15 && p.kinds().len() == 4);
        // A key that is gone (a catalogue change) falls back to the default.
        assert_eq!(Pick::from_key("gunung-garuda/WF-1X1X1X1").profile().name, catalog::default_profile().name);
    }

    #[test]
    fn changing_maker_or_kind_keeps_the_choice_consistent() {
        let mut p = Pick::from_key("gunung-garuda/WF-200X100X5.5X8");
        p.apply(&PickMsg::Kind(Kind::King));
        assert_eq!((p.kind, p.profile().kind), (Kind::King, Kind::King));
        // Perwaja has no King Cross: switching maker moves to its first kind.
        p.apply(&PickMsg::Mfr("perwaja-steel"));
        assert_eq!(p.mfr, "perwaja-steel");
        assert_eq!(p.kind, Kind::HBeam);
        assert_eq!(p.profile().mfr, "perwaja-steel");
        // Back to Gunung Garuda: H Beam exists there, so the kind stays.
        p.apply(&PickMsg::Mfr("gunung-garuda"));
        assert_eq!((p.kind, p.profile().kind), (Kind::HBeam, Kind::HBeam));
        // A kind the maker lacks is ignored; unknown names too.
        p.apply(&PickMsg::Mfr("perwaja-steel"));
        p.apply(&PickMsg::Kind(Kind::Queen));
        assert_eq!(p.kind, Kind::HBeam);
        let before = p.key.clone();
        p.apply(&PickMsg::Profile("nope"));
        p.apply(&PickMsg::Mfr("nobody"));
        assert_eq!(p.key, before);
    }

    #[test]
    fn choosing_a_profile_sets_the_key() {
        let mut p = Pick::from_key("gunung-garuda/WF-200X100X5.5X8");
        p.apply(&PickMsg::Profile("WF-300X150X6.5X9"));
        assert_eq!(p.key, "gunung-garuda/WF-300X150X6.5X9");
    }
}
