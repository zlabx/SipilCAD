//! The grid's row in the component registry.

use crate::sipil_arch::registry::ComponentInfo;

pub const INFO: ComponentInfo = ComponentInfo {
    id: "grid",
    label: "Grid",
    command: "GRID",
    icon: include_bytes!("../../../assets/icons/sipil/grid.svg"),
    xdata_app: super::model::XDATA_APP,
    summary: "Structural grid with bubbles and levels: plan, elevations and sections follow it",
};
