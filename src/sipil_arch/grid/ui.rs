//! The GRID popup: a template, the X lines, the Y lines and the levels as text
//! (coordinates and labels, as in SipilBIM), where the bubbles go. English on
//! purpose; it does not go through `t!`.
//!
//! One dialog serves a grid about to be drawn and a grid already in the drawing
//! (OK redraws it).

use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Border, Element, Fill, Length, Theme};

use super::form::Fields;
use super::spec::Bubble;
use super::templates::TEMPLATES;
use crate::app::Message;
use crate::sipil_arch::ArchMsg;
use crate::ui::modal::ModalSizing;
use crate::ui::style::form::{button_style, dialog_button};

pub const ID: &str = "grid";

#[derive(Clone, Debug)]
pub struct Popup {
    pub fields: Fields,
    /// The view the grid is drawn as (`ViewSpec` text): a plan unless the
    /// drawing is an elevation or a section of a project.
    pub view: String,
    /// The scale of the grid already in the drawing, if there is one.
    pub scale: Option<f64>,
    /// The entities of the grid already in the drawing.
    pub handles: Vec<codec::Handle>,
    /// `Some(on)` when a project is open: update its grid and tree too.
    pub project: Option<bool>,
    pub error: Option<String>,
}

/// What the grid component keeps between popups.
#[derive(Default)]
pub struct GridState {
    pub popup: Option<Popup>,
}

#[derive(Clone, Debug)]
pub enum GridMsg {
    XCoords(String),
    XLabels(String),
    XBubble(Bubble),
    YCoords(String),
    YLabels(String),
    YBubble(Bubble),
    ZCoords(String),
    ZLabels(String),
    Extend(String),
    Template(&'static str),
    Project(bool),
    Apply,
}

pub fn apply_msg(p: &mut Popup, msg: &GridMsg) {
    let f = &mut p.fields;
    match msg {
        GridMsg::XCoords(v) => f.x_coords = v.clone(),
        GridMsg::XLabels(v) => f.x_labels = v.clone(),
        GridMsg::XBubble(b) => f.x_bubble = *b,
        GridMsg::YCoords(v) => f.y_coords = v.clone(),
        GridMsg::YLabels(v) => f.y_labels = v.clone(),
        GridMsg::YBubble(b) => f.y_bubble = *b,
        GridMsg::ZCoords(v) => f.z_coords = v.clone(),
        GridMsg::ZLabels(v) => f.z_labels = v.clone(),
        GridMsg::Extend(v) => f.extend = v.clone(),
        GridMsg::Template(key) => f.apply_template(key),
        GridMsg::Project(on) => p.project = Some(*on),
        GridMsg::Apply => {}
    }
}

fn msg(m: GridMsg) -> Message {
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

const LABEL_W: f32 = 90.0;

fn choice<'a>(label: &str, selected: bool, on_press: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(12))
        .on_press(on_press)
        .padding([4, 12])
        .style(button_style(selected))
        .into()
}

/// One axis: its coordinates, its labels and where its bubbles go.
fn axis<'a>(
    coords: &str,
    labels: &str,
    bubble: Bubble,
    on_coords: impl Fn(String) -> Message + 'a,
    on_labels: impl Fn(String) -> Message + 'a,
    on_bubble: impl Fn(Bubble) -> Message + 'a,
) -> Element<'a, Message> {
    let mut bubbles = row![text("Bubbles".to_string())
        .size(11)
        .style(muted)
        .width(LABEL_W)]
    .spacing(6)
    .align_y(iced::Center);
    for b in Bubble::ALL {
        bubbles = bubbles.push(choice(b.label(), b == bubble, on_bubble(b)));
    }
    column![
        row![
            text("Coordinates".to_string())
                .size(11)
                .style(muted)
                .width(LABEL_W),
            text_input("0.00 2*3000.00", coords)
                .on_input(on_coords)
                .size(12)
                .padding([3, 6])
                .width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Center),
        row![
            text("Labels".to_string())
                .size(11)
                .style(muted)
                .width(LABEL_W),
            text_input("made up when empty", labels)
                .on_input(on_labels)
                .size(12)
                .padding([3, 6])
                .width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Center),
        bubbles,
    ]
    .spacing(6)
    .into()
}

pub fn popup_view<'a>(popup: &'a Popup, sizing: ModalSizing) -> Element<'a, Message> {
    let f = &popup.fields;

    let mut templates = row![].spacing(6);
    for t in TEMPLATES.iter() {
        templates = templates.push(choice(t.title, false, msg(GridMsg::Template(t.key))));
    }

    let x = axis(
        &f.x_coords,
        &f.x_labels,
        f.x_bubble,
        |v| msg(GridMsg::XCoords(v)),
        |v| msg(GridMsg::XLabels(v)),
        |b| msg(GridMsg::XBubble(b)),
    );
    let y = axis(
        &f.y_coords,
        &f.y_labels,
        f.y_bubble,
        |v| msg(GridMsg::YCoords(v)),
        |v| msg(GridMsg::YLabels(v)),
        |b| msg(GridMsg::YBubble(b)),
    );
    let levels = column![
        row![
            text("Elevations".to_string())
                .size(11)
                .style(muted)
                .width(LABEL_W),
            text_input("0.00 3000.00", &f.z_coords)
                .on_input(|v| msg(GridMsg::ZCoords(v)))
                .size(12)
                .padding([3, 6])
                .width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Center),
        row![
            text("Names".to_string())
                .size(11)
                .style(muted)
                .width(LABEL_W),
            text_input("made up when empty", &f.z_labels)
                .on_input(|v| msg(GridMsg::ZLabels(v)))
                .size(12)
                .padding([3, 6])
                .width(Fill),
        ]
        .spacing(8)
        .align_y(iced::Center),
    ]
    .spacing(6);
    let extend = row![
        text("Lines extend".to_string())
            .size(11)
            .style(muted)
            .width(LABEL_W),
        text_input("1000", &f.extend)
            .on_input(|v| msg(GridMsg::Extend(v)))
            .size(12)
            .padding([3, 6])
            .width(Length::Fixed(80.0)),
        text("mm beyond the outer lines".to_string())
            .size(11)
            .style(muted),
    ]
    .spacing(8)
    .align_y(iced::Center);

    // What the texts say, live.
    let summary = match f.to_def().and_then(|d| d.resolve()) {
        Ok(g) => text(format!(
            "{} X lines, {} Y lines, {} levels",
            g.x.len(),
            g.y.len(),
            g.z.len()
        ))
        .size(10)
        .style(muted),
        Err(e) => text(e).size(10).style(danger),
    };

    let mut body = column![
        text("Grid".to_string()).size(14),
        group("Template", templates.into()),
        group("X lines (A B C ...)", x),
        group("Y lines (1 2 3 ...)", y),
        group("Levels", levels.into()),
        extend,
        summary,
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
                    "Update the project grid and add its missing drawings",
                    on,
                    msg(GridMsg::Project(true))
                ),
                choice("This drawing only", !on, msg(GridMsg::Project(false))),
            ]
            .spacing(6)
            .align_y(iced::Center),
        );
    }
    body = body.push(
        text(
            if popup.handles.is_empty() {
                "OK draws the grid at the origin of this drawing."
            } else {
                "OK redraws the grid in this drawing."
            }
            .to_string(),
        )
        .size(10)
        .style(muted),
    );
    if let Some(err) = &popup.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let actions = row![
        Space::new().width(Fill),
        dialog_button("OK", msg(GridMsg::Apply), true),
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
