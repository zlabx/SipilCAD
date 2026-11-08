//! SipilCAD building components. First component: brick walls (`WALL`).
//!
//! Added by the SipilStock rebrand kit (`rebrand/opencadstudio/overlay`), not
//! by upstream. Everything that needs the host's private state lives in
//! `src/app/sipil_wall.rs`; this directory is the part with no such need.
//!
//! - `engine`   pure geometry (port of `dinding_bata.lsp`), unit-tested alone
//! - `entities` primitives <-> drawing entities and the XDATA tag
//! - `state`    settings and form fields, unit-tested alone
//! - `command`  the `WALL` draw command with live preview
//! - `ui`       the parameter popup (the tool's one dialog)
//! - `module`   the Architecture ribbon tab

pub mod command;
pub mod engine;
pub mod entities;
pub mod module;
pub mod state;
pub mod ui;

inventory::submit! {
    crate::command::CommandRegistration {
        names: &["WALL", "BW", "WALLPOPUP"],
    }
}
