//! SipilCAD wall tool: the part that needs the app's private state.
//!
//! Added by the SipilStock rebrand kit. It is a child of `app` so it can reach
//! the private tab/scene/modal fields; everything else of the tool lives in
//! `crate::sipil_wall`. Hooks into upstream are single lines (patch 0017).

use codec::{CadDocument, Handle};
use iced::Task;

use crate::app::{Message, ModalKind, OpenCADStudio};
use crate::sipil_wall::command::{host_pattern, WallCommand};
use crate::sipil_wall::engine::{self, ColumnBox, LAYERS};
use crate::sipil_wall::entities::{moved_by, read_tag, to_entities};
use crate::sipil_wall::state::{Fields, Popup, Target};
use crate::sipil_wall::ui::{self, Msg};
use crate::ui::dock::PanelId;

/// What the drawing already holds of walls.
struct Scan {
    /// Columns of other walls, reused where a new wall meets them.
    columns: Vec<ColumnBox>,
    /// Highest wall id in use.
    max_id: u32,
}

/// Walk the drawing once for wall columns and ids. `skip` leaves one wall's
/// own columns out (when it is about to be rebuilt).
fn scan(doc: &CadDocument, skip: Option<u32>) -> Scan {
    let mut out = Scan {
        columns: Vec::new(),
        max_id: 0,
    };
    for e in doc.entities() {
        let Some(tag) = read_tag(e) else { continue };
        out.max_id = out.max_id.max(tag.id);
        if skip == Some(tag.id) {
            continue;
        }
        if let Some(b) = tag.column {
            out.columns.push(b);
        }
    }
    out
}

impl OpenCADStudio {
    /// Command-line entry points: `BW`, `BWSET`, `BWPANEL`.
    pub(super) fn dispatch_sipil_wall(&mut self, cmd: &str, i: usize) -> Option<Task<Message>> {
        match cmd
            .trim()
            .trim_start_matches(['_', '.'])
            .to_ascii_uppercase()
            .as_str()
        {
            "BW" => Some(self.sipil_wall_start(i)),
            "BWSET" => {
                self.sipil_wall_open_defaults();
                Some(Task::none())
            }
            "BWPANEL" => {
                let open = !self.dock_panel_visible(PanelId::SipilWall);
                Some(self.dock_set_open(PanelId::SipilWall, open))
            }
            _ => None,
        }
    }

    /// The five wall layers, created with the LISP's colours when missing.
    fn ensure_sipil_layers(&mut self, i: usize) {
        let mut created = false;
        let scene = &mut self.tabs[i].scene;
        for (name, aci) in LAYERS {
            if scene.document.layers.contains(name) {
                continue;
            }
            scene.ensure_layer(name);
            if let Some(layer) = scene.document.layers.get_mut(name) {
                layer.color = codec::types::Color::from_index(aci);
            }
            created = true;
        }
        if created {
            self.refresh_layer_panel();
        }
    }

    fn sipil_wall_start(&mut self, i: usize) -> Task<Message> {
        use crate::command::CadCommand;
        self.ensure_sipil_layers(i);
        let found = scan(&self.tabs[i].scene.document, None);
        let upm = engine::units_per_metre(self.tabs[i].scene.document.header.insertion_units);
        let cmd = WallCommand::new(
            self.sipil_wall.settings.clone(),
            upm,
            found.columns,
            found.max_id.wrapping_add(1),
        );
        self.command_line.push_info(&cmd.prompt());
        self.tabs[i].active_cmd = Some(Box::new(cmd));
        Task::none()
    }

    fn sipil_wall_open_defaults(&mut self) {
        self.sipil_wall.popup = Some(Popup {
            target: Target::Defaults,
            fields: Fields::from_params(&self.sipil_wall.settings),
            def: None,
            error: None,
        });
        self.active_modal = Some(ModalKind::SipilWall);
    }

    /// Open the parameter popup for wall `id`. The definition comes from the
    /// wall's anchor entity, moved to where that entity is now, so a wall that
    /// was dragged is edited in place rather than snapped back.
    fn sipil_wall_open_wall(&mut self, i: usize, id: u32) -> bool {
        let found = self.tabs[i].scene.document.entities().find_map(|e| {
            let tag = read_tag(e).filter(|t| t.id == id)?;
            let mut def = tag.def.clone()?;
            if let Some([dx, dy]) = moved_by(e, &tag) {
                for p in def.pts.iter_mut() {
                    p[0] += dx;
                    p[1] += dy;
                }
            }
            Some(def)
        });
        let Some(def) = found else { return false };
        self.sipil_wall.popup = Some(Popup {
            target: Target::Wall(id),
            fields: Fields::from_params(&def.params),
            def: Some(def),
            error: None,
        });
        self.active_modal = Some(ModalKind::SipilWall);
        true
    }

    /// Double-click (or click) on `handle`: open its wall's popup if it is a wall part.
    pub(super) fn sipil_wall_open_at(&mut self, i: usize, handle: Handle) -> bool {
        let tag = self.tabs[i]
            .scene
            .document
            .get_entity(handle)
            .and_then(read_tag);
        let Some(tag) = tag else { return false };
        self.sipil_wall.opened_for = Some(handle);
        self.sipil_wall_open_wall(i, tag.id)
    }

    /// Called whenever the selection changed: a single selected wall part
    /// opens the popup right away (unless the user turned that off).
    pub(super) fn sipil_wall_on_selection(&mut self) {
        if !self.sipil_wall.popup_on_click || self.active_modal.is_some() {
            return;
        }
        let i = self.active_tab;
        // Runs from refresh_properties, which can fire before any tab exists.
        let Some(tab) = self.tabs.get(i) else { return };
        if tab.active_cmd.is_some() {
            return;
        }
        let handles = tab.scene.selected_handles_in_order();
        let [handle] = handles.as_slice() else {
            self.sipil_wall.opened_for = None;
            return;
        };
        if self.sipil_wall.opened_for == Some(*handle) {
            return;
        }
        self.sipil_wall_open_at(i, *handle);
    }

    fn sipil_wall_sync_panel(&mut self) {
        if let Ok(p) = self.sipil_wall.panel.to_params() {
            self.sipil_wall.settings = p;
        }
    }

    pub(super) fn sipil_wall_update(&mut self, msg: Msg) -> Task<Message> {
        match msg {
            Msg::Preset(idx) => {
                ui::apply_preset(&mut self.sipil_wall.panel, idx);
                self.sipil_wall_sync_panel();
            }
            Msg::Panel(field) => {
                ui::apply_field(&mut self.sipil_wall.panel, field);
                self.sipil_wall_sync_panel();
            }
            Msg::PopupField(field) => {
                if let Some(p) = self.sipil_wall.popup.as_mut() {
                    ui::apply_field(&mut p.fields, field);
                    p.error = None;
                }
            }
            Msg::PopupPreset(idx) => {
                if let Some(p) = self.sipil_wall.popup.as_mut() {
                    ui::apply_preset(&mut p.fields, idx);
                    p.error = None;
                }
            }
            Msg::OnClick(on) => self.sipil_wall.popup_on_click = on,
            Msg::PopupApply => return self.sipil_wall_apply(),
        }
        Task::none()
    }

    /// OK in the popup: store new defaults, or rebuild the wall in the drawing.
    fn sipil_wall_apply(&mut self) -> Task<Message> {
        let Some(mut popup) = self.sipil_wall.popup.take() else {
            self.active_modal = None;
            return Task::none();
        };
        let params = match popup.fields.to_params() {
            Ok(p) => p,
            Err(e) => {
                popup.error = Some(e);
                self.sipil_wall.popup = Some(popup);
                return Task::none();
            }
        };
        match popup.target.clone() {
            Target::Defaults => {
                self.sipil_wall.set_settings(params);
                self.active_modal = None;
            }
            Target::Wall(id) => {
                let Some(mut def) = popup.def.clone() else {
                    self.active_modal = None;
                    return Task::none();
                };
                def.params = params;
                let i = self.active_tab;
                let others = scan(&self.tabs[i].scene.document, Some(id));
                let out = match engine::build(&def, &others.columns) {
                    Ok(out) => out,
                    Err(e) => {
                        popup.error = Some(e);
                        self.sipil_wall.popup = Some(popup);
                        return Task::none();
                    }
                };
                let old: Vec<Handle> = self.tabs[i]
                    .scene
                    .document
                    .entities()
                    .filter(|e| read_tag(e).is_some_and(|t| t.id == id))
                    .map(|e| e.common().handle)
                    .collect();
                self.active_modal = None;
                self.push_undo_snapshot(i, "BWEDIT");
                self.ensure_sipil_layers(i);
                let scene = &mut self.tabs[i].scene;
                scene.erase_entities(&old);
                for entity in to_entities(&out, id, &def, &host_pattern) {
                    scene.add_entity(entity);
                }
                scene.deselect_all();
                scene.bump_geometry();
                self.tabs[i].dirty = true;
                self.refresh_properties();
            }
        }
        Task::none()
    }
}
