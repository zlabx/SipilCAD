//! New Project templates. Pure data: no host and no sheet set types, so it is
//! tested alone (test/projectcheck).
//!
//! A template is a list of categories (sheet set subsets), each with the drawings
//! (sheets) it starts with. Drawings are empty slots: the file is created the
//! first time the slot is opened (see `src/app/commands/sheet_set/sipil_project.rs`).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateId {
    Custom,
    House1,
    House2,
}

#[derive(Debug)]
pub struct Category {
    pub name: &'static str,
    pub drawings: &'static [&'static str],
}

#[derive(Debug)]
pub struct Template {
    pub id: TemplateId,
    pub label: &'static str,
    pub blurb: &'static str,
    pub categories: &'static [Category],
}

/// The project the app starts with ("Project 1").
pub const DEFAULT_TEMPLATE: TemplateId = TemplateId::House2;

const ELEVATIONS: Category = Category {
    name: "Elevations",
    drawings: &[
        "North Elevation",
        "South Elevation",
        "East Elevation",
        "West Elevation",
    ],
};

const SECTIONS: Category = Category {
    name: "Sections",
    drawings: &["Section A-A", "Section B-B"],
};

/// Starts empty: the person adds what the project needs.
const DETAILS: Category = Category {
    name: "Details",
    drawings: &[],
};

const FLOOR_PLANS_1: Category = Category {
    name: "Floor Plans",
    drawings: &["Site Plan", "Level 1", "Roof Plan"],
};

const CEILING_PLANS_1: Category = Category {
    name: "Ceiling Plans",
    drawings: &["Level 1"],
};

const STRUCTURAL_1: Category = Category {
    name: "Structural Plans",
    drawings: &["Foundation Plan", "Level 1 Framing", "Roof Framing"],
};

const FLOOR_PLANS_2: Category = Category {
    name: "Floor Plans",
    drawings: &["Site Plan", "Level 1", "Level 2", "Roof Plan"],
};

const CEILING_PLANS_2: Category = Category {
    name: "Ceiling Plans",
    drawings: &["Level 1", "Level 2"],
};

const STRUCTURAL_2: Category = Category {
    name: "Structural Plans",
    drawings: &[
        "Foundation Plan",
        "Level 1 Framing",
        "Level 2 Framing",
        "Roof Framing",
    ],
};

pub static TEMPLATES: [Template; 3] = [
    Template {
        id: TemplateId::Custom,
        label: "Custom (blank)",
        blurb: "An empty project. Add categories and drawings with right-click.",
        categories: &[],
    },
    Template {
        id: TemplateId::House1,
        label: "1-Story House",
        blurb: "Floor, ceiling and structural plans, elevations, sections and a Details category.",
        categories: &[
            FLOOR_PLANS_1,
            CEILING_PLANS_1,
            STRUCTURAL_1,
            ELEVATIONS,
            SECTIONS,
            DETAILS,
        ],
    },
    Template {
        id: TemplateId::House2,
        label: "2-Story House",
        blurb: "Same as the 1-story house, with a second level in every plan category.",
        categories: &[
            FLOOR_PLANS_2,
            CEILING_PLANS_2,
            STRUCTURAL_2,
            ELEVATIONS,
            SECTIONS,
            DETAILS,
        ],
    },
];

/// The template for `id` (the blank one if it is somehow missing).
pub fn template(id: TemplateId) -> &'static Template {
    TEMPLATES
        .iter()
        .find(|t| t.id == id)
        .unwrap_or(&TEMPLATES[0])
}

/// Number of drawings a template starts with.
pub fn drawing_count(t: &Template) -> usize {
    t.categories.iter().map(|c| c.drawings.len()).sum()
}
