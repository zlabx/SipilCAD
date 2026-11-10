//! The column's row in the component registry.

use crate::sipil_arch::registry::ComponentInfo;

pub const INFO: ComponentInfo = ComponentInfo {
    id: "column",
    label: "Column",
    command: "COLUMN",
    icon: include_bytes!("../../../assets/icons/sipil/column.svg"),
    xdata_app: super::model::XDATA_APP,
    summary: "Steel column in plan: section from SipilGen's profile tables, rotation, fill, label",
};
