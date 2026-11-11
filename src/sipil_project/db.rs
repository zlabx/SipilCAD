//! Builds and keeps in step the sheet set of a project. The only file of the
//! pure part that touches `codec::sheet_set`; it uses only calls the host itself
//! makes (`new`, `add_subset`, `add_sheet`, `set_custom_property`), so
//! test/wallcheck can run it against the public acadrust crate.
//!
//! What a project keeps in its sheet set:
//! - on the sheet set, the custom property [`GRID_PROP`] (the grid as one line
//!   of text) and, once set, [`TITLE_PROP`] (the title block every new drawing
//!   gets);
//! - on every sheet that came from the grid, [`VIEW_PROP`] (which view it shows).

use codec::sheet_set::{
    custom_properties, set_custom_property, ComponentKind, Element, SheetSetDatabase,
};

use super::templates::{outline, Category};
use crate::sipil_arch::grid::model::GridDef;
use crate::sipil_arch::grid::view::ViewSpec;
use crate::sipil_arch::titleblock::model::TitleBlockDef;

/// Custom property of the sheet set: the grid (`GridDef::to_text`).
pub const GRID_PROP: &str = "SipilGrid";
/// Custom property of the sheet set: the title block of new drawings.
pub const TITLE_PROP: &str = "SipilTitleBlock";
/// Custom property of a sheet: the view it shows (`ViewSpec::format`).
pub const VIEW_PROP: &str = "SipilView";

fn prop(el: &Element, name: &str) -> Option<String> {
    custom_properties(el)
        .into_iter()
        .find(|(n, _, _)| n == name)
        .map(|(_, v, _)| v)
}

/// A sheet set named `name`; with a grid, its tree and the grid kept in it.
pub fn build_project(name: &str, grid: Option<&GridDef>) -> Result<SheetSetDatabase, String> {
    let mut db = SheetSetDatabase::new(name, "");
    if let Some(def) = grid {
        set_grid(&mut db, def)?;
        sync_tree(&mut db, def)?;
    }
    Ok(db)
}

/// Keep `def` in the project.
pub fn set_grid(db: &mut SheetSetDatabase, def: &GridDef) -> Result<(), String> {
    def.resolve()?;
    set_custom_property(db.sheet_set_mut(), GRID_PROP, &def.to_text(), 0);
    Ok(())
}

/// The grid of the project, if it has one.
pub fn grid_of(db: &SheetSetDatabase) -> Option<GridDef> {
    GridDef::from_text(&prop(db.sheet_set(), GRID_PROP)?)
}

/// Keep the title block of new drawings in the project.
pub fn set_title_block(db: &mut SheetSetDatabase, def: &TitleBlockDef) {
    set_custom_property(db.sheet_set_mut(), TITLE_PROP, &def.to_text(), 0);
}

/// The title block of new drawings, if the project has one.
pub fn title_block_of(db: &SheetSetDatabase) -> Option<TitleBlockDef> {
    TitleBlockDef::from_text(&prop(db.sheet_set(), TITLE_PROP)?)
}

/// The view a sheet shows, if it came from the grid.
pub fn view_of(db: &SheetSetDatabase, sheet: &str) -> Option<ViewSpec> {
    ViewSpec::parse(&prop(db.find(sheet)?, VIEW_PROP)?)
}

/// Add what the grid asks for and the project does not have yet: the categories
/// and the drawings (a drawing is known by its title in its category). Nothing
/// is removed or renamed. Returns how many drawings were added.
pub fn sync_tree(db: &mut SheetSetDatabase, def: &GridDef) -> Result<usize, String> {
    let grid = def.resolve()?;
    let set = db.sheet_set().id().to_string();
    let mut added = 0;
    for Category { name, drawings } in outline(&grid) {
        let existing = db
            .sheet_set()
            .children
            .iter()
            .find(|c| {
                ComponentKind::of(c) == Some(ComponentKind::Subset)
                    && c.prop("Name") == Some(name.as_str())
            })
            .map(|c| c.id().to_string());
        let sub = match existing {
            Some(id) => id,
            None => db
                .add_subset(&set, &name, "")
                .ok_or("The category could not be added.")?,
        };
        for d in drawings {
            let has = db.find(&sub).is_some_and(|s| {
                s.children.iter().any(|c| {
                    ComponentKind::of(c) == Some(ComponentKind::Sheet)
                        && c.prop("Title") == Some(d.title.as_str())
                })
            });
            if has {
                continue;
            }
            let id = db
                .add_sheet(&sub, "", &d.title, "")
                .ok_or("The drawing could not be added.")?;
            if let Some(el) = db.find_mut(&id) {
                set_custom_property(el, VIEW_PROP, &d.view.format(), 0);
            }
            added += 1;
        }
    }
    Ok(added)
}

/// The categories of a project and their drawing titles, in tree order.
pub fn titles(db: &SheetSetDatabase) -> Vec<(String, Vec<String>)> {
    db.sheet_set()
        .children
        .iter()
        .filter(|c| ComponentKind::of(c) == Some(ComponentKind::Subset))
        .map(|c| {
            let drawings = c
                .children
                .iter()
                .filter(|s| ComponentKind::of(s) == Some(ComponentKind::Sheet))
                .map(|s| s.prop("Title").unwrap_or("").to_string())
                .collect();
            (c.prop("Name").unwrap_or("").to_string(), drawings)
        })
        .collect()
}
