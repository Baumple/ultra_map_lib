use std::{
    fs::File,
    io::{Read, Write},
    num::TryFromIntError,
};

use thiserror::Error;

const MAP_SIZE: usize = 256;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error occurred while converting map")]
    UltraMapConversionError,

    #[error("Error occurred while reading map")]
    UltraMapParsingError(#[from] TryFromIntError),

    #[error("Error occurred while writing to file")]
    UltraMapIoError(#[from] std::io::Error),

    #[error("Invalid coordinates")]
    UltraMapIndexOutOfBounds,
}

/// Each map is 16x16, each cell can range from -50 to 50 (0 is base height).
/// The level_map describes the height level while prefab_map indicates if and what prefabs should be placed on the cell.
#[derive(Debug)]
pub struct MapPattern {
    level_map: [i8; MAP_SIZE],
    prefab_map: [char; MAP_SIZE],
}

impl Default for MapPattern {
    fn default() -> Self {
        Self {
            level_map: [0; MAP_SIZE],
            prefab_map: ['0'; MAP_SIZE],
        }
    }
}

impl MapPattern {
    pub fn from(path: &str) -> Result<Self, Error> {
        let mut file = File::open(path)?;
        let mut input = String::new();

        file.read_to_string(&mut input)?;

        let mut level_map = [0; 256];
        let mut prefab_map = ['0'; 256];

        let mut in_parentheses = false;

        let mut temp = String::new();
        let mut index = 0;

        for c in input.chars() {
            if c.is_whitespace() {
                continue;
            }

            if c == '(' {
                in_parentheses = true;
                continue;
            } else if c == ')' {
                in_parentheses = false;
                level_map[index] = temp.parse().unwrap();
                temp.clear();
                index += 1;
                continue;
            }

            if in_parentheses {
                temp.push(c);
                continue;
            }

            if index < 256 {
                let digit = char::to_digit(c, 10).ok_or(Error::UltraMapConversionError)?;
                let digit: i8 = i8::try_from(digit)?;
                level_map[index] = digit;
            } else {
                prefab_map[index - 256] = c;
            }
            index += 1;
        }

        Ok(Self {
            level_map,
            prefab_map,
        })
    }
    pub fn level_map(&self) -> &[i8] {
        self.level_map.as_slice()
    }

    pub fn prefab_map(&self) -> &[char] {
        self.prefab_map.as_slice()
    }

    pub fn get_map_raw(&self) -> String {
        let mut returnee = String::new();
        for i in self.level_map.iter() {
            let c = char::from_digit(*i as u32, 10).unwrap();
            returnee.push(c);
        }

        returnee.push('\n');

        for c in self.prefab_map {
            returnee.push(c);
        }

        returnee
    }

    /// set height level of tile
    /// Note! level cannot be higher than 50 or lower than -50
    pub fn set_level_at(&mut self, x: usize, y: usize, level: i8) {
        let index = x * y;
        if index >= 256 {
            panic!("Invalid");
        }
        if !(-50..50).contains(&level) {
            panic!("Level cannot be greater than 50 or lower than -50")
        }
        self.level_map[x * y] = level;
    }

    pub fn set_level_at_index(&mut self, index: usize, level: i8) {
        if index >= 256 {
            panic!("Index out of bounds")
        }
        if level > 50 {
            panic!("Level cannot be higher than 50 or lower -50")
        }

        self.level_map[index] = level;
    }

    /// set prefab at given tile
    pub fn set_prefab_at(&mut self, x: usize, y: usize, prefab: Prefab) {
        self.prefab_map[x * y] = Prefab::match_char(prefab);
    }

    pub fn save_pattern(&self, name: &str) -> Result<(), Error> {
        let mut save = String::new();
        let mut f = File::create(format!("{}.cgp", name))?;

        for (index, i) in self.level_map.iter().enumerate() {
            let c = if i.to_string().len() > 1 {
                format!("({})", i)
            } else {
                i.to_string()
            };
            if index % 16 == 0 && index > 0 {
                save.push('\n');
            }
            save.push_str(&c);
        }

        save.push('\n');
        save.push('\n');

        for (index, c) in self.prefab_map.as_slice().iter().enumerate() {
            if index % 16 == 0 && index > 0 {
                save.push('\n');
            }
            save.push(*c);
        }

        writeln!(f, "{}", save)?;
        Ok(())
    }
}

pub enum Prefab {
    Melee,
    Projectile,
    JumpPad,
    Stairs,
    Hideous,
}

impl Prefab {
    fn match_char(prefab: Prefab) -> char {
        match prefab {
            Prefab::Melee => 'n',
            Prefab::Projectile => 'p',
            Prefab::JumpPad => 'J',
            Prefab::Stairs => 's',
            Prefab::Hideous => 'H',
        }
    }
}
