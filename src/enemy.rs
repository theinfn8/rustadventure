use crate::data;
use crate::mob::{Attack, Mob, DamageType};
use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum EnemyEvent {
    /// Halfway through the attack timer: warn the player.
    Prep { text: String },
    /// Timer finished: the attack lands.
    Attack {
        text: String,
        damage: u32,
        damage_type: DamageType,
    },
}
 
/// An attack that has been chosen and is counting down.
#[derive(Debug, Clone)]
struct PendingAttack {
    attack: &'static Attack,
    ticks: u32,
}

#[derive(Debug, Clone)]
pub struct Enemy {
    pub template: &'static Mob,          // shared, read-only definition
    pub current_hits: u32,                 // per-instance state below
    pending: Option<PendingAttack>,
}

impl Enemy {
    pub fn spawn(id: &str) -> Option<Self> {
        let template = data::mob(id)?;
        Some(Self {
            template,
            current_hits: template.max_hits,
            pending: None,
        })
    }

    pub fn name(&self) -> &str {
        &self.template.name
    }

    pub fn max_hits(&self) -> u32 {
        self.template.max_hits
    }

    pub fn is_dead(&self) -> bool {
        self.current_hits == 0
    }

    pub fn take_damage(&mut self, amount: u32) {
        self.current_hits = self.current_hits.saturating_sub(amount);
        if self.is_dead() {
            self.pending = None;
        }
    }

    pub fn heal(&mut self, amount: u32) {
        self.current_hits = (self.current_hits + amount).min(self.max_hits());
    }

    pub fn pending_attack(&self) -> Option<&'static Attack> {
        self.pending.as_ref().map(|p| p.attack)
    }

    pub fn choose_attack(&self, rng: &mut impl Rng) -> Option<&Attack> {
        self.template.choose_attack(rng)
    }

    pub fn ticks_remaining(&self) -> Option<u32> {
        self.pending
            .as_ref()
            .map(|p| p.attack.rate.saturating_sub(p.ticks))
    }

    pub fn tick(&mut self, rng: &mut impl Rng) -> Option<EnemyEvent> {
        if self.is_dead() {
            return None;
        }
 
        if self.pending.is_none() {
            let attack = self.template.choose_attack(rng)?;
            self.pending = Some(PendingAttack { attack, ticks: 0 });
        }
 
        let pending = self.pending.as_mut()?;
        pending.ticks += 1;
        let attack = pending.attack;
 
        if pending.ticks >= attack.rate {
            self.pending = None;
            return Some(EnemyEvent::Attack {
                text: attack.attack_text(self.name()),
                damage: attack.damage,
                damage_type: attack.damage_type,
            });
        }
 
        if pending.ticks == attack.prep_tick() {
            return Some(EnemyEvent::Prep {
                text: attack.prep_text.clone(),
            });
        }
 
        None
    }


}