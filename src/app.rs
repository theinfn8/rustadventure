use std::time::{Duration, Instant};
use ratatui::crossterm::event::{self, Event};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::{Constraint, Layout, Margin, Spacing};
use ratatui::symbols::merge::MergeStrategy;

use crate::enemy::{Enemy, EnemyEvent};
use crate::menus::{handle_key, hint_text};
use crate::mob::DamageType;

const HEART_RATE: Duration = Duration::from_millis(100);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppLocation {
    Town(TownLocation),
    Dungeon(DungeonLocation),
}

impl AppLocation {
    /// Name used in messages, e.g. "You enter the guild."
    pub fn name(&self) -> &'static str {
        match self {
            AppLocation::Town(TownLocation::TownSquare) => "town square",
            AppLocation::Town(TownLocation::Guild) => "guild",
            AppLocation::Town(TownLocation::Blacksmith) => "blacksmith",
            AppLocation::Town(TownLocation::Arcanist) => "arcanist",
            AppLocation::Town(TownLocation::Trainer) => "trainer",
            AppLocation::Dungeon(_) => "dungeon",
        }
    }

    /// Flavor text shown when the player arrives. `None` means no description.
    pub fn description(&self) -> Option<&'static str> {
        match self {
            AppLocation::Town(TownLocation::TownSquare) => Some(
                "Merchants shout over one another as adventurers come and go around the old fountain.",
            ),
            AppLocation::Town(TownLocation::Guild) => Some(
                "Smoke hangs under the rafters. Every table holds someone you'd rather not owe money to.",
            ),
            AppLocation::Town(TownLocation::Blacksmith) => Some(
                "Heat rolls out of the forge. The smith doesn't look up from the anvil.",
            ),
            AppLocation::Town(TownLocation::Arcanist) => Some(
                "Shelves of glowing vials line the walls, and the air tastes faintly of copper.",
            ),
            AppLocation::Town(TownLocation::Trainer) => Some(
                "The practice yard is packed dirt, battered dummies, and one very patient instructor.",
            ),
            AppLocation::Dungeon(DungeonLocation::Exploring) => Some(
                "Cold, damp stone closes in around you. Somewhere ahead, something skitters.",
            ),
            AppLocation::Dungeon(_) => None, // fights, chests and traps describe themselves
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TownLocation {
    TownSquare,
    Guild,
    Blacksmith,
    Arcanist,
    Trainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DungeonLocation {
    Entrance,
    Exploring,
    Monster,
    Treasure,
    Trap,

}
pub struct App {
    pub running: bool,
    pub location: AppLocation,
    pub active_enemy: Option<Enemy>,
    pub buffer: Vec<String>,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            location: AppLocation::Town(TownLocation::TownSquare),
            active_enemy: None,
            buffer: Vec::new(),
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> std::io::Result<()> {
        let mut last_heartbeat = Instant::now();
        self.buffer.push("Welcome to the adventure!".to_string());

        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            let timeout = HEART_RATE.saturating_sub(last_heartbeat.elapsed());

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    handle_key(self, key);
                }
            }

            if last_heartbeat.elapsed() >= HEART_RATE {
                // Add heartbeat logic here
                last_heartbeat = Instant::now();
                self.on_tick();
            }
        }

        Ok(())
    }

    

    fn on_tick(&mut self) {
        // Add heartbeat logic here
        let mut rng = rand::rng();

        let event = match self.location {
            AppLocation::Dungeon(DungeonLocation::Monster) => self
                .active_enemy
                .as_mut()
                .and_then(|enemy| enemy.tick(&mut rng)),
            _ => None,
        };
        match event {
            Some(EnemyEvent::Prep { text }) => self.on_enemy_prep(text),
            Some(EnemyEvent::Attack { text, damage, damage_type }) => {
                self.on_enemy_attack(text, damage, damage_type)
            },
            None => {}
        };
    }

    fn on_enemy_prep(&mut self, text: String) {
        self.buffer.push(text);
    }

    fn on_enemy_attack(&mut self, text: String, damage: u32, damage_type: DamageType) {
        // TODO: apply damage to the player once player stats exist.
        self.buffer.push(format!("{text} ({damage} {damage_type} damage)"));
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let [left, right] = Layout::horizontal([
            Constraint::Percentage(70),
            Constraint::Percentage(30)
        ])
            .spacing(Spacing::Overlap(1))
            .areas(frame.area());
        let [description, input] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(3)
        ])
            .spacing(Spacing::Overlap(1))
            .areas(left);
        let [stats, inv, abilities, map] = Layout::vertical([
            Constraint::Length(12),
            Constraint::Length(7),
            Constraint::Fill(1),
            Constraint::Length(5)
        ])
            .spacing(Spacing::Overlap(1))
            .areas(right);

        let content = Paragraph::new(self.buffer.join("\n"))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("")
                    .merge_borders(MergeStrategy::Exact)
            );

        // Count lines after word-wrapping at the pane's inner width (minus the
        // two side borders). line_count() includes the top/bottom borders, so
        // comparing it to the full pane height gives how far to scroll so the
        // newest text sits at the bottom.
        let inner_width = description.width.saturating_sub(2);
        let total_lines = content.line_count(inner_width);
        let max_scroll = total_lines.saturating_sub(description.height as usize);
        let content = content.scroll((u16::try_from(max_scroll).unwrap_or(u16::MAX), 0));

        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(max_scroll);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None);

        let hintbar = Paragraph::new(hint_text(self)).block(
            Block::default()
                .borders(Borders::ALL)
                .merge_borders(MergeStrategy::Exact),
        );

        // Placeholder stats for now
        let stat1 = Paragraph::new("Hits: 100\nMana: 50\nLevel: 1")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Stats")
                    .merge_borders(MergeStrategy::Exact)
            );

        frame.render_widget(content, description);
        if max_scroll > 0 {
            frame.render_stateful_widget(scrollbar, description.inner(Margin { vertical: 1, horizontal: 0 }), &mut scrollbar_state);
        }
        frame.render_widget(hintbar, input);
        //frame.render_widget(Paragraph::new("(Q)uit"), input);
        frame.render_widget(stat1, stats);
        frame.render_widget(Block::bordered().title("Inventory").merge_borders(MergeStrategy::Exact), inv);
        frame.render_widget(Block::bordered().title("Abilities").merge_borders(MergeStrategy::Exact), abilities);
        frame.render_widget(Block::bordered().title("Map").merge_borders(MergeStrategy::Exact), map);

    }

}