use std::fmt;

#[derive(Clone, PartialEq, Copy, Debug)]
pub enum Entity {
    FOOD,
    HERBIVORE,
    CARNIVORE,
    WALL,
    OTHER,
}

impl fmt::Display for Entity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}