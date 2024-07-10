use rand::Rng;
use std::{fmt, thread, time::Duration};

use crate::{
    check_life, italics, play_audio, remove_item, turn_screen_red, GameInfo, ItemEnum, TargetEnum,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExtraTurnVars {
    Handcuffed,
    None,
}
impl fmt::Display for ExtraTurnVars {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            ExtraTurnVars::Handcuffed => "Handcuffed",
            ExtraTurnVars::None => "Nothing!",
        };
        write!(f, "{printable}")
    }
}

struct DealerMinorInfo {
    extra_turn_var: ExtraTurnVars,
    shell_knowledge: bool,
}

fn dealer_item_logic(
    game_info: &mut GameInfo,
    mut damage: i8,
    shell_knowledge: bool,
    extra_turn_var: ExtraTurnVars,
) -> DealerMinorInfo {
    let mut dealer_minor_info: DealerMinorInfo = DealerMinorInfo {
        extra_turn_var,
        shell_knowledge,
    };
    let coinflip: bool = rand::thread_rng().gen();
    let mut lives = 0;
    let mut blanks = 0;

    for item in &game_info.shells_vector {
        if *item {
            lives += 1;
        } else {
            blanks += 1;
        }
    }
    'dealer_item_logic: loop {
        if game_info.dealer_stored_items.contains(&ItemEnum::Cigs) & { game_info.dealer_health < 3 }
        {
            item_use(ItemEnum::Cigs, game_info, &mut damage);
            play_audio("dealer_use_cigarettes.ogg");
            thread::sleep(Duration::from_millis(500));

            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::MagGlass)
            && !dealer_minor_info.shell_knowledge
        {
            dealer_minor_info.shell_knowledge =
                item_use(ItemEnum::MagGlass, game_info, &mut damage);
            play_audio("dealer_use_magnifier.ogg");
            thread::sleep(Duration::from_millis(500));
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Saws)
            & dealer_minor_info.shell_knowledge
            & game_info.shells_vector[0]
        {
            item_use(ItemEnum::Saws, game_info, &mut damage);
            play_audio("dealer_use_handsaw.ogg");
            thread::sleep(Duration::from_millis(500));
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Handcuffs) && {
            dealer_minor_info.extra_turn_var != ExtraTurnVars::Handcuffed
        } {
            item_use(ItemEnum::Handcuffs, game_info, &mut damage);
            play_audio("dealer_use_cigarettes.ogg");
            thread::sleep(Duration::from_millis(500));
            dealer_minor_info.extra_turn_var = ExtraTurnVars::Handcuffed;
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Beers)
            && !dealer_minor_info.shell_knowledge & coinflip
        {
            item_use(ItemEnum::Beers, game_info, &mut damage);
            play_audio("dealer_use_beer.ogg");
            thread::sleep(Duration::from_millis(500));
            continue 'dealer_item_logic;
        }
        if game_info.double_or_nothing {
            if game_info.dealer_stored_items.contains(&ItemEnum::Adren) && {
                !game_info.player_inventory.is_empty()
            } {
                item_use(ItemEnum::Adren, game_info, &mut damage);
                play_audio("dealer_use_adrenaline.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
            if game_info.dealer_stored_items.contains(&ItemEnum::BurnPho)
                && lives != 0
                && game_info.shells_vector.len() > 1
            {
                item_use(ItemEnum::BurnPho, game_info, &mut damage);
                play_audio("dealer_use_burner_phone.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
            if game_info.dealer_stored_items.contains(&ItemEnum::Invert) && {
                (dealer_minor_info.shell_knowledge && !game_info.shells_vector[0])
                    || (lives > blanks)
            } {
                item_use(ItemEnum::Invert, game_info, &mut damage);
                play_audio("dealer_use_inverter.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
            if game_info.dealer_stored_items.contains(&ItemEnum::ExpMed)
                && game_info.dealer_health == 2
            {
                item_use(ItemEnum::ExpMed, game_info, &mut damage);
                play_audio("dealer_use_medicine.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
        }
        break dealer_minor_info;
    }
}

pub fn turn(game_info: &mut GameInfo) -> bool {
    let damage: i8 = 1;
    // future goal: add logic for having dealer pick certain items
    let shell_knowledge = false;
    let extra_turn_var = ExtraTurnVars::None;
    let dealer_minor_info = dealer_item_logic(game_info, damage, shell_knowledge, extra_turn_var);

    let choice: bool = if game_info.perfect | dealer_minor_info.shell_knowledge {
        println!("perf");
        game_info.shells_vector[0]
    } else {
        //logic for the dealer's choice
        let mut lives = 0;
        let mut blanks = 0;

        for item in &game_info.shells_vector {
            if *item {
                lives += 1;
            } else {
                blanks += 1;
            }
        }
        //if there are more lives than blanks, choose to shoot player. Vice versa and such.
        lives >= blanks
    };
    let to_be_shot = if choice {
        TargetEnum::Player
    } else {
        TargetEnum::Dealer
    };
    //true means dealer shoots you, false means dealer shoots itself
    let mut extraturn = false;
    if dealer_minor_info.extra_turn_var == ExtraTurnVars::Handcuffed {
        extraturn = true;
    }
    
    match to_be_shot {
        TargetEnum::Player => {
            println!("The dealer points the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();
                println!("Dealer shot you.");
                game_info.player_health -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                italics("click");
            }
        }
        TargetEnum::Dealer => {
            println!("The dealer points the gun at its face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();
                println!("Dealer shot themselves.");
                game_info.dealer_health -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                italics("click");
                println!("Extra turn for dealer.");
                if dealer_minor_info.extra_turn_var != ExtraTurnVars::Handcuffed {
                    extraturn = true;
                }
            }
        }
    }
    
    game_info.shells_vector.remove(0);
    thread::sleep(Duration::from_secs(1));
    check_life(game_info.player_health, game_info.dealer_health);
    extraturn
}

pub fn item_use(item_type: ItemEnum, game_info: &mut GameInfo, damage: &mut i8) -> bool {
    let mut knowledge_of_shell = false;
    match item_type {
        ItemEnum::Cigs => {
            if game_info.dealer_health == 3 {
                panic!(
                "ERROR: THIS CODE SHOULD NOT BE REACHABLE! PLEASE REPORT THIS BUG. The dealer lights one of the cigs."
            );
            } else {
                println!("The dealer lights one of the cigs.");
                game_info.dealer_health += 1;
            }
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Saws => {
            println!("Shhk. The dealer slices off the tip of the gun. It'll do 2 damage now.");
            *damage = 2;
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::MagGlass => {
            println!(
                "The dealer looks down at the gun with an old magnifying glass. You see him smirk."
            );
            knowledge_of_shell = true;

            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Beers => {
            if game_info.shells_vector[0] {
                println!("The dealer gives the shotgun a pump. A live round drops out.");
            } else {
                println!("The dealer gives the shotgun a pump. A blank round drops out.");
            };
            game_info.shells_vector.remove(0);

            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Handcuffs => {
            println!("The dealer grabs onto your hand. When they let go, your hands are cuffed.");
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Nothing => {
            panic!("ERROR: THIS CODE SHOULD NOT BE REACHABLE! PLEASE REPORT THIS BUG.");
        }

        ItemEnum::Adren => {
            println!("The dealer takes a hit of the adrenaline.");
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::BurnPho => {
            println!("The dealer uses the burner phone.");
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Invert => {
            println!("The dealer uses the inverter.");
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::ExpMed => {
            println!("The dealer takes the expired medicine.");
            let coinflip: bool = rand::thread_rng().gen();
            if coinflip {
                game_info.dealer_health += 1;
                println!("The dealer smiles.");
            } else {
                game_info.dealer_health -= 2;
                println!("The dealer chokes and falls over.");
            }
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
    }
    thread::sleep(Duration::from_millis(500));
    knowledge_of_shell
}
pub fn picked_to_stored(
    mut picked_items_vec_dealer: Vec<ItemEnum>,
    dealer_stored_items: &mut [ItemEnum; 8],
) -> [ItemEnum; 8] {
    for dealer_item in dealer_stored_items.iter_mut() {
        if { *dealer_item == ItemEnum::Nothing } & { !picked_items_vec_dealer.is_empty() } {
            *dealer_item = picked_items_vec_dealer.remove(0);
        }
    }
    *dealer_stored_items
}
