//! SipilCAD building components (the Architecture tab) and their registry.
//!
//! One folder per component. The pure part of a component (geometry, tags,
//! planner, adapter) has no iced or host types and is tested in the scratch
//! crate (`test/wallcheck`); the host part (`command`, `ui`) is checked by the
//! build. See `rebrand/opencadstudio/COMPONENTS.md` for how to add one.

pub mod beam;
pub mod column;
pub mod door;
pub mod host;
pub mod registry;
pub mod steel;

pub use host::ArchMsg;
