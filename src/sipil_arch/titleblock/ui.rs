//! The TITLEBLOCK popup: the paper, the template, the margins and the texts of
//! the title block ("etiket"). English on purpose; it does not go through `t!`.
//!
//! One dialog edits the title block of the drawing's sheet (OK redraws it) or,
//! in a drawing that has none, draws one.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Border, Element, Fill, Length, Theme};

use super::form::FormFields;
use super::geometry::drawing_area;
use super::model::{Fields, Paper, Template, TitleBlockDef};
use crate::app::Message;
use crate::sipil_arch::ArchMsg;
use crate::ui::modal::ModalSizing;
use crate::ui::style::form::{button_style, dialog_button};

pub const ID: &str = "titleblock";

#[derive(Clone, Debug)]
pub struct Popup {
    pub fields: FormFields,
    /// The paper layout being edited.
    pub layout: String,
    /// The entities of the title block already on the sheet.
    pub handles: Vec<codec::Handle>,
    /// `Some(on)` when a project is open: keep this block as the one new
    /// drawings of the project get.
    pub project: Option<bool>,
    pub error: Option<String>,
}

/// What the title block component keeps between popups.
#[derive(Default)]
pub struct TbState {
    pub popup: Option<Popup>,
}

/// The texts of the block, in the order the popup lists them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextField {
    Logo,
    Notes,
    Kegiatan,
    Pekerjaan,
    Lokasi,
    Diperiksa,
    Digambar,
    Judul,
    Skala,
    Tanggal,
    NoLembar,
    JmlLembar,
}

pub const TEXT_FIELDS: [(TextField, &str); 12] = [
    (TextField::Logo, "Logo text"),
    (TextField::Notes, "Catatan (notes)"),
    (TextField::Kegiatan, "Kegiatan"),
    (TextField::Pekerjaan, "Pekerjaan"),
    (TextField::Lokasi, "Lokasi"),
    (TextField::Diperiksa, "Di periksa oleh"),
    (TextField::Digambar, "Di gambar oleh"),
    (TextField::Judul, "Judul gambar"),
    (TextField::Skala, "Skala"),
    (TextField::Tanggal, "Tanggal"),
    (TextField::NoLembar, "No. lembar"),
    (TextField::JmlLembar, "Jml. lembar"),
];

fn field_mut(f: &mut Fields, which: TextField) -> &mut String {
    match which {
        TextField::Logo => &mut f.logo,
        TextField::Notes => &mut f.notes,
        TextField::Kegiatan => &mut f.kegiatan,
        TextField::Pekerjaan => &mut f.pekerjaan,
        TextField::Lokasi => &mut f.lokasi,
        TextField::Diperiksa => &mut f.diperiksa,
        TextField::Digambar => &mut f.digambar,
        TextField::Judul => &mut f.judul,
        TextField::Skala => &mut f.skala,
        TextField::Tanggal => &mut f.tanggal,
        TextField::NoLembar => &mut f.no_lembar,
        TextField::JmlLembar => &mut f.jml_lembar,
    }
}

fn field_ref(f: &Fields, which: TextField) -> &str {
    match which {
        TextField::Logo => &f.logo,
        TextField::Notes => &f.notes,
        TextField::Kegiatan => &f.kegiatan,
        TextField::Pekerjaan => &f.pekerjaan,
        TextField::Lokasi => &f.lokasi,
        TextField::Diperiksa => &f.diperiksa,
        TextField::Digambar => &f.digambar,
        TextField::Judul => &f.judul,
        TextField::Skala => &f.skala,
        TextField::Tanggal => &f.tanggal,
        TextField::NoLembar => &f.no_lembar,
        TextField::JmlLembar => &f.jml_lembar,
    }
}

#[derive(Clone, Debug)]
pub enum TbMsg {
    Paper(Paper),
    Landscape(bool),
    Template(Template),
    MarginLeft(String),
    MarginRight(String),
    MarginTop(String),
    MarginBottom(String),
    BlockWidth(String),
    Text(TextField, String),
    Project(bool),
    Apply,
}

pub fn apply_msg(p: &mut Popup, msg: &TbMsg) {
    let f = &mut p.fields;
    match msg {
        TbMsg::Paper(v) => f.paper = *v,
        TbMsg::Landscape(v) => f.landscape = *v,
        TbMsg::Template(v) => f.template = *v,
        TbMsg::MarginLeft(v) => f.margin_left = v.clone(),
        TbMsg::MarginRight(v) => f.margin_right = v.clone(),
        TbMsg::MarginTop(v) => f.margin_top = v.clone(),
        TbMsg::MarginBottom(v) => f.margin_bottom = v.clone(),
        TbMsg::BlockWidth(v) => f.block_width = v.clone(),
        TbMsg::Text(which, v) => *field_mut(&mut f.texts, *which) = v.clone(),
        TbMsg::Project(on) => p.project = Some(*on),
        TbMsg::Apply => {}
    }
}

fn msg(m: TbMsg) -> Message {
    Message::SipilArch(ArchMsg::new(ID, m))
}

fn muted(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(theme.palette().background.base.text.scale_alpha(0.6)),
    }
}

fn danger(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(theme.palette().danger.base.color),
    }
}

fn group<'a>(title: &str, body: Element<'a, Message>) -> Element<'a, Message> {
    container(column![text(title.to_string()).size(11).style(muted), body].spacing(6))
        .padding(8)
        .width(Fill)
        .style(|theme: &Theme| container::Style {
            border: Border {
                width: 1.0,
                radius: 4.0.into(),
                color: theme.palette().background.strong.color,
            },
            ..Default::default()
        })
        .into()
}

const LABEL_W: f32 = 100.0;

fn choice<'a>(label: &str, selected: bool, on_press: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(12))
        .on_press(on_press)
        .padding([4, 12])
        .style(button_style(selected))
        .into()
}

fn number<'a>(
    label: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label.to_string()).size(11).style(muted).width(LABEL_W),
        text_input("", value)
            .on_input(on_input)
            .size(12)
            .padding([3, 6])
            .width(Length::Fixed(70.0)),
        text("mm".to_string()).size(11).style(muted),
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}

fn entry<'a>(which: TextField, label: &str, texts: &Fields) -> Element<'a, Message> {
    row![
        text(label.to_string()).size(11).style(muted).width(LABEL_W),
        text_input("", field_ref(texts, which))
            .on_input(move |v| msg(TbMsg::Text(which, v)))
            .size(12)
            .padding([3, 6])
            .width(Fill),
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}

pub fn popup_view<'a>(popup: &'a Popup, sizing: ModalSizing) -> Element<'a, Message> {
    let f = &popup.fields;

    let mut papers = row![].spacing(6);
    for p in Paper::ALL {
        papers = papers.push(choice(p.label(), p == f.paper, msg(TbMsg::Paper(p))));
    }
    let paper = column![
        papers,
        row![
            choice("Landscape", f.landscape, msg(TbMsg::Landscape(true))),
            choice("Portrait", !f.landscape, msg(TbMsg::Landscape(false))),
        ]
        .spacing(6),
    ]
    .spacing(6);

    let mut templates = row![].spacing(6);
    for t in Template::ALL {
        templates = templates.push(choice(t.label(), t == f.template, msg(TbMsg::Template(t))));
    }

    let margins = column![
        row![
            number("Left margin", &f.margin_left, |v| msg(TbMsg::MarginLeft(v))),
            number("Right margin", &f.margin_right, |v| msg(
                TbMsg::MarginRight(v)
            )),
        ]
        .spacing(16),
        row![
            number("Top margin", &f.margin_top, |v| msg(TbMsg::MarginTop(v))),
            number("Bottom margin", &f.margin_bottom, |v| msg(
                TbMsg::MarginBottom(v)
            )),
        ]
        .spacing(16),
        number("Block width", &f.block_width, |v| msg(TbMsg::BlockWidth(v))),
    ]
    .spacing(6);

    let mut texts = column![].spacing(6);
    for (which, label) in TEXT_FIELDS {
        texts = texts.push(entry(which, label, &f.texts));
    }

    // What the numbers leave for the drawing, live.
    let area = match f.to_def().and_then(|d| drawing_area(&d)) {
        Ok(a) => text(format!(
            "Room for the drawing: {:.0} x {:.0} mm",
            a.width(),
            a.height()
        ))
        .size(10)
        .style(muted),
        Err(e) => text(e).size(10).style(danger),
    };

    let sections = column![
        group("Paper", paper.into()),
        group("Template", templates.into()),
        group("Frame margins and block", margins.into()),
        area,
        group("Texts", texts.into()),
    ]
    .spacing(8);

    let mut body = column![
        text("Title Block".to_string()).size(14),
        scrollable(sections).height(Length::Fixed(430.0)),
    ]
    .spacing(8);
    if let Some(on) = popup.project {
        body = body.push(
            row![
                text("Project".to_string())
                    .size(11)
                    .style(muted)
                    .width(LABEL_W),
                choice(
                    "This drawing and the project's new drawings",
                    on,
                    msg(TbMsg::Project(true))
                ),
                choice("This drawing only", !on, msg(TbMsg::Project(false))),
            ]
            .spacing(6)
            .align_y(iced::Center),
        );
    }
    body = body.push(
        text(if popup.handles.is_empty() {
            "OK draws the frame and title block on the sheet of this drawing."
        } else {
            "OK redraws the frame and title block, and fits the drawing's viewport to the room left."
        }
        .to_string())
        .size(10)
        .style(muted),
    );
    if let Some(err) = &popup.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let actions = row![
        Space::new().width(Fill),
        dialog_button("OK", msg(TbMsg::Apply), true),
        dialog_button("Cancel", Message::CloseModal, false),
    ]
    .spacing(8)
    .align_y(iced::Center);

    column![body, actions]
        .spacing(10)
        .padding(10)
        .width(sizing.width)
        .into()
}

/// The definition the popup starts from when the sheet has no title block yet.
pub fn starting_def(project: Option<TitleBlockDef>) -> TitleBlockDef {
    project.unwrap_or_default()
}
