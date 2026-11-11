//! The grid component in the app: the ribbon command, the popup and what OK does.

use codec::Handle;
use iced::{Element, Task};

use super::HostComponent;
use crate::app::{Message, ModalKind, OpenCADStudio};
use crate::sipil_arch::grid::form::Fields;
use crate::sipil_arch::grid::ui::{self, GridMsg, GridState, Popup};
use crate::sipil_arch::grid::view::ViewSpec;
use crate::sipil_arch::grid::{adapter, plan};
use crate::sipil_arch::titleblock::{adapter as tb_adapter, geometry as tb_geometry};
use crate::sipil_arch::ArchMsg;
use crate::sipil_wall::engine::units_per_metre;
use crate::ui::modal::ModalSizing;

pub struct Grid;

impl OpenCADStudio {
    /// Make the grid layers exist (a layer already there keeps its own look).
    fn ensure_grid_layers(&mut self, i: usize) {
        let mut changed = false;
        let scene = &mut self.tabs[i].scene;
        for (name, aci) in plan::LAYERS {
            if scene.document.layers.contains(name) {
                continue;
            }
            scene.ensure_layer(name);
            if let Some(layer) = scene.document.layers.get_mut(name) {
                layer.color = codec::types::Color::from_index(aci);
                changed = true;
            }
        }
        if changed {
            self.refresh_layer_panel();
        }
    }

    /// The popup for the drawing in tab `tab`: the grid it holds, or else the
    /// project's, or else the house grid.
    fn sipil_grid_popup(&self, tab: usize) -> Popup {
        let found = adapter::scan(self.tabs[tab].scene.document.entities());
        let project = self.sipil_project_grid();
        match found {
            Some(f) => {
                // A grid that is the project's is kept in step with it.
                let same = project.as_ref() == Some(&f.def);
                Popup {
                    fields: Fields::from_def(&f.def),
                    view: f.view,
                    scale: Some(f.scale),
                    handles: f.handles,
                    project: self.sipil_has_project().then_some(same),
                    error: None,
                }
            }
            None => Popup {
                fields: Fields::from_def(&project.unwrap_or_default()),
                view: ViewSpec::plan().format(),
                scale: None,
                handles: Vec::new(),
                project: self.sipil_has_project().then_some(false),
                error: None,
            },
        }
    }

    /// OK in the grid popup: draw the grid (or redraw it), and keep the project in step.
    fn sipil_grid_apply(&mut self) -> Task<Message> {
        let Some(popup) = self.sipil_arch.state::<GridState>(ui::ID).popup.take() else {
            self.active_modal = None;
            return Task::none();
        };
        let (def, grid) = match popup
            .fields
            .to_def()
            .and_then(|d| d.resolve().map(|g| (d, g)))
        {
            Ok(v) => v,
            Err(e) => return self.sipil_grid_keep(popup, e),
        };
        let i = self.active_tab;
        if self.tabs.get(i).is_none() {
            self.active_modal = None;
            return Task::none();
        }
        let view = ViewSpec::parse(&popup.view).unwrap_or_else(ViewSpec::plan);
        let doc = &self.tabs[i].scene.document;
        let upm = units_per_metre(doc.header.insertion_units);
        // The scale of the grid already there, else the one that fits the room
        // the title block leaves (or A3 with the standard block).
        let scale = popup.scale.unwrap_or_else(|| {
            let tb = tb_adapter::scan(doc.entities())
                .map(|f| f.def)
                .unwrap_or_default();
            match tb_geometry::drawing_area(&tb) {
                Ok(a) => f64::from(plan::pick_scale(
                    &grid,
                    &view,
                    a.width() * 0.92,
                    a.height() * 0.92,
                )),
                Err(_) => 100.0,
            }
        });
        let prims = plan::build(&grid, &view, upm, scale);
        let entities = adapter::to_entities(&prims, &view.format(), scale, &def);
        self.active_modal = None;

        self.push_undo_snapshot(i, "GRID");
        self.ensure_grid_layers(i);
        let scene = &mut self.tabs[i].scene;
        let old: Vec<Handle> = popup.handles.clone();
        scene.erase_entities(&old);
        for entity in entities {
            scene.add_entity(entity);
        }
        scene.deselect_all();
        scene.bump_geometry();
        self.tabs[i].dirty = true;
        self.refresh_properties();

        let mut note = String::new();
        if popup.project == Some(true) {
            match self.sipil_project_set_grid(&def) {
                Ok(0) => note = " The project grid is updated.".to_string(),
                Ok(n) => {
                    note =
                        format!(" The project grid is updated and {n} drawing(s) were added to it.")
                }
                Err(e) => self
                    .command_line
                    .push_error(&format!("GRID  The project was not updated: {e}")),
            }
        }
        self.command_line.push_info(&format!(
            "GRID  {} X lines, {} Y lines, {} levels, drawn at 1:{}.{note}",
            grid.x.len(),
            grid.y.len(),
            grid.z.len(),
            scale
        ));
        Task::none()
    }

    /// Keep the popup open with `error` shown.
    fn sipil_grid_keep(&mut self, mut popup: Popup, error: String) -> Task<Message> {
        popup.error = Some(error);
        self.sipil_arch.state::<GridState>(ui::ID).popup = Some(popup);
        Task::none()
    }
}

impl HostComponent for Grid {
    fn id(&self) -> &'static str {
        ui::ID
    }

    fn run(&self, app: &mut OpenCADStudio, tab: usize, _command: &str) -> Task<Message> {
        if app.tabs.get(tab).is_none() {
            return Task::none();
        }
        let popup = app.sipil_grid_popup(tab);
        app.sipil_arch.state::<GridState>(ui::ID).popup = Some(popup);
        app.active_modal = Some(ModalKind::SipilArch(ui::ID));
        Task::none()
    }

    fn open_at(&self, app: &mut OpenCADStudio, tab: usize, handle: Handle) -> bool {
        let tagged = app.tabs[tab]
            .scene
            .document
            .get_entity(handle)
            .and_then(adapter::read_tag)
            .is_some();
        if !tagged {
            return false;
        }
        let popup = app.sipil_grid_popup(tab);
        app.sipil_arch.state::<GridState>(ui::ID).popup = Some(popup);
        app.active_modal = Some(ModalKind::SipilArch(ui::ID));
        true
    }

    fn update(&self, app: &mut OpenCADStudio, msg: &ArchMsg) -> Task<Message> {
        let Some(m) = msg.payload::<GridMsg>() else {
            return Task::none();
        };
        if let GridMsg::Apply = m {
            return app.sipil_grid_apply();
        }
        if let Some(p) = app.sipil_arch.state::<GridState>(ui::ID).popup.as_mut() {
            ui::apply_msg(p, m);
            p.error = None;
        }
        Task::none()
    }

    fn title(&self, _app: &OpenCADStudio) -> String {
        "Grid".to_string()
    }

    fn popup<'a>(
        &self,
        app: &'a OpenCADStudio,
        sizing: ModalSizing,
    ) -> Option<Element<'a, Message>> {
        let popup = app
            .sipil_arch
            .state_ref::<GridState>(ui::ID)?
            .popup
            .as_ref()?;
        Some(ui::popup_view(popup, sizing))
    }
}
