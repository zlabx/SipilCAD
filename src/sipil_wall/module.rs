//! The "SipilCAD" ribbon tab: wall tools and the building-components pallet.

use crate::modules::{CadModule, IconKind, ModuleEvent, RibbonGroup, RibbonItem, ToolDef};

pub struct SipilModule;

fn tool(id: &'static str, label: &'static str, icon: &'static [u8]) -> ToolDef {
    ToolDef {
        id,
        label,
        icon: IconKind::Svg(icon),
        event: ModuleEvent::Command(id.to_string()),
    }
}

impl CadModule for SipilModule {
    fn id(&self) -> &'static str {
        "sipil"
    }

    fn title(&self) -> &'static str {
        "SipilCAD"
    }

    fn ribbon_groups(&self) -> &[RibbonGroup] {
        static GROUPS: std::sync::OnceLock<Vec<RibbonGroup>> = std::sync::OnceLock::new();
        GROUPS.get_or_init(|| {
            vec![RibbonGroup {
                title: "Dinding",
                tools: vec![
                    RibbonItem::LargeTool(tool(
                        "BW",
                        "Dinding",
                        include_bytes!("../../assets/icons/sipil/wall.svg"),
                    )),
                    tool(
                        "BWSET",
                        "Pengaturan",
                        include_bytes!("../../assets/icons/sipil/wall_settings.svg"),
                    )
                    .into(),
                    tool(
                        "BWPANEL",
                        "Komponen",
                        include_bytes!("../../assets/icons/sipil/panel.svg"),
                    )
                    .into(),
                ],
            }]
        })
    }
}
