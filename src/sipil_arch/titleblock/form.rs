//! The fields of the TITLEBLOCK popup and how they become a definition. Pure.

use super::model::{Fields, Paper, Template, TitleBlockDef};

/// Everything of the popup as typed (numbers are text until they are checked).
#[derive(Clone, Debug, PartialEq)]
pub struct FormFields {
    pub paper: Paper,
    pub landscape: bool,
    pub template: Template,
    pub margin_left: String,
    pub margin_right: String,
    pub margin_top: String,
    pub margin_bottom: String,
    pub block_width: String,
    pub texts: Fields,
}

fn num(v: f64) -> String {
    format!("{v}")
}

fn parse(name: &str, text: &str) -> Result<f64, String> {
    text.trim()
        .replace(',', ".")
        .parse::<f64>()
        .map_err(|_| format!("{name} must be a number (mm)."))
}

impl FormFields {
    pub fn from_def(d: &TitleBlockDef) -> FormFields {
        FormFields {
            paper: d.paper,
            landscape: d.landscape,
            template: d.template,
            margin_left: num(d.margin_left),
            margin_right: num(d.margin_right),
            margin_top: num(d.margin_top),
            margin_bottom: num(d.margin_bottom),
            block_width: num(d.block_width),
            texts: d.fields.clone(),
        }
    }

    /// The definition the fields say, or what is wrong with them.
    pub fn to_def(&self) -> Result<TitleBlockDef, String> {
        let def = TitleBlockDef {
            paper: self.paper,
            landscape: self.landscape,
            template: self.template,
            margin_left: parse("Left margin", &self.margin_left)?,
            margin_right: parse("Right margin", &self.margin_right)?,
            margin_top: parse("Top margin", &self.margin_top)?,
            margin_bottom: parse("Bottom margin", &self.margin_bottom)?,
            block_width: parse("Block width", &self.block_width)?,
            fields: self.texts.clone(),
        };
        super::geometry::drawing_area(&def)?;
        Ok(def)
    }
}
