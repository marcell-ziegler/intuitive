mod creature;
mod monster;
mod player;
mod stats;
mod status;

pub use player::Player;

#[cfg(test)]
mod test {
    use super::*;
}
