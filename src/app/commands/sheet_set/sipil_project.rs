//! SipilCAD projects: the part of the project browser that needs the app's
//! private state.
//!
//! Added by the SipilStock rebrand kit. It is a child of the sheet set command
//! module (patch 0019) so it can use that module's private helpers
//! (`paper_layouts`, `utc_stamp`, `write_set`, `sheet_sets_changed`,
//! `save_current_sheet_set`); everything else lives in `crate::sipil_project`
//! and `crate::sipil_arch`. Hooks into upstream are one or two lines.
//!
//! A project is a sheet set: categories are subsets, drawings are sheets. A
//! drawing from a template starts as an empty slot (a sheet without a layout
//! reference) that remembers the view it shows (`SipilView`). Opening it makes
//! the drawing: an A3 landscape sheet with the title block (etiket), the grid
//! drawn in model space for that view (a plan of the level, a section along a
//! grid line, an elevation) and a viewport at a scale that fits.

use crate::app::{Message, ModalKind, OpenCADStudio};
use crate::scene::Scene;
use crate::sipil_arch::grid::model::{GridDef, XDATA_APP as GRID_APP};
use crate::sipil_arch::grid::view::ViewSpec;
use crate::sipil_arch::grid::{adapter as grid_adapter, plan as grid_plan};
use crate::sipil_arch::titleblock::model::{
    TitleBlockDef, LAYER as TB_LAYER, LAYER_ACI as TB_ACI, XDATA_APP as TB_APP,
};
use crate::sipil_arch::titleblock::{
    adapter as tb_adapter, geometry as tb_geometry, sheet as tb_sheet,
};
use crate::sipil_project::db as project_db;
use crate::sipil_project::state as project_state;
use crate::sipil_project::templates::DEFAULT_TEMPLATE;
use crate::sipil_project::ui::Msg;
use crate::sipil_wall::engine::units_per_metre;
use crate::ui::window::sheet_set::SsmTab;
use iced::Task;

use super::paper_layouts;
#[cfg(not(target_arch = "wasm32"))]
use super::{utc_stamp, write_set};
#[cfg(not(target_arch = "wasm32"))]
use crate::ui::window::sheet_set::SheetSetMsg;
#[cfg(not(target_arch = "wasm32"))]
use codec::sheet_set::{
    self as ss, ComponentKind, LayoutReference, SheetSetData, SheetSetDatabase,
};
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

/// What a new drawing of a project is made of.
pub(in crate::app) struct SheetSpec {
    /// The view the drawing shows, if it came from the grid.
    view: Option<ViewSpec>,
    grid: Option<GridDef>,
    /// The project's title block, before this sheet's own texts go in.
    title_block: TitleBlockDef,
    /// The text of JUDUL GAMBAR.
    title: String,
    /// 1-based place among the project's drawings, and how many there are.
    index: usize,
    total: usize,
}

fn now_unix() -> i64 {
    iced::time::SystemTime::now()
        .duration_since(iced::time::SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Make the layers exist (a layer already there keeps its own look).
fn ensure_layers(scene: &mut Scene, layers: &[(&str, i16)]) {
    for (name, aci) in layers {
        if scene.document.layers.contains(name) {
            continue;
        }
        scene.ensure_layer(name);
        if let Some(layer) = scene.document.layers.get_mut(name) {
            layer.color = codec::types::Color::from_index(*aci);
        }
    }
}

/// Make the sheet of `spec` in `scene`: the grid in model space, then on the
/// paper layout `layout` (if there is one) the paper, the title block and a
/// viewport on the grid. Returns the scale (1:n) the sheet was made at.
fn build_sheet(scene: &mut Scene, spec: &SheetSpec, layout: Option<&str>) -> Result<u32, String> {
    let upm = units_per_metre(scene.document.header.insertion_units);
    ensure_layers(scene, &grid_plan::LAYERS);
    ensure_layers(scene, &[(TB_LAYER, TB_ACI)]);
    let area = tb_geometry::drawing_area(&spec.title_block)?;

    // The grid, at the scale that fits the room the title block leaves.
    let mut scale = 100;
    let mut centre = [0.0, 0.0];
    if let (Some(def), Some(view)) = (&spec.grid, &spec.view) {
        let grid = def.resolve()?;
        scale = grid_plan::pick_scale(&grid, view, area.width() * 0.92, area.height() * 0.92);
        let prims = grid_plan::build(&grid, view, upm, f64::from(scale));
        if let Some((lo, hi)) = grid_plan::extents(&prims) {
            centre = [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0];
        }
        if scene.current_layout != "Model" {
            scene.set_current_layout("Model".to_string());
        }
        scene.ensure_app_id(GRID_APP);
        for e in grid_adapter::to_entities(&prims, &view.format(), f64::from(scale), def) {
            scene.add_entity(e);
        }
    }

    // The sheet.
    if let Some(layout) = layout {
        let tb = spec
            .title_block
            .for_sheet(&spec.title, scale, spec.index, spec.total, now_unix());
        let prims = tb_geometry::build(&tb)?;
        tb_sheet::set_paper(&mut scene.document, layout, &tb);
        scene.ensure_app_id(TB_APP);
        for e in tb_adapter::to_entities(&prims, &tb) {
            let _ = scene.document.add_entity_to_layout(e, layout);
        }
        // The overall sheet viewport first, then the one on the drawing.
        scene.ensure_sheet_viewport(layout);
        let factor = (1000.0 / upm) / f64::from(scale); // paper mm per model unit
        let vp = tb_sheet::content_viewport(area, centre, factor);
        let _ = scene
            .document
            .add_entity_to_layout(codec::EntityType::Viewport(Box::new(vp)), layout);
    }
    scene.bump_geometry();
    Ok(scale)
}

impl OpenCADStudio {
    /// At startup: "Project 1" (the default template) is open and the Sheet Set
    /// Manager is showing, so the project browser is there from the first frame.
    /// Skipped in unit tests, which expect a bare app.
    pub(in crate::app) fn sipil_project_startup(&mut self) {
        if cfg!(test) || !self.sheet_set.sets.is_empty() {
            return;
        }
        let name = project_state::next_project_name(&[]);
        let def = project_state::template_fields(DEFAULT_TEMPLATE)
            .to_def()
            .ok();
        let Ok(db) = project_db::build_project(&name, def.as_ref()) else {
            return;
        };
        self.sheet_set.sets.push(db);
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
                    f.set_template(t);
                    f.error = None;
                }
                Task::none()
            }
            Msg::Grid(which, v) => {
                if let Some(f) = self.sipil_project.form.as_mut() {
                    f.set_grid_field(which, v);
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
        let grid = form.grid_def()?;
        #[allow(unused_mut)]
        let mut database = project_db::build_project(&name, grid.as_ref())?;
        #[cfg(not(target_arch = "wasm32"))]
        save_new_project(&mut database, &name, &form.folder)?;
        self.sheet_set.sets.push(database);
        self.sheet_set.current = Some(self.sheet_set.sets.len() - 1);
        self.sheet_set.tab = SsmTab::SheetList;
        self.show_sheet_set_manager(true);
        self.sheet_sets_changed();
        Ok(())
    }

    // ── what the components ask of the project ───────────────────────────

    fn sipil_current_project(&self) -> Option<&codec::sheet_set::SheetSetDatabase> {
        self.sheet_set.sets.get(self.sheet_set.current?)
    }

    /// A project is open.
    pub(in crate::app) fn sipil_has_project(&self) -> bool {
        self.sipil_current_project().is_some()
    }

    /// The grid of the open project, if it has one.
    pub(in crate::app) fn sipil_project_grid(&self) -> Option<GridDef> {
        project_db::grid_of(self.sipil_current_project()?)
    }

    /// The title block new drawings of the open project get, if it keeps one.
    pub(in crate::app) fn sipil_project_title(&self) -> Option<TitleBlockDef> {
        project_db::title_block_of(self.sipil_current_project()?)
    }

    /// Keep `def` as the grid of the open project and add the plans, sections
    /// and elevations it asks for. Returns how many drawings were added.
    pub(in crate::app) fn sipil_project_set_grid(
        &mut self,
        def: &GridDef,
    ) -> Result<usize, String> {
        let at = self.sheet_set.current.ok_or("No project is open.")?;
        let db = self
            .sheet_set
            .sets
            .get_mut(at)
            .ok_or("No project is open.")?;
        project_db::set_grid(db, def)?;
        let added = project_db::sync_tree(db, def)?;
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(dir) = db
            .path
            .clone()
            .and_then(|p| Path::new(&p).parent().map(Path::to_path_buf))
        {
            set_category_folders(db, &dir);
        }
        self.save_current_sheet_set();
        self.sheet_sets_changed();
        Ok(added)
    }

    /// Keep `def` as the title block new drawings of the open project get.
    pub(in crate::app) fn sipil_project_set_title(&mut self, def: &TitleBlockDef) {
        let Some(db) = self
            .sheet_set
            .current
            .and_then(|at| self.sheet_set.sets.get_mut(at))
        else {
            return;
        };
        project_db::set_title_block(db, def);
        self.save_current_sheet_set();
    }

    /// Give the paper layout `layout` of tab `tab` the paper of `tb`.
    pub(in crate::app) fn sipil_set_paper(
        &mut self,
        tab: usize,
        layout: &str,
        tb: &TitleBlockDef,
    ) -> bool {
        match self.tabs.get_mut(tab) {
            Some(t) => tb_sheet::set_paper(&mut t.scene.document, layout, tb),
            None => false,
        }
    }

    // ── opening a drawing slot ───────────────────────────────────────────

    /// What the drawing slot `id` of the open project is made of, `None` when
    /// it is not a sheet of the open project.
    fn sipil_sheet_spec(&self, id: &str) -> Option<SheetSpec> {
        let db = self.sipil_current_project()?;
        let sheet = db.find(id)?;
        let title = sheet.prop("Title").unwrap_or("").trim().to_string();
        let number = sheet.prop("Number").unwrap_or("").trim().to_string();
        let label = format!("{number} {title}").trim().to_string();
        let parent = db.parent_of(id)?;
        let category = parent.prop("Name").unwrap_or("").trim().to_string();
        let in_category = codec::sheet_set::ComponentKind::of(parent)
            == Some(codec::sheet_set::ComponentKind::Subset);
        let sheets = db.sheets();
        let index = sheets
            .iter()
            .position(|s| s.id() == id)
            .map_or(1, |p| p + 1);
        Some(SheetSpec {
            view: project_db::view_of(db, id),
            grid: project_db::grid_of(db),
            title_block: project_db::title_block_of(db).unwrap_or_default(),
            title: if in_category && !category.is_empty() {
                format!("{category} - {label}")
            } else {
                label
            },
            index,
            total: sheets.len(),
        })
    }

    /// A drawing slot (a sheet with no layout yet) was opened. In a saved project
    /// the drawing is created as `<project>/<category>/<title>.dwg`, linked to the
    /// slot and opened. Otherwise (a browser, or a project not saved yet) a new
    /// tab opens with the same sheet in it, not linked to anything.
    pub(in crate::app) fn sipil_open_empty_sheet(&mut self, id: &str) -> Task<Message> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(task) = self.sipil_create_sheet_drawing(id) {
                return task;
            }
        }
        let spec = self.sipil_sheet_spec(id);
        let task = self.update(Message::TabNew);
        let tab = self.active_tab;
        if let (Some(spec), Some(t)) = (spec, self.tabs.get_mut(tab)) {
            let layout = paper_layouts(&t.scene.document)
                .into_iter()
                .next()
                .map(|(n, _)| n);
            match build_sheet(&mut t.scene, &spec, layout.as_deref()) {
                Ok(_) => t.dirty = true,
                Err(e) => self
                    .command_line
                    .push_error(&format!("The sheet could not be made: {e}")),
            }
            self.refresh_layer_panel();
        }
        self.command_line.push_info(
            "This project is not saved, so the drawing opens in a new tab and is not linked to it. \
             Use New Project in the desktop app to keep a project and its drawings on disk.",
        );
        task
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
        let spec = self.sipil_sheet_spec(id)?;
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
        let mut scene = Scene::new();
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
        // The grid, A3 landscape and the title block. If that fails the drawing is
        // still made, blank, and the reason is shown.
        if let Err(e) = build_sheet(&mut scene, &spec, Some(&label)) {
            self.command_line
                .push_error(&format!("The sheet of \"{label}\" is blank: {e}"));
        }
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

/// Each category's New Drawing location is its own subfolder of `dir`.
#[cfg(not(target_arch = "wasm32"))]
fn set_category_folders(database: &mut SheetSetDatabase, dir: &Path) {
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
    set_category_folders(database, &dir);
    database.write(&dst)
}
