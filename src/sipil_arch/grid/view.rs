//! Which view of the building a drawing shows: a plan of a level, a section
//! along a grid line, or an elevation. Written on the drawing's sheet in the
//! project and on its grid entities, so the grid knows how to draw itself.
//! Pure: `std` only.

use super::spec::Axis;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlanKind {
    Architectural,
    Structural,
}

/// The side of the building an elevation looks at from. North is +Y, east +X.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    North,
    South,
    East,
    West,
}

impl Side {
    pub const ALL: [Side; 4] = [Side::North, Side::South, Side::East, Side::West];

    pub fn name(self) -> &'static str {
        match self {
            Side::North => "North",
            Side::South => "South",
            Side::East => "East",
            Side::West => "West",
        }
    }

    fn code(self) -> &'static str {
        match self {
            Side::North => "north",
            Side::South => "south",
            Side::East => "east",
            Side::West => "west",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewSpec {
    /// A plan of the level at this index (in the grid's levels).
    Plan {
        kind: PlanKind,
        level: usize,
    },
    /// A section cut along a grid line: `along` X is a line of the letters
    /// (`A-A`), `along` Y one of the numbers (`1-1`); `label` is its label.
    Section {
        along: Axis,
        label: String,
    },
    Elevation(Side),
}

impl ViewSpec {
    /// The text kept in the sheet and on the entities, e.g. `plan:arch:0`,
    /// `section:x:A`, `elev:north`.
    pub fn format(&self) -> String {
        match self {
            ViewSpec::Plan { kind, level } => {
                let k = match kind {
                    PlanKind::Architectural => "arch",
                    PlanKind::Structural => "struct",
                };
                format!("plan:{k}:{level}")
            }
            ViewSpec::Section { along, label } => {
                let a = if *along == Axis::Y { "y" } else { "x" };
                format!("section:{a}:{label}")
            }
            ViewSpec::Elevation(side) => format!("elev:{}", side.code()),
        }
    }

    pub fn parse(text: &str) -> Option<ViewSpec> {
        let mut parts = text.splitn(3, ':');
        match (parts.next()?, parts.next(), parts.next()) {
            ("plan", Some(k), Some(level)) => {
                let kind = match k {
                    "arch" => PlanKind::Architectural,
                    "struct" => PlanKind::Structural,
                    _ => return None,
                };
                Some(ViewSpec::Plan {
                    kind,
                    level: level.parse().ok()?,
                })
            }
            ("section", Some(a), Some(label)) if !label.is_empty() => {
                let along = match a {
                    "x" => Axis::X,
                    "y" => Axis::Y,
                    _ => return None,
                };
                Some(ViewSpec::Section {
                    along,
                    label: label.to_string(),
                })
            }
            ("elev", Some(side), None) => Side::ALL
                .into_iter()
                .find(|s| s.code() == side)
                .map(ViewSpec::Elevation),
            _ => None,
        }
    }

    /// A plan view (what the `GRID` command draws in an ordinary drawing).
    pub fn plan() -> ViewSpec {
        ViewSpec::Plan {
            kind: PlanKind::Architectural,
            level: 0,
        }
    }

    pub fn is_plan(&self) -> bool {
        matches!(self, ViewSpec::Plan { .. })
    }
}
