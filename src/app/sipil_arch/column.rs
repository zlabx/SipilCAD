//! The column component in the app: the ribbon command, the popup and what OK does.

use codec::Handle;
use iced::{Element, Task};

use super::HostComponent;
use crate::app::{Message, ModalKind, OpenCADStudio};
use crate::command::CadCommand;
use crate::sipil_arch::column::command::ColumnCommand;
use crate::sipil_arch::column::form::Fields;
use crate::sipil_arch::column::model::{ColumnDef, LAYERS};
use crate::sipil_arch::column::ui::{self, ColumnMsg, ColumnState, Popup, Target};
use crate::sipil_arch::column::adapter;
use crate::sipil_arch::steel::catalog;
use crate::sipil_arch::ArchMsg;
use crate::sipil_wall::command::host_pattern;
use crate::sipil_wall::engine;
use crate::ui::modal::ModalSizing;

pub struct Column;

impl OpenCADStudio {
    /// OK in the column popup: start placing, or redraw the column in the drawing.
    fn sipil_column_apply(&mut self) -> Task<Message> {
        let Some(mut popup) = self.sipil_arch.state::<ColumnState>(ui::ID).popup.take() else {
            self.active_modal = None;
            return Task::none();
        };
        let def = match popup.fields.to_def(&popup.base) {
            Ok(d) => d,
            Err(e) => return self.sipil_column_keep(popup, e),
        };
        let i = self.active_tab;
        if self.tabs.get(i).is_none() {
            self.active_modal = None;
            return Task::none();
        }
        let model = adapter::scan(self.tabs[i].scene.document.entities());
        match popup.target.clone() {
            Target::New => {
                self.sipil_arch.state::<ColumnState>(ui::ID).template = Some(def.clone());
                self.active_modal = None;
                self.ensure_arch_layers(i, &LAYERS);
                let cmd = ColumnCommand::new(model, def);
                self.command_line.push_info(&cmd.prompt());
                self.tabs[i].active_cmd = Some(Box::new(cmd));
            }
            Target::Column(_) => match model.change(&def) {
                Ok(edit) => {
                    self.active_modal = None;
                    self.ensure_arch_layers(i, &LAYERS);
                    let entities = adapter::edit_entities(&edit, &host_pattern);
                    self.apply_arch_entities(i, "COLUMNEDIT", &edit.delete, entities);
                }
                Err(e) => return self.sipil_column_keep(popup, e),
            },
        }
        Task::none()
    }

    /// Keep the popup open with `error` shown.
    fn sipil_column_keep(&mut self, mut popup: Popup, error: String) -> Task<Message> {
        popup.error = Some(error);
        self.sipil_arch.state::<ColumnState>(ui::ID).popup = Some(popup);
        Task::none()
    }
}

impl HostComponent for Column {
    fn id(&self) -> &'static str {
        ui::ID
    }

    fn run(&self, app: &mut OpenCADStudio, tab: usize, _command: &str) -> Task<Message> {
        let Some(t) = app.tabs.get(tab) else {
            return Task::none();
        };
        let upm = engine::units_per_metre(t.scene.document.header.insertion_units);
        let state = app.sipil_arch.state::<ColumnState>(ui::ID);
        // The unit is the drawing's, not the one the settings were made in.
        let template = match state.template.clone() {
            Some(t) => ColumnDef { upm, ..t },
            None => ColumnDef::new(catalog::default_profile(), upm),
        };
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
        let Some(tag) = tag else {
            return false;
        };
        let model = adapter::scan(app.tabs[tab].scene.document.entities());
        let Some(def) = model.columns.iter().find(|c| c.def.id == tag.id).map(|c| c.def.clone()) else {
            return false;
        };
        app.sipil_arch.state::<ColumnState>(ui::ID).popup = Some(Popup {
            target: Target::Column(tag.id),
            fields: Fields::from_def(&def),
            base: def,
            error: None,
        });
        app.active_modal = Some(ModalKind::SipilArch(ui::ID));
        true
    }

    fn update(&self, app: &mut OpenCADStudio, msg: &ArchMsg) -> Task<Message> {
        let Some(m) = msg.payload::<ColumnMsg>() else {
            return Task::none();
        };
        if let ColumnMsg::Apply = m {
            return app.sipil_column_apply();
        }
        if let Some(p) = app.sipil_arch.state::<ColumnState>(ui::ID).popup.as_mut() {
            ui::apply_msg(&mut p.fields, m);
            p.error = None;
        }
        Task::none()
    }

    fn title(&self, app: &OpenCADStudio) -> String {
        match app.sipil_arch.state_ref::<ColumnState>(ui::ID).and_then(|s| s.popup.as_ref()).map(|p| &p.target) {
            Some(Target::Column(id)) => format!("Column #{id}"),
            _ => "Column".to_string(),
        }
    }

    fn popup<'a>(&self, app: &'a OpenCADStudio, sizing: ModalSizing) -> Option<Element<'a, Message>> {
        let popup = app.sipil_arch.state_ref::<ColumnState>(ui::ID)?.popup.as_ref()?;
        Some(ui::popup_view(popup, sizing))
    }
}
