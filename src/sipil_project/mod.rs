//! SipilCAD projects: the Sheet Set Manager as a Revit-style project browser.
//!
//! Added by the SipilStock rebrand kit (`rebrand/opencadstudio/overlay`), not by
//! upstream. A project IS a sheet set: its categories are subsets and its drawings
//! are sheets, so everything upstream already does with a sheet set (right-click
//! menus, Rename, Properties, the `.dst` file, Model Views) keeps working.
//!
//! - `templates` the New Project templates (Custom, 1-Story House, 2-Story House)
//! - `state`     the New Project form, naming and file-name rules, unit-tested alone
//! - `db`        builds a sheet set from a template (against `codec::sheet_set`)
//! - `ui`        the panel's New/Open buttons and the New Project popup
//!
//! The part that needs the app's private state lives in
//! `src/app/commands/sheet_set/sipil_project.rs`.

pub mod db;
pub mod state;
pub mod templates;
pub mod ui;
