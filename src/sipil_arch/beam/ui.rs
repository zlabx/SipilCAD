//! The beam popup: the SipilGen profile picker, drawing style and label.
//! English on purpose; it does not go through `t!`.
//!
//! One dialog serves both a beam about to be placed (Place starts the BEAM
//! command) and one already in the drawing (OK redraws it).

use iced::widget::{checkbox, column, row, text, Space};
use iced::{Element, Fill};

use super::form::Fields;
use super::model::{BeamDef, Style};
use crate::app::Message;
use crate::sipil_arch::steel::pick::PickMsg;
use crate::sipil_arch::steel::ui::{choice, danger, group, input_row, muted, picker, LABEL_W};
use crate::sipil_arch::ArchMsg;
use crate::ui::style::form::dialog_button;

pub const ID: &str = "beam";

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    /// Settings for the beams about to be placed.
    New,
    /// A beam in the drawing, by number.
    Beam(u32),
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub target: Target,
    pub fields: Fields,
    /// The beam's number, ends and unit.
    pub base: BeamDef,
    pub error: Option<String>,
}

/// What the beam component keeps between popups.
#[derive(Default)]
pub struct BeamState {
    /// Settings of the next beam; `None` until the first popup.
    pub template: Option<BeamDef>,
    pub popup: Option<Popup>,
}

#[derive(Clone, Debug)]
pub enum BeamMsg {
    Pick(PickMsg),
    Style(Style),
    Mark(String),
    Label(bool),
    Text(String),
    Apply,
}

pub fn apply_msg(f: &mut Fields, msg: &BeamMsg) {
    match msg {
        BeamMsg::Pick(m) => f.pick.apply(m),
        BeamMsg::Style(v) => f.style = *v,
        BeamMsg::Mark(v) => f.mark = v.clone(),
        BeamMsg::Label(v) => f.label = *v,
        BeamMsg::Text(v) => f.text_mm = v.clone(),
        BeamMsg::Apply => {}
    }
}

fn msg(m: BeamMsg) -> Message {
    Message::SipilArch(ArchMsg::new(ID, m))
}

fn wrap_pick(m: PickMsg) -> Message {
    msg(BeamMsg::Pick(m))
}

pub fn popup_view<'a>(popup: &'a Popup, sizing: crate::ui::modal::ModalSizing) -> Element<'a, Message> {
    let f = &popup.fields;
    let title = match &popup.target {
        Target::New => "Beam".to_string(),
        Target::Beam(id) => format!("Beam #{id}"),
    };

    let profile = group("Profile (SipilGen)", picker(&f.pick, wrap_pick));

    let style = row![
        text("Draw as".to_string()).size(11).style(muted).width(LABEL_W),
        choice("Outline (to scale)", f.style == Style::Outline, msg(BeamMsg::Style(Style::Outline))),
        choice("Centreline", f.style == Style::Centerline, msg(BeamMsg::Style(Style::Centerline))),
    ]
    .spacing(6)
    .align_y(iced::Center);
    let mut plan = column![style].spacing(6);
    if let Target::Beam(_) = popup.target {
        let len = popup.base.length() / popup.base.upm;
        plan = plan.push(text(format!("Length: {len:.3} m")).size(11).style(muted));
    }
    let plan = group("In the plan", plan.into());

    let label = group(
        "Label",
        column![
            row![
                Space::new().width(LABEL_W),
                checkbox(f.label).on_toggle(|on| msg(BeamMsg::Label(on))).size(14),
                text("Show the mark and profile name along the beam".to_string()).size(11),
            ]
            .spacing(6)
            .align_y(iced::Center),
            input_row("Mark", &f.mark, "e.g. B1", |v| msg(BeamMsg::Mark(v))),
            input_row("Text height", &f.text_mm, "mm", |v| msg(BeamMsg::Text(v))),
        ]
        .spacing(6)
        .into(),
    );

    let mut body = column![text(title).size(14), profile, plan, label].spacing(8);
    body = body.push(
        text(match popup.target {
            Target::New => "Place: click the start and the end of each beam; the next beam starts where the last ended (type B to break, Esc to finish). The outline is as wide as the flange.",
            Target::Beam(_) => "OK redraws this beam between the same two points.",
        })
        .size(10)
        .style(muted),
    );
    if let Some(err) = &popup.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let ok_label = match popup.target {
        Target::New => "Place",
        Target::Beam(_) => "OK",
    };
    let actions = row![
        Space::new().width(Fill),
        dialog_button(ok_label, msg(BeamMsg::Apply), true),
        dialog_button("Cancel", Message::CloseModal, false),
    ]
    .spacing(8)
    .align_y(iced::Center);

    column![body, actions].spacing(10).padding(10).width(sizing.width).into()
}
