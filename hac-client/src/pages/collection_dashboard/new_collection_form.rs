use crate::pages::input::Input;

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, Padding, Paragraph, StatefulWidget, Widget,
};

#[derive(Debug, Default, PartialEq, Eq)]
pub enum FormFocus {
    #[default]
    Name,
    Description,
    Confirm,
    Cancel,
}

struct FormLayout {
    name_input: Rect,
    desc_input: Rect,
    confirm_button: Rect,
    cancel_button: Rect,
    hint: Rect,
}

#[derive(Debug, Default)]
pub struct FormState {
    pub name: String,
    pub description: String,
    pub focused_field: FormFocus,
}

impl FormState {
    pub fn reset(&mut self) {
        self.name = String::default();
        self.description = String::default();
        self.focused_field = FormFocus::Name;
    }
}

#[derive(Debug)]
pub struct NewCollectionForm<'a> {
    colors: &'a hac_colors::Colors,
}

impl<'a> NewCollectionForm<'a> {
    pub fn new(colors: &'a hac_colors::Colors) -> Self {
        NewCollectionForm { colors }
    }

    fn build_layout(&self, size: &Rect) -> FormLayout {
        let size = Rect {
            x: size.x + 2,
            y: size.y + 1,
            width: size.width.saturating_sub(4),
            height: size.height.saturating_sub(2),
        };
        let [name_input, desc_input, _, buttons, _, hint] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .areas(size);

        let [confirm_button, _, cancel_button] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10),
                Constraint::Length(1),
                Constraint::Length(10),
            ])
            .flex(Flex::Center)
            .areas(buttons);

        FormLayout {
            name_input,
            desc_input,
            confirm_button,
            cancel_button,
            hint,
        }
    }
}

impl StatefulWidget for NewCollectionForm<'_> {
    type State = FormState;

    fn render(self, size: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let layout = self.build_layout(&size);
        Clear.render(size, buf);

        let mut name_input =
            Input::new(self.colors, "Name".into()).placeholder("My awesome API".into());

        let mut desc_input =
            Input::new(self.colors, "Description".into()).placeholder("Request testing".into());

        match state.focused_field {
            FormFocus::Name => name_input.focus(),
            FormFocus::Description => desc_input.focus(),
            _ => {}
        };

        let cancel_text = if state.focused_field.eq(&FormFocus::Cancel) {
            "Cancel"
                .fg(self.colors.normal.white)
                .bg(self.colors.normal.red)
        } else {
            "Cancel".fg(self.colors.normal.white)
        };

        let cancel_button = Paragraph::new(Line::from(cancel_text).centered()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.colors.bright.red))
                .border_type(BorderType::Rounded),
        );

        let confirm_text = if state.focused_field.eq(&FormFocus::Confirm) {
            "Create"
                .fg(self.colors.normal.white)
                .bg(self.colors.normal.magenta)
        } else {
            "Create".fg(self.colors.normal.white)
        };

        let confirm_button = Paragraph::new(Line::from(confirm_text).centered()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.colors.bright.magenta))
                .border_type(BorderType::Rounded),
        );

        let full_block = Block::default()
            .padding(Padding::uniform(1))
            .style(Style::default().bg(self.colors.primary.background));

        let hint = Paragraph::new("[Tab] to switch focus [Enter] to select a button")
            .centered()
            .fg(self.colors.normal.magenta);

        full_block.render(size, buf);
        name_input.render(layout.name_input, buf, &mut state.name);
        desc_input.render(layout.desc_input, buf, &mut state.description);
        cancel_button.render(layout.cancel_button, buf);
        confirm_button.render(layout.confirm_button, buf);
        hint.render(layout.hint, buf);
    }
}
