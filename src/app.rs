use std::time::{Duration, Instant};
use ratatui::crossterm::event::{self, Event};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{DefaultTerminal, Frame};
use ratatui::layout::{Constraint, Layout, Margin, Spacing};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::text::Line;

use crate::enemy::{Enemy, EnemyEvent};
use crate::menus::{handle_key, hint_text};
use crate::mob::DamageType;
use crate::character::Character;
use crate::inventory::InventoryItem;
use crate::popup::Popup;

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
            AppLocation::Town(TownLocation::Bank) => "bank",
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
            AppLocation::Town(TownLocation::Bank) => Some(
                "The bank is quiet with a stark solemnity as the bankers efficiently carry out their business.",
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
    Bank,
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
    pub character: Character,
    pub popup: Option<Popup>,
}

impl App {
    pub fn new() -> Self {
        // TEMP: starting test gear
        let mut character = Character::new("Chris");
        character.gear.main_hand = InventoryItem::new("superior_sword");
        character.gear.head = InventoryItem::new("leather_cap");
        character.gear.chest = InventoryItem::new("leather_jerkin");
        character.gear.gloves = InventoryItem::new("leather_gloves");
        character.gear.feet = InventoryItem::new("soft_leather_boots");
        for id in ["healing_potion", "healing_potion", "bandage", "lockpick"] {
            if let Some(item) = InventoryItem::new(id) {
                let _ = character.inventory.add(item);
            }
        }
        character.gold = 120;
        // TEMP: starting test gear

        Self {
            running: true,
            location: AppLocation::Town(TownLocation::TownSquare),
            active_enemy: None,
            buffer: Vec::new(),
            character: character,
            popup: None,
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

    fn panel(title: &str) -> Block<'_> {
        Block::bordered().title(title).merge_borders(MergeStrategy::Exact)
    }

    /// Name, level/XP, hits and skills. `inner` is the box's width inside its borders.
    fn stats_widget(&self, inner: u16) -> Paragraph<'static> {
        let c = &self.character;
        let base = c.skills;
        let max = c.total_skills();
        let current = max; // TODO: use the run's pools once DungeonRun exists

        let half = (inner / 2) as usize;
        let skill_line = |name: &str, base: i32, current: i32, max: i32| {
            Line::from(format!("{name:<9}({base}): {current}/{max}"))
        };

        let lines = vec![
            Line::from(c.name.clone()).centered(),
            Line::from(format!("{:<half$}{}", format!("Level: {}", c.level), format!("XP: {}", c.xp))),
            Line::from(format!("Hits: {}/{}", c.current_hits, c.max_hits)),
            skill_line("Combat", base.combat, current.combat, max.combat),
            skill_line("Defense", base.defense, current.defense, max.defense),
            skill_line("Arcana", base.arcana, current.arcana, max.arcana),
            skill_line("Thievery", base.thievery, current.thievery, max.thievery),
        ];
        Paragraph::new(lines).block(Self::panel("Stats"))
    }

    /// Four hotkey slots, numbered from `first_key`. Placeholder until abilities exist.
    fn slots_widget(title: &'static str, first_key: u32) -> Paragraph<'static> {
        let lines: Vec<Line> = (first_key..first_key + 4)
            .map(|key| Line::from(format!("{key} -")))
            .collect();
        Paragraph::new(lines).block(Self::panel(title))
    }

    /// Gold, then the bag's contents. The title shows slots used out of the maximum.
    fn inventory_widget(&self) -> Paragraph<'static> {
        let c = &self.character;
        let mut lines = vec![Line::from(format!("Gold: {}", c.gold)), Line::from("")];
        lines.extend(c.inventory.iter().map(|entry| Line::from(entry.label())));
        if c.inventory.is_empty() {
            lines.push(Line::from("(empty)"));
        }
        let title = format!("(I)nventory {}/{}", c.inventory.len(), c.inventory_slots());
        Paragraph::new(lines).block(Block::bordered().title(title).merge_borders(MergeStrategy::Exact))
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

        let [stats, attacks, defenses, inv] = Layout::vertical([
            Constraint::Length(9), // name, level/xp, hits, 4 skills + borders
            Constraint::Length(6), // 4 slots + borders
            Constraint::Length(6),
            Constraint::Fill(1),
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

        let stats_box = self.stats_widget(stats.width.saturating_sub(2));
        let attacks_box = Self::slots_widget("(A)ttack", 1);
        let defenses_box = Self::slots_widget("(D)efense", 5);
        let inventory_box = self.inventory_widget();

        frame.render_widget(content, description);
        if max_scroll > 0 {
            frame.render_stateful_widget(scrollbar, description.inner(Margin { vertical: 1, horizontal: 0 }), &mut scrollbar_state);
        }
        frame.render_widget(hintbar, input);
        frame.render_widget(stats_box, stats);
        frame.render_widget(attacks_box, attacks);
        frame.render_widget(defenses_box, defenses);
        frame.render_widget(inventory_box, inv);

        if let Some(popup) = self.popup.as_mut() {
            popup.render(frame);
        }
    }

}