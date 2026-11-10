//! The door component in the app: the ribbon command, the popup and what OK does.

use codec::Handle;
use iced::{Element, Task};

use super::HostComponent;
use crate::app::{Message, ModalKind, OpenCADStudio};
use crate::command::CadCommand;
use crate::sipil_arch::door::command::DoorCommand;
use crate::sipil_arch::door::form::Fields;
use crate::sipil_arch::door::model::{DTag, DoorDef, LAYERS};
use crate::sipil_arch::door::ui::{self, DoorMsg, DoorState, Popup, Target};
use crate::sipil_arch::door::{adapter, catalog};
use crate::sipil_arch::ArchMsg;
use crate::ui::modal::ModalSizing;

pub struct Door;

impl OpenCADStudio {
    /// Make the door layers exist (a layer already there keeps its own look).
    pub(super) fn ensure_door_layers(&mut self, i: usize) {
        self.ensure_arch_layers(i, &LAYERS);
    }

    /// OK in the door popup: start placing, or rebuild the door in the drawing.
    fn sipil_door_apply(&mut self) -> Task<Message> {
        let Some(mut popup) = self.sipil_arch.state::<DoorState>(ui::ID).popup.take() else {
            self.active_modal = None;
            return Task::none();
        };
        let def = match popup.fields.to_def(&popup.base) {
            Ok(d) => d,
            Err(e) => return self.sipil_door_keep(popup, e),
        };
        let i = self.active_tab;
        if self.tabs.get(i).is_none() {
            self.active_modal = None;
            return Task::none();
        }
        let model = adapter::scan(self.tabs[i].scene.document.entities());
        match popup.target.clone() {
            Target::New => {
                self.sipil_arch.state::<DoorState>(ui::ID).template = Some(def.clone());
                self.active_modal = None;
                if model.walls.is_empty() {
                    self.command_line.push_info("DOOR  There is no wall in this drawing. Draw a wall first (WALL).");
                    return Task::none();
                }
                self.ensure_door_layers(i);
                let cmd = DoorCommand::new(model, def);
                self.command_line.push_info(&cmd.prompt());
                self.tabs[i].active_cmd = Some(Box::new(cmd));
            }
            Target::Door(_) => match model.change_door(&def) {
                Ok(edit) => {
                    self.active_modal = None;
                    self.apply_sipil_edit(i, "DOOREDIT", &edit);
                }
                Err(e) => return self.sipil_door_keep(popup, e),
            },
        }
        Task::none()
    }

    /// Keep the popup open with `error` shown.
    fn sipil_door_keep(&mut self, mut popup: Popup, error: String) -> Task<Message> {
        popup.error = Some(error);
        self.sipil_arch.state::<DoorState>(ui::ID).popup = Some(popup);
        Task::none()
    }
}

impl HostComponent for Door {
    fn id(&self) -> &'static str {
        ui::ID
    }

    fn run(&self, app: &mut OpenCADStudio, tab: usize, _command: &str) -> Task<Message> {
        if app.tabs.get(tab).is_none() {
            return Task::none();
        }
        let state = app.sipil_arch.state::<DoorState>(ui::ID);
        let template = state.template.clone().unwrap_or_else(|| DoorDef::of_type(catalog::fallback()));
        state.popup = Some(Popup {
            target: Target::New,
            fields: Fields::from_def(&template),
            base: template,
            error: None,
        });
        app.active_modal = Some(ModalKind::SipilArch(ui::ID));
        Task::none()
    }

    fn open_at(&self, app: &mut OpenCADStudio, tab: usize, handle: Handle) -> bool {
        let tag = app.tabs[tab].scene.document.get_entity(handle).and_then(adapter::read_tag);
        let Some(DTag::Door { id, .. }) = tag else {
            return false;
        };
        let model = adapter::scan(app.tabs[tab].scene.document.entities());
        let Some(def) = model.doors.iter().find(|d| d.def.id == id).map(|d| d.def.clone()) else {
            return false;
        };
        app.sipil_arch.state::<DoorState>(ui::ID).popup = Some(Popup {
            target: Target::Door(id),
            fields: Fields::from_def(&def),
            base: def,
            error: None,
        });
        app.active_modal = Some(ModalKind::SipilArch(ui::ID));
        true
    }

    fn update(&self, app: &mut OpenCADStudio, msg: &ArchMsg) -> Task<Message> {
        let Some(m) = msg.payload::<DoorMsg>() else {
            return Task::none();
        };
        if let DoorMsg::Apply = m {
            return app.sipil_door_apply();
        }
        if let Some(p) = app.sipil_arch.state::<DoorState>(ui::ID).popup.as_mut() {
            ui::apply_msg(&mut p.fields, m);
            p.error = None;
        }
        Task::none()
    }

    fn title(&self, app: &OpenCADStudio) -> String {
        match app.sipil_arch.state_ref::<DoorState>(ui::ID).and_then(|s| s.popup.as_ref()).map(|p| &p.target) {
            Some(Target::Door(id)) => format!("Door #{id}"),
            _ => "Door".to_string(),
        }
    }

    fn popup<'a>(&self, app: &'a OpenCADStudio, sizing: ModalSizing) -> Option<Element<'a, Message>> {
        let popup = app.sipil_arch.state_ref::<DoorState>(ui::ID)?.popup.as_ref()?;
        Some(ui::popup_view(popup, sizing))
    }
}
