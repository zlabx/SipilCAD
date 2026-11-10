//! `DOOR`: click on a wall to put the door there, again and again until Esc.
//!
//! The command holds a [`Model`] of the drawing read when it started. A click
//! asks the planner for an edit (the wall rebuilt with an opening, the door,
//! the detail sheet of its type) and hands it to the host as one replace, which
//! is one undo step. The host tells the command the new handles, so the model
//! stays in step and the next click needs no second scan.

use codec::{EntityType, Handle};
use glam::DVec3;

use super::planner::{Edit, Model};
use super::{adapter, catalog, model::DoorDef};
use crate::command::{CadCommand, CmdOption, CmdResult};
use crate::sipil_wall::command::host_pattern;

pub struct DoorCommand {
    model: Model,
    door: DoorDef,
    /// Why the last click did not place a door, shown in the prompt.
    error: Option<String>,
    /// The edit handed to the host, waiting for the new handles.
    pending: Option<(Edit, Vec<usize>)>,
}

impl DoorCommand {
    pub fn new(model: Model, door: DoorDef) -> Self {
        Self { model, door, error: None, pending: None }
    }
}

impl CadCommand for DoorCommand {
    fn name(&self) -> &'static str {
        "DOOR"
    }

    fn prompt(&self) -> String {
        let code = catalog::find(&self.door.code).map_or("?", |t| t.code);
        match &self.error {
            Some(e) => format!("DOOR  {e}. Click on a wall to place door {code}:"),
            None => format!("DOOR  Click on a wall to place door {code} [Hinge/Open-close] <done>:"),
        }
    }

    fn options(&self) -> Vec<CmdOption> {
        vec![
            CmdOption::new("Hinge", "H"),
            CmdOption::new("Open-close", "O"),
            CmdOption::new("Done", ""),
        ]
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        self.error = None;
        let (edit, _) = match self.model.place(&self.door, [pt.x, pt.y]) {
            Ok(r) => r,
            Err(e) => {
                self.error = Some(e);
                return CmdResult::NeedPoint;
            }
        };
        let pieces: Vec<Vec<EntityType>> = edit.add.iter().map(|p| adapter::to_entities(p, &host_pattern)).collect();
        let counts: Vec<usize> = pieces.iter().map(Vec::len).collect();
        let added: Vec<EntityType> = pieces.into_iter().flatten().collect();
        // The first deleted entity is replaced by everything new; the others
        // are just deleted. The host answers with the new handles.
        let mut gone = edit.delete.clone().into_iter();
        let Some(first) = gone.next() else {
            return CmdResult::Cancel;
        };
        let mut replacements = vec![(first, added)];
        replacements.extend(gone.map(|h| (h, Vec::new())));
        self.pending = Some((edit, counts));
        CmdResult::ReplaceManyContinue(replacements)
    }

    fn on_entities_replaced(&mut self, replacements: &[(Handle, Vec<Handle>)]) {
        let Some((edit, counts)) = self.pending.take() else {
            return;
        };
        let new: Vec<Handle> = replacements.first().map(|(_, v)| v.clone()).unwrap_or_default();
        if new.len() == counts.iter().sum::<usize>() {
            self.model.apply(&edit, &counts, &new);
        } else {
            self.error = Some("The drawing could not be updated".to_string());
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
            "" => None,
            "H" | "HINGE" => {
                self.door.hinge_start ^= true;
                Some(CmdResult::NeedPoint)
            }
            "O" | "OPEN" | "CLOSE" | "OPEN-CLOSE" => {
                self.door.open ^= true;
                Some(CmdResult::NeedPoint)
            }
            "D" | "DONE" => Some(CmdResult::Cancel),
            _ => None,
        }
    }

    fn preview_entities(&mut self, _document: &codec::CadDocument, pt: DVec3) -> Vec<EntityType> {
        match self.model.preview(&self.door, [pt.x, pt.y]) {
            Some(prims) => adapter::preview_entities(&prims, &host_pattern),
            None => Vec::new(),
        }
    }

    // Doors carry their own layers and tags; the host must not restyle them.
    fn preserve_commit_layer(&self) -> bool {
        true
    }

    fn preserve_commit_style(&self) -> bool {
        true
    }
}
