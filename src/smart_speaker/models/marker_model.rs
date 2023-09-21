use std::fmt;

#[derive(Debug, PartialEq)]
pub(crate) enum IngredientMarker {
    Carrot,
    Tomato,
    Onion,
    Potato,
    Eggplant,
    Cabbage,
    Pumpkin,
    Broccoli,
    GreenPepper,
    Unknown,
}

impl fmt::Display for IngredientMarker {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            IngredientMarker::Carrot => write!(f, "carrot"),
            IngredientMarker::Tomato => write!(f, "tomato"),
            IngredientMarker::Onion => write!(f, "onion"),
            IngredientMarker::Potato => write!(f, "potato"),
            IngredientMarker::Eggplant => write!(f, "eggplant"),
            IngredientMarker::Cabbage => write!(f, "cabbage"),
            IngredientMarker::Pumpkin => write!(f, "pumpkin"),
            IngredientMarker::Broccoli => write!(f, "broccoli"),
            IngredientMarker::GreenPepper => write!(f, "green pepper"),
            IngredientMarker::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<u32> for IngredientMarker {
    fn from(value: u32) -> Self {
        match value {
            0 => IngredientMarker::Carrot,
            1 => IngredientMarker::Tomato,
            2 => IngredientMarker::Onion,
            3 => IngredientMarker::Potato,
            4 => IngredientMarker::Eggplant,
            5 => IngredientMarker::Cabbage,
            6 => IngredientMarker::Pumpkin,
            7 => IngredientMarker::Broccoli,
            8 => IngredientMarker::GreenPepper,
            _ => IngredientMarker::Unknown,
        }
    }
}
