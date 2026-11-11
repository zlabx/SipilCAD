//! The fields of the GRID popup and how they become a definition. Pure.

use super::model::{GridDef, DEFAULT_EXTEND_MM};
use super::spec::Bubble;
use super::templates;

/// Text of every field of the popup, as typed.
#[derive(Clone, Debug, PartialEq)]
pub struct Fields {
    pub x_coords: String,
    pub x_labels: String,
    pub x_bubble: Bubble,
    pub y_coords: String,
    pub y_labels: String,
    pub y_bubble: Bubble,
    pub z_coords: String,
    pub z_labels: String,
    pub extend: String,
}

impl Fields {
    pub fn from_def(d: &GridDef) -> Fields {
        Fields {
            x_coords: d.x_coords.clone(),
            x_labels: d.x_labels.clone(),
            x_bubble: d.x_bubble,
            y_coords: d.y_coords.clone(),
            y_labels: d.y_labels.clone(),
            y_bubble: d.y_bubble,
            z_coords: d.z_coords.clone(),
            z_labels: d.z_labels.clone(),
            extend: format!("{}", d.extend_mm),
        }
    }

    /// The definition the fields say, or what is wrong with them.
    pub fn to_def(&self) -> Result<GridDef, String> {
        let extend_mm = match self.extend.trim() {
            "" => DEFAULT_EXTEND_MM,
            t => t
                .replace(',', ".")
                .parse::<f64>()
                .map_err(|_| "The extension must be a number (mm).".to_string())?,
        };
        let def = GridDef {
            x_coords: self.x_coords.trim().to_string(),
            x_labels: self.x_labels.trim().to_string(),
            x_bubble: self.x_bubble,
            y_coords: self.y_coords.trim().to_string(),
            y_labels: self.y_labels.trim().to_string(),
            y_bubble: self.y_bubble,
            z_coords: self.z_coords.trim().to_string(),
            z_labels: self.z_labels.trim().to_string(),
            extend_mm,
        };
        def.resolve()?;
        Ok(def)
    }

    /// Replace the texts with a template's (the bubble places and the extension stay).
    pub fn apply_template(&mut self, key: &str) {
        if let Some(t) = templates::by_key(key) {
            self.x_coords = t.x.0.into();
            self.x_labels = t.x.1.into();
            self.y_coords = t.y.0.into();
            self.y_labels = t.y.1.into();
            self.z_coords = t.z.0.into();
            self.z_labels = t.z.1.into();
        }
    }
}
