//! The title block's row in the component registry.

use crate::sipil_arch::registry::ComponentInfo;

pub const INFO: ComponentInfo = ComponentInfo {
    id: "titleblock",
    label: "Title Block",
    command: "TITLEBLOCK",
    icon: include_bytes!("../../../assets/icons/sipil/titleblock.svg"),
    xdata_app: super::model::XDATA_APP,
    summary: "Sheet frame and title block (etiket): paper, margins, template and the texts in it",
};
