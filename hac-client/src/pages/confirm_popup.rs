use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Clear, Padding, Paragraph, Widget, Wrap};

pub struct ConfirmPopup<'a> {
    message: String,
    colors: &'a hac_colors::Colors,
}

impl<'a> ConfirmPopup<'a> {
    pub fn new(message: String, colors: &'a hac_colors::Colors) -> Self {
        ConfirmPopup { message, colors }
    }

    fn build_popup(&self) -> Paragraph<'_> {
        let lines = vec![
            self.message.clone().fg(self.colors.normal.yellow).into(),
            "".into(),
            Line::from(vec![
                "(y)es".fg(self.colors.normal.green),
                " ".into(),
                "(n)o".fg(self.colors.normal.red),
            ])
            .centered(),
        ];
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(self.colors.bright.black))
                .padding(Padding::new(2, 2, 1, 1))
                .bg(self.colors.normal.black),
        )
    }
}

impl Widget for ConfirmPopup<'_> {
    fn render(self, size: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        Clear.render(size, buf);
        let popup = self.build_popup();
        popup.render(size, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_with_correct_message() {
        let colors = hac_colors::Colors::default();
        let popup = ConfirmPopup::new("my confirmation message".into(), &colors);
        let lines = vec![
            "my confirmation message".fg(colors.normal.yellow).into(),
            "".into(),
            Line::from(vec![
                "(y)es".fg(colors.normal.green),
                " ".into(),
                "(n)o".fg(colors.normal.red),
            ])
            .centered(),
        ];
        let expected = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(colors.bright.black))
                .padding(Padding::new(2, 2, 1, 1))
                .bg(colors.normal.black),
        );

        let content = popup.build_popup();

        assert_eq!(expected, content);
    }
}
