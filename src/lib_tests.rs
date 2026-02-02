use super::model::*;

#[test]
fn test_player_new_and_getters() {
    let player = Player::new("Alice", 30, 15, None, None);
    assert_eq!(player.name(), "Alice");
    assert_eq!(player.max_hp(), 30);
    assert_eq!(player.hp(), 30);
    assert_eq!(player.ac(), 15);
    assert_eq!(*player.stats(), Stats::default());
    assert!(player.is_alive());
    assert!(!player.is_dead());
}

#[test]
fn test_player_new_with_current_hp() {
    let player = Player::new("Bob", 40, 12, Some(25), None);
    assert_eq!(player.hp(), 25);

    let player = Player::new("Carol", 40, 12, Some(50), None);
    assert_eq!(player.hp(), 40); // invalid current_hp defaults to max_hp
}

#[test]
fn test_heal_and_damage() {
    let mut player = Player::new("Dave", 20, 10, Some(10), None);
    player.heal(5);
    assert_eq!(player.hp(), 15);

    player.heal(10);
    assert_eq!(player.hp(), 20); // should not exceed max_hp

    let outcome = player.damage(5);
    assert_eq!(outcome, DamageOutcome::Survived);
    assert_eq!(player.hp(), 15);

    let outcome = player.damage(30);
    assert_eq!(outcome, DamageOutcome::Downed);
    assert_eq!(player.hp(), 0);

    let outcome = player.damage(20);
    assert_eq!(outcome, DamageOutcome::Died);
    assert!(player.is_dead());
}

#[test]
fn test_statuses() {
    let mut player = Player::new("Eve", 10, 10, None, None);
    player.add_status(Status::Blinded);
    assert!(player.get_statuses().contains(&Status::Blinded));

    player.add_status(Status::Blinded);
    assert_eq!(
        player
            .get_statuses()
            .iter()
            .filter(|s| **s == Status::Blinded)
            .count(),
        1
    );

    player.add_status(Status::Poisoned);
    assert!(player.get_statuses().contains(&Status::Poisoned));

    assert!(player.remove_status(Status::Blinded).is_some());
    assert!(!player.get_statuses().contains(&Status::Blinded));

    player.clear_status();
    assert!(player.get_statuses().is_empty());
}

#[test]
fn test_initiative() {
    let mut player = Player::new("Frank", 10, 10, None, None);
    let roll = player.roll_initiative();
    assert!((1..=20).contains(&roll));
    assert_eq!(player.get_initiative(), roll);

    player.set_initiative(15);
    assert_eq!(player.get_initiative(), 15);

    player.clear_initative();
    let new_roll = player.get_initiative();
    assert!((1..=20).contains(&new_roll));
}
