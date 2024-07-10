use crate::{
    check_life, generate_items, italics, play_audio, remove_item, turn_screen_red, GameInfo,
    ItemEnum, TargetEnum,
};
use dialoguer::Select;
use rand::Rng;
use std::{thread, time::Duration};


fn double_or_nothing_items(item_type: ItemEnum, game_info: &mut GameInfo) {

    match item_type {
        ItemEnum::Adren => {
            play_audio("player_use_adrenaline.ogg");
            println!("You jam the rusty needle into your thigh.");
            let stolen_item = game_info.dealer_stored_items[Select::new()
                .with_prompt("Pick an item to steal from the dealer")
                .items(&game_info.dealer_stored_items)
                .interact()
                .unwrap()];
            remove_item(&mut game_info.dealer_stored_items, stolen_item);
            remove_item(&mut game_info.player_inventory, item_type);
            todo!("give the player the item");
            
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
            println!("You flip open the phone. The {place} shell is {shell_reveal}");
            remove_item(&mut game_info.player_inventory, item_type);
            
        }
        ItemEnum::Invert => {
            println!("You flick the switch on the inverter.");
            play_audio("player_use_inverter.ogg");
            game_info.shells_vector[0] = !game_info.shells_vector[0];
            remove_item(&mut game_info.player_inventory, item_type);
            todo!("give the player the item");
            
        }
        ItemEnum::ExpMed => {
            play_audio("player_use_medicine.ogg");
            println!("You takes the expired medicine.");
            let coinflip: bool = rand::thread_rng().gen();
            if coinflip {
                game_info.player_health += 1;
                println!("You feel energy coursing through you.");
            } else {
                game_info.player_health -= 2;
                println!("You choke and fall over.");
            }
            remove_item(&mut game_info.dealer_stored_items, item_type);
            todo!("give the player the item");
            
        }
        _ => panic!("nooo (w error message chat?!?!??!)"),
    }
    
}

#[allow(clippy::too_many_lines)]
pub fn turn(game_info: &mut GameInfo) -> bool {
    let mut cuffed = false;
    let mut damage: i8 = 1;
    'item_selection_loop: loop {
        let selection = Select::new()
            .with_prompt("Pick an item to use")
            .items(&game_info.player_inventory)
            .interact()
            .unwrap();

        let item_type = game_info.player_inventory[selection];
        match item_type {
            ItemEnum::Cigs => {
                play_audio("player_use_cigarettes.ogg");
                if game_info.player_health == 3 {
                    println!(
                    "You light one of the cigs. Your head feels hazy. It doesn't seem to do much."
                );
                } else {
                    println!("You light one of the cigs. Your head feels hazy, but you feel power coursing through your veins.");
                    game_info.player_health += 1;
                }
                remove_item(&mut game_info.player_inventory, item_type);
                continue 'item_selection_loop;
            }
            ItemEnum::Saws => {
                play_audio("player_use_handsaw.ogg");
                println!("Shhk. You slice off the tip of the gun. It'll do 2 damage now.");
                damage = 2;
                remove_item(&mut game_info.player_inventory, item_type);
                continue 'item_selection_loop;
            }
            ItemEnum::MagGlass => {
                play_audio("player_use_magnifier.ogg");
                if game_info.shells_vector[0] {
                    println!(
                        "Upon closer inspection, you realize that there's a live round loaded."
                    );
                } else {
                    println!(
                        "Upon closer inspection, you realize that there's a blank round loaded."
                    );
                }
                remove_item(&mut game_info.player_inventory, item_type);
                continue 'item_selection_loop;
            }
            ItemEnum::Handcuffs => {
                play_audio("player_use_handcuffs.ogg");
                println!(
                    "The dealer grabs the handcuffs from your outstretched hand, putting them on."
                );
                cuffed = true;
                remove_item(&mut game_info.player_inventory, item_type);
                continue 'item_selection_loop;
            }
            ItemEnum::Beers => {
                play_audio("player_use_beer.ogg");
                if game_info.shells_vector.len() == 1 {
                    println!("You take the beer, but since there's only one round left, you're unable to rack the gun.");
                } else {
                    if game_info.shells_vector[0] {
                        println!("You give the shotgun a pump. A live round drops out.");
                    } else {
                        println!("You give the shotgun a pump. A blank round drops out.");
                    };
                    game_info.shells_vector.remove(0);
                }
                
                
                remove_item(&mut game_info.player_inventory, item_type);
                continue 'item_selection_loop;
            }
            ItemEnum::Nothing => {}
            _ => double_or_nothing_items(item_type, game_info)
        }
        break;
    }

    let targets: [TargetEnum; 2] = [TargetEnum::Player, TargetEnum::Dealer];
    let selection = Select::new()
        .with_prompt("Shoot self or Dealer")
        .items(&targets)
        .interact()
        .unwrap();

    let choice = targets[selection];

    let extraturn = resolve_player_choice(choice, damage, game_info, cuffed);
    thread::sleep(Duration::from_secs(1));
    check_life(game_info.player_health, game_info.dealer_health);

    extraturn
}

fn resolve_player_choice(
    choice: TargetEnum,
    damage: i8,
    game_info: &mut GameInfo,
    cuffed: bool,
) -> bool {
    let mut extraturn = false;
    if cuffed {
        extraturn = true;
    }
    match choice {
        TargetEnum::Player => {
            println!("You point the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();
                println!("You shot yourself.");
                
                game_info.player_health -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                italics("click");
                thread::sleep(Duration::from_secs(1));
                println!("Extra turn for you.");
                
                extraturn = true;
            }
        }
        TargetEnum::Dealer => {
            println!("You point the gun towards the dealer.");
            thread::sleep(Duration::from_secs(1));
            if game_info.shells_vector[0] {
                turn_screen_red();

                println!("You shot the dealer.");
                
                game_info.dealer_health -= damage;
                
            } else {
                play_audio("temp_gunshot_blank.wav");
                italics("click");
            }
        }
    }
    game_info.shells_vector.remove(0);
    extraturn
}

pub fn pick_items(player_inventory: &mut [ItemEnum; 8], doub_or_noth: bool) {
    let items_vec = &mut generate_items(8, doub_or_noth);
    println!("Take your items.");
    for item_num in 0..4 {
        let item_updated = item_num + 1;
        println!("Item #{item_updated} is {}. Place it in your inventory", items_vec[0]);
        let selection = Select::new().items(player_inventory).interact().unwrap();
        player_inventory[selection] = items_vec[0]; // replace item in player_inventory with items_vec[i]
        items_vec.remove(0);
    }
}
