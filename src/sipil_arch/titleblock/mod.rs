//! The title block component ("etiket"): the frame and block drawn in a
//! drawing's paper space, A3 landscape by default, with a popup for the paper,
//! margins, template and texts.

// Pure part (also compiled by the scratch crate, `test/wallcheck`).
pub mod adapter;
pub mod form;
pub mod geometry;
pub mod info;
pub mod model;
pub mod sheet;

// Host part (iced and app types): checked by the build.
pub mod ui;

pub use info::INFO;
