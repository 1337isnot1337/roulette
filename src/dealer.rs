use rand::Rng;
use std::{thread, time::Duration};

use crate::{
    check_life, italics, play_audio, remove_no_item, turn_screen_red, GameInfo, ItemEnum,
    TargetEnum,
};

fn dealer_item_logic(
    current_bullets_vec: &Vec<bool>,
    game_info: &mut GameInfo,
    mut damage: u8,
    mut shell_knowledge: bool,
    mut handcuff_player: bool,
) -> bool {
    let coinflip: bool = rand::thread_rng().gen();
    let mut lives = 0;
    let mut blanks = 0;

    for item in current_bullets_vec {
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
        if game_info.dealer_stored_items.contains(&ItemEnum::MagGlass) && !shell_knowledge {
            shell_knowledge = item_use(ItemEnum::MagGlass, game_info, &mut damage);
            play_audio("dealer_use_magnifier.ogg");
            thread::sleep(Duration::from_millis(500));
            println!("{shell_knowledge}");
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Saws)
            & shell_knowledge
            & game_info.shells_vector[0]
        {
            item_use(ItemEnum::Saws, game_info, &mut damage);
            play_audio("dealer_use_handsaw.ogg");
            thread::sleep(Duration::from_millis(500));
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Handcuffs) && !handcuff_player {
            item_use(ItemEnum::Handcuffs, game_info, &mut damage);
            play_audio("dealer_use_cigarettes.ogg");
            thread::sleep(Duration::from_millis(500));
            handcuff_player = !handcuff_player;
            continue 'dealer_item_logic;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Beers) && !shell_knowledge & coinflip {
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
                (shell_knowledge && !game_info.shells_vector[0]) || (lives > blanks)
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
        break shell_knowledge;
    }
}

pub fn turn(current_bullets_vec: Vec<bool>, game_info: &mut GameInfo) -> bool {
    let damage: u8 = 1;
    // future goal: add logic for having dealer pick certain items
    let mut shell_knowledge = false;
    let handcuff_player: bool = false;
    shell_knowledge = dealer_item_logic(
        &current_bullets_vec,
        game_info,
        damage,
        shell_knowledge,
        handcuff_player,
    );

    let choice: bool = if game_info.perfect | shell_knowledge {
        println!("perf");
        game_info.shells_vector[0]
    } else {
        //logic for the dealer's choice
        let mut lives = 0;
        let mut blanks = 0;

        for item in &current_bullets_vec {
            if *item {
                lives += 1;
            } else {
                blanks += 1;
            }
        }
        //if there are more lives than blanks, choose to shoot player. Vice versa and such.
        lives >= blanks
    };
    println!(
        "choice is {choice} and shell is {}",
        game_info.shells_vector[0]
    );
    //true means dealer shoots you, false means dealer shoots itself
    let mut extraturn = false;
    if choice {
        println!("The dealer points the gun at your face.");
        thread::sleep(Duration::from_secs(1));
        if game_info.shells_vector[0] {
            turn_screen_red();
            println!("Dealer shot you.");
            game_info.turn_owner = TargetEnum::Player;
            game_info.player_health -= 1;
        } else {
            play_audio("temp_gunshot_blank.wav");
            game_info.turn_owner = TargetEnum::Player;
            italics("click");
        }
    } else {
        println!("The dealer points the gun at its face.");
        thread::sleep(Duration::from_secs(1));
        if game_info.shells_vector[0] {
            turn_screen_red();
            println!("Dealer shot themselves.");
            game_info.turn_owner = TargetEnum::Player;
            game_info.dealer_health -= 1;
        } else {
            play_audio("temp_gunshot_blank.wav");
            italics("click");
            println!("Extra turn for dealer.");
            extraturn = true;
        }
    }

    thread::sleep(Duration::from_secs(1));
    check_life(game_info.player_health, game_info.dealer_health);
    extraturn
}

pub fn item_use(item_type: ItemEnum, game_info: &mut GameInfo, damage: &mut u8) -> bool {
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
            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::Cigs);
        }
        ItemEnum::Saws => {
            println!("Shhk. The dealer slices off the tip of the gun. It'll do 2 damage now.");
            *damage = 2;
            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::Saws);
        }
        ItemEnum::MagGlass => {
            println!(
                "The dealer looks down at the gun with an old magnifying glass. You see him smirk."
            );
            knowledge_of_shell = true;

            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::MagGlass);
        }
        ItemEnum::Beers => {
            if game_info.shells_vector[0] {
                println!("The dealer gives the shotgun a pump. A live round drops out.");
            } else {
                println!("The dealer gives the shotgun a pump. A blank round drops out.");
            };
            game_info.shells_vector.remove(0);

            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::Beers);
        }
        ItemEnum::Handcuffs => {
            println!("The dealer grabs onto your hand. When he lets go, your hands are cuffed.");
            if game_info.turn_owner == TargetEnum::Dealer {
                game_info.turn_owner = TargetEnum::Player;
            }
            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::Handcuffs);
        }
        ItemEnum::Nothing => {
            panic!("ERROR: THIS CODE SHOULD NOT BE REACHABLE! PLEASE REPORT THIS BUG.");
        }

        ItemEnum::Adren => {
            println!("The dealer takes a hit of the adrenaline.");
            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::Adren);
        }
        ItemEnum::BurnPho => {
            println!("The dealer uses the burner phone.");
            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::BurnPho);
        }
        ItemEnum::Invert => {
            println!("The dealer uses the inverter.");
            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::Invert);
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
            remove_no_item(&mut game_info.dealer_stored_items, ItemEnum::ExpMed);
        }
    }
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
