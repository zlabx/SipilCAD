//! The sheet side of a title block in a drawing: the paper of a layout and the
//! viewport on the drawing. `codec` types only (no iced, no host types), so it
//! is checked against the public `acadrust` crate in the scratch test.

use codec::entities::{Viewport, ViewportStatusFlags};
use codec::objects::{ObjectType, PlotRotation};
use codec::types::Vector3;
use codec::CadDocument;

use super::model::{Rect, TitleBlockDef};

/// Give the paper layout `layout` the paper and orientation of `tb`: the size,
/// its name, the rotation and the limits (what `apply_default_page_setup` does
/// for A4). False when there is no such layout.
pub fn set_paper(doc: &mut CadDocument, layout: &str, tb: &TitleBlockDef) -> bool {
    let (short, long) = tb.paper.size_mm();
    let (w, h) = tb.sheet_mm();
    for object in doc.objects.values_mut() {
        let ObjectType::Layout(l) = object else {
            continue;
        };
        if l.name != layout {
            continue;
        }
        l.paper_width = short;
        l.paper_height = long;
        l.paper_size = tb.paper.canonical();
        l.plot_rotation = if tb.landscape {
            PlotRotation::Degrees90.to_code()
        } else {
            PlotRotation::None.to_code()
        };
        l.min_limits = (0.0, 0.0);
        l.max_limits = (w, h);
        l.min_extents = (0.0, 0.0, 0.0);
        l.max_extents = (w, h, 0.0);
        return true;
    }
    false
}

/// A viewport filling `area` of the sheet, looking at `centre` of model space
/// at `factor` paper mm per model unit.
pub fn content_viewport(area: Rect, centre: [f64; 2], factor: f64) -> Viewport {
    let [cx, cy] = area.center();
    let mut vp = Viewport::new();
    vp.center = Vector3::new(cx, cy, 0.0);
    vp.width = area.width();
    vp.height = area.height();
    vp.view_target = Vector3::new(centre[0], centre[1], 0.0);
    vp.custom_scale = factor;
    vp.view_height = vp.height / factor;
    vp.id = 2;
    vp.status = ViewportStatusFlags::default_on();
    vp
}

/// Make an existing viewport fill `area`, keeping its scale. The overall sheet
/// viewport (id 1) is left alone; returns whether `vp` was changed.
pub fn refit_viewport(vp: &mut Viewport, area: Rect) -> bool {
    if vp.id == 1 {
        return false;
    }
    let [cx, cy] = area.center();
    vp.center = Vector3::new(cx, cy, 0.0);
    vp.width = area.width();
    vp.height = area.height();
    if vp.custom_scale > 1.0e-9 {
        vp.view_height = vp.height / vp.custom_scale;
    }
    true
}
