//! Wall tool state: settings, the pallet's quick fields and the parameter popup.
//!
//! No iced and no host types here (only `codec::Handle`), so the form logic is
//! unit-tested in the kit's scratch crate. Thickness is typed in centimetres
//! (how a site drawing reads) and stored in metres.

use super::engine::{format_number, parse_number, Pattern, Spacing, WallDef, WallParams};

/// The popup's and the pallet's text fields. Kept as text so a half-typed
/// number does not snap back while it is being typed.
#[derive(Clone, Debug, PartialEq)]
pub struct Fields {
    pub thickness_cm: String,
    pub plaster_cm: String,
    pub col_len_cm: String,
    pub interval_m: String,
    pub spacing: Spacing,
    pub pattern: Pattern,
    pub hatch_scale: String,
    pub col_hatch: bool,
    pub col_scale: String,
}

impl Fields {
    pub fn from_params(p: &WallParams) -> Fields {
        Fields {
            thickness_cm: format_number(p.thickness_m * 100.0),
            plaster_cm: format_number(p.plaster_m * 100.0),
            col_len_cm: format_number(p.col_len_m * 100.0),
            interval_m: format_number(p.interval_m),
            spacing: p.spacing,
            pattern: p.pattern,
            hatch_scale: format_number(p.hatch_scale),
            col_hatch: p.col_hatch,
            col_scale: format_number(p.col_scale),
        }
    }

    /// Parse and range-check the fields. An empty optional field counts as 0.
    pub fn to_params(&self) -> Result<WallParams, String> {
        fn num(label: &str, text: &str, min: f64, max: f64, optional: bool) -> Result<f64, String> {
            if text.trim().is_empty() && optional {
                return Ok(0.0);
            }
            let v = parse_number(text).ok_or_else(|| format!("{label}: isi dengan angka"))?;
            if v < min || v > max {
                return Err(format!(
                    "{label}: harus antara {} dan {}",
                    format_number(min),
                    format_number(max)
                ));
            }
            Ok(v)
        }
        let thickness = num("Tebal bata (cm)", &self.thickness_cm, 1.0, 200.0, false)?;
        let plaster = num("Plester per sisi (cm)", &self.plaster_cm, 0.0, 20.0, true)?;
        let col_len = num("Panjang kolom (cm)", &self.col_len_cm, 0.0, 200.0, true)?;
        let interval = num("Kolom tiap (m)", &self.interval_m, 0.0, 100.0, true)?;
        let hatch_scale = num("Skala arsir bata", &self.hatch_scale, 0.0, 10_000.0, true)?;
        let col_scale = num("Skala arsir kolom", &self.col_scale, 0.0, 10_000.0, true)?;
        Ok(WallParams {
            thickness_m: thickness / 100.0,
            plaster_m: plaster / 100.0,
            col_len_m: col_len / 100.0,
            interval_m: interval,
            spacing: self.spacing,
            pattern: self.pattern,
            hatch_scale,
            col_hatch: self.col_hatch,
            col_scale,
        })
    }
}

/// What the popup edits.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    /// The settings new walls are drawn with (BWSET).
    Defaults,
    /// A wall already in the drawing, by id.
    Wall(u32),
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub target: Target,
    pub fields: Fields,
    /// The wall being edited (axis points, unit); `None` for the defaults.
    pub def: Option<WallDef>,
    pub error: Option<String>,
}

pub struct State {
    /// The building-components pallet is open.
    pub show: bool,
    /// Settings new walls are drawn with.
    pub settings: WallParams,
    /// The pallet's quick fields.
    pub panel: Fields,
    pub popup: Option<Popup>,
    /// Open the popup as soon as a wall is clicked (otherwise only on double-click).
    pub popup_on_click: bool,
    /// The wall entity the popup was last opened for, so the same selection
    /// does not reopen it after it is closed.
    pub opened_for: Option<codec::Handle>,
}

impl Default for State {
    fn default() -> Self {
        let settings = WallParams::default();
        State {
            show: false,
            panel: Fields::from_params(&settings),
            settings,
            popup: None,
            popup_on_click: true,
            opened_for: None,
        }
    }
}

impl State {
    /// Make the pallet fields and the settings agree on `p`.
    pub fn set_settings(&mut self, p: WallParams) {
        self.panel = Fields::from_params(&p);
        self.settings = p;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fields_round_trip_through_centimetres() {
        let p = WallParams {
            thickness_m: 0.2,
            plaster_m: 0.015,
            interval_m: 3.0,
            ..WallParams::default()
        };
        let f = Fields::from_params(&p);
        assert_eq!(
            (
                f.thickness_cm.as_str(),
                f.plaster_cm.as_str(),
                f.interval_m.as_str()
            ),
            ("20", "1.5", "3")
        );
        let back = f.to_params().unwrap();
        assert!((back.thickness_m - 0.2).abs() < 1e-12 && (back.plaster_m - 0.015).abs() < 1e-12);
        assert_eq!(back.interval_m, 3.0);
    }

    #[test]
    fn bad_input_is_reported_in_the_users_words() {
        let mut f = Fields::from_params(&WallParams::default());
        f.thickness_cm = "abc".into();
        assert!(f.to_params().unwrap_err().contains("Tebal bata"));
        f.thickness_cm = "0".into();
        assert!(f.to_params().unwrap_err().contains("antara"));
        f.thickness_cm = "10".into();
        f.plaster_cm = "-1".into();
        assert!(f.to_params().unwrap_err().contains("Plester"));
        f.plaster_cm = "".into(); // optional: empty is zero
        assert_eq!(f.to_params().unwrap().plaster_m, 0.0);
        f.interval_m = "3,5".into(); // decimal comma
        assert_eq!(f.to_params().unwrap().interval_m, 3.5);
        f.thickness_cm = "".into(); // required
        assert!(f.to_params().is_err());
    }

    #[test]
    fn set_settings_updates_both() {
        let mut s = State::default();
        let p = WallParams {
            thickness_m: 0.2,
            ..WallParams::default()
        };
        s.set_settings(p.clone());
        assert_eq!(s.settings, p);
        assert_eq!(s.panel.thickness_cm, "20");
        assert!(s.popup_on_click && !s.show);
    }
}
