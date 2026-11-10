//! `COLUMN`: click to put a column of the chosen profile, again and again
//! until Esc or Enter. Type `R` to turn it by 90 degrees, or a number for any angle.

use codec::EntityType;
use glam::DVec3;

use super::planner::Model;
use super::{adapter, model::ColumnDef};
use crate::command::{CadCommand, CmdOption, CmdResult};
use crate::sipil_arch::steel::catalog;
use crate::sipil_wall::command::host_pattern;
use crate::sipil_wall::engine::parse_number;

pub struct ColumnCommand {
    model: Model,
    column: ColumnDef,
}

impl ColumnCommand {
    pub fn new(model: Model, column: ColumnDef) -> Self {
        Self { model, column }
    }
}

impl CadCommand for ColumnCommand {
    fn name(&self) -> &'static str {
        "COLUMN"
    }

    fn prompt(&self) -> String {
        let name = catalog::find_key(&self.column.profile).map_or("?", |p| p.name);
        format!("COLUMN  Click to place {name} at {:.0}° [Rotate 90/angle] <done>:", self.column.rot_deg)
    }

    fn options(&self) -> Vec<CmdOption> {
        vec![CmdOption::new("Rotate", "R"), CmdOption::new("Done", "")]
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        let (edit, def) = self.model.place(&self.column, [pt.x, pt.y]);
        let entities = adapter::edit_entities(&edit, &host_pattern);
        self.model.remember(def);
        CmdResult::CommitEntities(entities)
    }

    fn on_enter(&mut self) -> CmdResult {
        CmdResult::Cancel
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        let t = text.trim();
        match t.to_ascii_uppercase().as_str() {
            "" => None,
            "R" | "ROTATE" => {
                self.column.rot_deg = (self.column.rot_deg + 90.0).rem_euclid(360.0);
                Some(CmdResult::NeedPoint)
            }
            "D" | "DONE" => Some(CmdResult::Cancel),
            _ => {
                let angle = parse_number(t)?;
                self.column.rot_deg = angle.rem_euclid(360.0);
                Some(CmdResult::NeedPoint)
            }
        }
    }

    fn preview_entities(&mut self, _document: &codec::CadDocument, pt: DVec3) -> Vec<EntityType> {
        adapter::preview_entities(&self.model.preview(&self.column, [pt.x, pt.y]), &host_pattern)
    }

    // Columns carry their own layers and tags; the host must not restyle them.
    fn preserve_commit_layer(&self) -> bool {
        true
    }

    fn preserve_commit_style(&self) -> bool {
        true
    }
}
