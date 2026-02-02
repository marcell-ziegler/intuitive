#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stats {
    pub strength: u8,
    pub dexterity: u8,
    pub constitution: u8,
    pub intelligence: u8,
    pub wisdom: u8,
    pub charisma: u8,
}
impl Stats {
    /// Create a new `Stats` with values clamped to 0 <= x <= 30
    ///
    /// * `str`: Strength
    /// * `dex`: Dexterity
    /// * `con`: Constitution
    /// * `int`: Intelligence
    /// * `wis`: Wisdom
    /// * `cha`: Charisma
    pub fn new(str: u8, dex: u8, con: u8, int: u8, wis: u8, cha: u8) -> Self {
        let strength = str.min(30);
        let dexterity = dex.min(30);
        let constitution = con.min(30);
        let intelligence = int.min(30);
        let wisdom = wis.min(30);
        let charisma = cha.min(30);

        Stats {
            strength,
            dexterity,
            constitution,
            intelligence,
            wisdom,
            charisma,
        }
    }

    fn to_mod(stat: u8) -> i8 {
        if stat == 1 {
            -5
        } else if stat <= 3 {
            -4
        } else if stat <= 5 {
            -3
        } else if stat <= 7 {
            -2
        } else if stat <= 9 {
            -1
        } else if stat <= 11 {
            0
        } else if stat <= 13 {
            1
        } else if stat <= 15 {
            2
        } else if stat <= 17 {
            3
        } else if stat <= 19 {
            4
        } else if stat <= 21 {
            5
        } else if stat <= 23 {
            6
        } else if stat <= 25 {
            7
        } else if stat <= 27 {
            8
        } else if stat <= 29 {
            9
        } else {
            10
        }
    }

    /// Return the strength modifier
    pub fn str_mod(&self) -> i8 {
        Stats::to_mod(self.strength)
    }

    /// Return the dexterity modifier
    pub fn dex_mod(&self) -> i8 {
        Stats::to_mod(self.dexterity)
    }
    /// Return the constitution modifier
    pub fn con_mod(&self) -> i8 {
        Stats::to_mod(self.constitution)
    }
    /// Return the intelligence modifier
    pub fn int_mod(&self) -> i8 {
        Stats::to_mod(self.intelligence)
    }
    /// return the wisdom modifier
    pub fn wis_mod(&self) -> i8 {
        Stats::to_mod(self.wisdom)
    }
    /// Return the charisma modifier
    pub fn cha_mod(&self) -> i8 {
        Stats::to_mod(self.charisma)
    }
}
impl Default for Stats {
    fn default() -> Self {
        Stats {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}
