//! `WALL` (alias `BW`): draw a wall along clicked axis points, with a live preview.
//!
//! The preview is the real result (outline, plaster, hatch, columns) built by
//! the same engine that commits it, so what the user sees under the cursor is
//! what is drawn. Enter finishes, `C` closes the loop, `U` or Ctrl+Z removes
//! the last point, Esc cancels. This is the port of the LISP's `BW` loop.

use codec::EntityType;
use glam::DVec3;

use super::engine::{self, ColumnBox, Output, WallDef, WallParams, P};
use super::entities;
use crate::command::{CadCommand, CmdOption, CmdResult};

/// Catalog hatch pattern as the prebaked lines a hatch entity stores. The
/// host's own pattern helpers, so a wall hatch is the same kind of object as
/// one made by the HATCH command.
pub fn host_pattern(name: &str, scale: f64) -> Option<codec::entities::HatchPattern> {
    let entry = crate::scene::model::hatch_patterns::find(name)?;
    let mut pattern = crate::scene::model::hatch_patterns::build_dxf_pattern(entry);
    crate::entities::hatch::scale_pattern_geometry(&mut pattern, scale);
    Some(pattern)
}

pub struct WallCommand {
    params: WallParams,
    /// Drawing units per metre.
    upm: f64,
    /// Columns already in the drawing, reused where the wall meets them.
    existing: Vec<ColumnBox>,
    /// Id shared by every entity of the wall being drawn.
    id: u32,
    pts: Vec<P>,
}

impl WallCommand {
    pub fn new(params: WallParams, upm: f64, existing: Vec<ColumnBox>, id: u32) -> Self {
        Self {
            params,
            upm,
            existing,
            id,
            pts: Vec::new(),
        }
    }

    fn def(&self, pts: Vec<P>, closed: bool) -> WallDef {
        WallDef {
            params: self.params.clone(),
            upm: self.upm,
            pts,
            closed,
        }
    }

    fn entities(&self, out: &Output, def: &WallDef) -> Vec<EntityType> {
        entities::to_entities(out, self.id, def, &host_pattern)
    }

    /// Turn the points so far into the committed wall, or cancel if there is
    /// nothing to draw (a single point).
    fn finish(&mut self, closed: bool) -> CmdResult {
        if self.pts.len() < 2 {
            return CmdResult::Cancel;
        }
        let def = self.def(self.pts.clone(), closed);
        match engine::build(&def, &self.existing) {
            Ok(out) => CmdResult::CommitEntitiesAndExit(self.entities(&out, &def)),
            Err(_) => CmdResult::Cancel,
        }
    }
}

impl CadCommand for WallCommand {
    fn name(&self) -> &'static str {
        "WALL"
    }

    fn prompt(&self) -> String {
        if self.pts.is_empty() {
            "WALL  Specify start point of wall axis:".to_string()
        } else {
            "WALL  Specify next point [Close/Undo] <done>:".to_string()
        }
    }

    fn options(&self) -> Vec<CmdOption> {
        match self.pts.len() {
            0 => Vec::new(),
            1 | 2 => vec![CmdOption::new("Undo", "U"), CmdOption::new("Done", "")],
            _ => vec![
                CmdOption::new("Close", "C"),
                CmdOption::new("Undo", "U"),
                CmdOption::new("Done", ""),
            ],
        }
    }

    fn on_point(&mut self, pt: DVec3) -> CmdResult {
        let p = [pt.x, pt.y];
        // A repeated click on the same spot adds nothing.
        if self
            .pts
            .last()
            .map_or(true, |q| (q[0] - p[0]).hypot(q[1] - p[1]) > 1e-9)
        {
            self.pts.push(p);
        }
        CmdResult::NeedPoint
    }

    fn on_enter(&mut self) -> CmdResult {
        self.finish(false)
    }

    fn wants_text_input(&self) -> bool {
        true
    }

    fn on_text_input(&mut self, text: &str) -> Option<CmdResult> {
        match text.trim().to_ascii_uppercase().as_str() {
            "" => None,
            "C" | "CLOSE" if self.pts.len() >= 3 => Some(self.finish(true)),
            "U" | "UNDO" => {
                self.pts.pop();
                Some(CmdResult::NeedPoint)
            }
            "D" | "DONE" => Some(self.finish(false)),
            _ => None,
        }
    }

    fn on_undo_step(&mut self) -> Option<CmdResult> {
        if self.pts.is_empty() {
            return None;
        }
        self.pts.pop();
        Some(CmdResult::NeedPoint)
    }

    fn preview_entities(&mut self, _document: &codec::CadDocument, pt: DVec3) -> Vec<EntityType> {
        let cursor = [pt.x, pt.y];
        let (out, def) = if self.pts.is_empty() {
            (
                engine::marker(&self.params, self.upm, cursor),
                self.def(vec![cursor], false),
            )
        } else {
            let mut pts = self.pts.clone();
            if pts
                .last()
                .map_or(true, |q| (q[0] - cursor[0]).hypot(q[1] - cursor[1]) > 1e-9)
            {
                pts.push(cursor);
            }
            let def = self.def(pts, false);
            match engine::build(&def, &self.existing) {
                Ok(out) => (out, def),
                Err(_) => return Vec::new(),
            }
        };
        self.entities(&out, &def)
    }

    // Walls carry their own layers, colours and tags; the host must not
    // restyle them to the current layer on commit.
    fn preserve_commit_layer(&self) -> bool {
        true
    }

    fn preserve_commit_style(&self) -> bool {
        true
    }
}
