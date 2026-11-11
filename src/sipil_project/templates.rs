//! New Project templates. Pure data and the outline a grid gives: no host and no
//! sheet set types, so it is tested alone (test/wallcheck).
//!
//! A project is built from a grid (see `sipil_arch::grid`): its levels, its grid
//! lines. The tree follows it:
//!
//! - **Architectural Plans** and **Structural Plans**: one plan per level;
//! - **Sections**: one along every grid line, `Section 1-1` (a numbered line)
//!   and `Section A-A` (a lettered one);
//! - **Elevations**: North, South, East and West;
//! - **Details**: empty, for the drawings the person adds.
//!
//! A blank (Custom) project has no grid and no tree. Drawings are empty slots:
//! the file is created the first time the slot is opened (see
//! `src/app/commands/sheet_set/sipil_project.rs`).

use crate::sipil_arch::grid::model::Grid;
use crate::sipil_arch::grid::spec::Axis;
use crate::sipil_arch::grid::templates::GridTemplate;
use crate::sipil_arch::grid::view::{PlanKind, Side, ViewSpec};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateId {
    Custom,
    House1,
    House2,
    Building4,
}

#[derive(Debug)]
pub struct Template {
    pub id: TemplateId,
    pub label: &'static str,
    pub blurb: &'static str,
    /// The grid the project starts with; `None` for a blank project.
    pub grid: Option<GridTemplate>,
}

/// The project the app starts with ("Project 1").
pub const DEFAULT_TEMPLATE: TemplateId = TemplateId::House2;

pub static TEMPLATES: [Template; 4] = [
    Template {
        id: TemplateId::Custom,
        label: "Custom (blank)",
        blurb: "An empty project. Add categories and drawings with right-click.",
        grid: None,
    },
    Template {
        id: TemplateId::House1,
        label: "1-Story House",
        blurb: "A grid of 3 x 4 lines and one level: plans, sections along every grid line, four elevations.",
        grid: Some(GridTemplate {
            key: "house1",
            title: "1-Story House",
            x: ("0.00 2*3000.00", "A B C"),
            y: ("0.00 3*3000.00", "1 2 3 4"),
            z: ("0.00", "Level 1"),
        }),
    },
    Template {
        id: TemplateId::House2,
        label: "2-Story House",
        blurb: "The same grid with a second level: a plan of each level, in architecture and in structure.",
        grid: Some(GridTemplate {
            key: "house2",
            title: "2-Story House",
            x: ("0.00 2*3000.00", "A B C"),
            y: ("0.00 3*3000.00", "1 2 3 4"),
            z: ("0.00 3000.00", "Level 1, Level 2"),
        }),
    },
    Template {
        id: TemplateId::Building4,
        label: "4-Story Building",
        blurb: "A grid of 6 m bays (A-D by 1-3) and four levels of 3.5 m.",
        grid: Some(GridTemplate {
            key: "building4",
            title: "4-Story Building",
            x: ("0.00 3*6000.00", "A B C D"),
            y: ("0.00 2*6000.00", "1 2 3"),
            z: ("0.00 3*3500.00", "Level 1, Level 2, Level 3, Level 4"),
        }),
    },
];

/// The template for `id` (the blank one if it is somehow missing).
pub fn template(id: TemplateId) -> &'static Template {
    TEMPLATES
        .iter()
        .find(|t| t.id == id)
        .unwrap_or(&TEMPLATES[0])
}

/// One drawing slot of a project.
#[derive(Debug, Clone, PartialEq)]
pub struct Drawing {
    pub title: String,
    pub view: ViewSpec,
}

/// One category of a project and its drawings.
#[derive(Debug, Clone, PartialEq)]
pub struct Category {
    pub name: String,
    pub drawings: Vec<Drawing>,
}

pub const ARCHITECTURAL_PLANS: &str = "Architectural Plans";
pub const STRUCTURAL_PLANS: &str = "Structural Plans";
pub const SECTIONS: &str = "Sections";
pub const ELEVATIONS: &str = "Elevations";
pub const DETAILS: &str = "Details";

/// The tree a grid gives a project.
pub fn outline(g: &Grid) -> Vec<Category> {
    let levels = g.levels();
    let plans = |kind: PlanKind| -> Vec<Drawing> {
        levels
            .iter()
            .enumerate()
            .map(|(i, l)| Drawing {
                title: l.id.clone(),
                view: ViewSpec::Plan { kind, level: i },
            })
            .collect()
    };
    // Numbered lines (Y) first, then lettered ones (X): Section 1-1 ... Section A-A.
    let mut sections: Vec<Drawing> = Vec::new();
    for (axis, rows) in [(Axis::Y, &g.y), (Axis::X, &g.x)] {
        for r in rows.iter() {
            sections.push(Drawing {
                title: format!("Section {0}-{0}", r.id),
                view: ViewSpec::Section {
                    along: axis,
                    label: r.id.clone(),
                },
            });
        }
    }
    let elevations = Side::ALL
        .iter()
        .map(|s| Drawing {
            title: format!("{} Elevation", s.name()),
            view: ViewSpec::Elevation(*s),
        })
        .collect();
    vec![
        Category {
            name: ARCHITECTURAL_PLANS.into(),
            drawings: plans(PlanKind::Architectural),
        },
        Category {
            name: STRUCTURAL_PLANS.into(),
            drawings: plans(PlanKind::Structural),
        },
        Category {
            name: SECTIONS.into(),
            drawings: sections,
        },
        Category {
            name: ELEVATIONS.into(),
            drawings: elevations,
        },
        Category {
            name: DETAILS.into(),
            drawings: Vec::new(),
        },
    ]
}

/// Number of drawings in an outline.
pub fn drawing_count(o: &[Category]) -> usize {
    o.iter().map(|c| c.drawings.len()).sum()
}
