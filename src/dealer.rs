use crate::{local_ratatui::message_stats_func, message_top_func};
use rand::Rng;
use std::{fmt, thread, time::Duration};

use crate::{
    check_life, message_top, play_audio, remove_item, turn_screen_red, GameInfo, ItemEnum,
    TargetEnum,
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
    damage: i8,
}
#[allow(clippy::too_many_lines)]
fn dealer_item_logic(
    game_info: &mut GameInfo,
    damage: i8,
    mut shell_knowledge: bool,
    extra_turn_var: ExtraTurnVars,
) -> DealerMinorInfo {
    if game_info.perfect {shell_knowledge = true}
    message_stats_func(game_info);
    let mut dealer_minor_info: DealerMinorInfo = DealerMinorInfo {
        extra_turn_var,
        shell_knowledge,
        damage,
    };

    let coinflip: bool = rand::thread_rng().gen();
    let mut lives = 0;
    let mut blanks = 0;
    let mut saw_used = false;

    for item in &game_info.shells_vector {
        if *item {
            lives += 1;
        } else {
            blanks += 1;
        }
    }
    message_stats_func(game_info);
    'dealer_item_logic: loop {
        message_stats_func(game_info);
        if game_info.dealer_stored_items.contains(&ItemEnum::Cigs) & { game_info.dealer_health < 3 }
        {
            item_use(ItemEnum::Cigs, game_info, &mut dealer_minor_info);
            play_audio("dealer_use_cigarettes.ogg");
            thread::sleep(Duration::from_millis(500));

            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::MagGlass)
            && !dealer_minor_info.shell_knowledge
        {
            dealer_minor_info.shell_knowledge =
                item_use(ItemEnum::MagGlass, game_info, &mut dealer_minor_info);
            play_audio("dealer_use_magnifier.ogg");
            thread::sleep(Duration::from_millis(500));
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Saws)
            & dealer_minor_info.shell_knowledge
            & game_info.shells_vector[0]
            && !saw_used
        {
            saw_used = !saw_used;
            
            item_use(ItemEnum::Saws, game_info, &mut dealer_minor_info);
            play_audio("dealer_use_handsaw.ogg");
            thread::sleep(Duration::from_millis(500));
            dealer_minor_info.damage = 2;
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Handcuffs) && {
            dealer_minor_info.extra_turn_var != ExtraTurnVars::Handcuffed
        } {
            item_use(ItemEnum::Handcuffs, game_info, &mut dealer_minor_info);
            play_audio("dealer_use_cigarettes.ogg");
            thread::sleep(Duration::from_millis(500));
            dealer_minor_info.extra_turn_var = ExtraTurnVars::Handcuffed;
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Beers)
            && !dealer_minor_info.shell_knowledge
            && coinflip
            && { game_info.shells_vector.len() > 2 }
        {
            item_use(ItemEnum::Beers, game_info, &mut dealer_minor_info);
            play_audio("dealer_use_beer.ogg");
            thread::sleep(Duration::from_millis(500));
            continue 'dealer_item_logic;
        }
        if game_info.double_or_nothing {
            if game_info.dealer_stored_items.contains(&ItemEnum::Adren) && {
                !game_info.player_inventory.is_empty()
            } {
                item_use(ItemEnum::Adren, game_info, &mut dealer_minor_info);
                play_audio("dealer_use_adrenaline.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
            if game_info.dealer_stored_items.contains(&ItemEnum::BurnPho)
                && lives != 0
                && game_info.shells_vector.len() > 1
            {
                item_use(ItemEnum::BurnPho, game_info, &mut dealer_minor_info);
                play_audio("dealer_use_burner_phone.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
            if game_info.dealer_stored_items.contains(&ItemEnum::Invert) && {
                (dealer_minor_info.shell_knowledge && !game_info.shells_vector[0])
                    || (lives > blanks)
            } {
                item_use(ItemEnum::Invert, game_info, &mut dealer_minor_info);
                play_audio("dealer_use_inverter.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
            if game_info.dealer_stored_items.contains(&ItemEnum::ExpMed)
                && game_info.dealer_health == 2
            {
                item_use(ItemEnum::ExpMed, game_info, &mut dealer_minor_info);
                play_audio("dealer_use_medicine.ogg");
                thread::sleep(Duration::from_millis(500));
                continue 'dealer_item_logic;
            }
        }
        message_stats_func(game_info);
        break dealer_minor_info;
    }
}

pub fn turn(game_info: &mut GameInfo) -> bool {
    message_stats_func(game_info);
    let damage: i8 = 1;
    // future goal: add logic for having dealer pick certain items
    let shell_knowledge = false;
    let extra_turn_var = ExtraTurnVars::None;
    let dealer_minor_info = dealer_item_logic(game_info, damage, shell_knowledge, extra_turn_var);

    let choice: bool = if game_info.perfect | dealer_minor_info.shell_knowledge {
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
        //if there are more lives than blanks, choose to shoot player.
        lives >= blanks
    };
    message_stats_func(game_info);
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
    message_stats_func(game_info);
    match to_be_shot {
        TargetEnum::Player => {
            message_top!("The dealer points the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();
                message_top!("Dealer shot you.");
                game_info.player_health -= dealer_minor_info.damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                message_top!("click");
            }
        }
        TargetEnum::Dealer => {
            message_top!("The dealer points the gun at its face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();
                message_top!("Dealer shot themselves.");
                game_info.dealer_health -= dealer_minor_info.damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                message_top!("click");
                message_top!("Extra turn for dealer.");
                if dealer_minor_info.extra_turn_var != ExtraTurnVars::Handcuffed {
                    extraturn = true;
                }
            }
        }
    }
    message_stats_func(game_info);
    game_info.shells_vector.remove(0);
    thread::sleep(Duration::from_secs(1));
    check_life(game_info);
    message_stats_func(game_info);
    extraturn
}

fn item_use(
    item_type: ItemEnum,
    game_info: &mut GameInfo,
    dealer_minor_info: &mut DealerMinorInfo,
) -> bool {
    message_stats_func(game_info);
    let mut knowledge_of_shell = false;
    match item_type {
        ItemEnum::Cigs => {
            if game_info.dealer_health == 3 {
                unreachable!()
            } else {
                message_top!("The dealer lights one of the cigs.");
                game_info.dealer_health += 1;
            }
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Saws => {
            message_top!("Shhk. The dealer slices off the tip of the gun. It'll do 2 damage now.");
            dealer_minor_info.damage = 2;
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::MagGlass => {
            message_top!(
                "The dealer looks down at the gun with an old magnifying glass. You see them smirk."
            );
            knowledge_of_shell = true;

            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Beers => {
            if game_info.shells_vector[0] {
                message_top!("The dealer gives the shotgun a pump. A live round drops out.");
            } else {
                message_top!("The dealer gives the shotgun a pump. A blank round drops out.");
            };
            game_info.shells_vector.remove(0);

            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Handcuffs => {
            message_top!(
                "The dealer grabs onto your hand. When they let go, your hands are cuffed."
            );
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Nothing => {
            unreachable!()
        }

        ItemEnum::Adren => {
            message_top!("The dealer takes a hit of the adrenaline.");
            todo!();
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::BurnPho => {
            message_top!("The dealer uses the burner phone.");
            todo!();
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::Invert => {
            message_top!("The dealer uses the inverter.");
            game_info.shells_vector[0] = !game_info.shells_vector[0];
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
        ItemEnum::ExpMed => {
            message_top!("The dealer takes the expired medicine.");
            let coinflip: bool = rand::thread_rng().gen();
            if coinflip {
                game_info.dealer_health += 1;
                message_top!("The dealer smiles.");
            } else {
                game_info.dealer_health -= 2;
                message_top!("The dealer chokes and falls over.");
            }
            remove_item(&mut game_info.dealer_stored_items, item_type);
        }
    }
    thread::sleep(Duration::from_millis(500));
    message_stats_func(game_info);
    knowledge_of_shell
}
pub fn picked_to_stored(
    mut picked_items_vec_dealer: Vec<ItemEnum>,
    game_info: &mut GameInfo,
) -> [ItemEnum; 8] {
    message_top!("The dealer is picking items...");
    let mut index = 0;
    while !picked_items_vec_dealer.is_empty() {
        if game_info.dealer_stored_items[index] == ItemEnum::Nothing
            && !picked_items_vec_dealer.is_empty()
        {   
            let item = picked_items_vec_dealer[0];
            match item {
                ItemEnum::Cigs => play_audio("pick_up_cigarettes.ogg"),
                ItemEnum::Saws => play_audio("pick_up_metal.ogg"),
                ItemEnum::MagGlass => play_audio("pick_up_magnifier.ogg"),
                ItemEnum::Beers => play_audio("pick_up_beer.ogg"),
                ItemEnum::Handcuffs => play_audio("pick_up_handcuffs.ogg"),
                ItemEnum::Adren => play_audio("pick_up_adrenaline.ogg"),
                ItemEnum::BurnPho => play_audio("pick_up_burner_phone.ogg"),
                ItemEnum::Invert => play_audio("pick_up_inverter.ogg"),
                ItemEnum::ExpMed => play_audio("pick_up_medicine.ogg"),
                ItemEnum::Nothing => {}
            }

            thread::sleep(Duration::from_millis(300));
            match item {
                ItemEnum::Cigs => play_audio("place_down_cigarettes.ogg"),
                ItemEnum::Saws => play_audio("place_down_handsaw.ogg"),
                ItemEnum::MagGlass => play_audio("place_down_magnifier.ogg"),
                ItemEnum::Beers => play_audio("place_down_beer.ogg"),
                ItemEnum::Handcuffs => play_audio("place_down_handcuffs.ogg"),
                ItemEnum::Adren => play_audio("place_down_adrenaline.ogg"),
                ItemEnum::BurnPho => play_audio("place_down_burner_phone.ogg"),
                ItemEnum::Invert => play_audio("place_down_inverter.ogg"),
                ItemEnum::ExpMed => play_audio("place_down_medicine.ogg"),
                ItemEnum::Nothing => {}
            }
            game_info.dealer_stored_items[index] = picked_items_vec_dealer.remove(0);
            
        }

        message_stats_func(game_info);
        index += 1;
    }
    game_info.dealer_stored_items
}
