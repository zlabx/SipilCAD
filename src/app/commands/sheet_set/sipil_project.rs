//! SipilCAD projects: the part of the project browser that needs the app's
//! private state.
//!
//! Added by the SipilStock rebrand kit. It is a child of the sheet set command
//! module (patch 0019) so it can use that module's private helpers
//! (`paper_layouts`, `utc_stamp`, `write_set`, `sheet_sets_changed`); everything
//! else lives in `crate::sipil_project`. Hooks into upstream are one or two lines.
//!
//! A project is a sheet set: categories are subsets, drawings are sheets. A
//! drawing from a template starts as an empty slot (a sheet without a layout
//! reference); opening it creates the DWG.

use crate::app::{Message, ModalKind, OpenCADStudio};
use crate::sipil_project::db as project_db;
use crate::sipil_project::state as project_state;
use crate::sipil_project::templates::DEFAULT_TEMPLATE;
use crate::sipil_project::ui::Msg;
use crate::ui::window::sheet_set::SsmTab;
use iced::Task;

#[cfg(not(target_arch = "wasm32"))]
use super::{paper_layouts, utc_stamp, write_set};
#[cfg(not(target_arch = "wasm32"))]
use crate::ui::window::sheet_set::SheetSetMsg;
#[cfg(not(target_arch = "wasm32"))]
use codec::sheet_set::{
    self as ss, ComponentKind, LayoutReference, SheetSetData, SheetSetDatabase,
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

impl OpenCADStudio {
    /// At startup: "Project 1" (the default template) is open and the Sheet Set
    /// Manager is showing, so the project browser is there from the first frame.
    /// Skipped in unit tests, which expect a bare app.
    pub(in crate::app) fn sipil_project_startup(&mut self) {
        if cfg!(test) || !self.sheet_set.sets.is_empty() {
            return;
        }
        let name = project_state::next_project_name(&[]);
        self.sheet_set
            .sets
            .push(project_db::build_project(&name, DEFAULT_TEMPLATE));
        self.sheet_set.current = Some(0);
        self.sheet_set.tab = SsmTab::SheetList;
        self.show_sheet_set_manager(true);
        self.sheet_sets_changed();
    }

    /// The panel buttons and the New Project popup.
    pub(in crate::app) fn sipil_project_update(&mut self, m: Msg) -> Task<Message> {
        match m {
            Msg::New => {
                self.sipil_project_new();
                Task::none()
            }
            Msg::Open => self.sipil_project_open(),
            Msg::Name(v) => {
                if let Some(f) = self.sipil_project.form.as_mut() {
                    f.name = v;
                    f.error = None;
                }
                Task::none()
            }
            Msg::Folder(v) => {
                if let Some(f) = self.sipil_project.form.as_mut() {
                    f.folder = v;
                    f.error = None;
                }
                Task::none()
            }
            Msg::Template(t) => {
                if let Some(f) = self.sipil_project.form.as_mut() {
                    f.template = t;
                }
                Task::none()
            }
            Msg::Browse => self.sipil_project_browse(),
            Msg::Picked(picked) => {
                if let (Some(path), Some(f)) = (picked, self.sipil_project.form.as_mut()) {
                    f.folder = path.to_string_lossy().to_string();
                    f.error = None;
                }
                Task::none()
            }
            Msg::Create => {
                self.sipil_project_create();
                Task::none()
            }
        }
    }

    fn sipil_project_new(&mut self) {
        let names: Vec<String> = self
            .sheet_set
            .sets
            .iter()
            .map(|db| db.name().to_string())
            .collect();
        self.sipil_project.form = Some(project_state::ProjectForm::new(
            &names,
            project_state::default_folder(),
        ));
        self.active_modal = Some(ModalKind::SipilProject);
    }

    /// Open Project: the sheet set file picker (the same one as the combo's "Open...").
    #[cfg(not(target_arch = "wasm32"))]
    fn sipil_project_open(&mut self) -> Task<Message> {
        Task::done(Message::SheetSet(SheetSetMsg::PickSet(None)))
    }

    /// A browser cannot read a `.dst` yet (the panel disables the button).
    #[cfg(target_arch = "wasm32")]
    fn sipil_project_open(&mut self) -> Task<Message> {
        self.command_line.push_info(
            "Open Project needs the desktop app: a browser cannot read project files (.dst) yet.",
        );
        Task::none()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn sipil_project_browse(&mut self) -> Task<Message> {
        Task::perform(
            async {
                crate::sys::file_dialog()
                    .set_title("Project location")
                    .pick_folder()
                    .await
                    .map(|h| crate::sys::handle_path(&h))
            },
            |p| Message::SipilProject(Msg::Picked(p)),
        )
    }

    #[cfg(target_arch = "wasm32")]
    fn sipil_project_browse(&mut self) -> Task<Message> {
        Task::none()
    }

    /// Create from the popup: close it on success, else show why in the popup.
    fn sipil_project_create(&mut self) {
        let Some(mut form) = self.sipil_project.form.clone() else {
            return;
        };
        match self.sipil_project_make(&form) {
            Ok(()) => {
                self.sipil_project.form = None;
                self.active_modal = None;
            }
            Err(e) => {
                form.error = Some(e);
                self.sipil_project.form = Some(form);
            }
        }
    }

    fn sipil_project_make(&mut self, form: &project_state::ProjectForm) -> Result<(), String> {
        let name = project_state::validate_name(&form.name)?;
        if self
            .sheet_set
            .sets
            .iter()
            .any(|db| db.name().eq_ignore_ascii_case(&name))
        {
            return Err(format!("A project named \"{name}\" is already open."));
        }
        #[allow(unused_mut)]
        let mut database = project_db::build_project(&name, form.template);
        #[cfg(not(target_arch = "wasm32"))]
        save_new_project(&mut database, &name, &form.folder)?;
        self.sheet_set.sets.push(database);
        self.sheet_set.current = Some(self.sheet_set.sets.len() - 1);
        self.sheet_set.tab = SsmTab::SheetList;
        self.show_sheet_set_manager(true);
        self.sheet_sets_changed();
        Ok(())
    }

    /// A drawing slot (a sheet with no layout yet) was opened. In a saved project
    /// the drawing is created as `<project>/<category>/<title>.dwg`, linked to the
    /// slot and opened. Otherwise (a browser, or a project not saved yet) a blank
    /// drawing opens and nothing is linked.
    pub(in crate::app) fn sipil_open_empty_sheet(&mut self, id: &str) -> Task<Message> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(task) = self.sipil_create_sheet_drawing(id) {
                return task;
            }
        }
        self.command_line.push_info(
            "This project is not saved, so the drawing opens blank and is not linked to it. \
             Use New Project in the desktop app to keep a project and its drawings on disk.",
        );
        Task::done(Message::TabNew)
    }

    /// Create the DWG of the slot `id` next to the project file. `None`: the
    /// project has no file (or the slot is not found), so there is nothing to
    /// create here. `Some`: it was handled, with an error shown if it failed.
    #[cfg(not(target_arch = "wasm32"))]
    fn sipil_create_sheet_drawing(&mut self, id: &str) -> Option<Task<Message>> {
        let set = self.sheet_set.current?;
        let db = self.sheet_set.sets.get(set)?;
        let dst = db.path.clone()?;
        let sheet = db.find(id)?;
        let title = sheet.prop("Title").unwrap_or("").trim().to_string();
        let number = sheet.prop("Number").unwrap_or("").trim().to_string();
        let parent = db.parent_of(id)?;
        let parent_id = parent.id().to_string();
        let category = parent.prop("Name").unwrap_or("").to_string();
        let in_category = ComponentKind::of(parent) == Some(ComponentKind::Subset);
        let project_folder = Path::new(&dst).parent()?.to_path_buf();
        // The category's own folder (set when the project was created), else
        // `<project>/<category>`, else the project folder.
        let folder = db
            .file_reference(&parent_id, "NewSheetLocation")
            .filter(|f| !f.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                if in_category {
                    project_folder.join(project_state::file_stem(&category))
                } else {
                    project_folder.clone()
                }
            });
        let label = format!("{number} {title}").trim().to_string();
        if label.is_empty() {
            return None;
        }
        let path = folder.join(format!("{}.dwg", project_state::file_stem(&label)));
        if path.exists() {
            self.command_line.push_error(&format!(
                "{} already exists. Use Add Drawing (DWG) to attach it.",
                path.display()
            ));
            return Some(Task::none());
        }
        if let Err(e) = std::fs::create_dir_all(&folder) {
            self.command_line
                .push_error(&format!("{}: {e}", folder.display()));
            return Some(Task::none());
        }
        // A blank drawing with one paper layout named after the slot, as New Sheet makes it.
        let mut scene = crate::scene::Scene::new();
        scene.populate_new_drawing_defaults();
        let source_layout = paper_layouts(&scene.document)
            .into_iter()
            .next()
            .map(|(n, _)| n)
            .unwrap_or_else(|| "Layout1".into());
        for (name, _) in paper_layouts(&scene.document) {
            if !name.eq_ignore_ascii_case(&source_layout) {
                scene.delete_layout(&name);
            }
        }
        scene.rename_layout(&source_layout, &label);
        let handle = paper_layouts(&scene.document)
            .into_iter()
            .find(|(n, _)| *n == label)
            .map(|(_, h)| h)
            .unwrap_or_default();
        let file = ss::native_path(&path.to_string_lossy());
        let revision = self.sheet_set.sets.get(set)?.file_revision() + 1;
        scene.document.set_sheet_set_data(&SheetSetData {
            layout_handle: handle.to_lowercase(),
            layout_name: label.clone(),
            sheet_dwg_name: file.clone(),
            sheet_set_file_name: ss::native_path(&dst),
            sheet_set_version: revision,
            update_count: 1,
            update_time: utc_stamp(),
        });
        crate::entities::field::stamp_save_dates(&mut scene.document);
        // The drawing is saved first: a failed save leaves the slot empty.
        if let Err(e) = crate::io::save(&scene.document, &path) {
            self.command_line.push_error(&format!("{file}: {e}"));
            return Some(Task::none());
        }
        let db = self.sheet_set.sets.get_mut(set)?;
        db.set_layout_reference(
            id,
            "Layout",
            &LayoutReference {
                file_name: file,
                name: label,
                handle,
            },
        );
        if let Err(e) = write_set(db, &dst) {
            self.command_line.push_error(&format!("{dst}: {e}"));
        }
        self.sheet_set.selected = Some(id.to_string());
        self.sheet_sets_changed();
        // The slot has a layout now: opening it is the ordinary path.
        Some(Task::done(Message::SheetSet(SheetSetMsg::Activate(
            id.to_string(),
        ))))
    }
}

/// Give a new project its place on disk: `<folder>/<name>/<name>.dst`, with each
/// category's New Drawing location set to its own subfolder.
#[cfg(not(target_arch = "wasm32"))]
fn save_new_project(
    database: &mut SheetSetDatabase,
    name: &str,
    folder: &str,
) -> Result<(), String> {
    let parent = folder.trim();
    if parent.is_empty() {
        return Err("Choose a location for the project.".to_string());
    }
    let stem = project_state::file_stem(name);
    let dir = Path::new(parent).join(&stem);
    let path = dir.join(format!("{stem}.dst"));
    if path.exists() {
        return Err(format!("{} already exists.", path.display()));
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let dst = path.to_string_lossy().to_string();
    database.path = Some(dst.clone());
    let set = database.sheet_set().id().to_string();
    database.set_file_reference(&set, "NewSheetLocation", &dir.to_string_lossy());
    let categories: Vec<(String, String)> = database
        .sheet_set()
        .children
        .iter()
        .filter(|c| ComponentKind::of(c) == Some(ComponentKind::Subset))
        .map(|c| (c.id().to_string(), c.prop("Name").unwrap_or("").to_string()))
        .collect();
    for (id, category) in categories {
        let sub = dir.join(project_state::file_stem(&category));
        database.set_file_reference(&id, "NewSheetLocation", &sub.to_string_lossy());
    }
    database.write(&dst)
}
