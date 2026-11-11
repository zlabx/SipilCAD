//! The grid component: lines labelled A B C / 1 2 3, bubbles, levels. A project
//! is built from it: plans per level, sections along grid lines, elevations.
//! Port of the grid of the SipilBIM plugin (zlabx/FT-CES), for 2-D drawings.

// Pure part (also compiled by the scratch crate, `test/wallcheck`).
pub mod adapter;
pub mod bubble;
pub mod form;
pub mod info;
pub mod model;
pub mod plan;
pub mod spec;
pub mod templates;
pub mod view;

// Host part (iced and app types): checked by the build.
pub mod ui;

pub use info::INFO;
