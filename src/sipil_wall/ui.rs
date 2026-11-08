//! The wall parameter popup: the one dialog of the wall tool.
//!
//! It opens from the ribbon's Wall button (settings for the next wall, OK
//! starts drawing) and from a click on a wall already in the drawing (OK
//! redraws that wall). Widgets follow `ui/window/drawing_units.rs` (same
//! helpers, same look). The UI text is English on purpose and does not go
//! through `t!`.

use std::fmt;

use iced::widget::{checkbox, column, container, pick_list, row, text, text_input, Space};
use iced::{Background, Border, Element, Fill, Length, Theme};

use super::engine::{self, lineweight_label, Pattern, Spacing, LINEWEIGHTS, PRESETS};
use super::state::{Fields, LayerFields, Popup, Target, LAYER_LABELS};
use crate::app::Message;
use crate::ui::style::form::dialog_button;

#[derive(Debug, Clone)]
pub enum Field {
    Thickness(String),
    Plaster(String),
    ColLen(String),
    Interval(String),
    Spacing(Spacing),
    Pattern(Pattern),
    HatchScale(String),
    ColHatch(bool),
    ColScale(String),
    /// Layer table row: name, colour (as typed), lineweight.
    LayerName(usize, String),
    LayerColor(usize, String),
    LayerWeight(usize, i16),
}

#[derive(Debug, Clone)]
pub enum Msg {
    /// A field in the popup.
    PopupField(Field),
    /// A wall type button in the popup.
    PopupPreset(usize),
    PopupApply,
}

pub fn apply_field(f: &mut Fields, field: Field) {
    match field {
        Field::Thickness(v) => f.thickness_cm = v,
        Field::Plaster(v) => f.plaster_cm = v,
        Field::ColLen(v) => f.col_len_cm = v,
        Field::Interval(v) => f.interval_m = v,
        Field::Spacing(v) => f.spacing = v,
        Field::Pattern(v) => f.pattern = v,
        Field::HatchScale(v) => f.hatch_scale = v,
        Field::ColHatch(v) => f.col_hatch = v,
        Field::ColScale(v) => f.col_scale = v,
        Field::LayerName(i, v) => {
            if let Some(l) = f.layers.get_mut(i) {
                l.name = v;
            }
        }
        Field::LayerColor(i, v) => {
            if let Some(l) = f.layers.get_mut(i) {
                l.color = v;
            }
        }
        Field::LayerWeight(i, v) => {
            if let Some(l) = f.layers.get_mut(i) {
                l.lineweight = v;
            }
        }
    }
}

/// Set the thickness and plaster fields from a wall type.
pub fn apply_preset(f: &mut Fields, index: usize) {
    if let Some(p) = PRESETS.get(index) {
        f.thickness_cm = engine::format_number(p.thickness_m * 100.0);
        f.plaster_cm = engine::format_number(p.plaster_m * 100.0);
    }
}

// ── small widgets ─────────────────────────────────────────────────────────

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

const LABEL_W: f32 = 130.0;

fn input_row<'a>(
    label: &str,
    value: &str,
    unit: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
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

#[derive(Clone, PartialEq)]
struct PatternChoice(Pattern);

impl fmt::Display for PatternChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.label())
    }
}

#[derive(Clone, PartialEq)]
struct SpacingChoice(Spacing);

impl fmt::Display for SpacingChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.0 {
            Spacing::Even => "Even (equal bays)",
            Spacing::Fixed => "Fixed (remainder at the end)",
        })
    }
}

#[derive(Clone, PartialEq)]
struct WeightChoice(i16);

impl fmt::Display for WeightChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&lineweight_label(self.0))
    }
}

fn pattern_row<'a>(label: &str, selected: Pattern) -> Element<'a, Message> {
    let options: Vec<PatternChoice> = Pattern::CHOICES.iter().map(|p| PatternChoice(*p)).collect();
    let picker = pick_list(Some(PatternChoice(selected)), options, |c| c.to_string())
        .on_select(|c: PatternChoice| popup_msg(Field::Pattern(c.0)))
        .text_size(12)
        .padding([3, 6])
        .width(Fill);
    row![text(label.to_string()).size(11).style(muted).width(LABEL_W), picker]
        .spacing(8)
        .align_y(iced::Center)
        .into()
}

fn spacing_row<'a>(label: &str, selected: Spacing) -> Element<'a, Message> {
    let options = vec![SpacingChoice(Spacing::Even), SpacingChoice(Spacing::Fixed)];
    let picker = pick_list(Some(SpacingChoice(selected)), options, |c| c.to_string())
        .on_select(|c: SpacingChoice| popup_msg(Field::Spacing(c.0)))
        .text_size(12)
        .padding([3, 6])
        .width(Fill);
    row![text(label.to_string()).size(11).style(muted).width(LABEL_W), picker]
        .spacing(8)
        .align_y(iced::Center)
        .into()
}

fn popup_msg(f: Field) -> Message {
    Message::SipilWall(Msg::PopupField(f))
}

/// "Total thickness: 13 cm", live from the typed fields.
fn total_label(f: &Fields) -> String {
    match f.to_params() {
        Ok(p) => format!(
            "Total thickness (brick + plaster): {} cm",
            engine::format_number(p.total_m() * 100.0)
        ),
        Err(_) => "Total thickness: —".to_string(),
    }
}

/// A small square of the layer's AutoCAD colour, so a typed index is seen.
fn swatch<'a>(color_text: &str) -> Element<'a, Message> {
    let aci = engine::parse_number(color_text)
        .filter(|v| v.fract() == 0.0 && (1.0..=255.0).contains(v))
        .map(|v| v as i16);
    let fill = aci.map(|i| {
        crate::ui::properties::acad_color_display(codec::types::Color::from_index(i)).0
    });
    container(
        Space::new()
            .width(Length::Fixed(18.0))
            .height(Length::Fixed(18.0)),
    )
    .style(move |theme: &Theme| container::Style {
        background: fill.map(Background::Color),
        border: Border {
            width: 1.0,
            radius: 2.0.into(),
            color: theme.palette().background.strong.color,
        },
        ..Default::default()
    })
    .into()
}

fn layer_row<'a>(i: usize, l: &'a LayerFields) -> Element<'a, Message> {
    let weights: Vec<WeightChoice> = LINEWEIGHTS.iter().map(|w| WeightChoice(*w)).collect();
    row![
        text(LAYER_LABELS[i].to_string())
            .size(11)
            .style(muted)
            .width(LABEL_W - 40.0),
        text_input("", &l.name)
            .on_input(move |v| popup_msg(Field::LayerName(i, v)))
            .size(12)
            .padding([3, 6])
            .width(Fill),
        swatch(&l.color),
        text_input("", &l.color)
            .on_input(move |v| popup_msg(Field::LayerColor(i, v)))
            .size(12)
            .padding([3, 6])
            .width(Length::Fixed(48.0)),
        pick_list(Some(WeightChoice(l.lineweight)), weights, |c| c.to_string())
            .on_select(move |c: WeightChoice| popup_msg(Field::LayerWeight(i, c.0)))
            .text_size(12)
            .padding([3, 6])
            .width(Length::Fixed(100.0)),
    ]
    .spacing(6)
    .align_y(iced::Center)
    .into()
}

// ── popup ─────────────────────────────────────────────────────────────────

pub fn popup_view<'a>(
    popup: &'a Popup,
    sizing: crate::ui::modal::ModalSizing,
) -> Element<'a, Message> {
    let f = &popup.fields;
    let title = match &popup.target {
        Target::New => "Wall".to_string(),
        Target::Wall(id) => format!("Wall #{id}"),
    };

    let mut presets = column![].spacing(4);
    for (row_idx, chunk) in PRESETS.chunks(2).enumerate() {
        let mut r = row![].spacing(4);
        for (j, p) in chunk.iter().enumerate() {
            let idx = row_idx * 2 + j;
            r = r.push(
                dialog_button(p.label, Message::SipilWall(Msg::PopupPreset(idx)), false)
                    .width(Fill),
            );
        }
        presets = presets.push(r);
    }

    let size = group(
        "Size",
        column![
            presets,
            input_row("Brick thickness", &f.thickness_cm, "cm", |v| popup_msg(
                Field::Thickness(v)
            )),
            input_row("Plaster per face", &f.plaster_cm, "cm (0 = none)", |v| {
                popup_msg(Field::Plaster(v))
            }),
            input_row("Column every", &f.interval_m, "m (0 = none, e.g. 3 or 4)", |v| {
                popup_msg(Field::Interval(v))
            }),
            text(total_label(f)).size(11).style(muted),
        ]
        .spacing(6)
        .into(),
    );

    let hatch = group(
        "Hatch",
        column![
            pattern_row("Brick pattern", f.pattern),
            input_row("Brick hatch scale", &f.hatch_scale, "0 = auto", |v| {
                popup_msg(Field::HatchScale(v))
            }),
            row![
                Space::new().width(Length::Fixed(LABEL_W)),
                checkbox(f.col_hatch)
                    .on_toggle(|on| popup_msg(Field::ColHatch(on)))
                    .size(14),
                text("Hatch columns with concrete (AR-CONC)".to_string()).size(11),
            ]
            .spacing(6)
            .align_y(iced::Center),
            input_row("Column hatch scale", &f.col_scale, "0 = auto", |v| {
                popup_msg(Field::ColScale(v))
            }),
        ]
        .spacing(6)
        .into(),
    );

    let columns = group(
        "Practical columns",
        column![
            text("Columns are always placed at corners, ends and bends.".to_string())
                .size(10)
                .style(muted),
            input_row("Column length", &f.col_len_cm, "cm (0 = total thickness)", |v| {
                popup_msg(Field::ColLen(v))
            }),
            spacing_row("Extra column spacing", f.spacing),
        ]
        .spacing(6)
        .into(),
    );

    let mut layers = column![row![
        text("Part".to_string()).size(10).style(muted).width(LABEL_W - 40.0),
        text("Layer name".to_string()).size(10).style(muted).width(Fill),
        text("Colour (1-255)".to_string()).size(10).style(muted).width(Length::Fixed(72.0)),
        text("Lineweight".to_string()).size(10).style(muted).width(Length::Fixed(100.0)),
    ]
    .spacing(6)]
    .spacing(6);
    for (i, l) in f.layers.iter().enumerate() {
        layers = layers.push(layer_row(i, l));
    }
    let layers = group("Layers", layers.into());

    let mut body = column![text(title).size(14), size, hatch, columns, layers].spacing(8);
    if let Target::Wall(_) = popup.target {
        body = body.push(
            text("Changes redraw this whole wall (the axis points stay).".to_string())
                .size(10)
                .style(muted),
        );
    }
    if let Some(err) = &popup.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let ok_label = match popup.target {
        Target::New => "Draw",
        Target::Wall(_) => "OK",
    };
    let actions = row![
        Space::new().width(Fill),
        dialog_button(ok_label, Message::SipilWall(Msg::PopupApply), true),
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
