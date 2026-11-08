//! Wall tool state: settings and the parameter popup's form.
//!
//! No iced and no host types here (only `codec::Handle`), so the form logic is
//! unit-tested in the kit's scratch crate. Thickness is typed in centimetres
//! (how a site drawing reads) and stored in metres.

use super::engine::{
    default_layers, format_number, parse_number, LayerSpec, Pattern, Spacing, WallDef, WallParams,
};

/// What each row of the layer table is for, in `Role` order.
pub const LAYER_LABELS: [&str; 5] = ["Brick", "Plaster", "Brick hatch", "Column", "Column hatch"];

/// One row of the layer table as typed. The colour stays text so a
/// half-typed number does not snap back; the lineweight is picked from a list.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerFields {
    pub name: String,
    pub color: String,
    pub lineweight: i16,
}

impl LayerFields {
    fn from_spec(l: &LayerSpec) -> LayerFields {
        LayerFields {
            name: l.name.clone(),
            color: l.color.to_string(),
            lineweight: l.lineweight,
        }
    }
}

/// The popup's text fields. Kept as text so a half-typed number does not snap
/// back while it is being typed.
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
    pub layers: [LayerFields; 5],
}

/// Characters AutoCAD does not allow in a layer name.
const BAD_NAME_CHARS: &[char] = &['<', '>', '/', '\\', '"', ':', ';', '?', '*', '|', '=', ',', '`'];

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
            layers: [0, 1, 2, 3, 4].map(|i| LayerFields::from_spec(&p.layers[i])),
        }
    }

    /// Parse and range-check the fields. An empty optional field counts as 0.
    pub fn to_params(&self) -> Result<WallParams, String> {
        fn num(label: &str, text: &str, min: f64, max: f64, optional: bool) -> Result<f64, String> {
            if text.trim().is_empty() && optional {
                return Ok(0.0);
            }
            let v = parse_number(text).ok_or_else(|| format!("{label}: enter a number"))?;
            if v < min || v > max {
                return Err(format!(
                    "{label}: must be between {} and {}",
                    format_number(min),
                    format_number(max)
                ));
            }
            Ok(v)
        }
        let thickness = num("Brick thickness (cm)", &self.thickness_cm, 1.0, 200.0, false)?;
        let plaster = num("Plaster per face (cm)", &self.plaster_cm, 0.0, 20.0, true)?;
        let col_len = num("Column length (cm)", &self.col_len_cm, 0.0, 200.0, true)?;
        let interval = num("Column every (m)", &self.interval_m, 0.0, 100.0, true)?;
        let hatch_scale = num("Brick hatch scale", &self.hatch_scale, 0.0, 10_000.0, true)?;
        let col_scale = num("Column hatch scale", &self.col_scale, 0.0, 10_000.0, true)?;

        let mut layers = default_layers();
        for (i, (spec, f)) in layers.iter_mut().zip(&self.layers).enumerate() {
            let label = LAYER_LABELS[i];
            let name = f.name.trim();
            if name.is_empty() {
                return Err(format!("{label} layer: enter a layer name"));
            }
            if name.chars().count() > 255 || name.contains(BAD_NAME_CHARS) {
                return Err(format!(
                    "{label} layer: the name is too long or has a character a layer name cannot have"
                ));
            }
            let color = num(&format!("{label} layer colour"), &f.color, 1.0, 255.0, false)?;
            if color.fract() != 0.0 {
                return Err(format!("{label} layer colour: use a whole number (1-255)"));
            }
            *spec = LayerSpec {
                name: name.to_string(),
                color: color as i16,
                lineweight: f.lineweight,
            };
        }
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
            layers,
        })
    }
}

/// What the popup edits.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    /// The settings new walls are drawn with; OK starts drawing.
    New,
    /// A wall already in the drawing, by id.
    Wall(u32),
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub target: Target,
    pub fields: Fields,
    /// The wall being edited (axis points, unit); `None` for a new wall.
    pub def: Option<WallDef>,
    pub error: Option<String>,
}

pub struct State {
    /// Settings new walls are drawn with.
    pub settings: WallParams,
    pub popup: Option<Popup>,
    /// Open the popup as soon as a wall is clicked (otherwise only on double-click).
    pub popup_on_click: bool,
    /// The wall entity the popup was last opened for, so the same selection
    /// does not reopen it after it is closed.
    pub opened_for: Option<codec::Handle>,
}

impl Default for State {
    fn default() -> Self {
        State {
            settings: WallParams::default(),
            popup: None,
            popup_on_click: true,
            opened_for: None,
        }
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
        assert_eq!(back.layers, p.layers);
    }

    #[test]
    fn bad_input_is_reported_in_the_users_words() {
        let mut f = Fields::from_params(&WallParams::default());
        f.thickness_cm = "abc".into();
        assert!(f.to_params().unwrap_err().contains("Brick thickness"));
        f.thickness_cm = "0".into();
        assert!(f.to_params().unwrap_err().contains("must be between"));
        f.thickness_cm = "10".into();
        f.plaster_cm = "-1".into();
        assert!(f.to_params().unwrap_err().contains("Plaster"));
        f.plaster_cm = "".into(); // optional: empty is zero
        assert_eq!(f.to_params().unwrap().plaster_m, 0.0);
        f.interval_m = "3,5".into(); // decimal comma
        assert_eq!(f.to_params().unwrap().interval_m, 3.5);
        f.thickness_cm = "".into(); // required
        assert!(f.to_params().is_err());
    }

    #[test]
    fn layer_rows_are_checked() {
        let mut f = Fields::from_params(&WallParams::default());
        f.layers[2].name = "  ".into();
        assert!(f.to_params().unwrap_err().contains("Brick hatch layer"));
        f.layers[2].name = "A/B".into();
        assert!(f.to_params().unwrap_err().contains("cannot have"));
        f.layers[2].name = " A-HATCH ".into();
        f.layers[3].color = "300".into();
        assert!(f.to_params().unwrap_err().contains("Column layer colour"));
        f.layers[3].color = "2.5".into();
        assert!(f.to_params().unwrap_err().contains("whole number"));
        f.layers[3].color = "30".into();
        f.layers[3].lineweight = 35;
        let p = f.to_params().unwrap();
        assert_eq!(p.layers[2].name, "A-HATCH"); // trimmed
        assert_eq!((p.layers[3].color, p.layers[3].lineweight), (30, 35));
    }

    #[test]
    fn popup_opens_on_click_by_default() {
        let s = State::default();
        assert!(s.popup_on_click && s.popup.is_none());
    }
}
