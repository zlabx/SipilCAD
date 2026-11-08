//! SipilCAD wall engine: brick wall + plaster + hatch + practical columns.
//!
//! A Rust port of the user's `dinding_bata.lsp` (BW), with a few
//! corrections noted below. This file is deliberately **pure**: no iced, no
//! `codec`, no host types, so it compiles and is unit-tested on its own
//! (`cargo test` in a scratch crate; see `rebrand/opencadstudio/test/`).
//! `entities.rs` turns the primitives into drawing entities.
//!
//! Differences from the LISP:
//! - Parameters are stored in **metres** (`WallParams`) and converted with the
//!   drawing's units-per-metre, so a wall keeps its meaning if the drawing unit
//!   changes and the popup can speak centimetres.
//! - T-junctions: a column that already exists where a new wall starts, ends or
//!   bends is reused (no duplicate), and the new wall's pieces stop at *that*
//!   column's real face (the LISP assumed the new wall's own column size).
//! - Every wall carries a small tag (`Tag`, stored as XDATA by the adapter) so
//!   clicking any part of it can reopen its parameters and rebuild it.
//! - Hatch scale is derived from the pattern's real spacing in the metric
//!   pattern catalog (ANSI31 = 3.175), not from the CADCraft x25.4 rule.

use std::f64::consts::PI;

pub type P = [f64; 2];

/// Default layer table, one row per [`Role`] in the same order: name, ACI
/// colour and lineweight (1/100 mm, `-3` = the drawing's default weight).
pub const DEFAULT_LAYERS: [(&str, i16, i16); 5] = [
    ("WALL-BRICK", 8, -3),
    ("WALL-PLASTER", 5, -3),
    ("WALL-HATCH", 1, -3),
    ("WALL-COLUMN", 5, -3),
    ("WALL-COLUMN-HATCH", 7, -3),
];

/// Layer names walls used before the layers became editable (tag version 1).
const LEGACY_LAYER_NAMES: [&str; 5] = [
    "BW-BATA",
    "BW-PLASTER",
    "BW-ARSIR",
    "BW-KOLOM",
    "BW-ARSIR-KOLOM",
];

/// Lineweights (1/100 mm) a layer can have, as the drawing's lineweight list
/// offers them. `-3` is the default weight.
pub const LINEWEIGHTS: [i16; 25] = [
    -3, 0, 5, 9, 13, 15, 18, 20, 25, 30, 35, 40, 50, 53, 60, 70, 80, 90, 100, 106, 120, 140, 158,
    200, 211,
];

/// Label of a lineweight value: `Default`, or millimetres with two decimals.
pub fn lineweight_label(w: i16) -> String {
    if w < 0 {
        "Default".to_string()
    } else {
        format!("{:.2} mm", f64::from(w) / 100.0)
    }
}

/// One wall layer: what it is called, its colour (AutoCAD colour index,
/// 1-255) and its lineweight.
#[derive(Clone, Debug, PartialEq)]
pub struct LayerSpec {
    pub name: String,
    pub color: i16,
    pub lineweight: i16,
}

/// The default layers, in [`Role`] order.
pub fn default_layers() -> [LayerSpec; 5] {
    DEFAULT_LAYERS.map(|(name, color, lineweight)| LayerSpec {
        name: name.to_string(),
        color,
        lineweight,
    })
}

/// The layers of walls drawn before layers were editable: the old names with
/// the old colours.
fn legacy_layers() -> [LayerSpec; 5] {
    let mut layers = default_layers();
    for (layer, name) in layers.iter_mut().zip(LEGACY_LAYER_NAMES) {
        layer.name = name.to_string();
    }
    layers
}

/// Application name of the XDATA record (a registered APPID).
pub const XDATA_APP: &str = "SIPILWALL";
/// 2 added the layer table; version 1 tags are still read.
const XDATA_VERSION: i16 = 2;

/// AR-CONC calibration of the LISP (`*bw-ck*`): scale = CK * total / 25.4.
const CONCRETE_K: f64 = 0.012;
const CONCRETE_PATTERN: &str = "AR-CONC";

// ── Parameters ────────────────────────────────────────────────────────────

/// Brick hatch choices offered in the UI.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pattern {
    Ansi31,
    Ansi32,
    Ansi37,
    Solid,
    None,
}

impl Pattern {
    pub const CHOICES: [Pattern; 5] = [
        Pattern::Ansi31,
        Pattern::Ansi37,
        Pattern::Ansi32,
        Pattern::Solid,
        Pattern::None,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Pattern::Ansi31 => "ANSI31",
            Pattern::Ansi32 => "ANSI32",
            Pattern::Ansi37 => "ANSI37",
            Pattern::Solid => "SOLID",
            Pattern::None => "",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Pattern::Ansi31 => "Diagonal (ANSI31)",
            Pattern::Ansi37 => "Cross (ANSI37)",
            Pattern::Ansi32 => "Double diagonal (ANSI32)",
            Pattern::Solid => "Solid fill (SOLID)",
            Pattern::None => "No hatch",
        }
    }

    /// Distance between neighbouring lines at scale 1 in the metric catalog.
    fn base_spacing(self) -> f64 {
        match self {
            Pattern::Ansi31 | Pattern::Ansi37 => 3.175,
            Pattern::Ansi32 => 4.7625,
            Pattern::Solid | Pattern::None => 1.0,
        }
    }

    fn code(self) -> i16 {
        match self {
            Pattern::Ansi31 => 0,
            Pattern::Ansi32 => 1,
            Pattern::Ansi37 => 2,
            Pattern::Solid => 3,
            Pattern::None => 4,
        }
    }

    fn from_code(code: i16) -> Pattern {
        match code {
            1 => Pattern::Ansi32,
            2 => Pattern::Ansi37,
            3 => Pattern::Solid,
            4 => Pattern::None,
            _ => Pattern::Ansi31,
        }
    }
}

/// How the extra columns of a straight wall are spread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spacing {
    /// Equal bays, none longer than the interval ("Even").
    Even,
    /// Exactly the interval from the start, remainder at the end ("Fixed").
    Fixed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WallParams {
    /// Brick masonry thickness, metres.
    pub thickness_m: f64,
    /// Plaster per face, metres (0 = no plaster).
    pub plaster_m: f64,
    /// Column length along the wall, metres (0 = same as total thickness).
    pub col_len_m: f64,
    /// Extra column every N metres on straight runs (0 = none).
    pub interval_m: f64,
    pub spacing: Spacing,
    pub pattern: Pattern,
    /// Brick hatch scale override (0 = automatic).
    pub hatch_scale: f64,
    /// Hatch the columns with AR-CONC.
    pub col_hatch: bool,
    /// Column hatch scale override (0 = automatic).
    pub col_scale: f64,
    /// Layer of each kind of part, in [`Role`] order.
    pub layers: [LayerSpec; 5],
}

impl Default for WallParams {
    fn default() -> Self {
        Self {
            thickness_m: 0.10,
            plaster_m: 0.015,
            col_len_m: 0.0,
            interval_m: 0.0,
            spacing: Spacing::Even,
            pattern: Pattern::Ansi31,
            hatch_scale: 0.0,
            col_hatch: true,
            col_scale: 0.0,
            layers: default_layers(),
        }
    }
}

impl WallParams {
    /// Total wall thickness (brick + plaster on both faces), metres.
    pub fn total_m(&self) -> f64 {
        self.thickness_m + 2.0 * self.plaster_m
    }

    /// Column length actually used, metres.
    pub fn column_len_m(&self) -> f64 {
        if self.col_len_m > 0.0 {
            self.col_len_m
        } else {
            self.total_m()
        }
    }

    /// The layer a part of this wall is drawn on.
    pub fn layer(&self, role: Role) -> &LayerSpec {
        &self.layers[role.index()]
    }

    /// Clamp to values the geometry can use; NaN and negatives become safe ones.
    pub fn sanitized(&self) -> WallParams {
        let fix = |v: f64, min: f64| if v.is_finite() { v.max(min) } else { min };
        let mut layers = self.layers.clone();
        for (layer, (name, _, _)) in layers.iter_mut().zip(DEFAULT_LAYERS) {
            if layer.name.trim().is_empty() {
                layer.name = name.to_string();
            }
            layer.color = layer.color.clamp(1, 255);
            layer.lineweight = if layer.lineweight < 0 {
                -3
            } else {
                layer.lineweight.min(211)
            };
        }
        WallParams {
            layers,
            thickness_m: fix(self.thickness_m, 0.01),
            plaster_m: fix(self.plaster_m, 0.0),
            col_len_m: fix(self.col_len_m, 0.0),
            interval_m: fix(self.interval_m, 0.0),
            hatch_scale: fix(self.hatch_scale, 0.0),
            col_scale: fix(self.col_scale, 0.0),
            ..self.clone()
        }
    }
}

/// A ready-made wall type for the popup.
pub struct Preset {
    pub label: &'static str,
    pub thickness_m: f64,
    pub plaster_m: f64,
}

pub const PRESETS: [Preset; 4] = [
    Preset {
        label: "1/2 brick + plaster",
        thickness_m: 0.10,
        plaster_m: 0.015,
    },
    Preset {
        label: "1 brick + plaster",
        thickness_m: 0.20,
        plaster_m: 0.015,
    },
    Preset {
        label: "1/2 brick (no plaster)",
        thickness_m: 0.10,
        plaster_m: 0.0,
    },
    Preset {
        label: "1 brick (no plaster)",
        thickness_m: 0.20,
        plaster_m: 0.0,
    },
];

/// Drawing units per metre for an INSUNITS code. The LISP knew mm/cm/m and
/// defaulted to mm; inches and feet are added.
pub fn units_per_metre(insunits: i16) -> f64 {
    match insunits {
        1 => 39.370_078_74,
        2 => 3.280_839_895,
        5 => 100.0,
        6 => 1.0,
        _ => 1000.0,
    }
}

pub fn unit_name(upm: f64) -> &'static str {
    if (upm - 1000.0).abs() < 1e-6 {
        "mm"
    } else if (upm - 100.0).abs() < 1e-6 {
        "cm"
    } else if (upm - 1.0).abs() < 1e-6 {
        "m"
    } else {
        "unit"
    }
}

// ── Definition and output ─────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub struct WallDef {
    pub params: WallParams,
    /// Drawing units per metre when the wall was drawn.
    pub upm: f64,
    /// Axis points (the centre line), drawing units.
    pub pts: Vec<P>,
    pub closed: bool,
}

/// A column's box: centre, direction of its length, half length, half width.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColumnBox {
    pub c: P,
    pub dir: f64,
    pub hl: f64,
    pub hw: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Brick,
    Plaster,
    Hatch,
    Column,
    ColumnHatch,
}

impl Role {
    /// Position in the layer table ([`WallParams::layers`]).
    pub fn index(self) -> usize {
        self.code() as usize
    }

    fn code(self) -> i16 {
        match self {
            Role::Brick => 0,
            Role::Plaster => 1,
            Role::Hatch => 2,
            Role::Column => 3,
            Role::ColumnHatch => 4,
        }
    }

    fn from_code(code: i16) -> Option<Role> {
        Some(match code {
            0 => Role::Brick,
            1 => Role::Plaster,
            2 => Role::Hatch,
            3 => Role::Column,
            4 => Role::ColumnHatch,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    Poly {
        pts: Vec<P>,
        closed: bool,
    },
    Line {
        a: P,
        b: P,
    },
    /// `name` is the catalog pattern; `solid` ignores `scale`.
    Hatch {
        name: &'static str,
        solid: bool,
        scale: f64,
        boundary: Vec<P>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Prim {
    pub role: Role,
    pub shape: Shape,
    /// Set for columns: the box to remember for T-junctions.
    pub column: Option<ColumnBox>,
}

#[derive(Clone, Debug, Default)]
pub struct Output {
    pub prims: Vec<Prim>,
    /// Columns this build created (not the reused ones).
    pub new_columns: Vec<ColumnBox>,
    pub n_pieces: usize,
    pub n_columns: usize,
}

// ── Geometry helpers (names follow the LISP) ──────────────────────────────

fn polar(p: P, ang: f64, d: f64) -> P {
    [p[0] + d * ang.cos(), p[1] + d * ang.sin()]
}

fn dist(a: P, b: P) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

fn angle(a: P, b: P) -> f64 {
    (b[1] - a[1]).atan2(b[0] - a[0])
}

/// `bw:normang`: fold into (-PI, PI].
pub fn normalize_angle(mut a: f64) -> f64 {
    if !a.is_finite() {
        return 0.0;
    }
    while a > PI {
        a -= 2.0 * PI;
    }
    while a <= -PI {
        a += 2.0 * PI;
    }
    a
}

/// `bw:ext`: half the extent of a box measured along a wall of direction `ar`.
fn extent(ar: f64, b: &ColumnBox) -> f64 {
    b.hl * (ar - b.dir).cos().abs() + b.hw * (ar - b.dir).sin().abs()
}

/// `bw:box`: four corners of a rectangle (centre, direction, half length, half width).
fn rect(c: P, dir: f64, hl: f64, hw: f64) -> Vec<P> {
    let n = dir + PI / 2.0;
    vec![
        polar(polar(c, dir, hl), n, hw),
        polar(polar(c, dir + PI, hl), n, hw),
        polar(polar(c, dir + PI, hl), n + PI, hw),
        polar(polar(c, dir, hl), n + PI, hw),
    ]
}

fn find_existing(c: P, existing: &[ColumnBox], tol: f64) -> Option<ColumnBox> {
    existing.iter().copied().find(|e| dist(e.c, c) < tol)
}

// ── Build ─────────────────────────────────────────────────────────────────

struct Dims {
    t: f64,
    tot: f64,
    cl: f64,
}

/// Column box at vertex `j` (`bw:vcol`).
fn vertex_column(j: usize, n: usize, closed: bool, angs: &[f64], pt: P, d: &Dims) -> ColumnBox {
    let (hl, hw) = (d.cl / 2.0, d.tot / 2.0);
    let has_in = closed || j > 0;
    let has_out = closed || j + 1 < n;
    let ain = has_in.then(|| angs[if j > 0 { j - 1 } else { n - 1 }]);
    let aout = has_out.then(|| angs[j]);
    let (dir, l, w) = match (ain, aout) {
        (Some(ai), Some(ao)) => {
            if normalize_angle(ao - ai).abs() < 1e-4 {
                (ao, hl, hw) // straight (mid point / T)
            } else {
                (ao, hw, hw) // corner: square column
            }
        }
        (None, Some(ao)) => (ao, hl, hw), // start end
        (Some(ai), None) => (ai, hl, hw), // finish end
        (None, None) => (0.0, hl, hw),
    };
    ColumnBox {
        c: pt,
        dir,
        hl: l,
        hw: w,
    }
}

/// Distances along a straight run where an extra column goes (`bw:stations`).
fn stations(len: f64, interval_du: f64, spacing: Spacing, tot: f64) -> Vec<f64> {
    if interval_du <= 0.0 || len <= interval_du {
        return Vec::new();
    }
    match spacing {
        Spacing::Even => {
            let n = (len / interval_du + 0.999_999).floor() as usize;
            (1..n).map(|k| k as f64 * (len / n as f64)).collect()
        }
        Spacing::Fixed => {
            let mut out = Vec::new();
            let mut k = 1.0;
            while k * interval_du < len - tot {
                out.push(k * interval_du);
                k += 1.0;
            }
            out
        }
    }
}

fn hatch_fill(role: Role, pattern: Pattern, scale: f64, boundary: Vec<P>) -> Option<Prim> {
    match pattern {
        Pattern::None => None,
        Pattern::Solid => Some(Prim {
            role,
            shape: Shape::Hatch {
                name: "SOLID",
                solid: true,
                scale: 1.0,
                boundary,
            },
            column: None,
        }),
        p => Some(Prim {
            role,
            shape: Shape::Hatch {
                name: p.name(),
                solid: false,
                scale,
                boundary,
            },
            column: None,
        }),
    }
}

/// Build the wall. `existing` are the columns already in the drawing (from
/// their tags); a vertex or station that lands on one reuses it.
pub fn build(def: &WallDef, existing: &[ColumnBox]) -> Result<Output, String> {
    let upm = if def.upm.is_finite() && def.upm > 0.0 {
        def.upm
    } else {
        1000.0
    };
    let params = def.params.sanitized();

    // Axis: drop repeated points and a closing duplicate.
    let mut pts: Vec<P> = Vec::with_capacity(def.pts.len());
    for &p in &def.pts {
        if !(p[0].is_finite() && p[1].is_finite()) {
            return Err("titik tidak valid".into());
        }
        if pts.last().map_or(true, |&q| dist(p, q) > 1e-6) {
            pts.push(p);
        }
    }
    let mut closed = def.closed;
    if closed && pts.len() > 2 && dist(pts[0], pts[pts.len() - 1]) <= 1e-6 {
        pts.pop();
    }
    if closed && pts.len() < 3 {
        closed = false;
    }
    let n = pts.len();
    if n < 2 {
        return Err("minimal 2 titik".into());
    }
    let nr = if closed { n } else { n - 1 };

    let dims = Dims {
        t: params.thickness_m * upm,
        tot: params.total_m() * upm,
        cl: params.column_len_m() * upm,
    };
    let tol = 0.05 * dims.tot;
    let hscale = if params.hatch_scale > 0.0 {
        params.hatch_scale
    } else {
        (dims.t / 3.0) / params.pattern.base_spacing()
    };
    let cscale = if params.col_scale > 0.0 {
        params.col_scale
    } else {
        CONCRETE_K * dims.tot / 25.4
    };

    let angs: Vec<f64> = (0..nr).map(|i| angle(pts[i], pts[(i + 1) % n])).collect();

    let mut out = Output::default();
    let add_column = |out: &mut Output, b: ColumnBox| {
        let corners = rect(b.c, b.dir, b.hl, b.hw);
        out.prims.push(Prim {
            role: Role::Column,
            shape: Shape::Poly {
                pts: corners.clone(),
                closed: true,
            },
            column: Some(b),
        });
        if params.col_hatch {
            // Concrete is always AR-CONC, whatever the brick hatch is.
            out.prims.push(Prim {
                role: Role::ColumnHatch,
                shape: Shape::Hatch {
                    name: CONCRETE_PATTERN,
                    solid: false,
                    scale: cscale,
                    boundary: corners,
                },
                column: None,
            });
        }
        out.new_columns.push(b);
        out.n_columns += 1;
    };

    // Columns at every point; reuse the ones already there.
    let mut vcols: Vec<ColumnBox> = Vec::with_capacity(n);
    for (j, &pt) in pts.iter().enumerate() {
        let own = vertex_column(j, n, closed, &angs, pt, &dims);
        match find_existing(pt, existing, tol) {
            Some(found) => vcols.push(found),
            None => {
                add_column(&mut out, own);
                vcols.push(own);
            }
        }
    }

    // Walls and the columns between points.
    let interval_du = params.interval_m * upm;
    for i in 0..nr {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let ang = angs[i];
        let len = dist(a, b);
        let vs = extent(ang, &vcols[i]);
        let ve = extent(ang, &vcols[(i + 1) % n]);

        // (distance along the run, half extent of the column there)
        let mut stl: Vec<(f64, f64)> = vec![(0.0, vs)];
        for st in stations(len, interval_du, params.spacing, dims.tot) {
            let c = polar(a, ang, st);
            let own = ColumnBox {
                c,
                dir: ang,
                hl: dims.cl / 2.0,
                hw: dims.tot / 2.0,
            };
            let used = match find_existing(c, existing, tol) {
                Some(found) => found,
                None => {
                    add_column(&mut out, own);
                    own
                }
            };
            stl.push((st, extent(ang, &used)));
        }
        stl.push((len, ve));

        for w in stl.windows(2) {
            let start = w[0].0 + w[0].1;
            let end = w[1].0 - w[1].1;
            if end - start > 1e-6 {
                piece(
                    &mut out,
                    polar(a, ang, start),
                    polar(a, ang, end),
                    ang,
                    &dims,
                    &params,
                    hscale,
                );
            }
        }
    }
    if out.prims.is_empty() {
        return Err("tidak ada yang bisa digambar".into());
    }
    Ok(out)
}

/// A lone column outline at `c`: what the cursor shows before the first click.
pub fn marker(params: &WallParams, upm: f64, c: P) -> Output {
    let params = params.sanitized();
    let upm = if upm.is_finite() && upm > 0.0 {
        upm
    } else {
        1000.0
    };
    let b = ColumnBox {
        c,
        dir: 0.0,
        hl: params.column_len_m() * upm / 2.0,
        hw: params.total_m() * upm / 2.0,
    };
    let mut out = Output::default();
    out.prims.push(Prim {
        role: Role::Column,
        shape: Shape::Poly {
            pts: rect(c, 0.0, b.hl, b.hw),
            closed: true,
        },
        column: Some(b),
    });
    out.new_columns.push(b);
    out.n_columns = 1;
    out
}

/// One wall piece between two columns: brick outline, plaster lines, hatch.
fn piece(out: &mut Output, p0: P, p1: P, ang: f64, d: &Dims, params: &WallParams, hscale: f64) {
    let half = dist(p0, p1) / 2.0;
    let mid = polar(p0, ang, half);
    let outline = rect(mid, ang, half, d.t / 2.0);
    out.prims.push(Prim {
        role: Role::Brick,
        shape: Shape::Poly {
            pts: outline.clone(),
            closed: true,
        },
        column: None,
    });
    if params.plaster_m > 0.0 {
        let n = ang + PI / 2.0;
        let hp = d.tot / 2.0;
        for side in [n, n + PI] {
            out.prims.push(Prim {
                role: Role::Plaster,
                shape: Shape::Line {
                    a: polar(p0, side, hp),
                    b: polar(p1, side, hp),
                },
                column: None,
            });
        }
    }
    if let Some(h) = hatch_fill(Role::Hatch, params.pattern, hscale, outline) {
        out.prims.push(h);
    }
    out.n_pieces += 1;
}

// ── Tag (what the adapter stores as XDATA) ────────────────────────────────

/// Neutral XDATA values; the adapter maps them to the codec's own type.
#[derive(Clone, Debug, PartialEq)]
pub enum XVal {
    I16(i16),
    I32(i32),
    Real(f64),
    Str(String),
    Pt(P),
}

/// What every wall entity remembers. Only the anchor (the first primitive)
/// carries the full definition; the rest carry just the wall id.
#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub id: u32,
    pub role: Role,
    pub column: Option<ColumnBox>,
    pub def: Option<WallDef>,
    /// Anchor only: centre of the anchor outline when the wall was built. A
    /// MOVE later shows up as the difference to its centre now, so editing a
    /// moved wall rebuilds it where it is, not where it was drawn.
    pub origin: Option<P>,
}

impl Tag {
    pub fn encode(&self) -> Vec<XVal> {
        let mut v = vec![
            XVal::I16(XDATA_VERSION),
            XVal::I32(self.id as i32),
            XVal::I16(self.role.code()),
            XVal::I16(i16::from(self.def.is_some())),
        ];
        if self.role == Role::Column {
            let b = self.column.unwrap_or(ColumnBox {
                c: [0.0, 0.0],
                dir: 0.0,
                hl: 0.0,
                hw: 0.0,
            });
            v.extend([XVal::Real(b.dir), XVal::Real(b.hl), XVal::Real(b.hw)]);
        }
        if let Some(def) = &self.def {
            let p = &def.params;
            v.extend([
                XVal::Real(def.upm),
                XVal::Real(p.thickness_m),
                XVal::Real(p.plaster_m),
                XVal::Real(p.col_len_m),
                XVal::Real(p.interval_m),
                XVal::I16(match p.spacing {
                    Spacing::Even => 0,
                    Spacing::Fixed => 1,
                }),
                XVal::I16(p.pattern.code()),
                XVal::Real(p.hatch_scale),
                XVal::I16(i16::from(p.col_hatch)),
                XVal::Real(p.col_scale),
            ]);
            for layer in &p.layers {
                v.extend([
                    XVal::Str(layer.name.clone()),
                    XVal::I16(layer.color),
                    XVal::I16(layer.lineweight),
                ]);
            }
            v.extend([
                XVal::I16(i16::from(def.closed)),
                XVal::I32(def.pts.len() as i32),
            ]);
            v.extend(def.pts.iter().map(|&pt| XVal::Pt(pt)));
            v.push(XVal::Pt(
                self.origin
                    .unwrap_or(def.pts.first().copied().unwrap_or([0.0, 0.0])),
            ));
        }
        v
    }

    /// Read a tag back. `None` for anything that is not a wall tag; the
    /// column centre is the caller's job (centroid of the polyline).
    pub fn decode(vals: &[XVal]) -> Option<Tag> {
        type It<'a> = std::slice::Iter<'a, XVal>;
        fn i16_of(it: &mut It) -> Option<i16> {
            match it.next()? {
                XVal::I16(v) => Some(*v),
                _ => None,
            }
        }
        fn i32_of(it: &mut It) -> Option<i32> {
            match it.next()? {
                XVal::I32(v) => Some(*v),
                _ => None,
            }
        }
        fn real_of(it: &mut It) -> Option<f64> {
            match it.next()? {
                XVal::Real(v) => Some(*v),
                _ => None,
            }
        }
        fn str_of(it: &mut It) -> Option<String> {
            match it.next()? {
                XVal::Str(v) => Some(v.clone()),
                _ => None,
            }
        }
        let mut it = vals.iter();
        let version = i16_of(&mut it)?;
        if !(1..=XDATA_VERSION).contains(&version) {
            return None;
        }
        let id = i32_of(&mut it)? as u32;
        let role = Role::from_code(i16_of(&mut it)?)?;
        let has_def = i16_of(&mut it)? == 1;
        let column = if role == Role::Column {
            Some(ColumnBox {
                c: [0.0, 0.0],
                dir: real_of(&mut it)?,
                hl: real_of(&mut it)?,
                hw: real_of(&mut it)?,
            })
        } else {
            None
        };
        let def = if has_def {
            let upm = real_of(&mut it)?;
            let thickness_m = real_of(&mut it)?;
            let plaster_m = real_of(&mut it)?;
            let col_len_m = real_of(&mut it)?;
            let interval_m = real_of(&mut it)?;
            let spacing = if i16_of(&mut it)? == 1 {
                Spacing::Fixed
            } else {
                Spacing::Even
            };
            let pattern = Pattern::from_code(i16_of(&mut it)?);
            let hatch_scale = real_of(&mut it)?;
            let col_hatch = i16_of(&mut it)? == 1;
            let col_scale = real_of(&mut it)?;
            // Version 1 had fixed layers; they keep their old names.
            let mut layers = legacy_layers();
            if version >= 2 {
                for layer in layers.iter_mut() {
                    *layer = LayerSpec {
                        name: str_of(&mut it)?,
                        color: i16_of(&mut it)?,
                        lineweight: i16_of(&mut it)?,
                    };
                }
            }
            let closed = i16_of(&mut it)? == 1;
            let count = usize::try_from(i32_of(&mut it)?).ok()?;
            let mut pts = Vec::with_capacity(count.min(100_000));
            for _ in 0..count {
                match it.next()? {
                    XVal::Pt(p) => pts.push(*p),
                    _ => return None,
                }
            }
            let origin = match it.next()? {
                XVal::Pt(p) => *p,
                _ => return None,
            };
            Some((
                WallDef {
                    params: WallParams {
                        thickness_m,
                        plaster_m,
                        col_len_m,
                        interval_m,
                        spacing,
                        pattern,
                        hatch_scale,
                        col_hatch,
                        col_scale,
                        layers,
                    },
                    upm,
                    pts,
                    closed,
                },
                origin,
            ))
        } else {
            None
        };
        let (def, origin) = match def {
            Some((d, o)) => (Some(d), Some(o)),
            None => (None, None),
        };
        Some(Tag {
            id,
            role,
            column,
            def,
            origin,
        })
    }
}

/// Centre of a polygon from its corner points (what the LISP did with the
/// column polylines): the mean of the vertices.
pub fn centroid(pts: &[P]) -> Option<P> {
    if pts.is_empty() {
        return None;
    }
    let n = pts.len() as f64;
    Some([
        pts.iter().map(|p| p[0]).sum::<f64>() / n,
        pts.iter().map(|p| p[1]).sum::<f64>() / n,
    ])
}

/// Parse a number typed in the popup; accepts a decimal comma.
pub fn parse_number(text: &str) -> Option<f64> {
    let v = text.trim().replace(',', ".").parse::<f64>().ok()?;
    v.is_finite().then_some(v)
}

/// Format a number for a popup field without trailing zeros.
pub fn format_number(v: f64) -> String {
    let s = format!("{:.4}", v);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MM: f64 = 1000.0;

    fn def(pts: &[P], closed: bool, params: WallParams) -> WallDef {
        WallDef {
            params,
            upm: MM,
            pts: pts.to_vec(),
            closed,
        }
    }

    fn bricks(out: &Output) -> Vec<&Vec<P>> {
        out.prims
            .iter()
            .filter_map(|p| match (&p.role, &p.shape) {
                (Role::Brick, Shape::Poly { pts, .. }) => Some(pts),
                _ => None,
            })
            .collect()
    }

    fn x_range(pts: &[P]) -> (f64, f64) {
        let xs = pts.iter().map(|p| p[0]);
        (
            xs.clone().fold(f64::MAX, f64::min),
            xs.fold(f64::MIN, f64::max),
        )
    }

    fn y_range(pts: &[P]) -> (f64, f64) {
        let ys = pts.iter().map(|p| p[1]);
        (
            ys.clone().fold(f64::MAX, f64::min),
            ys.fold(f64::MIN, f64::max),
        )
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn straight_wall_has_two_end_columns_and_one_piece() {
        let out = build(
            &def(&[[0.0, 0.0], [3000.0, 0.0]], false, WallParams::default()),
            &[],
        )
        .unwrap();
        assert_eq!((out.n_columns, out.n_pieces), (2, 1));
        let (x0, x1) = x_range(bricks(&out)[0]);
        // total 130 -> end column half length 65 on each end
        assert!(close(x0, 65.0) && close(x1, 2935.0), "{x0} {x1}");
        let (y0, y1) = y_range(bricks(&out)[0]);
        assert!(close(y0, -50.0) && close(y1, 50.0)); // brick is 100 thick
    }

    #[test]
    fn plaster_lines_sit_on_the_outer_faces_and_vanish_at_zero() {
        let out = build(
            &def(&[[0.0, 0.0], [3000.0, 0.0]], false, WallParams::default()),
            &[],
        )
        .unwrap();
        let lines: Vec<_> = out
            .prims
            .iter()
            .filter_map(|p| match &p.shape {
                Shape::Line { a, .. } if p.role == Role::Plaster => Some(a[1]),
                _ => None,
            })
            .collect();
        assert_eq!(lines.len(), 2);
        assert!(lines.iter().any(|&y| close(y, 65.0)) && lines.iter().any(|&y| close(y, -65.0)));

        let bare = WallParams {
            plaster_m: 0.0,
            ..WallParams::default()
        };
        let out = build(&def(&[[0.0, 0.0], [3000.0, 0.0]], false, bare), &[]).unwrap();
        assert!(out.prims.iter().all(|p| p.role != Role::Plaster));
    }

    #[test]
    fn right_angle_corner_gets_a_square_column_and_pieces_stop_at_its_faces() {
        let pts = [[0.0, 0.0], [3000.0, 0.0], [3000.0, 2000.0]];
        let out = build(&def(&pts, false, WallParams::default()), &[]).unwrap();
        assert_eq!((out.n_columns, out.n_pieces), (3, 2));
        let corner = out.new_columns[1];
        assert!(close(corner.hl, 65.0) && close(corner.hw, 65.0));
        let b = bricks(&out);
        assert!(close(x_range(b[0]).1, 3000.0 - 65.0));
        assert!(close(y_range(b[1]).0, 65.0));
        assert!(close(y_range(b[1]).1, 2000.0 - 65.0));
    }

    #[test]
    fn closed_loop_has_a_column_and_a_piece_per_side() {
        let pts = [[0.0, 0.0], [4000.0, 0.0], [4000.0, 3000.0], [0.0, 3000.0]];
        let out = build(&def(&pts, true, WallParams::default()), &[]).unwrap();
        assert_eq!((out.n_columns, out.n_pieces), (4, 4));
        // A repeated closing point changes nothing.
        let mut again = pts.to_vec();
        again.push(pts[0]);
        let out2 = build(&def(&again, true, WallParams::default()), &[]).unwrap();
        assert_eq!((out2.n_columns, out2.n_pieces), (4, 4));
    }

    #[test]
    fn even_spacing_splits_into_equal_bays() {
        let p = WallParams {
            interval_m: 3.0,
            spacing: Spacing::Even,
            ..WallParams::default()
        };
        let out = build(&def(&[[0.0, 0.0], [10_000.0, 0.0]], false, p), &[]).unwrap();
        // ceil(10/3) = 4 bays -> columns at 2500, 5000, 7500
        assert_eq!((out.n_columns, out.n_pieces), (5, 4));
        let xs: Vec<f64> = out.new_columns.iter().map(|c| c.c[0]).collect();
        for want in [2500.0, 5000.0, 7500.0] {
            assert!(xs.iter().any(|&x| close(x, want)), "{xs:?}");
        }
    }

    #[test]
    fn fixed_spacing_keeps_the_interval_and_leaves_the_remainder_at_the_end() {
        let p = WallParams {
            interval_m: 4.0,
            spacing: Spacing::Fixed,
            ..WallParams::default()
        };
        let out = build(&def(&[[0.0, 0.0], [10_000.0, 0.0]], false, p), &[]).unwrap();
        assert_eq!((out.n_columns, out.n_pieces), (4, 3));
        let xs: Vec<f64> = out.new_columns.iter().map(|c| c.c[0]).collect();
        assert!(xs.iter().any(|&x| close(x, 4000.0)) && xs.iter().any(|&x| close(x, 8000.0)));
        // A run no longer than the interval gets no extra column.
        let out = build(
            &def(
                &[[0.0, 0.0], [3000.0, 0.0]],
                false,
                WallParams {
                    interval_m: 4.0,
                    ..WallParams::default()
                },
            ),
            &[],
        )
        .unwrap();
        assert_eq!(out.n_columns, 2);
    }

    #[test]
    fn a_branch_reuses_the_column_it_starts_on_and_stops_at_its_real_face() {
        let params = WallParams {
            col_len_m: 0.30,
            ..WallParams::default()
        };
        let main = build(
            &def(
                &[[0.0, 0.0], [2000.0, 0.0], [4000.0, 0.0]],
                false,
                params.clone(),
            ),
            &[],
        )
        .unwrap();
        assert_eq!(main.n_columns, 3);
        let branch = build(
            &def(&[[2000.0, 0.0], [2000.0, 1500.0]], false, params),
            &main.new_columns,
        )
        .unwrap();
        // Only the far end is new: the start column already exists.
        assert_eq!(branch.n_columns, 1);
        // The mid column is 300 long along the main wall, 130 across; the branch
        // runs across it, so it must start 65 from the axis (not 150).
        assert!(
            close(y_range(bricks(&branch)[0]).0, 65.0),
            "{:?}",
            y_range(bricks(&branch)[0])
        );
    }

    #[test]
    fn hatch_scale_follows_the_metric_catalog() {
        let out = build(
            &def(&[[0.0, 0.0], [3000.0, 0.0]], false, WallParams::default()),
            &[],
        )
        .unwrap();
        let scales: Vec<(&str, f64)> = out
            .prims
            .iter()
            .filter_map(|p| match &p.shape {
                Shape::Hatch { name, scale, .. } => Some((*name, *scale)),
                _ => None,
            })
            .collect();
        let brick = scales.iter().find(|(n, _)| *n == "ANSI31").unwrap().1;
        let conc = scales.iter().find(|(n, _)| *n == "AR-CONC").unwrap().1;
        // spacing = t/3 = 33.3 at base 3.175
        assert!(close(brick, (100.0 / 3.0) / 3.175));
        assert!(close(conc, 0.012 * 130.0 / 25.4));
        let manual = WallParams {
            hatch_scale: 5.0,
            col_scale: 0.2,
            ..WallParams::default()
        };
        let out = build(&def(&[[0.0, 0.0], [3000.0, 0.0]], false, manual), &[]).unwrap();
        assert!(out.prims.iter().any(
            |p| matches!(&p.shape, Shape::Hatch { name: "ANSI31", scale, .. } if close(*scale, 5.0))
        ));
    }

    #[test]
    fn solid_and_none_patterns() {
        let solid = WallParams {
            pattern: Pattern::Solid,
            ..WallParams::default()
        };
        let out = build(&def(&[[0.0, 0.0], [3000.0, 0.0]], false, solid), &[]).unwrap();
        assert!(out
            .prims
            .iter()
            .any(|p| matches!(&p.shape, Shape::Hatch { solid: true, .. })));
        let none = WallParams {
            pattern: Pattern::None,
            col_hatch: false,
            ..WallParams::default()
        };
        let out = build(&def(&[[0.0, 0.0], [3000.0, 0.0]], false, none), &[]).unwrap();
        assert!(out
            .prims
            .iter()
            .all(|p| p.role != Role::Hatch && p.role != Role::ColumnHatch));
    }

    #[test]
    fn units_follow_the_drawing() {
        let p = WallParams::default();
        let cm = WallDef {
            params: p.clone(),
            upm: units_per_metre(5),
            pts: vec![[0.0, 0.0], [300.0, 0.0]],
            closed: false,
        };
        let out = build(&cm, &[]).unwrap();
        let (x0, x1) = x_range(bricks(&out)[0]);
        assert!(close(x0, 6.5) && close(x1, 293.5), "{x0} {x1}"); // 13 cm total
        assert_eq!(
            (units_per_metre(4), units_per_metre(6), units_per_metre(0)),
            (1000.0, 1.0, 1000.0)
        );
        assert_eq!(unit_name(100.0), "cm");
    }

    #[test]
    fn degenerate_input_is_an_error_not_a_panic() {
        assert!(build(&def(&[[0.0, 0.0]], false, WallParams::default()), &[]).is_err());
        assert!(build(
            &def(&[[1.0, 1.0], [1.0, 1.0]], false, WallParams::default()),
            &[]
        )
        .is_err());
        assert!(build(
            &def(&[[0.0, f64::NAN], [1.0, 1.0]], false, WallParams::default()),
            &[]
        )
        .is_err());
        // Too short for its own columns: columns only, no piece.
        let out = build(
            &def(&[[0.0, 0.0], [100.0, 0.0]], false, WallParams::default()),
            &[],
        )
        .unwrap();
        assert_eq!((out.n_columns, out.n_pieces), (2, 0));
        // Two points cannot be closed.
        let out = build(
            &def(&[[0.0, 0.0], [3000.0, 0.0]], true, WallParams::default()),
            &[],
        )
        .unwrap();
        assert_eq!(out.n_pieces, 1);
        // Nonsense parameters are clamped.
        let bad = WallParams {
            thickness_m: -1.0,
            plaster_m: f64::NAN,
            interval_m: f64::INFINITY,
            ..WallParams::default()
        };
        assert!(build(&def(&[[0.0, 0.0], [3000.0, 0.0]], false, bad), &[]).is_ok());
    }

    #[test]
    fn oblique_corner_still_builds() {
        let pts = [[0.0, 0.0], [3000.0, 0.0], [4500.0, 1500.0]];
        let out = build(&def(&pts, false, WallParams::default()), &[]).unwrap();
        assert_eq!((out.n_columns, out.n_pieces), (3, 2));
    }

    #[test]
    fn tag_round_trips_for_anchor_and_column() {
        let d = WallDef {
            params: WallParams {
                thickness_m: 0.2,
                plaster_m: 0.02,
                col_len_m: 0.25,
                interval_m: 3.0,
                spacing: Spacing::Fixed,
                pattern: Pattern::Ansi37,
                hatch_scale: 1.5,
                col_hatch: false,
                col_scale: 0.07,
                layers: {
                    let mut l = default_layers();
                    l[0] = LayerSpec {
                        name: "A-WALL".to_string(),
                        color: 30,
                        lineweight: 35,
                    };
                    l
                },
            },
            upm: 100.0,
            pts: vec![[0.0, 0.0], [300.0, 0.0], [300.0, 200.0]],
            closed: true,
        };
        let anchor = Tag {
            id: 77,
            role: Role::Brick,
            column: None,
            def: Some(d.clone()),
            origin: Some([12.5, -3.25]),
        };
        assert_eq!(Tag::decode(&anchor.encode()), Some(anchor));
        let b = ColumnBox {
            c: [0.0, 0.0],
            dir: 1.25,
            hl: 6.5,
            hw: 6.5,
        };
        let col = Tag {
            id: 77,
            role: Role::Column,
            column: Some(b),
            def: None,
            origin: None,
        };
        assert_eq!(Tag::decode(&col.encode()), Some(col));
        let both = Tag {
            id: 1,
            role: Role::Column,
            column: Some(b),
            def: Some(d),
            origin: Some([0.0, 0.0]),
        };
        assert_eq!(Tag::decode(&both.encode()), Some(both));
        // Foreign or truncated data is not a tag.
        assert_eq!(Tag::decode(&[]), None);
        assert_eq!(Tag::decode(&[XVal::I16(9), XVal::I32(1)]), None);
        let mut cut = anchor_values();
        cut.truncate(cut.len() - 2);
        assert_eq!(Tag::decode(&cut), None);
    }

    fn anchor_values() -> Vec<XVal> {
        Tag {
            id: 5,
            role: Role::Hatch,
            column: None,
            def: Some(def(
                &[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0]],
                false,
                WallParams::default(),
            )),
            origin: Some([1.0, 2.0]),
        }
        .encode()
    }

    #[test]
    fn version_1_tags_keep_their_old_layers() {
        // A wall drawn before layers were editable: drop the layer table and
        // mark the tag as version 1.
        let mut v = anchor_values();
        v[0] = XVal::I16(1);
        v.drain(14..29);
        let tag = Tag::decode(&v).expect("version 1 tag is still read");
        let layers = &tag.def.unwrap().params.layers;
        let names: Vec<&str> = layers.iter().map(|l| l.name.as_str()).collect();
        assert_eq!(
            names,
            ["BW-BATA", "BW-PLASTER", "BW-ARSIR", "BW-KOLOM", "BW-ARSIR-KOLOM"]
        );
        assert_eq!(layers[0].color, 8);
    }

    #[test]
    fn layers_are_sanitized() {
        let mut p = WallParams::default();
        p.layers[1].name = "  ".to_string();
        p.layers[2].color = 0;
        p.layers[3].color = 999;
        p.layers[4].lineweight = 500;
        p.layers[0].lineweight = -1;
        let s = p.sanitized();
        assert_eq!(s.layers[1].name, DEFAULT_LAYERS[1].0);
        assert_eq!((s.layers[2].color, s.layers[3].color), (1, 255));
        assert_eq!((s.layers[0].lineweight, s.layers[4].lineweight), (-3, 211));
    }

    #[test]
    fn every_role_has_its_own_layer_row() {
        let roles = [
            Role::Brick,
            Role::Plaster,
            Role::Hatch,
            Role::Column,
            Role::ColumnHatch,
        ];
        for (i, role) in roles.iter().enumerate() {
            assert_eq!(role.index(), i);
        }
        assert_eq!(
            WallParams::default().layer(Role::Column).name,
            "WALL-COLUMN"
        );
        assert_eq!(lineweight_label(-3), "Default");
        assert_eq!(lineweight_label(25), "0.25 mm");
        assert!(LINEWEIGHTS.contains(&-3) && LINEWEIGHTS.contains(&211));
    }

    #[test]
    fn numbers_parse_and_format() {
        assert_eq!(parse_number(" 1,5 "), Some(1.5));
        assert_eq!(parse_number("abc"), None);
        assert_eq!(parse_number("inf"), None);
        assert_eq!(format_number(0.015), "0.015");
        assert_eq!(format_number(3.0), "3");
        assert!(close(normalize_angle(3.0 * PI), PI));
    }

    #[test]
    fn marker_is_one_column_outline() {
        let m = marker(&WallParams::default(), MM, [100.0, 50.0]);
        assert_eq!((m.prims.len(), m.n_columns), (1, 1));
        let Shape::Poly { pts, .. } = &m.prims[0].shape else {
            panic!()
        };
        assert!(close(x_range(pts).0, 35.0) && close(x_range(pts).1, 165.0));
    }

    #[test]
    fn presets_are_sane() {
        for p in PRESETS.iter() {
            assert!(p.thickness_m > 0.0 && p.plaster_m >= 0.0, "{}", p.label);
        }
    }
}
