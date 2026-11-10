//! Steel profiles shared by the columns and the beams (SipilGen's tables).
//!
//! A library, not a component: it has no ribbon button or command of its own.
//! `data.rs` is generated, see `scripts/gen_steel_profiles.py`.

// Pure part (also compiled by the scratch crate, `test/wallcheck`).
pub mod catalog;
pub mod data;
pub mod draw;
pub mod pick;
pub mod shape;

// Host part (iced): checked by the build.
pub mod ui;
