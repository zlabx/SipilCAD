//! A drawing sheet's title block ("etiket"): paper, margins, template and the
//! texts in it. Pure (`codec`-free): the adapter maps the tag to XDATA.
//!
//! The definition is kept in the drawing itself, on the first entity of the
//! title block, so a sheet can be read back and edited exactly as it was set.

use crate::sipil_wall::engine::XVal;

pub const XDATA_APP: &str = "SIPILTITLE";
pub const XDATA_VERSION: i16 = 1;
pub const LAYER: &str = "TITLEBLOCK";
/// White/black (ACI 7).
pub const LAYER_ACI: i16 = 7;

/// The longest text of a field. An XDATA string holds 255 bytes.
pub const MAX_TEXT: usize = 200;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Paper {
    A4,
    A3,
    A2,
    A1,
    A0,
}

impl Paper {
    pub const ALL: [Paper; 5] = [Paper::A4, Paper::A3, Paper::A2, Paper::A1, Paper::A0];

    pub fn label(self) -> &'static str {
        match self {
            Paper::A4 => "A4",
            Paper::A3 => "A3",
            Paper::A2 => "A2",
            Paper::A1 => "A1",
            Paper::A0 => "A0",
        }
    }

    /// `(short, long)` side in mm.
    pub fn size_mm(self) -> (f64, f64) {
        match self {
            Paper::A4 => (210.0, 297.0),
            Paper::A3 => (297.0, 420.0),
            Paper::A2 => (420.0, 594.0),
            Paper::A1 => (594.0, 841.0),
            Paper::A0 => (841.0, 1189.0),
        }
    }

    /// The medium name a layout stores (the portrait name, as upstream does).
    pub fn canonical(self) -> String {
        let (s, l) = self.size_mm();
        format!("ISO_{}_({s:.2}_x_{l:.2}_MM)", self.label())
    }

    pub fn code(self) -> i16 {
        Paper::ALL.iter().position(|p| *p == self).unwrap_or(1) as i16
    }

    pub fn from_code(code: i16) -> Paper {
        usize::try_from(code)
            .ok()
            .and_then(|i| Paper::ALL.get(i).copied())
            .unwrap_or(Paper::A3)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Template {
    /// A block down the right side, the whole height of the frame (the
    /// Indonesian office standard).
    Side,
    /// A strip along the bottom of the frame.
    Strip,
    /// The frame only.
    FrameOnly,
}

impl Template {
    pub const ALL: [Template; 3] = [Template::Side, Template::Strip, Template::FrameOnly];

    pub fn label(self) -> &'static str {
        match self {
            Template::Side => "Standard (side block)",
            Template::Strip => "Bottom strip",
            Template::FrameOnly => "Frame only",
        }
    }

    pub fn code(self) -> i16 {
        Template::ALL.iter().position(|t| *t == self).unwrap_or(0) as i16
    }

    pub fn from_code(code: i16) -> Template {
        usize::try_from(code)
            .ok()
            .and_then(|i| Template::ALL.get(i).copied())
            .unwrap_or(Template::Side)
    }
}

/// The texts of the block. Headings ("KEGIATAN"...) are fixed by the template.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Fields {
    pub logo: String,
    pub notes: String,
    pub kegiatan: String,
    pub pekerjaan: String,
    pub lokasi: String,
    pub diperiksa: String,
    pub digambar: String,
    pub judul: String,
    pub skala: String,
    pub tanggal: String,
    pub no_lembar: String,
    pub jml_lembar: String,
}

impl Fields {
    fn all(&self) -> [&String; 12] {
        [
            &self.logo,
            &self.notes,
            &self.kegiatan,
            &self.pekerjaan,
            &self.lokasi,
            &self.diperiksa,
            &self.digambar,
            &self.judul,
            &self.skala,
            &self.tanggal,
            &self.no_lembar,
            &self.jml_lembar,
        ]
    }

    fn from_all(v: Vec<String>) -> Option<Fields> {
        let mut it = v.into_iter();
        Some(Fields {
            logo: it.next()?,
            notes: it.next()?,
            kegiatan: it.next()?,
            pekerjaan: it.next()?,
            lokasi: it.next()?,
            diperiksa: it.next()?,
            digambar: it.next()?,
            judul: it.next()?,
            skala: it.next()?,
            tanggal: it.next()?,
            no_lembar: it.next()?,
            jml_lembar: it.next()?,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TitleBlockDef {
    pub paper: Paper,
    pub landscape: bool,
    pub template: Template,
    /// Frame margins from the paper edge, mm.
    pub margin_left: f64,
    pub margin_right: f64,
    pub margin_top: f64,
    pub margin_bottom: f64,
    /// Width of the side block, mm.
    pub block_width: f64,
    pub fields: Fields,
}

impl Default for TitleBlockDef {
    /// A3 landscape, the standard side block; left margin 20 for binding.
    fn default() -> Self {
        TitleBlockDef {
            paper: Paper::A3,
            landscape: true,
            template: Template::Side,
            margin_left: 20.0,
            margin_right: 10.0,
            margin_top: 10.0,
            margin_bottom: 10.0,
            block_width: 60.0,
            fields: Fields {
                logo: "LOGO".into(),
                ..Fields::default()
            },
        }
    }
}

/// A rectangle on the sheet, mm, origin at the lower left corner of the paper.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x0: f64,
    pub y0: f64,
    pub x1: f64,
    pub y1: f64,
}

impl Rect {
    pub fn width(&self) -> f64 {
        self.x1 - self.x0
    }

    pub fn height(&self) -> f64 {
        self.y1 - self.y0
    }

    pub fn center(&self) -> [f64; 2] {
        [(self.x0 + self.x1) / 2.0, (self.y0 + self.y1) / 2.0]
    }
}

impl TitleBlockDef {
    /// The whole definition as one line of text (a custom property of a project).
    pub fn to_text(&self) -> String {
        let mut v = vec![
            self.paper.code().to_string(),
            i16::from(self.landscape).to_string(),
            self.template.code().to_string(),
            self.margin_left.to_string(),
            self.margin_right.to_string(),
            self.margin_top.to_string(),
            self.margin_bottom.to_string(),
            self.block_width.to_string(),
        ];
        v.extend(self.fields.all().into_iter().cloned());
        crate::sipil_arch::fields::join(&v)
    }

    /// The definition [`TitleBlockDef::to_text`] wrote, or `None` for anything else.
    pub fn from_text(text: &str) -> Option<TitleBlockDef> {
        let f = crate::sipil_arch::fields::split(text);
        if f.len() != 8 + 12 {
            return None;
        }
        let num = |i: usize| f[i].parse::<f64>().ok();
        Some(TitleBlockDef {
            paper: Paper::from_code(f[0].parse().ok()?),
            landscape: f[1].parse::<i16>().ok()? != 0,
            template: Template::from_code(f[2].parse().ok()?),
            margin_left: num(3)?,
            margin_right: num(4)?,
            margin_top: num(5)?,
            margin_bottom: num(6)?,
            block_width: num(7)?,
            fields: Fields::from_all(f[8..].to_vec())?,
        })
    }

    /// The sheet `(width, height)` in mm as it lies.
    pub fn sheet_mm(&self) -> (f64, f64) {
        let (s, l) = self.paper.size_mm();
        if self.landscape {
            (l, s)
        } else {
            (s, l)
        }
    }

    /// The frame: the sheet less the margins.
    pub fn frame(&self) -> Rect {
        let (w, h) = self.sheet_mm();
        Rect {
            x0: self.margin_left,
            y0: self.margin_bottom,
            x1: w - self.margin_right,
            y1: h - self.margin_top,
        }
    }

    pub fn check_numbers(&self) -> Result<(), String> {
        for (name, v) in [
            ("Left margin", self.margin_left),
            ("Right margin", self.margin_right),
            ("Top margin", self.margin_top),
            ("Bottom margin", self.margin_bottom),
        ] {
            if !v.is_finite() || !(0.0..=100.0).contains(&v) {
                return Err(format!("{name} must be between 0 and 100 mm."));
            }
        }
        if !self.block_width.is_finite() || !(40.0..=150.0).contains(&self.block_width) {
            return Err("The block width must be between 40 and 150 mm.".to_string());
        }
        for t in self.fields.all() {
            if t.len() > MAX_TEXT {
                return Err(format!("A text can have at most {MAX_TEXT} characters."));
            }
        }
        let f = self.frame();
        if f.width() < 100.0 || f.height() < 100.0 {
            return Err("The margins leave too little room for the frame.".to_string());
        }
        Ok(())
    }
}

/// `DD/MM/YYYY` (UTC) of a Unix time.
pub fn date_text(unix_secs: i64) -> String {
    // Days since 1970-01-01 to a civil date (Howard Hinnant's algorithm).
    let z = unix_secs.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{day:02}/{month:02}/{year}")
}

impl TitleBlockDef {
    /// This title block for one sheet of a project: the title, the scale
    /// (`1:n`), the date and the sheet number (`index` of `total`, 1-based)
    /// filled in; every other text stays as it is.
    pub fn for_sheet(
        &self,
        title: &str,
        scale: u32,
        index: usize,
        total: usize,
        unix_secs: i64,
    ) -> TitleBlockDef {
        let mut d = self.clone();
        d.fields.judul = title.to_string();
        d.fields.skala = format!("1:{scale}");
        d.fields.tanggal = date_text(unix_secs);
        d.fields.no_lembar = index.to_string();
        d.fields.jml_lembar = total.to_string();
        d
    }
}

/// What a title block entity remembers; the anchor (first entity) carries the
/// whole definition.
#[derive(Clone, Debug, PartialEq)]
pub struct TTag {
    pub def: Option<TitleBlockDef>,
}

impl TTag {
    pub fn encode(&self) -> Vec<XVal> {
        let mut v = vec![
            XVal::I16(XDATA_VERSION),
            XVal::I16(i16::from(self.def.is_some())),
        ];
        if let Some(d) = &self.def {
            v.extend([
                XVal::I16(d.paper.code()),
                XVal::I16(i16::from(d.landscape)),
                XVal::I16(d.template.code()),
                XVal::Real(d.margin_left),
                XVal::Real(d.margin_right),
                XVal::Real(d.margin_top),
                XVal::Real(d.margin_bottom),
                XVal::Real(d.block_width),
            ]);
            v.extend(d.fields.all().into_iter().map(|s| XVal::Str(s.clone())));
        }
        v
    }

    pub fn decode(vals: &[XVal]) -> Option<TTag> {
        let [XVal::I16(XDATA_VERSION), XVal::I16(has), rest @ ..] = vals else {
            return None;
        };
        if *has == 0 {
            return rest.is_empty().then_some(TTag { def: None });
        }
        let [XVal::I16(paper), XVal::I16(landscape), XVal::I16(template), XVal::Real(ml), XVal::Real(mr), XVal::Real(mt), XVal::Real(mb), XVal::Real(bw), strings @ ..] =
            rest
        else {
            return None;
        };
        let texts: Vec<String> = strings
            .iter()
            .map(|v| match v {
                XVal::Str(s) => Some(s.clone()),
                _ => None,
            })
            .collect::<Option<_>>()?;
        if texts.len() != 12 {
            return None;
        }
        Some(TTag {
            def: Some(TitleBlockDef {
                paper: Paper::from_code(*paper),
                landscape: *landscape != 0,
                template: Template::from_code(*template),
                margin_left: *ml,
                margin_right: *mr,
                margin_top: *mt,
                margin_bottom: *mb,
                block_width: *bw,
                fields: Fields::from_all(texts)?,
            }),
        })
    }
}
