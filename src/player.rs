use crate::local_ratatui::{dialogue, message_stats_func};
use crate::message_top_func;
use crate::{
    check_life, generate_items, message_top, play_audio, remove_item, turn_screen_red, GameInfo,
    ItemEnum, TargetEnum,
};
use rand::Rng;
use std::{thread, time::Duration};

pub fn turn(game_info: &mut GameInfo) -> (bool, bool) {
    let mut empty_due_to_beer = false;
    message_stats_func(game_info);
    let mut cuffed = false;
    let mut damage: i8 = 1;
    message_top!("Select items to use");
    'item_selection_loop: loop {
        if empty_due_to_beer {
            message_top!("init break recieved");
            break 'item_selection_loop;
        }
        message_stats_func(game_info);
        let selection = dialogue(&game_info.player_inventory, "Items list:");

        let item_type = game_info.player_inventory[selection];
        remove_item(&mut game_info.player_inventory, item_type);
        match item_type {
            ItemEnum::Cigs => {
                play_audio("player_use_cigarettes.ogg");
                if game_info.player_health == 3 {
                    message_top!(
                    "You light one of the cigs. Your head feels hazy. It doesn't seem to do much."
                );
                } else {
                    message_top!("You light one of the cigs. Your head feels hazy, but you feel power coursing through your veins.");
                    game_info.player_health += 1;
                }

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
                if game_info.shells_vector[0] {
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
                play_audio("player_use_beer.ogg");
                if game_info.shells_vector.len() == 1 {
                    empty_due_to_beer = true;
                }
                if game_info.shells_vector[0] {
                    message_top!("You give the shotgun a pump. A live round drops out.");
                } else {
                    message_top!("You give the shotgun a pump. A blank round drops out.");
                };
                game_info.shells_vector.remove(0);

                continue 'item_selection_loop;
            }
            ItemEnum::Nothing => {}
            _ => double_or_nothing_items(item_type, game_info),
        }
        break;
    }
    let mut extraturn = false;
    if !empty_due_to_beer {
        message_stats_func(game_info);
        let targets: [TargetEnum; 2] = [TargetEnum::Player, TargetEnum::Dealer];
        let selection = dialogue(&targets, "Who to shoot?");
        message_stats_func(game_info);

        let choice = targets[selection];
        extraturn = resolve_player_choice(choice, damage, game_info, cuffed);
        
        thread::sleep(Duration::from_secs(1));
        check_life(game_info);
        message_stats_func(game_info);
    }
    

    (extraturn, empty_due_to_beer)
}

fn double_or_nothing_items(item_type: ItemEnum, game_info: &mut GameInfo) {
    message_stats_func(game_info);
    match item_type {
        ItemEnum::Adren => {
            play_audio("player_use_adrenaline.ogg");
            message_top!("You jam the rusty needle into your thigh.");
            let stolen_item = game_info.dealer_stored_items[dialogue(
                &game_info.dealer_stored_items,
                "Pick one of the dealer's items",
            )];
            if stolen_item == ItemEnum::Adren {
                message_top!("You can't grab the adrenaline.");
            } else {
                message_top!("You grab the {item_type}, and use it.");

                remove_item(&mut game_info.dealer_stored_items, stolen_item);
                todo!("you need to code the usage part bozo");
            }
        }
        ItemEnum::BurnPho => {
            play_audio("player_use_burner_phone.ogg");
            let shell_number: usize =
                rand::thread_rng().gen_range(0..{ game_info.shells_vector.len() });
            let shell_reveal = if game_info.shells_vector[shell_number] {
                "live"
            } else {
                "blank"
            };

            let place = match shell_number.try_into().unwrap() {
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
            game_info.shells_vector[0] = !game_info.shells_vector[0];
        }
        ItemEnum::ExpMed => {
            play_audio("player_use_medicine.ogg");
            message_top!("You takes the expired medicine.");
            let coinflip: bool = rand::thread_rng().gen();
            if coinflip {
                game_info.player_health += 1;
                message_top!("You feel energy coursing through you.");
            } else {
                game_info.player_health -= 2;
                message_top!("You choke and fall over.");
            }

            todo!("give the player the item");
        }
        _ => unreachable!(),
    }
    remove_item(&mut game_info.player_inventory, item_type);
}

fn resolve_player_choice(
    choice: TargetEnum,
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
        TargetEnum::Player => {
            message_top!("You point the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();
                message_top!("You shot yourself.");

                game_info.player_health -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                message_top!("click");
                thread::sleep(Duration::from_secs(1));
                message_top!("Extra turn for you.");

                extraturn = true;
            }
        }
        TargetEnum::Dealer => {
            message_top!("You point the gun towards the dealer.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();

                message_top!("You shot the dealer.");

                game_info.dealer_health -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                message_top!("click");
            }
        }
    }
    game_info.shells_vector.remove(0);
    message_stats_func(game_info);
    extraturn
}

pub fn pick_items(game_info: &mut GameInfo) {
    let items_vec = &mut generate_items(8, game_info);

    message_top!("Take your items.");
    for item_num in 0..4 {
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
        let selection = dialogue(&game_info.player_inventory, "Inventory");

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
