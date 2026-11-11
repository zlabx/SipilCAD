//! The drawing of a title block as primitives (lines and texts) in paper
//! millimetres. Pure geometry: no host and no `codec`, so it is tested alone.
//!
//! The **Standard (side block)** template follows the office standard: a block
//! down the right of the frame, the full height, rows from the top
//! LOGO 37.5, CATATAN 7.5 + notes (whatever is left, 50 on A3), then KEGIATAN,
//! PEKERJAAN, LOKASI (7.5 + 15 each), DI PERIKSA OLEH and DI GAMBAR OLEH
//! (7.5 + 27.5 each), JUDUL GAMBAR | SKALA (7.5 + 17.5) and
//! TANGGAL | NO. LEMBAR | JML. LEMBAR (7.5 + 12). On A3 landscape with 10 mm
//! top and bottom margins that is exactly the 277 mm of the frame.

use std::collections::BTreeSet;

use super::model::{Rect, Template, TitleBlockDef};

pub type P = [f64; 2];

/// Line weights, 1/100 mm.
pub const W_FRAME: i16 = 70;
pub const W_BLOCK: i16 = 50;
pub const W_CELL: i16 = 25;

/// Height of the bottom strip, mm.
pub const STRIP_H: f64 = 30.0;

const HEADER_TEXT_H: f64 = 2.0;
const VALUE_TEXT_H: f64 = 3.5;
const LOGO_TEXT_H: f64 = 8.0;
const MIN_TEXT_H: f64 = 1.8;
const PAD: f64 = 1.5;
/// Width of a character as a fraction of the text height (the drawing font is
/// narrower than this, so the estimate leaves a little room).
const CHAR_W: f64 = 0.6;
/// Line pitch as a fraction of the text height.
const PITCH: f64 = 1.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Anchor {
    Center,
    Left,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TShape {
    Line {
        a: P,
        b: P,
    },
    Text {
        at: P,
        height: f64,
        text: String,
        anchor: Anchor,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct TPrim {
    /// Line weight, 1/100 mm (0 for text).
    pub weight: i16,
    pub shape: TShape,
}

/// One box of the block: a heading strip on top (may be 0 high) and the room
/// for the value under it.
struct Cell<'a> {
    r: Rect,
    header: &'a str,
    header_h: f64,
    value: &'a str,
    anchor: Anchor,
    /// Largest text height for the value.
    max_h: f64,
    /// Lines start at the top of the value area instead of being centred in it.
    top: bool,
}

fn cell<'a>(r: Rect, header: &'a str, header_h: f64, value: &'a str) -> Cell<'a> {
    Cell {
        r,
        header,
        header_h,
        value,
        anchor: Anchor::Left,
        max_h: VALUE_TEXT_H,
        top: false,
    }
}

// ── text fitting ──────────────────────────────────────────────────────────

/// `text` broken into lines of at most `max_chars` characters, at spaces where
/// it can be, inside a word where it cannot.
pub fn wrap(text: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    let mut lines: Vec<String> = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let mut word: Vec<char> = word.chars().collect();
            loop {
                let have = line.chars().count();
                let gap = usize::from(have > 0);
                if have + gap + word.len() <= max_chars {
                    if gap == 1 {
                        line.push(' ');
                    }
                    line.extend(word.iter());
                    break;
                }
                if have > 0 {
                    lines.push(std::mem::take(&mut line));
                    continue;
                }
                let rest = word.split_off(max_chars);
                lines.push(word.iter().collect());
                word = rest;
            }
        }
        lines.push(line);
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines
}

/// The largest text height (down to the minimum) at which `text` wrapped to
/// `width` fits `height`, with its lines. At the minimum height too long a
/// text is cut to the lines that fit, the last one ending in `...`.
pub fn fit(text: &str, width: f64, height: f64, max_h: f64) -> (Vec<String>, f64) {
    let inner_w = (width - 2.0 * PAD).max(1.0);
    let inner_h = (height - 2.0 * PAD).max(1.0);
    let mut h = max_h;
    loop {
        let chars = (inner_w / (CHAR_W * h)).floor() as usize;
        let lines = wrap(text, chars);
        let room = (inner_h / (PITCH * h)).floor().max(1.0) as usize;
        if lines.len() <= room {
            return (lines, h);
        }
        if h - 0.1 < MIN_TEXT_H {
            let mut cut: Vec<String> = lines.into_iter().take(room).collect();
            if let Some(last) = cut.last_mut() {
                let keep = chars.saturating_sub(3);
                *last = format!("{}...", last.chars().take(keep).collect::<String>());
            }
            return (cut, MIN_TEXT_H);
        }
        h -= 0.1;
    }
}

// ── cells ─────────────────────────────────────────────────────────────────

fn side_cells<'a>(def: &'a TitleBlockDef, f: Rect) -> Result<Vec<Cell<'a>>, String> {
    let fields = &def.fields;
    let bw = def.block_width;
    let (x0, x1) = (f.x1 - bw, f.x1);
    // Everything but the notes room: LOGO, the CATATAN heading, three 7.5+15
    // stacks, two 7.5+27.5 stacks, and the two split rows.
    let fixed = 37.5 + 7.5 + 3.0 * (7.5 + 15.0) + 2.0 * (7.5 + 27.5) + (7.5 + 17.5) + (7.5 + 12.0);
    let notes_h = f.height() - fixed;
    if notes_h < 10.0 {
        return Err(
            "The frame is too short for the side block: use a bigger paper or another template."
                .to_string(),
        );
    }
    let mut cells = Vec::new();
    let mut y = f.y1;
    let row = |cells: &mut Vec<Cell<'a>>,
               y: &mut f64,
               parts: &[(f64, &'a str, &'a str)],
               head: f64,
               value_h: f64| {
        let total_w: f64 = parts.iter().map(|p| p.0).sum();
        let mut x = x0;
        for (w, header, value) in parts {
            let wx = bw * w / total_w;
            let r = Rect {
                x0: x,
                y0: *y - head - value_h,
                x1: x + wx,
                y1: *y,
            };
            cells.push(cell(r, header, head, value));
            x += wx;
        }
        *y -= head + value_h;
    };
    // LOGO: a box with just the text, large and centred.
    let r = Rect {
        x0,
        y0: y - 37.5,
        x1,
        y1: y,
    };
    cells.push(Cell {
        anchor: Anchor::Center,
        max_h: LOGO_TEXT_H,
        ..cell(r, "", 0.0, fields.logo.as_str())
    });
    y -= 37.5;
    row(
        &mut cells,
        &mut y,
        &[(1.0, "CATATAN", fields.notes.as_str())],
        7.5,
        notes_h,
    );
    if let Some(last) = cells.last_mut() {
        last.top = true;
    }
    row(
        &mut cells,
        &mut y,
        &[(1.0, "KEGIATAN", fields.kegiatan.as_str())],
        7.5,
        15.0,
    );
    row(
        &mut cells,
        &mut y,
        &[(1.0, "PEKERJAAN", fields.pekerjaan.as_str())],
        7.5,
        15.0,
    );
    row(
        &mut cells,
        &mut y,
        &[(1.0, "LOKASI", fields.lokasi.as_str())],
        7.5,
        15.0,
    );
    row(
        &mut cells,
        &mut y,
        &[(1.0, "DI PERIKSA OLEH :", fields.diperiksa.as_str())],
        7.5,
        27.5,
    );
    row(
        &mut cells,
        &mut y,
        &[(1.0, "DI GAMBAR OLEH :", fields.digambar.as_str())],
        7.5,
        27.5,
    );
    row(
        &mut cells,
        &mut y,
        &[
            (1.0, "JUDUL GAMBAR", fields.judul.as_str()),
            (1.0, "SKALA", fields.skala.as_str()),
        ],
        7.5,
        17.5,
    );
    row(
        &mut cells,
        &mut y,
        &[
            (1.0, "TANGGAL", fields.tanggal.as_str()),
            (1.0, "NO. LEMBAR", fields.no_lembar.as_str()),
            (1.0, "JML. LEMBAR", fields.jml_lembar.as_str()),
        ],
        7.5,
        12.0,
    );
    Ok(cells)
}

fn strip_cells<'a>(def: &'a TitleBlockDef, f: Rect) -> Result<Vec<Cell<'a>>, String> {
    let fields = &def.fields;
    const LOGO: f64 = 40.0;
    const PROJECT: f64 = 100.0;
    const PEOPLE: f64 = 60.0;
    const SKALA: f64 = 25.0;
    const TANGGAL: f64 = 25.0;
    const NO: f64 = 20.0;
    const JML: f64 = 20.0;
    let judul = f.width() - (LOGO + PROJECT + PEOPLE + SKALA + TANGGAL + NO + JML);
    if judul < 40.0 {
        return Err(
            "The frame is too narrow for the bottom strip: use a bigger paper or another template."
                .to_string(),
        );
    }
    let (y0, y1) = (f.y0, f.y0 + STRIP_H);
    let mut cells = Vec::new();
    let mut x = f.x0;
    let r = Rect {
        x0: x,
        y0,
        x1: x + LOGO,
        y1,
    };
    cells.push(Cell {
        anchor: Anchor::Center,
        max_h: 6.0,
        ..cell(r, "", 0.0, fields.logo.as_str())
    });
    x += LOGO;
    let stack =
        |cells: &mut Vec<Cell<'a>>, x: f64, w: f64, items: &[(&'a str, &'a str)], head: f64| {
            let h = STRIP_H / items.len() as f64;
            for (i, (header, value)) in items.iter().enumerate() {
                let top = y1 - i as f64 * h;
                cells.push(cell(
                    Rect {
                        x0: x,
                        y0: top - h,
                        x1: x + w,
                        y1: top,
                    },
                    header,
                    head,
                    value,
                ));
            }
        };
    stack(
        &mut cells,
        x,
        PROJECT,
        &[
            ("KEGIATAN", fields.kegiatan.as_str()),
            ("PEKERJAAN", fields.pekerjaan.as_str()),
            ("LOKASI", fields.lokasi.as_str()),
        ],
        3.5,
    );
    x += PROJECT;
    stack(
        &mut cells,
        x,
        PEOPLE,
        &[
            ("DI PERIKSA OLEH :", fields.diperiksa.as_str()),
            ("DI GAMBAR OLEH :", fields.digambar.as_str()),
        ],
        5.0,
    );
    x += PEOPLE;
    for (w, header, value) in [
        (judul, "JUDUL GAMBAR", fields.judul.as_str()),
        (SKALA, "SKALA", fields.skala.as_str()),
        (TANGGAL, "TANGGAL", fields.tanggal.as_str()),
        (NO, "NO. LEMBAR", fields.no_lembar.as_str()),
        (JML, "JML. LEMBAR", fields.jml_lembar.as_str()),
    ] {
        cells.push(cell(
            Rect {
                x0: x,
                y0,
                x1: x + w,
                y1,
            },
            header,
            7.5,
            value,
        ));
        x += w;
    }
    Ok(cells)
}

fn cells<'a>(def: &'a TitleBlockDef, f: Rect) -> Result<Vec<Cell<'a>>, String> {
    match def.template {
        Template::Side => side_cells(def, f),
        Template::Strip => strip_cells(def, f),
        Template::FrameOnly => Ok(Vec::new()),
    }
}

/// The room left for the drawing itself (what a viewport shows): the frame less
/// the block.
pub fn drawing_area(def: &TitleBlockDef) -> Result<Rect, String> {
    def.check_numbers()?;
    let f = def.frame();
    cells(def, f)?;
    Ok(match def.template {
        Template::Side => Rect {
            x1: f.x1 - def.block_width,
            ..f
        },
        Template::Strip => Rect {
            y0: f.y0 + STRIP_H,
            ..f
        },
        Template::FrameOnly => f,
    })
}

// ── primitives ────────────────────────────────────────────────────────────

struct Out {
    prims: Vec<TPrim>,
    seen: BTreeSet<[i64; 4]>,
}

impl Out {
    fn line(&mut self, a: P, b: P, weight: i16) {
        let k = |p: P| [(p[0] * 1e4).round() as i64, (p[1] * 1e4).round() as i64];
        let (ka, kb) = (k(a), k(b));
        let (lo, hi) = if ka <= kb { (ka, kb) } else { (kb, ka) };
        if lo == hi || !self.seen.insert([lo[0], lo[1], hi[0], hi[1]]) {
            return; // a line the heavier one before it already drew
        }
        self.prims.push(TPrim {
            weight,
            shape: TShape::Line { a, b },
        });
    }

    fn rect(&mut self, r: Rect, weight: i16) {
        self.line([r.x0, r.y0], [r.x1, r.y0], weight);
        self.line([r.x1, r.y0], [r.x1, r.y1], weight);
        self.line([r.x1, r.y1], [r.x0, r.y1], weight);
        self.line([r.x0, r.y1], [r.x0, r.y0], weight);
    }

    fn text(&mut self, at: P, height: f64, text: String, anchor: Anchor) {
        if !text.is_empty() {
            self.prims.push(TPrim {
                weight: 0,
                shape: TShape::Text {
                    at,
                    height,
                    text,
                    anchor,
                },
            });
        }
    }
}

/// The primitives of the title block, frame first (so the frame's heavy lines
/// win where a cell shares an edge with it).
pub fn build(def: &TitleBlockDef) -> Result<Vec<TPrim>, String> {
    def.check_numbers()?;
    let f = def.frame();
    let list = cells(def, f)?;
    let mut out = Out {
        prims: Vec::new(),
        seen: BTreeSet::new(),
    };
    out.rect(f, W_FRAME);
    match def.template {
        Template::Side => out.rect(
            Rect {
                x0: f.x1 - def.block_width,
                ..f
            },
            W_BLOCK,
        ),
        Template::Strip => out.rect(
            Rect {
                y1: f.y0 + STRIP_H,
                ..f
            },
            W_BLOCK,
        ),
        Template::FrameOnly => {}
    }
    for c in &list {
        out.rect(c.r, W_CELL);
        let value_top = c.r.y1 - c.header_h;
        if c.header_h > 0.0 {
            out.line([c.r.x0, value_top], [c.r.x1, value_top], W_CELL);
            let h = HEADER_TEXT_H.min(c.header_h * 0.6);
            out.text(
                [(c.r.x0 + c.r.x1) / 2.0, c.r.y1 - c.header_h / 2.0],
                h,
                c.header.to_string(),
                Anchor::Center,
            );
        }
        let (w, vh) = (c.r.width(), value_top - c.r.y0);
        if c.value.trim().is_empty() {
            continue;
        }
        let (lines, h) = fit(c.value, w, vh, c.max_h);
        let pitch = PITCH * h;
        let x = match c.anchor {
            Anchor::Center => (c.r.x0 + c.r.x1) / 2.0,
            Anchor::Left => c.r.x0 + PAD,
        };
        let block = lines.len() as f64 * pitch;
        // The centre of the first line.
        let first = if c.top {
            value_top - PAD - pitch / 2.0
        } else {
            value_top - (vh - block) / 2.0 - pitch / 2.0
        };
        for (i, line) in lines.into_iter().enumerate() {
            out.text([x, first - i as f64 * pitch], h, line, c.anchor);
        }
    }
    Ok(out.prims)
}

/// `1:100`.
pub fn scale_text(scale: u32) -> String {
    format!("1:{scale}")
}
