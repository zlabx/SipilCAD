//! The door popup: type cards with pictures, open/closed with pictures,
//! size and hinge. English on purpose; it does not go through `t!`.
//!
//! One dialog serves both a door about to be placed (Place starts the DOOR
//! command) and a door already in the drawing (OK rebuilds it).

use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Border, Element, Fill, Length, Theme};

use super::catalog::{self, DoorType};
use super::form::Fields;
use super::model::DoorDef;
use crate::app::Message;
use crate::sipil_arch::ArchMsg;
use crate::ui::icons::semantic;
use crate::ui::style::form::{button_style, dialog_button};

pub const ID: &str = "door";

const PLAN_OPEN: &[u8] = include_bytes!("../../../assets/icons/sipil/door/plan_open.svg");
const PLAN_CLOSED: &[u8] = include_bytes!("../../../assets/icons/sipil/door/plan_closed.svg");

#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    /// Settings for the doors about to be placed.
    New,
    /// A door in the drawing, by number.
    Door(u32),
}

#[derive(Clone, Debug)]
pub struct Popup {
    pub target: Target,
    pub fields: Fields,
    /// The door's number, wall and place (zero for a new door).
    pub base: DoorDef,
    pub error: Option<String>,
}

/// What the door component keeps between popups.
#[derive(Default)]
pub struct DoorState {
    /// Settings of the next door; `None` until the first popup.
    pub template: Option<DoorDef>,
    pub popup: Option<Popup>,
}

#[derive(Clone, Debug)]
pub enum DoorMsg {
    Type(&'static str),
    Open(bool),
    Angle(String),
    Width(String),
    Height(String),
    HingeStart(bool),
    SwingLeft(bool),
    Apply,
}

pub fn apply_msg(f: &mut Fields, msg: &DoorMsg) {
    match msg {
        DoorMsg::Type(code) => f.set_type(code),
        DoorMsg::Open(v) => f.open = *v,
        DoorMsg::Angle(v) => f.angle = v.clone(),
        DoorMsg::Width(v) => f.width_cm = v.clone(),
        DoorMsg::Height(v) => f.height_cm = v.clone(),
        DoorMsg::HingeStart(v) => f.hinge_start = *v,
        DoorMsg::SwingLeft(v) => f.swing_left = *v,
        DoorMsg::Apply => {}
    }
}

fn msg(m: DoorMsg) -> Message {
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

const LABEL_W: f32 = 120.0;

fn input_row<'a>(label: &str, value: &str, unit: &str, on_input: impl Fn(String) -> Message + 'a) -> Element<'a, Message> {
    row![
        text(label.to_string()).size(11).style(muted).width(LABEL_W),
        text_input("", value)
            .on_input(on_input)
            .size(12)
            .padding([3, 6])
            .width(Length::Fixed(80.0)),
        text(unit.to_string()).size(11).style(muted),
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}

/// A picture button: the picture over a caption, highlighted when chosen.
fn card<'a>(picture: &'static [u8], size: f32, title: String, sub: String, selected: bool, on_press: Message) -> Element<'a, Message> {
    let content = column![
        semantic(picture, size),
        text(title).size(13),
        text(sub).size(10).style(muted),
    ]
    .spacing(3)
    .align_x(iced::Center)
    .width(Fill);
    button(content)
        .on_press(on_press)
        .padding(8)
        .width(Fill)
        .style(button_style(selected))
        .into()
}

fn type_card<'a>(ty: &'static DoorType, selected: bool) -> Element<'a, Message> {
    let sub = format!("{} · {} leaf{}", ty.name, ty.leaves, if ty.leaves > 1 { "ves" } else { "" });
    card(ty.thumb, 64.0, ty.code.to_string(), sub, selected, msg(DoorMsg::Type(ty.code)))
}

fn choice<'a>(label: &str, selected: bool, on_press: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(12))
        .on_press(on_press)
        .padding([4, 14])
        .style(button_style(selected))
        .into()
}

pub fn popup_view<'a>(popup: &'a Popup, sizing: crate::ui::modal::ModalSizing) -> Element<'a, Message> {
    let f = &popup.fields;
    let title = match &popup.target {
        Target::New => "Door".to_string(),
        Target::Door(id) => format!("Door #{id}"),
    };
    let ty = catalog::find(&f.code).unwrap_or_else(catalog::fallback);

    let mut cards = row![].spacing(6);
    for t in catalog::TYPES.iter() {
        cards = cards.push(type_card(t, t.code == f.code));
    }

    let state = row![
        card(PLAN_OPEN, 56.0, "Open".to_string(), "leaf swung, with arc".to_string(), f.open, msg(DoorMsg::Open(true))),
        card(PLAN_CLOSED, 56.0, "Closed".to_string(), "leaf in the frame".to_string(), !f.open, msg(DoorMsg::Open(false))),
    ]
    .spacing(6);
    let mut plan = column![state].spacing(6);
    if f.open {
        plan = plan.push(input_row("Opening angle", &f.angle, "degrees", |v| msg(DoorMsg::Angle(v))));
    }

    let frame_cm = ty.frame_m * 100.0;
    let opening_cm = f.width_cm.trim().replace(',', ".").parse::<f64>().map_or(f64::NAN, |w| w + 2.0 * frame_cm);
    let mut size = column![
        input_row("Clear width", &f.width_cm, "cm", |v| msg(DoorMsg::Width(v))),
        input_row("Clear height", &f.height_cm, "cm", |v| msg(DoorMsg::Height(v))),
        text(if opening_cm.is_finite() {
            format!("Opening in the wall: {} cm (frame {} cm each side)", crate::sipil_wall::engine::format_number(opening_cm), crate::sipil_wall::engine::format_number(frame_cm))
        } else {
            "Opening in the wall: —".to_string()
        })
        .size(10)
        .style(muted),
    ]
    .spacing(6);
    if ty.leaves < 2 {
        size = size.push(
            row![
                text("Hinge at".to_string()).size(11).style(muted).width(LABEL_W),
                choice("Start of wall run", f.hinge_start, msg(DoorMsg::HingeStart(true))),
                choice("End of wall run", !f.hinge_start, msg(DoorMsg::HingeStart(false))),
            ]
            .spacing(6)
            .align_y(iced::Center),
        );
    }
    if let Target::Door(_) = popup.target {
        size = size.push(
            row![
                text("Swings to".to_string()).size(11).style(muted).width(LABEL_W),
                choice("Left of the run", f.swing_left, msg(DoorMsg::SwingLeft(true))),
                choice("Right of the run", !f.swing_left, msg(DoorMsg::SwingLeft(false))),
            ]
            .spacing(6)
            .align_y(iced::Center),
        );
    }

    let mut body = column![
        text(title).size(14),
        group("Type", cards.into()),
        group("In the plan", plan.into()),
        group("Size", size.into()),
    ]
    .spacing(8);
    body = body.push(
        text(match popup.target {
            Target::New => "Place: click on a wall. The door swings to the side you click on, and the wall gets an opening. A detail sheet of the type is drawn below the plan, with the number of doors.",
            Target::Door(_) => "OK redraws this door in its opening and updates the detail sheets.",
        })
        .size(10)
        .style(muted),
    );
    if let Some(err) = &popup.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let ok_label = match popup.target {
        Target::New => "Place",
        Target::Door(_) => "OK",
    };
    let actions = row![
        Space::new().width(Fill),
        dialog_button(ok_label, msg(DoorMsg::Apply), true),
        dialog_button("Cancel", Message::CloseModal, false),
    ]
    .spacing(8)
    .align_y(iced::Center);

    column![body, actions].spacing(10).padding(10).width(sizing.width).into()
}
