use crate::local_ratatui::{dialogue, message_stats_func};
use crate::{
    check_life, generate_items, message_top, play_audio, remove_item, turn_screen_red, GameInfo,
    ItemEnum, PlayerDealer,
};
use crate::{message_top_func, PREVIOUS_INDEX};
use rand::Rng;
use std::{thread, time::Duration};

pub fn turn(game_info: &mut GameInfo) -> (bool, bool) {
    let mut empty_due_to_beer = false;
    message_stats_func(game_info);
    let mut cuffed = false;
    let mut damage: i8 = 1;
    if game_info.round == 1 {
        message_top!("Your turn.");
    } else {
        message_top!("Select items to use");
    };
    let mut extraturn = false;
    'might_return: loop {
        let (new_empty_due_to_beer, new_cuffed, new_damage, broke) =
            match_item(false, game_info, empty_due_to_beer, damage, cuffed, None);
        
        //message_top!("Matched at least. New: beer {new_empty_due_to_beer}, cuff {new_cuffed}, damage {new_damage}.");
        //message_top!(
         //   "Matched at least. Old: beer {empty_due_to_beer}, cuff {cuffed}, damage {damage}."
        //);
        
        if new_cuffed && !cuffed {
           // message_top!("DEBUG INFO cuff");
            cuffed = true;
        }
        if new_damage == 2 && damage == 1 {
           // message_top!("DEBUG INFO damage");
            damage = 2;
        }
        if new_empty_due_to_beer && !cuffed {
           // message_top!("DEBUG INFO beer");
            empty_due_to_beer = true;
        }
        if broke {
            continue 'might_return;
        }

        if !empty_due_to_beer {
            message_stats_func(game_info);
            let mut targets: [PlayerDealer; 2] = [PlayerDealer::Player, PlayerDealer::Dealer];

            let selection: usize = if game_info.round == 1 {
                dialogue(&mut targets, "Who to shoot?", None, false, false).unwrap()
            } else {
                match dialogue(&mut targets, "Who to shoot?", None, false, true) {
                    Some(index) => index,
                    None => continue 'might_return,
                }
                // add impl for none here
            };

            message_stats_func(game_info);

            let choice = targets[selection];
            extraturn = resolve_player_choice(choice, damage, game_info, cuffed);

            thread::sleep(Duration::from_secs(1));
            check_life(game_info);
            message_stats_func(game_info);
            break;
        }
        break;
    }

    (extraturn, empty_due_to_beer)
}
#[allow(clippy::too_many_lines)]
fn match_item(
    mut adren_pick: bool,
    game_info: &mut GameInfo,
    mut empty_due_to_beer: bool,
    mut damage: i8,
    mut cuffed: bool,
    adren_item: Option<ItemEnum>,
) -> (bool, bool, i8, bool) {
    let mut broke = false;

    'item_selection_loop: loop {
        message_stats_func(game_info);

        let shoot_or_pick_items: Option<ItemEnum> = if adren_pick {
            adren_pick = false;
            adren_item
        } else {
            if empty_due_to_beer {
                break 'item_selection_loop;
            }
            let might_just_shoot: Option<ItemEnum> = if game_info.round == 1 {
                None
            } else {
                let selection_shoot = dialogue(
                    &mut ["Use the gun", "Use items"],
                    "Shoot or use inventory",
                    Some(PlayerDealer::Player),
                    false,
                    false,
                )
                .unwrap();

                if selection_shoot == 1 {
                    let Some(selection) = dialogue(
                        &mut game_info.player_inventory,
                        "Your Inventory",
                        Some(PlayerDealer::Player),
                        true,
                        true,
                    ) else {
                        broke = true;
                        break 'item_selection_loop;
                    };
                    // add impl for none here

                    *PREVIOUS_INDEX.try_lock().unwrap() = selection;

                    let result = game_info.player_inventory[selection];
                    remove_item(&mut game_info.player_inventory, selection);
                    Some(result)
                } else {
                    None
                }
            };

            might_just_shoot
        };

        message_stats_func(game_info);

        if let Some(item) = shoot_or_pick_items {
            match item {
                ItemEnum::Cigs => {
                    play_audio("player_use_cigarettes.ogg");
                    if game_info.player_charges_cap > game_info.player_charges {
                        message_top!("You light one of the cigs. Your head feels hazy, but you feel power coursing through your veins.");
                        game_info.player_charges += 1;
                    } else {
                        message_top!(
                "You light one of the cigs. Your head feels hazy. It doesn't seem to do much.");
                    }
                    game_info.score_info.cigs_taken += 1;

                    continue 'item_selection_loop;
                }
                ItemEnum::Saws => {
                    play_audio("player_use_handsaw.ogg");
                    message_top!("Shhk. You slice off the tip of the gun. It'll do 2 damage now.");
                    damage = 2;

                    continue 'item_selection_loop;
                }
                ItemEnum::MagGlass => {
                    play_audio("player_use_magnifier.ogg");
                    if game_info.shells_vector[game_info.shell_index] {
                        message_top!(
                            "Upon closer inspection, you realize that there's a live round loaded."
                        );
                    } else {
                        message_top!(
                    "Upon closer inspection, you realize that there's a blank round loaded."
                );
                    }

                    continue 'item_selection_loop;
                }
                ItemEnum::Handcuffs => {
                    play_audio("player_use_handcuffs.ogg");
                    message_top!(
                "The dealer grabs the handcuffs from your outstretched hand, putting them on."
            );
                    cuffed = true;

                    continue 'item_selection_loop;
                }
                ItemEnum::Beers => {
                    game_info.score_info.shells_ejec += 1;
                    game_info.score_info.beers += 1;
                    play_audio("player_use_beer.ogg");
                    if (game_info.shells_vector.len() - 1) == game_info.shell_index {
                        empty_due_to_beer = true;
                    }
                    if game_info.shells_vector[game_info.shell_index] {
                        message_top!("You give the shotgun a pump. A live round drops out.");
                    } else {
                        message_top!("You give the shotgun a pump. A blank round drops out.");
                    };
                    game_info.shell_index += 1;

                    continue 'item_selection_loop;
                }
                ItemEnum::Nothing => {}
                double_item => {
                    (empty_due_to_beer, cuffed, damage) = double_or_nothing_items(
                        double_item,
                        game_info,
                        empty_due_to_beer,
                        damage,
                        cuffed,
                    );

                    continue 'item_selection_loop;
                }
            }

            break;
        }
        empty_due_to_beer = false;
        break;
    }
    //message_top!(
      //  "beer {}, cuff {}, dam {}",
        //empty_due_to_beer,
        //cuffed,
    //    damage
    //);

    (empty_due_to_beer, cuffed, damage, broke)
}

fn double_or_nothing_items(
    item_type: ItemEnum,
    game_info: &mut GameInfo,
    mut empty_due_to_beer: bool,
    mut damage: i8,
    mut cuffed: bool,
) -> (bool, bool, i8) {
    message_stats_func(game_info);
    match item_type {
        ItemEnum::Adren => {
            play_audio("player_use_adrenaline.ogg");
            message_top!("You jam the rusty needle into your thigh.");
            let sel_index = dialogue(
                &mut game_info.dealer_inventory,
                "Pick one of the dealer's items",
                Some(PlayerDealer::Dealer),
                false,
                false,
            )
            .unwrap();

            let stolen_item = game_info.dealer_inventory[sel_index];
            if stolen_item == ItemEnum::Adren {
                message_top!("You can't grab the adrenaline.");
            } else {
                message_top!("You grab the {stolen_item}, and use it.");

                remove_item(&mut game_info.dealer_inventory, sel_index);
                let (new_empty_due_to_beer, new_cuffed, new_damage, _) = match_item(
                    true,
                    game_info,
                    empty_due_to_beer,
                    damage,
                    cuffed,
                    Some(stolen_item),
                );
                if new_cuffed && !cuffed {
                    message_top!("DEBUG INFO cuff");
                    cuffed = true;
                }
                if new_damage == 2 && damage == 1 {
                    message_top!("DEBUG INFO damage");
                    damage = 2;
                }
                if new_empty_due_to_beer && !cuffed {
                    message_top!("DEBUG INFO beer");
                    empty_due_to_beer = true;
                }
            }
        }
        ItemEnum::BurnPho => {
            play_audio("player_use_burner_phone.ogg");
            let abs_shell_number: usize =
                rand::thread_rng().gen_range(game_info.shell_index..game_info.shells_vector.len());
            let rel_shell_num = abs_shell_number - game_info.shell_index;
            let shell_reveal = if game_info.shells_vector[abs_shell_number] {
                "live"
            } else {
                "blank"
            };

            let place = match rel_shell_num {
                0 => "first",
                1 => "second",
                2 => "third",
                3 => "fourth",
                4 => "fifth",
                5 => "sixth",
                6 => "seventh",
                7 => "eigth",
                _ => panic!("Burner phone panic; number larger than 8. Report this error!"),
            };
            message_top!("You flip open the phone. The {place} shell is {shell_reveal}");
        }
        ItemEnum::Invert => {
            message_top!("You flick the switch on the inverter.");

            play_audio("player_use_inverter.ogg");
            game_info.shells_vector[game_info.shell_index] =
                !game_info.shells_vector[game_info.shell_index];
        }
        ItemEnum::ExpMed => {
            play_audio("player_use_medicine.ogg");
            message_top!("You takes the expired medicine.");
            let coinflip: bool = rand::thread_rng().gen();

            if coinflip {
                if game_info.player_charges < 3 {
                    game_info.player_charges += 2;
                    message_top!("You feel energy coursing through you.");
                } else {
                    message_top!("The pills don't do anything.");
                }
            } else {
                game_info.player_charges -= 1;
                message_top!("You choke and fall over.");
            }
        }
        _ => unreachable!(),
    }
    (empty_due_to_beer, cuffed, damage)
}

fn resolve_player_choice(
    choice: PlayerDealer,
    damage: i8,
    game_info: &mut GameInfo,
    cuffed: bool,
) -> bool {
    message_stats_func(game_info);
    let mut extraturn = false;
    if cuffed {
        extraturn = true;
    }
    match choice {
        PlayerDealer::Player => {
            message_top!("You point the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[game_info.shell_index] {
                turn_screen_red();
                message_top!("You shot yourself.");
                game_info.score_info.shots_fir += 1;
                game_info.score_info.shells_ejec += 1;

                game_info.player_charges -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                message_top!("click");
                thread::sleep(Duration::from_secs(1));
                message_top!("Extra turn for you.");
                game_info.score_info.shells_ejec += 1;
                extraturn = true;
            }
        }
        PlayerDealer::Dealer => {
            message_top!("You point the gun towards the dealer.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[game_info.shell_index] {
                turn_screen_red();

                message_top!("You shot the dealer.");
                game_info.score_info.shots_fir += 1;
                game_info.score_info.shells_ejec += 1;
                game_info.dealer_charges -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                message_top!("click");
                game_info.score_info.shells_ejec += 1;
            }
        }
    }
    game_info.shell_index += 1;
    message_stats_func(game_info);
    extraturn
}

pub fn pick_items(game_info: &mut GameInfo) {
    let items_vec = &mut generate_items(8, game_info);
    let num_of_items: u8 = match game_info.round {
        2 => 2,
        3 => 4,
        _ => unreachable!(),
    };
    message_top!("Take your items.");
    for item_num in 0..num_of_items {
        let item_updated = item_num + 1;
        match items_vec.first().unwrap() {
            ItemEnum::Cigs => play_audio("pick_up_cigarettes.ogg"),
            ItemEnum::Saws => play_audio("pick_up_metal.ogg"),
            ItemEnum::MagGlass => play_audio("pick_up_magnifier.ogg"),
            ItemEnum::Beers => play_audio("pick_up_beer.ogg"),
            ItemEnum::Handcuffs => play_audio("pick_up_handcuffs.ogg"),
            ItemEnum::Adren => play_audio("pick_up_adrenaline.ogg"),
            ItemEnum::BurnPho => play_audio("pick_up_burner_phone.ogg"),
            ItemEnum::Invert => play_audio("pick_up_inverter.ogg"),
            ItemEnum::ExpMed => play_audio("pick_up_medicine.ogg"),
            ItemEnum::Nothing => unreachable!(),
        }
        message_top!(
            "Item #{item_updated} is {}. Place it in your inventory",
            items_vec[0]
        );
        let selection = dialogue(
            &mut game_info.player_inventory,
            "Your Inventory",
            Some(PlayerDealer::Player),
            true,
            false,
        )
        .unwrap();
        *PREVIOUS_INDEX.try_lock().unwrap() = selection;

        match items_vec.first().unwrap() {
            ItemEnum::Cigs => play_audio("place_down_cigarettes.ogg"),
            ItemEnum::Saws => play_audio("place_down_handsaw.ogg"),
            ItemEnum::MagGlass => play_audio("place_down_magnifier.ogg"),
            ItemEnum::Beers => play_audio("place_down_beer.ogg"),
            ItemEnum::Handcuffs => play_audio("place_down_handcuffs.ogg"),
            ItemEnum::Adren => play_audio("place_down_adrenaline.ogg"),
            ItemEnum::BurnPho => play_audio("place_down_burner_phone.ogg"),
            ItemEnum::Invert => play_audio("place_down_inverter.ogg"),
            ItemEnum::ExpMed => play_audio("place_down_medicine.ogg"),
            ItemEnum::Nothing => unreachable!(),
        }

        game_info.player_inventory[selection] = items_vec[0]; // replace item in player_inventory with items_vec[0]
        message_stats_func(game_info);
        items_vec.remove(0);
    }
}
