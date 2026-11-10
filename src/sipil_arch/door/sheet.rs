//! The detail sheet of one door type: elevation, section B, plan detail A,
//! their dimensions and the material table with the number of doors.
//!
//! Drawn at true size (the sheet is laid out in centimetres and scaled to the
//! drawing's unit), below the plan. Pure geometry; text is Indonesian because
//! it is what goes on a construction drawing.

use super::catalog::{DoorType, Elevation};
use super::model::{DPrim, DRole, DShape};
use crate::sipil_wall::engine::{Shape, P};

type Pc = (f64, f64);

/// Gap between two sheets side by side and under the plan, centimetres.
pub const GAP_CM: f64 = 150.0;

/// Where the parts of a sheet go, centimetres from its lower-left corner.
struct Layout {
    w: f64,   // clear width
    f: f64,   // frame
    h: f64,   // clear height
    lt: f64,  // leaf thickness
    t: f64,   // wall brick thickness
    tot: f64, // wall total thickness
    wf: f64,  // overall width (clear + 2 frames)
    ht: f64,  // overall height (clear + head frame)
    elev_x0: f64,
    sec_x0: f64,
    plan_x0: f64,
    plan_x1: f64,
    floor_y: f64,
    width: f64,
    height: f64,
}

const TABLE_H: f64 = 96.0;
const WALL_STUB: f64 = 45.0;

fn layout(ty: &DoorType, t_m: f64, tot_m: f64) -> Layout {
    let (w, f, h, lt) = (ty.width_m * 100.0, ty.frame_m * 100.0, ty.height_m * 100.0, ty.leaf_m * 100.0);
    let (t, tot) = (t_m * 100.0, tot_m.max(t_m) * 100.0);
    let (wf, ht) = (w + 2.0 * f, h + f);
    let elev_x0 = 60.0;
    let sec_x0 = elev_x0 + wf + 130.0;
    let plan_x0 = sec_x0 + t.max(tot) + 120.0;
    let plan_x1 = plan_x0 + wf + 2.0 * WALL_STUB;
    let floor_y = TABLE_H + 85.0;
    Layout {
        w,
        f,
        h,
        lt,
        t,
        tot,
        wf,
        ht,
        elev_x0,
        sec_x0,
        plan_x0,
        plan_x1,
        floor_y,
        width: plan_x1 + 40.0,
        height: floor_y + ht + 100.0,
    }
}

/// Width and height of a sheet, centimetres.
pub fn size_cm(ty: &DoorType, t_m: f64, tot_m: f64) -> (f64, f64) {
    let l = layout(ty, t_m, tot_m);
    (l.width, l.height)
}

struct B {
    prims: Vec<DPrim>,
    o: P,
    k: f64,
    hatch_scale: f64,
}

fn tw(s: &str, h: f64) -> f64 {
    0.6 * h * s.chars().count() as f64
}

/// A length in centimetres as a dimension label.
fn cm(v: f64) -> String {
    if (v - v.round()).abs() < 0.05 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.1}")
    }
}

impl B {
    fn p(&self, (x, y): Pc) -> P {
        [self.o[0] + x * self.k, self.o[1] + y * self.k]
    }

    fn push(&mut self, role: DRole, shape: Shape) {
        self.prims.push(DPrim { role, shape: DShape::Geo(shape) });
    }

    fn line(&mut self, role: DRole, a: Pc, b: Pc) {
        let (a, b) = (self.p(a), self.p(b));
        self.push(role, Shape::Line { a, b });
    }

    fn poly(&mut self, role: DRole, pts: &[Pc], closed: bool) {
        let pts = pts.iter().map(|&q| self.p(q)).collect();
        self.push(role, Shape::Poly { pts, closed });
    }

    fn rect(&mut self, role: DRole, x0: f64, y0: f64, x1: f64, y1: f64) {
        self.poly(role, &[(x0, y0), (x1, y0), (x1, y1), (x0, y1)], true);
    }

    fn circle(&mut self, role: DRole, c: Pc, r: f64) {
        let pts: Vec<Pc> = (0..16)
            .map(|i| {
                let a = std::f64::consts::TAU * i as f64 / 16.0;
                (c.0 + r * a.cos(), c.1 + r * a.sin())
            })
            .collect();
        self.poly(role, &pts, true);
    }

    fn hatch(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) {
        let boundary = vec![self.p((x0, y0)), self.p((x1, y0)), self.p((x1, y1)), self.p((x0, y1))];
        let scale = self.hatch_scale;
        self.push(
            DRole::DetailHatch,
            Shape::Hatch { name: "ANSI31", solid: false, scale, boundary },
        );
    }

    fn text(&mut self, x: f64, y: f64, h: f64, s: &str) {
        self.text_rot(x, y, h, s, 0.0);
    }

    fn text_rot(&mut self, x: f64, y: f64, h: f64, s: &str, rot: f64) {
        let at = self.p((x, y));
        self.prims.push(DPrim {
            role: DRole::Text,
            shape: DShape::Text { at, height: h * self.k, text: s.to_string(), rot },
        });
    }

    /// Text centred on `cx`.
    fn text_c(&mut self, cx: f64, y: f64, h: f64, s: &str) {
        self.text(cx - tw(s, h) / 2.0, y, h, s);
    }

    fn dashed(&mut self, role: DRole, a: Pc, b: Pc, dash: f64, gap: f64) {
        let len = (b.0 - a.0).hypot(b.1 - a.1);
        if len < 1e-9 {
            return;
        }
        let (ux, uy) = ((b.0 - a.0) / len, (b.1 - a.1) / len);
        let mut s = 0.0;
        while s < len {
            let e = (s + dash).min(len);
            self.line(role, (a.0 + ux * s, a.1 + uy * s), (a.0 + ux * e, a.1 + uy * e));
            s += dash + gap;
        }
    }

    /// Horizontal dimension between `x0` and `x1` on the line at `y`; the
    /// extension lines run down to `from_y`.
    fn dim_h(&mut self, x0: f64, x1: f64, y: f64, from_y: f64, label: &str) {
        for x in [x0, x1] {
            self.line(DRole::Dim, (x, from_y), (x, y + 3.0));
            self.line(DRole::Dim, (x - 2.0, y - 2.0), (x + 2.0, y + 2.0));
        }
        self.line(DRole::Dim, (x0 - 4.0, y), (x1 + 4.0, y));
        self.text_c((x0 + x1) / 2.0, y + 2.0, 5.5, label);
    }

    /// Vertical dimension between `y0` and `y1` on the line at `x`; the
    /// extension lines run left to `from_x`.
    fn dim_v(&mut self, y0: f64, y1: f64, x: f64, from_x: f64, label: &str) {
        for y in [y0, y1] {
            self.line(DRole::Dim, (from_x, y), (x + 3.0, y));
            self.line(DRole::Dim, (x - 2.0, y - 2.0), (x + 2.0, y + 2.0));
        }
        self.line(DRole::Dim, (x, y0 - 4.0), (x, y1 + 4.0));
        self.text_rot(x - 2.0, (y0 + y1) / 2.0 - tw(label, 5.5) / 2.0, 5.5, label, std::f64::consts::FRAC_PI_2);
    }

    fn marker(&mut self, c: Pc, label: &str) {
        self.circle(DRole::Detail, c, 9.0);
        self.text_c(c.0, c.1 - 3.0, 8.0, label);
    }
}

/// Lever handle and key hole at the latch edge `x` of a leaf, at height `y`.
fn handle(b: &mut B, x: f64, toward_left: bool, y: f64) {
    let s = if toward_left { -1.0 } else { 1.0 };
    b.rect(DRole::Detail, x.min(x + s * 16.0), y - 1.2, x.max(x + s * 16.0), y + 1.2);
    b.circle(DRole::Detail, (x, y), 2.4);
    b.circle(DRole::Detail, (x, y - 12.0), 1.5);
}

fn elevation(b: &mut B, ty: &DoorType, l: &Layout, hinge_left: bool) {
    let (x0, y0) = (l.elev_x0, l.floor_y);
    let (x1, yt) = (x0 + l.wf, y0 + l.ht);
    let (ix0, ix1, iy1) = (x0 + l.f, x1 - l.f, y0 + l.h);
    b.line(DRole::Detail, (x0 - 30.0, y0), (x1 + 30.0, y0));
    b.rect(DRole::Detail, x0, y0, x1, yt);
    b.rect(DRole::Detail, ix0, y0, ix1, iy1);

    let n = ty.leaves.max(1) as usize;
    let lw = l.w / n as f64;
    for i in 0..n {
        let (lx0, lx1) = (ix0 + lw * i as f64, ix0 + lw * (i + 1) as f64);
        if i > 0 {
            b.line(DRole::Detail, (lx0, y0), (lx0, iy1));
        }
        match ty.elevation {
            Elevation::Panel => {
                let (m, mid) = (10.0, y0 + 0.45 * l.h);
                b.rect(DRole::Detail, lx0 + m, mid + 4.0, lx1 - m, iy1 - m);
                b.rect(DRole::Detail, lx0 + m, y0 + m + 20.0, lx1 - m, mid - 4.0);
            }
            Elevation::GlassLouvre => {
                let m = 8.0;
                let louvre_top = y0 + 0.27 * l.h;
                b.rect(DRole::Detail, lx0 + m, louvre_top + 3.0, lx1 - m, iy1 - m);
                b.rect(DRole::Detail, lx0 + m, y0 + m, lx1 - m, louvre_top);
                let mut y = y0 + m + 3.5;
                while y < louvre_top - 1.0 {
                    b.line(DRole::Detail, (lx0 + m, y), (lx1 - m, y));
                    y += 3.5;
                }
            }
            Elevation::Glass => {
                let m = 10.0;
                b.rect(DRole::Detail, lx0 + m, y0 + 25.0, lx1 - m, iy1 - m);
                b.line(DRole::Detail, (lx0 + m, y0 + 25.0 - 4.0), (lx1 - m, y0 + 25.0 - 4.0));
            }
        }
        // Which side this leaf is hinged on.
        let hl = if n >= 2 { i == 0 } else { hinge_left };
        let (hx, lx) = if hl { (lx0, lx1) } else { (lx1, lx0) };
        // Swing symbol: apex at the hinge, ends at the latch corners.
        let mid = y0 + l.h / 2.0;
        b.dashed(DRole::Swing, (hx, mid), (lx, iy1), 6.0, 4.0);
        b.dashed(DRole::Swing, (hx, mid), (lx, y0), 6.0, 4.0);
        handle(b, lx + if hl { -5.0 } else { 5.0 }, hl, y0 + l.h * 0.45);
    }

    // Level of the head and the section marks.
    b.line(DRole::Dim, (x0 - 50.0, iy1), (x0 - 4.0, iy1));
    b.text(x0 - 52.0, iy1 + 3.0, 5.0, &format!("+{:.2} dari", l.h / 100.0));
    b.text(x0 - 52.0, iy1 - 6.0, 5.0, "lantai");
    let cut_y = y0 + 110.0;
    b.dashed(DRole::Detail, (x0 - 40.0, cut_y), (x1 + 8.0, cut_y), 10.0, 3.0);
    b.marker((x0 - 56.0, cut_y), "A");
    b.line(DRole::Detail, (x0 + l.wf / 2.0, y0), (x0 + l.wf / 2.0, y0 - 30.0));
    b.marker((x0 + l.wf / 2.0, y0 - 40.0), "B");

    // Widths above: frame | clear | frame, then overall.
    let dy = yt + 22.0;
    b.dim_h(x0, x0 + l.f, dy, yt + 3.0, &cm(l.f));
    b.dim_h(x0 + l.f, x1 - l.f, dy, yt + 3.0, &cm(l.w));
    b.dim_h(x1 - l.f, x1, dy, yt + 3.0, &cm(l.f));
    b.dim_h(x0, x1, dy + 22.0, yt + 3.0, &cm(l.wf));
}

fn section(b: &mut B, ty: &DoorType, l: &Layout) {
    let (x0, y0) = (l.sec_x0, l.floor_y);
    let d = l.t;
    let (hx0, hx1) = (x0, x0 + d);
    b.line(DRole::Detail, (x0 - 30.0, y0), (hx1 + 30.0, y0));
    // Head frame and leaf.
    b.rect(DRole::Detail, hx0, y0 + l.h, hx1, y0 + l.ht);
    let lx0 = x0 + (d - l.lt) / 2.0;
    b.rect(DRole::Detail, lx0, y0, lx0 + l.lt, y0 + l.h);
    if matches!(ty.elevation, Elevation::GlassLouvre) {
        b.hatch(lx0, y0 + 8.0, lx0 + l.lt, y0 + 0.27 * l.h);
    }
    // Handle through the leaf.
    b.rect(DRole::Detail, lx0 - 6.0, y0 + l.h * 0.45 - 1.2, lx0 + l.lt + 6.0, y0 + l.h * 0.45 + 1.2);
    // Lintel above the head.
    let (lw0, lw1) = (x0 + d / 2.0 - l.tot / 2.0, x0 + d / 2.0 + l.tot / 2.0);
    b.rect(DRole::Detail, lw0, y0 + l.ht, lw1, y0 + l.ht + 25.0);
    b.hatch(lw0, y0 + l.ht, lw1, y0 + l.ht + 25.0);
    b.text(lw0 - 52.0, y0 + l.ht + 12.0, 5.5, "balok");
    b.text(lw0 - 52.0, y0 + l.ht + 4.0, 5.5, "latai");
    // Heights to the right, depth below.
    let rx = lw1.max(hx1) + 30.0;
    b.dim_v(y0, y0 + l.h, rx, hx1 + 2.0, &cm(l.h));
    b.dim_v(y0, y0 + l.ht, rx + 28.0, hx1 + 2.0, &cm(l.ht));
    b.dim_h(hx0, hx1, y0 - 16.0, y0 - 1.0, &cm(d));
    // Title.
    let ty_ = y0 - 56.0;
    b.text(x0 - 6.0, ty_, 9.0, "POT. B");
    b.line(DRole::Detail, (x0 - 8.0, ty_ - 3.0), (x0 + 52.0, ty_ - 3.0));
}

fn plan_detail(b: &mut B, ty: &DoorType, l: &Layout) {
    let yc = l.floor_y + l.ht * 0.55;
    let (wx0, wx1) = (l.plan_x0 + WALL_STUB, l.plan_x0 + WALL_STUB + l.wf);
    // Wall stubs both sides: brick hatch and plaster lines.
    for (a, c) in [(l.plan_x0, wx0), (wx1, wx1 + WALL_STUB)] {
        b.rect(DRole::Detail, a, yc - l.t / 2.0, c, yc + l.t / 2.0);
        b.hatch(a, yc - l.t / 2.0, c, yc + l.t / 2.0);
        for s in [-1.0, 1.0] {
            b.line(DRole::Detail, (a, yc + s * l.tot / 2.0), (c, yc + s * l.tot / 2.0));
        }
    }
    // Frame jambs and the closed leaves.
    b.rect(DRole::Frame, wx0, yc - l.tot / 2.0, wx0 + l.f, yc + l.tot / 2.0);
    b.rect(DRole::Frame, wx1 - l.f, yc - l.tot / 2.0, wx1, yc + l.tot / 2.0);
    let n = ty.leaves.max(1) as usize;
    let lw = l.w / n as f64;
    for i in 0..n {
        let (a, c) = (wx0 + l.f + lw * i as f64, wx0 + l.f + lw * (i + 1) as f64);
        b.rect(DRole::Leaf, a, yc - l.lt / 2.0, c, yc + l.lt / 2.0);
        b.line(DRole::Leaf, (a, yc), (c, yc));
    }
    // Dimensions below.
    let dy = yc - l.tot / 2.0 - 22.0;
    let from = yc - l.tot / 2.0 - 3.0;
    b.dim_h(wx0, wx0 + l.f, dy, from, &cm(l.f));
    b.dim_h(wx0 + l.f, wx1 - l.f, dy, from, &cm(l.w));
    b.dim_h(wx1 - l.f, wx1, dy, from, &cm(l.f));
    b.dim_h(wx0, wx1, dy - 22.0, from, &cm(l.wf));
    // Title.
    let ty_ = dy - 70.0;
    b.text(l.plan_x1 - 80.0, ty_, 10.0, "POT. A");
    b.line(DRole::Detail, (l.plan_x1 - 84.0, ty_ - 3.0), (l.plan_x1 + 2.0, ty_ - 3.0));
}

fn table(b: &mut B, ty: &DoorType, l: &Layout, count: usize) {
    let w = l.width;
    let (ax, bx) = (24.0, 110.0);
    let head = 26.0;
    let row = 14.0;
    b.rect(DRole::Detail, 0.0, 0.0, w, TABLE_H);
    // Header: TYPE / JUMLAH with a diagonal, then the count.
    let top = TABLE_H - head;
    b.line(DRole::Detail, (0.0, top), (w, top));
    b.line(DRole::Detail, (bx + ax, top), (bx + ax, TABLE_H));
    b.line(DRole::Detail, (0.0, TABLE_H), (bx + ax, top));
    b.text(6.0, TABLE_H - 9.0, 6.5, "TYPE");
    b.text(bx + ax - 36.0, top + 3.0, 6.5, "JUMLAH");
    b.text(bx + ax + 14.0, top + 7.0, 9.0, &format!("{}  =  {}  buah", ty.code, count));
    // Body: BAHAN (rotated) over the first four rows, ACCESSORIES alone.
    let mut y = top;
    for (i, r) in ty.spec.iter().enumerate() {
        let (y1, y0) = (y, y - row);
        b.line(DRole::Detail, (if i == 4 { 0.0 } else { 0.0 }, y0), (w, y0));
        let tx = if i == 4 { 28.0 } else { ax + 4.0 };
        b.text(tx, y0 + 4.0, 6.0, r.label);
        b.text(bx + ax + 4.0, y0 + 4.0, 5.5, r.text);
        y = y0;
        let _ = y1;
    }
    b.line(DRole::Detail, (ax, top), (ax, top - 4.0 * row));
    b.line(DRole::Detail, (bx + ax, top), (bx + ax, 0.0));
    b.text_rot(ax - 6.0, top - 4.0 * row + 6.0, 6.5, "BAHAN", std::f64::consts::FRAC_PI_2);
}

/// The sheet of `ty` for `count` doors with its lower-left corner at
/// `origin` (drawing units, `upm` per metre). `t_m` and `tot_m` are the wall's
/// brick and total thickness, for the plan detail.
pub fn sheet(ty: &DoorType, count: usize, origin: P, upm: f64, t_m: f64, tot_m: f64) -> Vec<DPrim> {
    let l = layout(ty, t_m, tot_m);
    let mut b = B {
        prims: Vec::new(),
        o: origin,
        k: upm / 100.0,
        hatch_scale: (t_m * upm / 3.0) / 3.175,
    };
    elevation(&mut b, ty, &l, true);
    section(&mut b, ty, &l);
    plan_detail(&mut b, ty, &l);
    table(&mut b, ty, &l, count);
    // Big type mark, top right.
    b.text(l.plan_x1 - 30.0, l.height - 30.0, 26.0, ty.code);
    b.prims
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sipil_arch::door::catalog;

    fn texts(p: &[DPrim]) -> Vec<String> {
        p.iter()
            .filter_map(|q| match &q.shape {
                DShape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn the_sheet_has_every_part_and_the_count() {
        let ty = catalog::find("P2").unwrap();
        let prims = sheet(ty, 46, [0.0, 0.0], 100.0, 0.10, 0.13);
        let t = texts(&prims);
        for want in ["P2  =  46  buah", "POT. A", "POT. B", "KUSEN", "PINTU", "BAHAN", "TYPE", "JUMLAH", "+2.10 dari", "lantai", "balok", "latai", "A", "B"] {
            assert!(t.iter().any(|s| s == want), "missing {want}: {t:?}");
        }
        // The dimensions of the sample: 5 | 80 | 5 = 90, 210, 215, 10.
        for want in ["5", "80", "90", "210", "215", "10"] {
            assert!(t.iter().any(|s| s == want), "missing dimension {want}");
        }
        assert!(t.iter().any(|s| s.starts_with("ALUMINIUM 4")));
        assert!(prims.iter().any(|p| matches!(p.shape, DShape::Geo(Shape::Hatch { .. }))));
    }

    #[test]
    fn everything_stays_inside_the_sheet_and_scales_with_the_unit() {
        for ty in catalog::TYPES.iter() {
            let (w, h) = size_cm(ty, 0.10, 0.13);
            for upm in [1.0, 100.0, 1000.0] {
                let k = upm / 100.0;
                let prims = sheet(ty, 3, [1000.0, -500.0], upm, 0.10, 0.13);
                let pts = |s: &Shape| -> Vec<P> {
                    match s {
                        Shape::Poly { pts, .. } => pts.clone(),
                        Shape::Line { a, b } => vec![*a, *b],
                        Shape::Hatch { boundary, .. } => boundary.clone(),
                    }
                };
                for p in &prims {
                    if let DShape::Geo(s) = &p.shape {
                        for q in pts(s) {
                            // Dimension text and markers stay within a margin of the sheet.
                            let (x, y) = ((q[0] - 1000.0) / k, (q[1] + 500.0) / k);
                            assert!(x >= -80.0 - 1e-6 && x <= w + 1e-6, "{} x={x} of {w}", ty.code);
                            assert!(y >= -1e-6 && y <= h + 1e-6, "{} y={y} of {h}", ty.code);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn double_doors_get_two_leaves_in_the_elevation() {
        let single = sheet(catalog::find("P2").unwrap(), 1, [0.0, 0.0], 100.0, 0.1, 0.13);
        let double = sheet(catalog::find("P3").unwrap(), 1, [0.0, 0.0], 100.0, 0.1, 0.13);
        let swing = |p: &[DPrim]| p.iter().filter(|q| q.role == DRole::Swing).count();
        assert!(swing(&double) > swing(&single));
        assert!(texts(&double).iter().any(|s| s == "160"));
    }

    #[test]
    fn sizes_are_in_centimetres() {
        let (w, h) = size_cm(catalog::find("P2").unwrap(), 0.10, 0.13);
        assert!(w > 400.0 && w < 700.0 && h > 400.0 && h < 600.0, "{w} x {h}");
        assert_eq!(cm(80.0), "80");
        assert_eq!(cm(7.5), "7.5");
    }
}
