use crate::model::creature::CreatureId;
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Status {
    Blinded,
    Charmed,
    Deafened,
    Exhaustion(u8),
    Frightened,
    Grappled(CreatureId),
    Incapacitated,
    Invisible,
    Paralyzed,
    Petrified,
    Poisoned,
    Prone,
    Restrained,
    Stunned,
    Unconscious,
}
