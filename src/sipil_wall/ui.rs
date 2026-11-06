//! The wall parameter popup and the building-components pallet (sidebar).
//!
//! Widgets follow `ui/window/drawing_units.rs` (same helpers, same look). The
//! UI text is Indonesian on purpose and does not go through `t!`.

use std::fmt;

use iced::widget::{checkbox, column, container, row, scrollable, text, text_input, Space};
use iced::{Border, Element, Fill, Length, Theme};

use super::engine::{self, Pattern, Spacing, PRESETS};
use super::state::{Fields, Popup, State, Target};
use crate::app::Message;
use crate::ui::dock::{Chrome, PanelId};
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
}

#[derive(Debug, Clone)]
pub enum Msg {
    /// A wall type in the pallet.
    Preset(usize),
    /// A quick field in the pallet.
    Panel(Field),
    /// A field in the popup.
    PopupField(Field),
    /// A wall type inside the popup.
    PopupPreset(usize),
    PopupApply,
    /// "Open the popup when a wall is clicked" toggle.
    OnClick(bool),
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

fn input_row<'a>(
    label: &str,
    value: &str,
    unit: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label.to_string()).size(11).style(muted).width(130),
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
            Spacing::Even => "Rata (bentang sama panjang)",
            Spacing::Fixed => "Tetap (sisa di ujung)",
        })
    }
}

fn pattern_row<'a>(
    label: &str,
    selected: Pattern,
    wrap: fn(Field) -> Message,
) -> Element<'a, Message> {
    let options: Vec<PatternChoice> = Pattern::CHOICES.iter().map(|p| PatternChoice(*p)).collect();
    let picker = iced::widget::pick_list(Some(PatternChoice(selected)), options, |c| c.to_string())
        .on_select(move |c: PatternChoice| wrap(Field::Pattern(c.0)))
        .text_size(12)
        .padding([3, 6])
        .width(Fill);
    row![
        text(label.to_string()).size(11).style(muted).width(130),
        picker
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}

fn spacing_row<'a>(
    label: &str,
    selected: Spacing,
    wrap: fn(Field) -> Message,
) -> Element<'a, Message> {
    let options = vec![SpacingChoice(Spacing::Even), SpacingChoice(Spacing::Fixed)];
    let picker = iced::widget::pick_list(Some(SpacingChoice(selected)), options, |c| c.to_string())
        .on_select(move |c: SpacingChoice| wrap(Field::Spacing(c.0)))
        .text_size(12)
        .padding([3, 6])
        .width(Fill);
    row![
        text(label.to_string()).size(11).style(muted).width(130),
        picker
    ]
    .spacing(8)
    .align_y(iced::Center)
    .into()
}

fn popup_msg(f: Field) -> Message {
    Message::SipilWall(Msg::PopupField(f))
}

fn panel_msg(f: Field) -> Message {
    Message::SipilWall(Msg::Panel(f))
}

/// "Tebal total: 13 cm", live from the typed fields.
fn total_label(f: &Fields) -> String {
    match f.to_params() {
        Ok(p) => format!(
            "Tebal total (bata + plester): {} cm",
            engine::format_number(p.total_m() * 100.0)
        ),
        Err(_) => "Tebal total: —".to_string(),
    }
}

// ── popup ─────────────────────────────────────────────────────────────────

pub fn popup_view<'a>(
    popup: &'a Popup,
    sizing: crate::ui::modal::ModalSizing,
) -> Element<'a, Message> {
    let f = &popup.fields;
    let title = match &popup.target {
        Target::Defaults => "Pengaturan dinding baru".to_string(),
        Target::Wall(id) => format!("Parameter dinding #{id}"),
    };

    let mut presets = column![].spacing(4);
    for pair in PRESETS.chunks(2).enumerate() {
        let (row_idx, chunk) = pair;
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
        "Dimensi",
        column![
            presets,
            input_row("Tebal bata", &f.thickness_cm, "cm", |v| popup_msg(
                Field::Thickness(v)
            )),
            input_row("Plester per sisi", &f.plaster_cm, "cm (0 = tanpa)", |v| {
                popup_msg(Field::Plaster(v))
            }),
            text(total_label(f)).size(11).style(muted),
        ]
        .spacing(6)
        .into(),
    );

    let hatch = group(
        "Arsiran",
        column![
            pattern_row("Pola bata", f.pattern, popup_msg),
            input_row("Skala arsir bata", &f.hatch_scale, "0 = otomatis", |v| {
                popup_msg(Field::HatchScale(v))
            }),
            row![
                Space::new().width(Length::Fixed(130.0)),
                checkbox(f.col_hatch)
                    .on_toggle(|on| popup_msg(Field::ColHatch(on)))
                    .size(14),
                text("Arsir beton kolom (AR-CONC)".to_string()).size(11),
            ]
            .spacing(6)
            .align_y(iced::Center),
            input_row("Skala arsir kolom", &f.col_scale, "0 = otomatis", |v| {
                popup_msg(Field::ColScale(v))
            }),
        ]
        .spacing(6)
        .into(),
    );

    let columns = group(
        "Kolom praktis",
        column![
            text("Kolom selalu dipasang di sudut, ujung, dan titik belok.".to_string())
                .size(10)
                .style(muted),
            input_row(
                "Panjang kolom",
                &f.col_len_cm,
                "cm (0 = tebal total)",
                |v| popup_msg(Field::ColLen(v))
            ),
            input_row(
                "Kolom tambahan tiap",
                &f.interval_m,
                "m (0 = tanpa, mis. 3 atau 4)",
                |v| popup_msg(Field::Interval(v))
            ),
            spacing_row("Penyebaran", f.spacing, popup_msg),
        ]
        .spacing(6)
        .into(),
    );

    let mut body = column![text(title).size(14), size, hatch, columns].spacing(8);
    if let Target::Wall(_) = popup.target {
        body = body.push(
            text("Perubahan menggambar ulang seluruh dinding ini (titik as tetap).".to_string())
                .size(10)
                .style(muted),
        );
    }
    if let Some(err) = &popup.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let actions = row![
        Space::new().width(Fill),
        dialog_button("OK", Message::SipilWall(Msg::PopupApply), true),
        dialog_button("Batal", Message::CloseModal, false),
    ]
    .spacing(8)
    .align_y(iced::Center);

    column![body, actions]
        .spacing(10)
        .padding(10)
        .width(sizing.width)
        .into()
}

// ── pallet ────────────────────────────────────────────────────────────────

fn preset_matches(state: &State, index: usize) -> bool {
    PRESETS.get(index).is_some_and(|p| {
        (state.settings.thickness_m - p.thickness_m).abs() < 1e-9
            && (state.settings.plaster_m - p.plaster_m).abs() < 1e-9
    })
}

/// One category of the pallet. More building components (doors, windows, ...)
/// become more of these.
fn wall_category<'a>(state: &'a State) -> Element<'a, Message> {
    let mut presets = column![].spacing(4);
    for (i, p) in PRESETS.iter().enumerate() {
        presets = presets.push(
            dialog_button(
                p.label,
                Message::SipilWall(Msg::Preset(i)),
                preset_matches(state, i),
            )
            .width(Fill),
        );
    }
    let f = &state.panel;
    group(
        "Dinding",
        column![
            presets,
            input_row("Tebal bata", &f.thickness_cm, "cm", |v| panel_msg(
                Field::Thickness(v)
            )),
            input_row("Plester", &f.plaster_cm, "cm", |v| panel_msg(
                Field::Plaster(v)
            )),
            input_row("Kolom tiap", &f.interval_m, "m", |v| panel_msg(
                Field::Interval(v)
            )),
            text(total_label(f)).size(10).style(muted),
            row![
                dialog_button("Gambar dinding", Message::Command("BW".to_string()), true),
                dialog_button("Pengaturan…", Message::Command("BWSET".to_string()), false),
            ]
            .spacing(6),
            row![
                checkbox(state.popup_on_click)
                    .on_toggle(|on| Message::SipilWall(Msg::OnClick(on)))
                    .size(14),
                text("Popup parameter saat dinding diklik".to_string()).size(11),
            ]
            .spacing(6)
            .align_y(iced::Center),
        ]
        .spacing(6)
        .into(),
    )
}

pub fn panel_view<'a>(state: &'a State, width: f32, chrome: Chrome) -> Element<'a, Message> {
    let title_bar =
        crate::ui::dock::title_bar(PanelId::SipilWall, "Komponen Bangunan".to_string(), chrome);
    let body = column![
        wall_category(state),
        text("Komponen lain (kolom, pintu, jendela) menyusul.".to_string())
            .size(10)
            .style(muted),
    ]
    .spacing(8);
    crate::ui::dock::frame(title_bar, scrollable(body), width)
}
