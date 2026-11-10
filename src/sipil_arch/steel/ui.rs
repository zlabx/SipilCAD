//! The profile picker widget shared by the column and beam popups: maker,
//! kind gallery (SipilGen's pictures), profile list, and a drawing of the
//! chosen section with its sizes. English on purpose; no `t!`.

use std::fmt;

use iced::widget::{button, column, container, pick_list, row, svg, text, text_input, Space};
use iced::{Border, Element, Fill, Length, Theme};

use super::catalog;
use super::pick::{Pick, PickMsg};
use super::shape;
use crate::app::Message;
use crate::sipil_wall::engine::format_number;
use crate::ui::icons::semantic;
use crate::ui::style::form::button_style;

#[derive(Clone, PartialEq)]
struct Name(&'static str);

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

pub fn muted(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(theme.palette().background.base.text.scale_alpha(0.6)),
    }
}

pub fn danger(theme: &Theme) -> iced::widget::text::Style {
    iced::widget::text::Style {
        color: Some(theme.palette().danger.base.color),
    }
}

/// A titled box around a block of fields.
pub fn group<'a>(title: &str, body: Element<'a, Message>) -> Element<'a, Message> {
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

pub const LABEL_W: f32 = 120.0;

/// A labelled text field with its unit.
pub fn input_row<'a>(label: &str, value: &str, unit: &str, on_input: impl Fn(String) -> Message + 'a) -> Element<'a, Message> {
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

/// A text button of a choice, highlighted when chosen.
pub fn choice<'a>(label: &str, selected: bool, on_press: Message) -> Element<'a, Message> {
    button(text(label.to_string()).size(12))
        .on_press(on_press)
        .padding([4, 14])
        .style(button_style(selected))
        .into()
}

/// The picker. `wrap` turns the picker's messages into the component's own.
pub fn picker<'a>(pick: &'a Pick, wrap: fn(PickMsg) -> Message) -> Element<'a, Message> {
    let mut makers = row![].spacing(6);
    for m in catalog::manufacturers() {
        makers = makers.push(
            button(text(format!("{} ({})", m.name, m.country.to_uppercase())).size(12))
                .on_press(wrap(PickMsg::Mfr(m.id)))
                .padding([4, 12])
                .style(button_style(m.id == pick.mfr)),
        );
    }

    // The kinds this maker makes, five to a row.
    let kinds = pick.kinds();
    let mut gallery = column![].spacing(6);
    for chunk in kinds.chunks(5) {
        let mut r = row![].spacing(6);
        for (kind, label) in chunk {
            let content = column![semantic(kind.icon(), 30.0), text(label.to_string()).size(10)]
                .spacing(2)
                .align_x(iced::Center)
                .width(Fill);
            r = r.push(
                button(content)
                    .on_press(wrap(PickMsg::Kind(*kind)))
                    .padding(5)
                    .width(Fill)
                    .style(button_style(*kind == pick.kind)),
            );
        }
        for _ in chunk.len()..5 {
            r = r.push(Space::new().width(Fill));
        }
        gallery = gallery.push(r);
    }

    let chosen = pick.profile();
    let names: Vec<Name> = pick.profiles().iter().map(|p| Name(p.name)).collect();
    let list = pick_list(Some(Name(chosen.name)), names, |n| n.to_string())
        .on_select(move |n: Name| wrap(PickMsg::Profile(n.0)))
        .text_size(12)
        .padding([3, 6])
        .width(Fill);

    let mut sizes = vec![format!("h {}", format_number(chosen.h)), format!("b {}", format_number(chosen.b))];
    for (label, v) in [("tw", chosen.tw), ("tf", chosen.tf), ("t", chosen.t)] {
        if v > 0.0 {
            sizes.push(format!("{label} {}", format_number(v)));
        }
    }
    let info = column![
        text(format!("{} mm", sizes.join("  ·  "))).size(11).style(muted),
        text(format!("A = {:.2} cm²   ·   {:.2} kg/m", chosen.area_cm2, chosen.kg_m)).size(11).style(muted),
    ]
    .spacing(2);

    let preview = svg(svg::Handle::from_memory(shape::svg(chosen, 110.0).into_bytes()))
        .width(Length::Fixed(110.0))
        .height(Length::Fixed(110.0));

    column![
        makers,
        gallery,
        row![column![list, info].spacing(6).width(Fill), preview].spacing(10).align_y(iced::Center),
    ]
    .spacing(8)
    .into()
}
