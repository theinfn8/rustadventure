use ratatui::layout::Constraint;
use ratatui::style::{Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Clear, List, ListItem, ListState};
use ratatui::Frame;

/// What the popup is for. Decides what happens when the player picks an entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    Inventory,
    Attacks,
    Defenses,
}

/// A centered list the player scrolls with the arrow keys and picks from with Enter.
#[derive(Debug, Clone)]
pub struct Popup {
    pub kind: PopupKind,
    pub title: String,
    items: Vec<String>,
    state: ListState,
}

impl Popup {
    pub fn new(kind: PopupKind, title: impl Into<String>, items: Vec<String>) -> Self {
        let selected = if items.is_empty() { None } else { Some(0) };
        Self {
            kind,
            title: title.into(),
            items,
            state: ListState::default().with_selected(selected),
        }
    }

    /// Index of the highlighted entry, or None if the list is empty.
    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    /// Move the highlight down, wrapping from the last entry to the first.
    pub fn next(&mut self) {
        if let Some(i) = self.state.selected() {
            self.state.select(Some((i + 1) % self.items.len()));
        }
    }

    /// Move the highlight up, wrapping from the first entry to the last.
    pub fn previous(&mut self) {
        if let Some(i) = self.state.selected() {
            let len = self.items.len();
            self.state.select(Some((i + len - 1) % len));
        }
    }

    /// Draw on top of everything else, centered on the screen.
    pub fn render(&mut self, frame: &mut Frame) {
        // Two border rows plus one row per entry (at least one, for the "empty" text).
        let height = self.items.len().max(1) as u16 + 2;
        let area = frame
            .area()
            .centered(Constraint::Percentage(40), Constraint::Length(height));

        let block = Block::bordered()
            .title(Line::from(self.title.as_str()).centered())
            .border_style(Style::new().yellow());

        frame.render_widget(Clear, area); // erase what's underneath

        if self.items.is_empty() {
            let empty = List::new([ListItem::new("(nothing here)")]).block(block);
            frame.render_widget(empty, area);
        } else {
            let list = List::new(self.items.iter().map(|s| ListItem::new(s.as_str())))
                .block(block)
                .highlight_symbol("> ")
                .highlight_style(Style::new().reversed());
            frame.render_stateful_widget(list, area, &mut self.state);
        }
    }
}