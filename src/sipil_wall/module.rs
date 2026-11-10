//! The "Architecture" ribbon tab: the brick wall, then every component of the
//! registry (`sipil_arch::registry`), one large button each.
//!
//! The leftmost group holds the "popup on click" toggle (highlighted while on,
//! like View Cube or UCS Icon). Wall opens the one parameter popup and draws;
//! the toggle decides whether a click on a wall (or a door) already in the
//! drawing opens its popup.

use crate::modules::{CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, ToolDef};
use crate::sipil_arch::registry;

pub struct ArchitectureModule;

/// Command of the "popup on click" toggle. The ribbon highlights the button
/// from its own state (`sipil_wall_popup`), see patch 0018.
pub const POPUP_TOGGLE: &str = "WALLPOPUP";

fn tool(id: &'static str, label: &'static str, icon: &'static [u8]) -> ToolDef {
    ToolDef {
        id,
        label,
        icon: IconKind::Svg(icon),
        event: ModuleEvent::Command(id.to_string()),
    }
}

impl CadModule for ArchitectureModule {
    fn id(&self) -> &'static str {
        "architecture"
    }

    fn title(&self) -> &'static str {
        "Architecture"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        static GROUPS: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        GROUPS.get_or_init(|| {
            vec![
                RibbonGroup {
                    title: "Options",
                    tools: vec![RibbonItem::LargeTool(tool(
                        POPUP_TOGGLE,
                        "Click Popup",
                        include_bytes!("../../assets/icons/sipil/wall_settings.svg"),
                    ))],
                },
                RibbonGroup {
                    title: "Components",
                    tools: std::iter::once(RibbonItem::LargeTool(tool(
                        "WALL",
                        "Wall",
                        include_bytes!("../../assets/icons/sipil/wall.svg"),
                    )))
                    .chain(
                        registry::COMPONENTS
                            .iter()
                            .map(|c| RibbonItem::LargeTool(tool(c.command, c.label, c.icon))),
                    )
                    .collect(),
                },
            ]
        })
    }
}
