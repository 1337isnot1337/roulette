use std::{fmt, sync::{mpsc::Receiver, Mutex, OnceLock}};
use crossterm::event::Event;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
    Play,
    Help,
    Credits,
}
impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Selection::Play => "Play",
            Selection::Help => "Help",
            Selection::Credits => "Credits",
        };
        write!(f, "{printable}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEnum {
    Cigs,
    Saws,
    MagGlass,
    Beers,
    Handcuffs,
    Adren,
    BurnPho,
    Invert,
    ExpMed,
    Nothing,
}
impl fmt::Display for ItemEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            ItemEnum::Cigs => "Cigarettes",
            ItemEnum::Saws => "Saw",
            ItemEnum::MagGlass => "Magnifying Glass",
            ItemEnum::Beers => "Beer",
            ItemEnum::Handcuffs => "Handcuffs",
            ItemEnum::Adren => "Adrenaline",
            ItemEnum::BurnPho => "Burner Phone",
            ItemEnum::Invert => "Inverter",
            ItemEnum::ExpMed => "Expired Medicine",
            ItemEnum::Nothing => "No item",
        };
        write!(f, "{printable}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInfo {
    pub dealer_health: i8,
    pub player_health: i8,
    pub turn_owner: PlayerDealer,
    pub player_inventory: [ItemEnum; 8],
    pub dealer_stored_items: [ItemEnum; 8],
    pub perfect: bool,
    pub double_or_nothing: bool,
    pub debug: bool,
    pub shells_vector: Vec<bool>,
    pub current_turn: i32,
    pub shell_index: usize,
    pub dealer_shell_knowledge_vec: Vec<Option<bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerDealer {
    Player,
    Dealer,
}
impl fmt::Display for PlayerDealer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            PlayerDealer::Player => "self",
            PlayerDealer::Dealer => "dealer",
        };
        write!(f, "{printable}")
    }
}

pub static STDIN: OnceLock<Mutex<Receiver<Event>>> = OnceLock::new();