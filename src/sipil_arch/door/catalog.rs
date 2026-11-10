//! The door types the popup offers, and the text of their schedule rows.
//!
//! Data only. To add a type, add a row to [`TYPES`] (and its thumbnail under
//! `assets/icons/sipil/door/`); nothing else changes. The schedule text is
//! Indonesian on purpose: it is what goes on the drawing.

/// How the leaf looks in the elevation of the detail sheet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Elevation {
    /// Two raised panels (wooden door).
    Panel,
    /// Glass above, louvre at the bottom.
    GlassLouvre,
    /// Glass over the whole leaf.
    Glass,
}

/// One row of the material table under the detail.
#[derive(Clone, Copy, Debug)]
pub struct SpecRow {
    pub label: &'static str,
    pub text: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct DoorType {
    /// Type mark on the drawing (`P1`).
    pub code: &'static str,
    /// Name in the popup.
    pub name: &'static str,
    /// 1 or 2 leaves.
    pub leaves: u8,
    /// Default clear width between the frames, metres.
    pub width_m: f64,
    /// Default clear height, metres.
    pub height_m: f64,
    /// Frame (kusen) width on each side and above, metres.
    pub frame_m: f64,
    /// Leaf thickness, metres.
    pub leaf_m: f64,
    pub elevation: Elevation,
    /// Material rows: KUSEN, PINTU, JENDELA, BOUVENLICHT, ACCESSORIES.
    pub spec: [SpecRow; 5],
    /// Thumbnail shown in the popup (SVG, two-colour icon style).
    pub thumb: &'static [u8],
}

const DASH: &str = "-";

pub const TYPES: [DoorType; 3] = [
    DoorType {
        code: "P1",
        name: "Wooden panel door",
        leaves: 1,
        width_m: 0.80,
        height_m: 2.10,
        frame_m: 0.05,
        leaf_m: 0.04,
        elevation: Elevation::Panel,
        spec: [
            SpecRow { label: "KUSEN", text: "KAYU KAMPER 5/12 FINISHING CAT KAYU" },
            SpecRow { label: "PINTU", text: "PANEL KAYU KAMPER TEBAL 4 cm" },
            SpecRow { label: "JENDELA", text: DASH },
            SpecRow { label: "BOUVENLICHT", text: DASH },
            SpecRow { label: "ACCESSORIES", text: "HANDLE + KUNCI (1 bh), ENGSEL (3 bh)" },
        ],
        thumb: include_bytes!("../../../assets/icons/sipil/door/p1.svg"),
    },
    DoorType {
        code: "P2",
        name: "Aluminium door, glass + louvre",
        leaves: 1,
        width_m: 0.80,
        height_m: 2.10,
        frame_m: 0.05,
        leaf_m: 0.04,
        elevation: Elevation::GlassLouvre,
        spec: [
            SpecRow { label: "KUSEN", text: "ALUMINIUM 4\" TEBAL 1,2 mm POWDER COATING" },
            SpecRow { label: "PINTU", text: "RANGKA ALUMINIUM POWDER COATING PANIL ALUMINIUM" },
            SpecRow { label: "JENDELA", text: DASH },
            SpecRow { label: "BOUVENLICHT", text: DASH },
            SpecRow { label: "ACCESSORIES", text: "HANDLE + KUNCI (1 bh), ENGSEL (3 bh)" },
        ],
        thumb: include_bytes!("../../../assets/icons/sipil/door/p2.svg"),
    },
    DoorType {
        code: "P3",
        name: "Aluminium double door, glass",
        leaves: 2,
        width_m: 1.60,
        height_m: 2.10,
        frame_m: 0.05,
        leaf_m: 0.04,
        elevation: Elevation::Glass,
        spec: [
            SpecRow { label: "KUSEN", text: "ALUMINIUM 4\" TEBAL 1,2 mm POWDER COATING" },
            SpecRow { label: "PINTU", text: "RANGKA ALUMINIUM POWDER COATING KACA CLEAR 6 mm" },
            SpecRow { label: "JENDELA", text: DASH },
            SpecRow { label: "BOUVENLICHT", text: DASH },
            SpecRow { label: "ACCESSORIES", text: "HANDLE + KUNCI (2 bh), ENGSEL (6 bh)" },
        ],
        thumb: include_bytes!("../../../assets/icons/sipil/door/p3.svg"),
    },
];

/// The type with this mark.
pub fn find(code: &str) -> Option<&'static DoorType> {
    TYPES.iter().find(|t| t.code == code)
}

/// The first type, used when a drawing names one that no longer exists.
pub fn fallback() -> &'static DoorType {
    &TYPES[0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_are_unique_and_sane() {
        for (i, t) in TYPES.iter().enumerate() {
            assert!(t.leaves == 1 || t.leaves == 2, "{}", t.code);
            assert!(t.width_m > 0.3 && t.height_m > 1.5 && t.frame_m > 0.0 && t.leaf_m > 0.0);
            assert!(!t.thumb.is_empty());
            assert_eq!(t.spec.iter().map(|r| r.label).collect::<Vec<_>>(),
                ["KUSEN", "PINTU", "JENDELA", "BOUVENLICHT", "ACCESSORIES"]);
            assert!(TYPES[..i].iter().all(|o| o.code != t.code));
        }
        assert_eq!(find("P2").unwrap().elevation, Elevation::GlassLouvre);
        assert!(find("P9").is_none());
    }
}
