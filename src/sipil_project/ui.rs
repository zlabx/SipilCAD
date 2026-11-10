//! The project browser's two buttons (New Project, Open Project) under the
//! Sheet Set Manager tree, and the New Project popup (name, template, folder).
//!
//! Widgets follow `sipil_wall/ui.rs` and `ui/window/drawing_units.rs` (same helpers,
//! same look). The UI text is English on purpose and does not go through `t!`.

use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Border, Element, Fill, Length, Theme};

use super::state::ProjectForm;
use super::templates::{template, Template, TemplateId, TEMPLATES};
use crate::app::Message;
use crate::ui::modal::ModalSizing;
use crate::ui::style::form::{dialog_button, dialog_button_styled_opt, form_radio};

#[derive(Debug, Clone)]
pub enum Msg {
    /// New Project button: open the popup.
    New,
    /// Open Project button: pick a sheet set file (desktop).
    Open,
    /// Popup fields.
    Name(String),
    Folder(String),
    Template(TemplateId),
    /// Browse for the folder, and the folder picked.
    Browse,
    Picked(Option<std::path::PathBuf>),
    /// Create the project.
    Create,
}

fn msg(m: Msg) -> Message {
    Message::SipilProject(m)
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

const LABEL_W: f32 = 100.0;

// ── panel buttons ─────────────────────────────────────────────────────────

/// New Project and Open Project, at the bottom of the Sheet Set Manager.
/// A browser cannot read a `.dst` yet, so Open Project is disabled there.
pub fn panel_buttons<'a>() -> Element<'a, Message> {
    #[cfg(not(target_arch = "wasm32"))]
    let open_press = Some(msg(Msg::Open));
    #[cfg(target_arch = "wasm32")]
    let open_press: Option<Message> = None;
    row![
        dialog_button("New Project", msg(Msg::New), true).width(Fill),
        dialog_button_styled_opt("Open Project", open_press, button::secondary).width(Fill),
    ]
    .spacing(6)
    .into()
}

// ── New Project popup ─────────────────────────────────────────────────────

/// The tree the template will create, as plain lines.
fn preview<'a>(t: &'static Template) -> Element<'a, Message> {
    if t.categories.is_empty() {
        return text("Nothing yet: add categories and drawings with right-click.".to_string())
            .size(11)
            .style(muted)
            .into();
    }
    let mut lines = column![].spacing(2);
    for c in t.categories {
        lines = lines.push(text(format!("{}  ({})", c.name, c.drawings.len())).size(11));
        for d in c.drawings {
            lines = lines.push(text(format!("      {d}")).size(10).style(muted));
        }
    }
    scrollable(lines).height(Length::Fixed(170.0)).into()
}

#[cfg(not(target_arch = "wasm32"))]
fn location<'a>(form: &'a ProjectForm) -> Element<'a, Message> {
    let file = super::state::project_file(&form.folder, &form.name);
    column![
        row![
            text("Location".to_string())
                .size(11)
                .style(muted)
                .width(LABEL_W),
            text_input("", &form.folder)
                .on_input(|v| msg(Msg::Folder(v)))
                .size(12)
                .padding([3, 6])
                .width(Fill),
            dialog_button("Browse...", msg(Msg::Browse), false),
        ]
        .spacing(8)
        .align_y(iced::Center),
        text(format!("Project file: {file}")).size(10).style(muted),
    ]
    .spacing(4)
    .into()
}

pub fn new_project_view<'a>(form: &'a ProjectForm, sizing: ModalSizing) -> Element<'a, Message> {
    let current = template(form.template);

    let name = row![
        text("Project name".to_string())
            .size(11)
            .style(muted)
            .width(LABEL_W),
        text_input("Project 1", &form.name)
            .on_input(|v| msg(Msg::Name(v)))
            .on_submit(msg(Msg::Create))
            .size(12)
            .padding([3, 6])
            .width(Fill),
    ]
    .spacing(8)
    .align_y(iced::Center);

    let mut picks = column![].spacing(6);
    for t in TEMPLATES.iter() {
        picks = picks.push(form_radio(t.label, t.id, Some(form.template), |id| {
            msg(Msg::Template(id))
        }));
    }
    let templates = group(
        "Template",
        column![picks, text(current.blurb.to_string()).size(10).style(muted)]
            .spacing(6)
            .into(),
    );
    let tree = group("Project browser", preview(current));

    let mut body = column![
        text("New Project".to_string()).size(14),
        name,
        templates,
        tree
    ]
    .spacing(8);
    #[cfg(not(target_arch = "wasm32"))]
    {
        body = body.push(location(form));
    }
    #[cfg(target_arch = "wasm32")]
    {
        body = body.push(
            text(
                "In the browser a project lasts until you close the tab. \
                 Use the desktop app to keep projects as files."
                    .to_string(),
            )
            .size(10)
            .style(muted),
        );
    }
    if let Some(err) = &form.error {
        body = body.push(text(err.clone()).size(11).style(danger));
    }
    let actions = row![
        Space::new().width(Fill),
        dialog_button("Create", msg(Msg::Create), true),
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
