//! The title block component in the app: the ribbon command, the popup and what
//! OK does.

use iced::{Element, Task};

use super::HostComponent;
use crate::app::{Message, ModalKind, OpenCADStudio};
use crate::sipil_arch::titleblock::form::FormFields;
use crate::sipil_arch::titleblock::model::{LAYER, LAYER_ACI};
use crate::sipil_arch::titleblock::ui::{self, Popup, TbMsg, TbState};
use crate::sipil_arch::titleblock::{adapter, geometry, sheet as tb_sheet};
use crate::sipil_arch::ArchMsg;
use crate::ui::modal::ModalSizing;

pub struct TitleBlock;

impl OpenCADStudio {
    /// The paper layout to edit in tab `tab`: the one showing, else the first.
    fn sipil_title_layout(&self, tab: usize) -> Option<String> {
        let scene = &self.tabs[tab].scene;
        if scene.current_layout != "Model" {
            return Some(scene.current_layout.clone());
        }
        scene.layout_names().into_iter().find(|n| n != "Model")
    }

    /// The popup for the sheet of tab `tab`: the title block it has, else the
    /// project's, else the standard one.
    fn sipil_title_popup(&self, tab: usize) -> Option<Popup> {
        let layout = self.sipil_title_layout(tab)?;
        let scene = &self.tabs[tab].scene;
        let on_sheet: Vec<_> = scene
            .layout_paper_entities(&layout)
            .into_iter()
            .filter_map(|h| scene.document.get_entity(h))
            .collect();
        let found = adapter::scan(on_sheet);
        let (def, handles) = match found {
            Some(f) => (f.def, f.handles),
            None => (ui::starting_def(self.sipil_project_title()), Vec::new()),
        };
        Some(Popup {
            fields: FormFields::from_def(&def),
            layout,
            handles,
            project: self.sipil_has_project().then_some(false),
            error: None,
        })
    }

    /// OK in the title block popup: redraw the frame and block on the sheet,
    /// give it the paper, and fit the drawing's viewport to the room left.
    fn sipil_title_apply(&mut self) -> Task<Message> {
        let Some(popup) = self.sipil_arch.state::<TbState>(ui::ID).popup.take() else {
            self.active_modal = None;
            return Task::none();
        };
        let checked = popup.fields.to_def().and_then(|d| {
            geometry::build(&d).and_then(|p| geometry::drawing_area(&d).map(|a| (d, p, a)))
        });
        let (def, prims, area) = match checked {
            Ok(v) => v,
            Err(e) => return self.sipil_title_keep(popup, e),
        };
        let i = self.active_tab;
        if self.tabs.get(i).is_none() {
            self.active_modal = None;
            return Task::none();
        }
        self.active_modal = None;
        self.push_undo_snapshot(i, "TITLEBLOCK");
        let layout = popup.layout.clone();
        let mut new_layer = false;
        let scene = &mut self.tabs[i].scene;
        if !scene.document.layers.contains(LAYER) {
            scene.ensure_layer(LAYER);
            if let Some(layer) = scene.document.layers.get_mut(LAYER) {
                layer.color = codec::types::Color::from_index(LAYER_ACI);
            }
            new_layer = true;
        }
        let old = popup.handles.clone();
        scene.erase_entities(&old);
        scene.ensure_app_id(crate::sipil_arch::titleblock::model::XDATA_APP);
        for e in adapter::to_entities(&prims, &def) {
            let _ = scene.document.add_entity_to_layout(e, &layout);
        }
        self.sipil_set_paper(i, &layout, &def);
        let scene = &mut self.tabs[i].scene;
        // The viewport(s) on the drawing take the room the block leaves, at the scale they have.
        scene.ensure_sheet_viewport(&layout);
        for h in scene.layout_paper_entities(&layout) {
            if let Some(codec::EntityType::Viewport(vp)) = scene.document.get_entity_mut(h) {
                tb_sheet::refit_viewport(vp, area);
            }
        }
        scene.fit_layout_sheet(&layout);
        scene.set_current_layout(layout.clone());
        scene.deselect_all();
        scene.bump_geometry();
        self.tabs[i].dirty = true;
        if new_layer {
            self.refresh_layer_panel();
        }
        self.refresh_properties();
        if popup.project == Some(true) {
            self.sipil_project_set_title(&def);
        }
        self.command_line.push_info(&format!(
            "TITLEBLOCK  {} {}, frame {:.0} x {:.0} mm, room for the drawing {:.0} x {:.0} mm.",
            def.paper.label(),
            if def.landscape {
                "landscape"
            } else {
                "portrait"
            },
            def.frame().width(),
            def.frame().height(),
            area.width(),
            area.height()
        ));
        Task::none()
    }

    /// Keep the popup open with `error` shown.
    fn sipil_title_keep(&mut self, mut popup: Popup, error: String) -> Task<Message> {
        popup.error = Some(error);
        self.sipil_arch.state::<TbState>(ui::ID).popup = Some(popup);
        Task::none()
    }
}

impl HostComponent for TitleBlock {
    fn id(&self) -> &'static str {
        ui::ID
    }

    fn run(&self, app: &mut OpenCADStudio, tab: usize, _command: &str) -> Task<Message> {
        if app.tabs.get(tab).is_none() {
            return Task::none();
        }
        let Some(popup) = app.sipil_title_popup(tab) else {
            app.command_line
                .push_info("TITLEBLOCK  This drawing has no paper layout.");
            return Task::none();
        };
        app.sipil_arch.state::<TbState>(ui::ID).popup = Some(popup);
        app.active_modal = Some(ModalKind::SipilArch(ui::ID));
        Task::none()
    }

    fn update(&self, app: &mut OpenCADStudio, msg: &ArchMsg) -> Task<Message> {
        let Some(m) = msg.payload::<TbMsg>() else {
            return Task::none();
        };
        if let TbMsg::Apply = m {
            return app.sipil_title_apply();
        }
        if let Some(p) = app.sipil_arch.state::<TbState>(ui::ID).popup.as_mut() {
            ui::apply_msg(p, m);
            p.error = None;
        }
        Task::none()
    }

    fn title(&self, _app: &OpenCADStudio) -> String {
        "Title Block".to_string()
    }

    fn popup<'a>(
        &self,
        app: &'a OpenCADStudio,
        sizing: ModalSizing,
    ) -> Option<Element<'a, Message>> {
        let popup = app
            .sipil_arch
            .state_ref::<TbState>(ui::ID)?
            .popup
            .as_ref()?;
        Some(ui::popup_view(popup, sizing))
    }
}
