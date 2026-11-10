//! Host side of the component registry (the part that needs the app's
//! private state). Added by the SipilStock rebrand kit; hooks into upstream
//! are single lines (patch 0019).
//!
//! A component implements [`HostComponent`] in `app/sipil_arch/<id>.rs` and
//! adds one row to [`components`]. Its own state lives in [`Host`], keyed by
//! its id, so the app's state struct never changes when a component is added.

use std::any::Any;
use std::collections::HashMap;

use codec::{EntityType, Handle};
use iced::{Element, Task};

use crate::app::{Message, OpenCADStudio};
use crate::sipil_arch::door::adapter;
use crate::sipil_arch::door::planner::Edit;
use crate::sipil_arch::{registry, ArchMsg};
use crate::sipil_wall::command::host_pattern;
use crate::ui::modal::ModalSizing;

mod beam;
mod column;
mod door;

/// State of every component, by component id.
#[derive(Default)]
pub struct Host {
    states: HashMap<&'static str, Box<dyn Any + Send>>,
}

impl Host {
    /// The state of component `id`, created empty on first use.
    pub fn state<T: Default + Send + 'static>(&mut self, id: &'static str) -> &mut T {
        self.states
            .entry(id)
            .or_insert_with(|| Box::new(T::default()))
            .downcast_mut::<T>()
            .expect("a component keeps one type of state")
    }

    pub fn state_ref<T: 'static>(&self, id: &'static str) -> Option<&T> {
        self.states.get(id)?.downcast_ref::<T>()
    }
}

/// What a component does inside the app.
pub trait HostComponent {
    /// Same as its row in `sipil_arch::registry::COMPONENTS`.
    fn id(&self) -> &'static str;
    /// Its ribbon button or command ran.
    fn run(&self, app: &mut OpenCADStudio, tab: usize, command: &str) -> Task<Message>;
    /// An entity was clicked: open the popup if it belongs to this component.
    fn open_at(&self, _app: &mut OpenCADStudio, _tab: usize, _handle: Handle) -> bool {
        false
    }
    /// A message from this component's popup.
    fn update(&self, app: &mut OpenCADStudio, msg: &ArchMsg) -> Task<Message>;
    /// Title of the popup window.
    fn title(&self, app: &OpenCADStudio) -> String;
    /// The popup, while it is open.
    fn popup<'a>(&self, app: &'a OpenCADStudio, sizing: ModalSizing) -> Option<Element<'a, Message>>;
}

/// Every component, in the same order as the registry. Add yours here.
fn components() -> Vec<Box<dyn HostComponent>> {
    vec![Box::new(door::Door), Box::new(column::Column), Box::new(beam::Beam)]
}

impl OpenCADStudio {
    /// A command of a registered component (its ribbon button or typed name).
    pub(super) fn dispatch_sipil_arch(&mut self, cmd: &str, i: usize) -> Option<Task<Message>> {
        let info = registry::by_command(cmd)?;
        let comp = components().into_iter().find(|c| c.id() == info.id)?;
        Some(comp.run(self, i, cmd))
    }

    /// An entity was clicked: let the components open their popup for it.
    pub(super) fn sipil_arch_open_at(&mut self, i: usize, handle: Handle) -> bool {
        components().into_iter().any(|c| c.open_at(self, i, handle))
    }

    pub(super) fn sipil_arch_update(&mut self, msg: ArchMsg) -> Task<Message> {
        match components().into_iter().find(|c| c.id() == msg.id) {
            Some(c) => c.update(self, &msg),
            None => Task::none(),
        }
    }

    pub(super) fn sipil_arch_title(&self, id: &str) -> String {
        components()
            .into_iter()
            .find(|c| c.id() == id)
            .map_or_else(|| id.to_string(), |c| c.title(self))
    }

    pub(super) fn sipil_arch_popup_view<'a>(&'a self, id: &str, sizing: ModalSizing) -> Element<'a, Message> {
        components()
            .into_iter()
            .find(|c| c.id() == id)
            .and_then(|c| c.popup(self, sizing))
            .unwrap_or_else(|| iced::widget::Space::new().into())
    }

    /// Make the layers of a component exist: a layer already in the drawing
    /// keeps its own look, a missing one is created with the colour given.
    pub(super) fn ensure_arch_layers(&mut self, i: usize, layers: &[(&str, i16)]) {
        let mut changed = false;
        let scene = &mut self.tabs[i].scene;
        for (name, aci) in layers {
            let name: &str = name;
            if scene.document.layers.contains(name) {
                continue;
            }
            scene.ensure_layer(name);
            if let Some(layer) = scene.document.layers.get_mut(name) {
                layer.color = codec::types::Color::from_index(*aci);
                changed = true;
            }
        }
        if changed {
            self.refresh_layer_panel();
        }
    }

    /// Erase `delete` and add `entities` as one undo step, then redraw. For a
    /// component whose command is not running (an edit from its popup).
    pub(super) fn apply_arch_entities(&mut self, i: usize, label: &str, delete: &[Handle], entities: Vec<EntityType>) {
        self.push_undo_snapshot(i, label);
        let scene = &mut self.tabs[i].scene;
        scene.erase_entities(delete);
        for entity in entities {
            scene.add_entity(entity);
        }
        scene.deselect_all();
        scene.bump_geometry();
        self.tabs[i].dirty = true;
        self.refresh_properties();
    }

    /// Apply a door planner edit (wall, doors and sheets) as one undo step.
    pub(super) fn apply_sipil_edit(&mut self, i: usize, label: &str, edit: &Edit) {
        self.ensure_door_layers(i);
        let entities = adapter::edit_entities(edit, &host_pattern);
        self.apply_arch_entities(i, label, &edit.delete, entities);
    }
}
