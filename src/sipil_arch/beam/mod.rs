//! The beam component: a steel beam in plan between two points, its section
//! from SipilGen's profile tables (`../steel`).

// Pure part (also compiled by the scratch crate, `test/wallcheck`).
pub mod adapter;
pub mod form;
pub mod info;
pub mod model;
pub mod plan;
pub mod planner;

// Host part (iced and app types): checked by the build.
pub mod command;
pub mod ui;

pub use info::INFO;
