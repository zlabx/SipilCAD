//! The column popup: the SipilGen profile picker, rotation, fill and label.
//! English on purpose; it does not go through `t!`.
//!
//! One dialog serves both a column about to be placed (Place starts the
//! COLUMN command) and one already in the drawing (OK redraws it).

use iced::widget::{checkbox, column, row, text, Space};
use iced::{Element, Fill};

use super::form::Fields;
use super::model::{ColumnDef, Fill as FillKind};
use crate::app::Message;
use crate::sipil_arch::steel::pick::PickMsg;
use crate::sipil_arch::steel::ui::{choice, danger, group, input_row, muted, picker, LABEL_W};
use crate::sipil_arch::ArchMsg;
use crate::ui::style::form::dialog_button;

pub const ID: &str = "column";

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    /// Settings for the columns about to be placed.
    New,
    /// A column in the drawing, by number.
    Column(u32),
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub target: Target,
    pub fields: Fields,
    /// The column's number, place and unit.
    pub base: ColumnDef,
    pub error: Option<String>,
}

/// What the column component keeps between popups.
#[derive(Default)]
pub struct ColumnState {
    /// Settings of the next column; `None` until the first popup.
    pub template: Option<ColumnDef>,
    pub popup: Option<Popup>,
}

#[derive(Clone, Debug)]
pub enum ColumnMsg {
    Pick(PickMsg),
    Rot(String),
    Fill(FillKind),
    Mark(String),
    Label(bool),
    Text(String),
    Apply,
}

pub fn apply_msg(f: &mut Fields, msg: &ColumnMsg) {
    match msg {
        ColumnMsg::Pick(m) => f.pick.apply(m),
        ColumnMsg::Rot(v) => f.rot = v.clone(),
        ColumnMsg::Fill(v) => f.fill = *v,
        ColumnMsg::Mark(v) => f.mark = v.clone(),
        ColumnMsg::Label(v) => f.label = *v,
        ColumnMsg::Text(v) => f.text_mm = v.clone(),
        ColumnMsg::Apply => {}
    }
}

fn msg(m: ColumnMsg) -> Message {
    Message::SipilArch(ArchMsg::new(ID, m))
}

fn wrap_pick(m: PickMsg) -> Message {
    msg(ColumnMsg::Pick(m))
}

pub fn popup_view<'a>(popup: &'a Popup, sizing: crate::ui::modal::ModalSizing) -> Element<'a, Message> {
    let f = &popup.fields;
    let title = match &popup.target {
        Target::New => "Column".to_string(),
        Target::Column(id) => format!("Column #{id}"),
    };

    let profile = group("Profile (SipilGen)", picker(&f.pick, wrap_pick));

    let angle = f.rot.trim().replace(',', ".");
    let rotation = row![
        text("Rotation".to_string()).size(11).style(muted).width(LABEL_W),
        choice("0°", angle == "0", msg(ColumnMsg::Rot("0".into()))),
        choice("90°", angle == "90", msg(ColumnMsg::Rot("90".into()))),
    ]
    .spacing(6)
    .align_y(iced::Center);
    let fill = row![
        text("Fill".to_string()).size(11).style(muted).width(LABEL_W),
        choice("Solid", f.fill == FillKind::Solid, msg(ColumnMsg::Fill(FillKind::Solid))),
        choice("Steel hatch", f.fill == FillKind::Hatch, msg(ColumnMsg::Fill(FillKind::Hatch))),
        choice("None", f.fill == FillKind::None, msg(ColumnMsg::Fill(FillKind::None))),
    ]
    .spacing(6)
    .align_y(iced::Center);
    let plan = group(
        "In the plan",
        column![
            input_row("Rotation (any angle)", &f.rot, "degrees", |v| msg(ColumnMsg::Rot(v))),
            rotation,
            fill,
        ]
        .spacing(6)
        .into(),
    );

    let label = group(
        "Label",
        column![
            row![
                Space::new().width(LABEL_W),
                checkbox(f.label).on_toggle(|on| msg(ColumnMsg::Label(on))).size(14),
                text("Show the mark and profile name".to_string()).size(11),
            ]
            .spacing(6)
            .align_y(iced::Center),
            input_row("Mark", &f.mark, "e.g. K1", |v| msg(ColumnMsg::Mark(v))),
            input_row("Text height", &f.text_mm, "mm", |v| msg(ColumnMsg::Text(v))),
        ]
        .spacing(6)
        .into(),
    );

    let mut body = column![text(title).size(14), profile, plan, label].spacing(8);
    body = body.push(
        text(match popup.target {
            Target::New => "Place: click in the plan, again and again (Esc to finish; type R to turn 90°). The click is the centre of gravity of the section; the section is drawn to scale.",
            Target::Column(_) => "OK redraws this column in place.",
        })
        .size(10)
        .style(muted),
    );
    if let Some(err) = &popup.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let ok_label = match popup.target {
        Target::New => "Place",
        Target::Column(_) => "OK",
    };
    let actions = row![
        Space::new().width(Fill),
        dialog_button(ok_label, msg(ColumnMsg::Apply), true),
        dialog_button("Cancel", Message::CloseModal, false),
    ]
    .spacing(8)
    .align_y(iced::Center);

    column![body, actions].spacing(10).padding(10).width(sizing.width).into()
}
