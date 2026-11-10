//! `BEAM`: click the start and end of a beam; the next beam starts where the
//! last one ended (type `B` to break the chain). Esc or Enter finishes.

use codec::EntityType;
use glam::DVec3;

use super::planner::Model;
use super::{adapter, model::BeamDef};
use crate::command::{CadCommand, CmdOption, CmdResult};
use crate::sipil_arch::steel::catalog;
use crate::sipil_wall::command::host_pattern;
use crate::sipil_wall::engine::P;

pub struct BeamCommand {
    model: Model,
    beam: BeamDef,
    start: Option<P>,
    /// Why the last click did not place a beam, shown in the prompt.
    error: Option<String>,
}

impl BeamCommand {
    pub fn new(model: Model, beam: BeamDef) -> Self {
        Self { model, beam, start: None, error: None }
    }
}

impl CadCommand for BeamCommand {
    fn name(&self) -> &'static str {
        "BEAM"
    }

    fn prompt(&self) -> String {
        let name = catalog::find_key(&self.beam.profile).map_or("?", |p| p.name);
        match (&self.start, &self.error) {
            (_, Some(e)) => format!("BEAM  {e}. Specify the point:"),
            (None, None) => format!("BEAM  Specify start point of {name}:"),
            (Some(_), None) => format!("BEAM  Specify end point of {name} [Break] <done>:"),
        }
    }

    fn options(&self) -> Vec<CmdOption> {
        if self.start.is_some() {
            vec![CmdOption::new("Break", "B"), CmdOption::new("Done", "")]
        } else {
            vec![CmdOption::new("Done", "")]
        }
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        self.error = None;
        let p = [pt.x, pt.y];
        let Some(a) = self.start else {
            self.start = Some(p);
            return CmdResult::NeedPoint;
        };
        match self.model.place(&self.beam, a, p) {
            Ok((edit, def)) => {
                let entities = adapter::edit_entities(&edit, &host_pattern);
                self.model.remember(def);
                self.start = Some(p); // the next beam starts where this one ended
                CmdResult::CommitEntities(entities)
            }
            Err(e) => {
                self.error = Some(e);
                CmdResult::NeedPoint
            }
        }
    }

    fn on_enter(&mut self) -> CmdResult {
        CmdResult::Cancel
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        match text.trim().to_ascii_uppercase().as_str() {
            "B" | "BREAK" | "U" | "UNDO" => {
                self.start = None;
                Some(CmdResult::NeedPoint)
            }
            "D" | "DONE" => Some(CmdResult::Cancel),
            _ => None,
        }
    }

    fn preview_entities(&mut self, _document: &codec::CadDocument, pt: DVec3) -> Vec<EntityType> {
        match self.start {
            Some(a) => adapter::preview_entities(&self.model.preview(&self.beam, a, [pt.x, pt.y]), &host_pattern),
            None => Vec::new(),
        }
    }

    // Beams carry their own layers and tags; the host must not restyle them.
    fn preserve_commit_layer(&self) -> bool {
        true
    }

    fn preserve_commit_style(&self) -> bool {
        true
    }
}
