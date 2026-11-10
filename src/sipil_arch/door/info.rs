//! The door's row in the component registry.

use crate::sipil_arch::registry::ComponentInfo;

pub const INFO: ComponentInfo = ComponentInfo {
    id: "door",
    label: "Door",
    command: "DOOR",
    icon: include_bytes!("../../../assets/icons/sipil/door.svg"),
    xdata_app: super::model::XDATA_APP,
    summary: "Door in a wall: type, open/closed in plan, opening in the wall, detail sheet per type",
};
