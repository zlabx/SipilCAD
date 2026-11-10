//! The beam's row in the component registry.

use crate::sipil_arch::registry::ComponentInfo;

pub const INFO: ComponentInfo = ComponentInfo {
    id: "beam",
    label: "Beam",
    command: "BEAM",
    icon: include_bytes!("../../../assets/icons/sipil/beam.svg"),
    xdata_app: super::model::XDATA_APP,
    summary: "Steel beam in plan between two points: profile from SipilGen's tables, outline or centreline, label",
};
