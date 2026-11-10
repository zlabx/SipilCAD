//! The door component: type catalog, plan symbol, opening in the wall,
//! detail sheet per type, and its command and popup.

// Pure part (also compiled by the scratch crate, `test/wallcheck`).
pub mod adapter;
pub mod catalog;
pub mod form;
pub mod info;
pub mod model;
pub mod plan;
pub mod planner;
pub mod sheet;

// Host part (iced and app types): checked by the build.
pub mod command;
pub mod ui;

pub use info::INFO;
