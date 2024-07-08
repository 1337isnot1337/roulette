use ansi_term::Style;
use core::fmt;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{Clear, ClearType},
};
use dialoguer::FuzzySelect;
use once_cell::sync::Lazy;
use rand::{seq::SliceRandom, Rng};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};
use std::{
    env,
    fs::{self, File},
    io::{self, BufReader, Write},
    mem, process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use crate::{check_life, italics, play_audio, remove_no_item, turn_screen_red, GameInfo, ItemEnum, TargetEnum};


pub fn dealer_turn(current_bullets_vec: Vec<bool>, game_info: &mut GameInfo) -> bool {
    let mut damage = 1;
    // future goal: add logic for having dealer pick certain items
    let mut shell_knowledge = false;
    let mut handcuff_player: bool = false;
    let coinflip: bool = rand::thread_rng().gen();
    'dealer_use_items: loop {
        if game_info.dealer_stored_items.contains(&ItemEnum::Cigs) & { game_info.dealer_health < 3 }
        {
            dealer_item_use(ItemEnum::Cigs, game_info, &mut damage);
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::MagGlass) && !shell_knowledge {
            shell_knowledge = dealer_item_use(ItemEnum::MagGlass, game_info, &mut damage);
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Saws)
            & shell_knowledge
            & game_info.shell
        {
            dealer_item_use(ItemEnum::Saws, game_info, &mut damage);
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Handcuffs) && !handcuff_player {
            dealer_item_use(ItemEnum::Handcuffs, game_info, &mut damage);
            handcuff_player = !handcuff_player;
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Beers) && !shell_knowledge & coinflip {
            dealer_item_use(ItemEnum::Beers, game_info, &mut damage);
            break 'dealer_use_items;
        }
        break;
    }

    let choice: bool = if game_info.perfect | shell_knowledge {
        game_info.shell
    } else {
        //logic for the dealer's choice
        let mut lives = 0;
        let mut blanks = 0;

        for item in current_bullets_vec {
            if item {
                lives += 1;
            } else {
                blanks += 1;
            }
        }
        //if there are more lives than blanks, choose to shoot player. Vice versa and such.
        lives >= blanks
    };
    //true means dealer shoots you, false means dealer shoots itself
    let mut extraturn = false;
    if choice {
        println!("The dealer points the gun at your face.");
        thread::sleep(Duration::from_secs(1));
        if game_info.shell {
            turn_screen_red();
            println!("Dealer shot you.");
            game_info.player_health -= 1;
        } else {
            play_audio("temp_gunshot_blank.wav");
            italics("click");
        }
    } else {
        println!("The dealer points the gun at its face.");
        thread::sleep(Duration::from_secs(1));
        if game_info.shell {
            turn_screen_red();
            println!("Dealer shot themselves.");
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


pub fn dealer_item_use(item_type: ItemEnum, game_info: &mut GameInfo, damage: &mut u8) -> bool {
    let mut knowledge_of_shell = false;
    match item_type {
        ItemEnum::Cigs => {
            if game_info.dealer_health == 3 {
                println!(
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
            if game_info.shell {
                println!("The dealer gives the shotgun a pump. A live round drops out.");
            } else {
                println!("The dealer gives the shotgun a pump. A blank round drops out.");
            };
            match game_info.turn_owner {
                TargetEnum::Player => {
                    game_info.turn_owner = TargetEnum::Dealer;
                }
                TargetEnum::Dealer => {
                    game_info.turn_owner = TargetEnum::Player;
                }
            }
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
            println!("ERROR: THIS CODE SHOULD NOT BE REACHABLE! PLEASE REPORT THIS BUG.");
        }
        ItemEnum::Adren => todo!(),
        ItemEnum::BurnPho => todo!(),
        ItemEnum::Invert => todo!(),
        ItemEnum::ExpMed => todo!(),
    }
    knowledge_of_shell
}
pub fn picked_to_stored(
    mut picked_items_vec_dealer: Vec<ItemEnum>,
    dealer_stored_items: &mut [ItemEnum; 8],
) -> [ItemEnum; 8] {
    // iterate through each item in dealer_stored_items
    for dealer_item in dealer_stored_items.iter_mut() {
        // check if the dealer_stored_item is Nothing and picked_items_vec_dealer isnt empty
        if { *dealer_item == ItemEnum::Nothing } & { !picked_items_vec_dealer.is_empty() } {
            // replace the Nothing with first item from picked_items_vec_dealer
            *dealer_item = picked_items_vec_dealer.remove(0);
        }
    }
    *dealer_stored_items
}