use crate::{local_ratatui::message_stats_func, message_top_func};
use itertools::Itertools;
use rand::Rng;
use std::{fmt, thread, time::Duration};

use crate::{
    check_life, message_top, play_audio, remove_item, turn_screen_red, GameInfo, ItemEnum,
    PlayerDealer,
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
    damage: i8,
}
#[allow(clippy::too_many_lines)]
fn dealer_item_logic(
    game_info: &mut GameInfo,
    damage: i8,
    extra_turn_var: ExtraTurnVars,
) -> DealerMinorInfo {
    message_stats_func(game_info);
    let mut dealer_minor_info: DealerMinorInfo = DealerMinorInfo {
        extra_turn_var,
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
        if game_info.dealer_inventory.contains(&ItemEnum::Cigs) & { game_info.dealer_health < 3 } {
            item_use(ItemEnum::Cigs, game_info, &mut dealer_minor_info, false);
            play_audio("dealer_use_cigarettes.ogg");

            continue 'dealer_item_logic;
        }
        if game_info.dealer_inventory.contains(&ItemEnum::MagGlass)
            && !game_info.dealer_shell_knowledge_vec[game_info.shell_index].unwrap_or(false)
        {
            item_use(ItemEnum::MagGlass, game_info, &mut dealer_minor_info, false);
            play_audio("dealer_use_magnifier.ogg");

            continue 'dealer_item_logic;
        }
        if game_info.dealer_inventory.contains(&ItemEnum::Saws)
            & game_info.dealer_shell_knowledge_vec[game_info.shell_index].unwrap_or(false)
            & game_info.shells_vector[game_info.shell_index]
            && !saw_used
        {
            saw_used = true;

            item_use(ItemEnum::Saws, game_info, &mut dealer_minor_info, false);
            play_audio("dealer_use_handsaw.ogg");

            dealer_minor_info.damage = 2;
            continue 'dealer_item_logic;
        }
        if game_info.dealer_inventory.contains(&ItemEnum::Handcuffs) && {
            dealer_minor_info.extra_turn_var != ExtraTurnVars::Handcuffed
        } {
            item_use(
                ItemEnum::Handcuffs,
                game_info,
                &mut dealer_minor_info,
                false,
            );
            play_audio("dealer_use_cigarettes.ogg");
            continue 'dealer_item_logic;
        }
        if game_info.dealer_inventory.contains(&ItemEnum::Beers)
            && !game_info.dealer_shell_knowledge_vec[game_info.shell_index].unwrap_or(false)
            && coinflip
            && { game_info.shells_vector.len() - game_info.shell_index > 2 }
        {
            item_use(ItemEnum::Beers, game_info, &mut dealer_minor_info, false);
            play_audio("dealer_use_beer.ogg");

            continue 'dealer_item_logic;
        }
        if game_info.double_or_nothing {
            if game_info.dealer_inventory.contains(&ItemEnum::Adren) && {
                game_info
                    .player_inventory
                    .iter()
                    .any(|&v| v != ItemEnum::Nothing && v != ItemEnum::Adren)
            } {
                item_use(ItemEnum::Adren, game_info, &mut dealer_minor_info, false);
                play_audio("dealer_use_adrenaline.ogg");

                continue 'dealer_item_logic;
            }
            if game_info.dealer_inventory.contains(&ItemEnum::BurnPho)
                && lives != 0
                && game_info.shells_vector.len() > 1
            {
                item_use(ItemEnum::BurnPho, game_info, &mut dealer_minor_info, false);
                play_audio("dealer_use_burner_phone.ogg");

                continue 'dealer_item_logic;
            }
            if game_info.dealer_inventory.contains(&ItemEnum::Invert) && {
                (game_info.dealer_shell_knowledge_vec[game_info.shell_index].unwrap_or(false)
                    && !game_info.shells_vector[game_info.shell_index])
                    || (lives > blanks)
            } {
                item_use(ItemEnum::Invert, game_info, &mut dealer_minor_info, false);
                play_audio("dealer_use_inverter.ogg");

                continue 'dealer_item_logic;
            }
            if game_info.dealer_inventory.contains(&ItemEnum::ExpMed)
                && game_info.dealer_health == 2
            {
                item_use(ItemEnum::ExpMed, game_info, &mut dealer_minor_info, false);
                play_audio("dealer_use_medicine.ogg");

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
    let extra_turn_var = ExtraTurnVars::None;
    let dealer_minor_info = dealer_item_logic(game_info, damage, extra_turn_var);

    let choice: bool = if game_info.perfect
        | game_info.dealer_shell_knowledge_vec[game_info.shell_index].unwrap_or(false)
    {
        game_info.shells_vector[game_info.shell_index]
    } else {
        //logic for the dealer's choice
        let mut lives = 0;
        let mut blanks = 0;
        let cur_svec = &game_info.shells_vector[game_info.shell_index..];
        for item in cur_svec {
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
        PlayerDealer::Player
    } else {
        PlayerDealer::Dealer
    };
    //true means dealer shoots you, false means dealer shoots itself
    let mut extraturn = false;
    if dealer_minor_info.extra_turn_var == ExtraTurnVars::Handcuffed {
        extraturn = true;
    }
    message_stats_func(game_info);
    match to_be_shot {
        PlayerDealer::Player => {
            message_top!("The dealer points the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[game_info.shell_index] {
                turn_screen_red();
                message_top!("Dealer shot you.");
                game_info.player_health -= dealer_minor_info.damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                message_top!("click");
            }
        }
        PlayerDealer::Dealer => {
            message_top!("The dealer points the gun at its face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[game_info.shell_index] {
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
    game_info.shell_index += 1;
    message_stats_func(game_info);
    thread::sleep(Duration::from_secs(1));
    check_life(game_info);
    message_stats_func(game_info);
    extraturn
}

fn item_use(
    item_type: ItemEnum,
    game_info: &mut GameInfo,
    dealer_minor_info: &mut DealerMinorInfo,
    adren_item: bool,
) {
    message_stats_func(game_info);
    let mut index: Option<usize> = None;
    if !adren_item {
        index = game_info
            .dealer_inventory
            .iter()
            .position(|x| *x == item_type);
    }

    match item_type {
        ItemEnum::Cigs => {
            if game_info.dealer_health == 3 {
                unreachable!()
            } else {
                message_top!("The dealer lights one of the cigs.");
                game_info.dealer_health += 1;
            }
        }
        ItemEnum::Saws => {
            message_top!("Shhk. The dealer slices off the tip of the gun. It'll do 2 damage now.");
            dealer_minor_info.damage = 2;
        }
        ItemEnum::MagGlass => {
            message_top!(
                "The dealer looks down at the gun with an old magnifying glass. You see them smirk."
            );
            game_info.dealer_shell_knowledge_vec[game_info.shell_index] =
                Some(game_info.shells_vector[game_info.shell_index]);
        }
        ItemEnum::Beers => {
            if game_info.shells_vector[game_info.shell_index] {
                message_top!("The dealer gives the shotgun a pump. A live round drops out.");
            } else {
                message_top!("The dealer gives the shotgun a pump. A blank round drops out.");
            };
            game_info.shell_index += 1;
        }
        ItemEnum::Handcuffs => {
            message_top!(
                "The dealer grabs onto your hand. When they let go, your hands are cuffed."
            );
            dealer_minor_info.extra_turn_var = ExtraTurnVars::Handcuffed;
        }
        ItemEnum::Nothing => {
            unreachable!()
        }

        ItemEnum::Adren => {
            message_top!("The dealer takes a hit of the adrenaline.");
            let real_player_items: Vec<&ItemEnum> = game_info
                .player_inventory
                .iter()
                .filter(|item| item != &&ItemEnum::Nothing && item != &&ItemEnum::Adren)
                .collect_vec();

            let stolen_item: &ItemEnum =
                real_player_items[rand::thread_rng().gen_range(0..real_player_items.len())];

            message_top!("The dealer took your {stolen_item}!");
            let index_of_stolen: usize = match game_info
                .player_inventory
                .iter()
                .position(|&x| x == *stolen_item)
            {
                Some(index) => index,
                None => unreachable!(),
            };

            drop(real_player_items);
            let test = *stolen_item;
            remove_item(&mut game_info.player_inventory, index_of_stolen);

            remove_item(&mut game_info.dealer_inventory, index.unwrap());

            //let the dealer use the item stolen_item
            item_use(test, game_info, dealer_minor_info, true);
        }
        ItemEnum::BurnPho => {
            let shell_number: usize =
                rand::thread_rng().gen_range(0..{ game_info.shells_vector.len() });
            //shell number is a rand number from 0 to the length of the current shell vec
            //ie, if there are 3 bullets left the number could be 0,1,2.

            game_info.dealer_shell_knowledge_vec[shell_number] =
                Some(game_info.shells_vector[shell_number]);
            message_top!("The dealer uses the burner phone."); // for debug add ->> it thinks index {shell_number} is {}", game_info.shells_vector[shell_number]
        }
        ItemEnum::Invert => {
            message_top!("The dealer uses the inverter.");
            game_info.shells_vector[game_info.shell_index] =
                !game_info.shells_vector[game_info.shell_index];
        }
        ItemEnum::ExpMed => {
            message_top!("The dealer takes the expired medicine.");
            let coinflip: bool = rand::thread_rng().gen();
            if coinflip {
                game_info.dealer_health += 2;
                message_top!("The dealer smiles.");
            } else {
                game_info.dealer_health -= 1;
                message_top!("The dealer chokes and falls over.");
            }
        }
    }
    if adren_item {
    } else {
        remove_item(&mut game_info.dealer_inventory, index.unwrap());
    }

    thread::sleep(Duration::from_millis(1000));
    message_stats_func(game_info);
}

pub fn picked_to_stored(
    mut picked_items_vec_dealer: Vec<ItemEnum>,
    game_info: &mut GameInfo,
) -> [ItemEnum; 8] {
    message_top!("The dealer is picking items...");
    let mut index = 0;
    while !picked_items_vec_dealer.is_empty() {
        if game_info.dealer_inventory[index] == ItemEnum::Nothing
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
            game_info.dealer_inventory[index] = picked_items_vec_dealer.remove(0);
        }

        message_stats_func(game_info);
        index += 1;
    }
    game_info.dealer_inventory
}
