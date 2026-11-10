//! Builds the sheet set of a new project from a template. The only file of the
//! pure part that touches `codec::sheet_set`; it uses only calls the host itself
//! makes (`new`, `add_subset`, `add_sheet`), so test/projectcheck can run it
//! against the public acadrust crate.

use codec::sheet_set::{ComponentKind, SheetSetDatabase};

use super::templates::{template, TemplateId};

/// A sheet set named `name` with the template's categories and drawings.
/// Drawings carry no sheet number, so the tree shows just their titles.
pub fn build_project(name: &str, id: TemplateId) -> SheetSetDatabase {
    let mut db = SheetSetDatabase::new(name, "");
    let set = db.sheet_set().id().to_string();
    for category in template(id).categories {
        let Some(sub) = db.add_subset(&set, category.name, "") else {
            continue;
        };
        for drawing in category.drawings {
            db.add_sheet(&sub, "", drawing, "");
        }
    }
    db
}

/// The categories of a project and their drawing titles, in tree order.
pub fn outline(db: &SheetSetDatabase) -> Vec<(String, Vec<String>)> {
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
