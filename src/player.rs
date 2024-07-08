use dialoguer::FuzzySelect;
use rand::Rng;
use std::{
    thread,
    time::Duration,
};
use crate::{check_life, generate_items, italics, play_audio, remove_no_item, turn_screen_red, GameInfo, ItemEnum, TargetEnum};

#[allow(clippy::too_many_lines)]
pub fn turn(game_info: &mut GameInfo) -> bool {
    let mut damage: i8 = 1;
    'item_selection_loop: loop {
        let selection = FuzzySelect::new()
            .with_prompt("Pick an item to use")
            .items(&game_info.player_stored_items)
            .interact()
            .unwrap();

        let item_use = &mut game_info.player_stored_items[selection];
        match item_use {
            ItemEnum::Cigs => {
                if game_info.player_health == 3 {
                    println!(
                    "You light one of the cigs. Your head feels hazy. It doesn't seem to do much."
                );
                } else {
                    println!("You light one of the cigs. Your head feels hazy, but you feel power coursing through your veins.");
                    game_info.player_health += 1;
                }
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Cigs);
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing);
                continue 'item_selection_loop;
            }
            ItemEnum::Saws => {
                println!("Shhk. You slice off the tip of the gun. It'll do 2 damage now.");
                damage = 2;
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Saws);
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing);
                continue 'item_selection_loop;
            }
            ItemEnum::MagGlass => {
                if game_info.shell {
                    println!(
                        "Upon closer inspection, you realize that there's a live round loaded."
                    );
                } else {
                    println!(
                        "Upon closer inspection, you realize that there's a blank round loaded."
                    );
                }
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::MagGlass);
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing);
                continue 'item_selection_loop;
            }
            ItemEnum::Handcuffs => {
                println!(
                    "The dealer grabs the handcuffs from your outstretched hand, putting them on."
                );
                match game_info.turn_owner {
                    TargetEnum::Player => {
                        game_info.turn_owner = TargetEnum::Dealer;
                    }
                    TargetEnum::Dealer => {
                        game_info.turn_owner = TargetEnum::Player;
                    }
                }
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Handcuffs);
            }
            ItemEnum::Beers => {
                if game_info.shell {
                    println!("You give the shotgun a pump. A live round drops out.");
                } else {
                    println!("You give the shotgun a pump. A blank round drops out.");
                };
                match game_info.turn_owner {
                    TargetEnum::Player => {
                        game_info.turn_owner = TargetEnum::Dealer;
                    }
                    TargetEnum::Dealer => {
                        game_info.turn_owner = TargetEnum::Player;
                    }
                }
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Beers);
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing);
            }
            ItemEnum::Adren => {
                println!("You jam the rusty needle into your thigh.");
                let stolen_item = game_info.dealer_stored_items[FuzzySelect::new()
                    .with_prompt("Pick an item to steal from the dealer")
                    .items(&game_info.dealer_stored_items)
                    .interact()
                    .unwrap()];

                remove_no_item(&mut game_info.dealer_stored_items, stolen_item);
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Adren);
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing);
                todo!("give the player the item")
            }
            ItemEnum::BurnPho => {
                let shell_number: usize =
                    rand::thread_rng().gen_range(0..{ game_info.shells_vector.len() });
                let shell_reveal = if game_info.shells_vector[shell_number] {
                    "live"
                } else {
                    "blank"
                };
                let place = match shell_number.try_into().unwrap() {
                    1 => "first",
                    2 => "second",
                    3 => "third",
                    4 => "fourth",
                    5 => "fifth",
                    6 => "sixth",
                    7 => "seventh",
                    8 => "eigth",
                    _ => panic!("Burner phone panic; number larger than 8. Report this error!"),
                };
                println!("You flip open the phone. The {place} shell is {shell_reveal}");
            }
            ItemEnum::Invert => {
                todo!()
            }
            ItemEnum::ExpMed => {}
            ItemEnum::Nothing => {
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing);
            }
        }
        break;
    }

    let targets: [TargetEnum; 2] = [TargetEnum::Player, TargetEnum::Dealer];
    let selection = FuzzySelect::new()
        .with_prompt("Shoot self or Dealer")
        .items(&targets)
        .interact()
        .unwrap();

    let choice = targets[selection];

    let extraturn = resolve_player_choice(
        choice,
        game_info.shell,
        &mut game_info.player_health,
        &mut game_info.dealer_health,
        damage,
    );
    thread::sleep(Duration::from_secs(1));
    check_life(game_info.player_health, game_info.dealer_health);
    extraturn
}

fn resolve_player_choice(
    choice: TargetEnum,
    shell: bool,
    player_health: &mut i8,
    dealer_health: &mut i8,
    damage: i8,
) -> bool {
    let mut extraturn = false;
    match choice {
        TargetEnum::Player => {
            println!("You point the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if shell {
                turn_screen_red();

                println!("You shot yourself.");
                *player_health -= 1;
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
            if shell {
                turn_screen_red();

                println!("You shot the dealer.");

                *dealer_health -= damage;
            } else {
                play_audio("temp_gunshot_blank.wav");
                italics("click");
            }
        }
    }
    extraturn
}

pub fn pick_items(player_stored_items: &mut [ItemEnum; 8], doub_or_noth: bool) {
    let mut items_vec = generate_items(8, doub_or_noth);
    for i in 0..4 {
        println!("You got {}, where are you going to place it?", items_vec[i]);
        let selection = FuzzySelect::new()
            .with_prompt("Store the item")
            .report(false)
            .items(player_stored_items)
            .interact()
            .unwrap();

        player_stored_items[selection] = items_vec[i]; // replace item in player_stored_items with items_vec[i]
        let index = items_vec.iter().position(|&x| x == items_vec[i]).unwrap();
        items_vec.remove(index);
    }
}
