//! The grid templates of the GRID popup: a starting grid and levels for a kind
//! of building. The same four as SipilBIM's Grid & Level window
//! (`gridtemplates.py`). The values are planning assumptions (except
//! Warehouse, which is the brief); the person edits them in the popup.
//!
//! | Template  | X (along the length) | Y (across)         | Levels                    |
//! |-----------|----------------------|--------------------|---------------------------|
//! | House     | A-C, 2 bays of 3 m   | 1-4, 3 bays of 3 m | 2: 0, 3.00 m              |
//! | Medium    | A-F, 5 bays of 6 m   | 1-4, 3 bays of 6 m | 8: 0, 4.00 m, then 3.50 m |
//! | High      | A-G, 6 bays of 8 m   | 1-5, 4 bays of 8 m | 20: 0, 4.50 m, then 3.80 m|
//! | Warehouse | A-F, 5 bays of 6 m   | 1-2, one 20 m span | 2: 0 and the eaves, 8.00 m|

use super::model::{GridDef, DEFAULT_EXTEND_MM};
use super::spec::Bubble;

#[derive(Debug)]
pub struct GridTemplate {
    pub key: &'static str,
    pub title: &'static str,
    /// `(coordinates, labels)` of X, Y and the levels; empty labels are made up.
    pub x: (&'static str, &'static str),
    pub y: (&'static str, &'static str),
    pub z: (&'static str, &'static str),
}

pub const TEMPLATES: [GridTemplate; 4] = [
    GridTemplate {
        key: "house",
        title: "Basic House",
        x: ("0.00 2*3000.00", "A B C"),
        y: ("0.00 3*3000.00", "1 2 3 4"),
        z: ("0.00 3000.00", "Level 1, Level 2"),
    },
    GridTemplate {
        key: "medium",
        title: "Medium Rise Building",
        x: ("0.00 5*6000.00", ""),
        y: ("0.00 3*6000.00", ""),
        z: ("0.00 4000.00 6*3500.00", ""),
    },
    GridTemplate {
        key: "high",
        title: "High Rise Building",
        x: ("0.00 6*8000.00", ""),
        y: ("0.00 4*8000.00", ""),
        z: ("0.00 4500.00 18*3800.00", ""),
    },
    GridTemplate {
        key: "warehouse",
        title: "Warehouse",
        x: ("0.00 5*6000.00", ""),
        y: ("0.00 20000.00", ""),
        z: ("0.00 8000.00", "Level 1, Eaves"),
    },
];

impl GridTemplate {
    pub fn def(&self) -> GridDef {
        GridDef {
            x_coords: self.x.0.into(),
            x_labels: self.x.1.into(),
            x_bubble: Bubble::Start,
            y_coords: self.y.0.into(),
            y_labels: self.y.1.into(),
            y_bubble: Bubble::Start,
            z_coords: self.z.0.into(),
            z_labels: self.z.1.into(),
            extend_mm: DEFAULT_EXTEND_MM,
        }
    }
}

pub fn by_key(key: &str) -> Option<&'static GridTemplate> {
    TEMPLATES.iter().find(|t| t.key == key)
}
