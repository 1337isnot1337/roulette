use ansi_term::Style;
use rand::seq::SliceRandom;
use rand::Rng;
use std::io::{self};
use std::process;
use std::thread;
use std::time::Duration;

macro_rules! italics {
    ($text:expr) => {{
        let styled_text = Style::new().italic().paint($text);
        println!("{}", styled_text);
    }};
}

fn main() {
    clearscreen::clear().expect("Failed to clear screen");
    let mut player_health: u8 = 3;
    let mut dealer_health: u8 = 3;
    loop {
        let live: u8 = rand::thread_rng().gen_range(2..=5);
        let blanks: u8 = rand::thread_rng().gen_range(2..=5);
        println!("----------------\n{live} lives and {blanks} blanks are loaded into the shotgun.\n----------------");
        let shell_vec = load_shells(live, blanks);
        //turn owner is used to switch between turns for player/dealer.
        //true means it is the players turn, false the dealer's turn.
        let mut turn_owner: bool = true;
        let mut turn = 1;
        //if perfect is on, the dealer will make optimal decisions every round.
        let perfect = false;
        let iter_shell_vec = shell_vec.clone();
        let size = 6;
        let mut items_vec = generate_items(size);
        let mut picked_items_vec: Vec<ItemEnum> = Vec::new();
        let mut amount_to_pick = 3;
        'item_pick_loop:  loop {
            
            println!("Pick {amount_to_pick} of the items.");
            let final_index = items_vec.len() - 1;

            for (index, item) in items_vec.iter().enumerate() {
                print!("{}", item);
                if index < final_index {
                    print!(", ");
                }
            }

            println!("");
            

            let mut item_choice = String::new();
            let _ = io::stdin().read_line(&mut item_choice).unwrap();
            let item_choice = item_choice.trim().split(' ');
            for item in item_choice {
                if amount_to_pick < 1 {
                    println!("You picked too many items. Automatically choosing your first picks.");
                    break 'item_pick_loop;
                }
                match item {
                    "cigs" => {
                        if let Some(index) = items_vec.iter().position(|&x| x == "cigs") {
                            items_vec.remove(index);
                            picked_items_vec.push(ItemEnum::Cigs);
                            amount_to_pick -= 1;
                        } else {
                            println!("Re-pick your items, and make sure you have enough. You picked an extra cig, it seems.");
                            continue 'item_pick_loop;
                        }
                    }
                    "beers" => {
                        if let Some(index) = items_vec.iter().position(|&x| x == "beers") {
                            items_vec.remove(index);
                            picked_items_vec.push(ItemEnum::Beers);
                            amount_to_pick -= 1;
                        } else {
                            println!("Re-pick your items, and make sure you have enough. You picked an extra beer, it seems.");
                            continue 'item_pick_loop;
                        }
                    }
                    "mag_glass" => {
                        if let Some(index) = items_vec.iter().position(|&x| x == "mag_glass") {
                            items_vec.remove(index);
                            picked_items_vec.push(ItemEnum::MagGlass);
                            amount_to_pick -= 1;
                        } else {
                            println!("Re-pick your items, and make sure you have enough. You picked an extra magnifying glass, it seems.");
                            continue 'item_pick_loop;
                        }
                    }
                    "saws" => {
                        if let Some(index) = items_vec.iter().position(|&x| x == "saws") {
                            items_vec.remove(index);
                            picked_items_vec.push(ItemEnum::Saws);
                            amount_to_pick -= 1;
                        } else {
                            println!("Re-pick your items, and make sure you have enough. You picked an extra saw, it seems.");
                            continue 'item_pick_loop;
                        }
                    }
                    _ => {
                        println!("Invalid item choice. Try again.");
                        continue 'item_pick_loop;
                    }
                }
            }
            break;
        }

        for shell in iter_shell_vec {
            //current bullets vec holds the bullets currently loaded
            let current_bullets_vec: Vec<bool> = shell_vec[turn - 1..].to_vec();
            println!("{}", Style::new().bold().paint(format!("Turn {turn}\n")));
            check_life(&player_health, &dealer_health);
            if turn_owner {
                your_turn(
                    shell,
                    &mut dealer_health,
                    &mut player_health,
                    &mut turn_owner,
                    &mut picked_items_vec,
                );
                check_life(&player_health, &dealer_health);
            } else {
                dealer_turn(
                    current_bullets_vec,
                    shell,
                    &mut dealer_health,
                    &mut player_health,
                    &mut turn_owner,
                    perfect,
                );
                check_life(&player_health, &dealer_health);
            }
            turn += 1;
            turn_owner = !turn_owner;
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn generate_items(len: usize) -> Vec<&'static str> {
    let saws: u8 = rand::thread_rng().gen_range(1..=3);
    let beers: u8 = rand::thread_rng().gen_range(1..6);
    let cigs: u8 = rand::thread_rng().gen_range(1..4);
    let mag_glass: u8 = rand::thread_rng().gen_range(1..4);
    let mut items_vec: Vec<&str> = Vec::new();
    for _ in 0..saws {
        items_vec.push("saws");
    }
    for _ in 0..beers {
        items_vec.push("beers");
    }
    for _ in 0..cigs {
        items_vec.push("cigs");
    }
    for _ in 0..mag_glass {
        items_vec.push("mag_glass");
    }
    let mut rng = rand::thread_rng();
    items_vec.as_mut_slice().shuffle(&mut rng);
    let trimmed_vec = items_vec.iter().take(len).copied().collect::<Vec<_>>();

    trimmed_vec
}
fn dealer_turn(
    current_bullets_vec: Vec<bool>,
    shell: bool,
    dealer_health: &mut u8,
    player_health: &mut u8,
    turn_owner: &mut bool,
    perfect: bool,
) {
    let choice: bool = if perfect {
        shell
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
    match choice {
        false => {
            println!("The dealer points the gun at its face.");
            thread::sleep(Duration::from_secs(1));
            if shell {
                println!("Dealer shot themselves.");
                *dealer_health -= 1;
            } else {
                italics!("click");
                println!("Extra turn for dealer.");
                *turn_owner = !*turn_owner;
            }
        }
        true => {
            println!("The dealer points the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if shell {
                println!("Dealer shot you.");
                *player_health -= 1;
            } else {
                italics!("click");
            }
        }
    }
    thread::sleep(Duration::from_secs(1));
}

/*

        "cigs" => *player_health += 1,
        "saws" => {
            damage = 2;
        }
        "mag_glass" => {}
        "beers" => {}
*/
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ItemEnum {
    Cigs,
    Saws,
    MagGlass,
    Beers,
    Nothing,
}

fn your_turn(
    shell: bool,
    dealer_health: &mut u8,
    player_health: &mut u8,
    turn_owner: &mut bool,
    picked_items_vec: &mut Vec<ItemEnum>,
) {
    let mut damage = 1;
    let mut choice = String::new();
    let item_use = loop {
        print!("Pick an item to use (or no). You have ( ");
        for item in &mut *picked_items_vec {
            print!("{item:?} ")
        }
        println!(")");

        

        let item_use = loop {
            let mut item_use = String::new();
            let _ = io::stdin().read_line(&mut item_use).unwrap();
            let item_use = item_use.trim();
            let item_use = match item_use {
                "cigs" => ItemEnum::Cigs,
                "beers" => ItemEnum::Beers,
                "mag_glass" => ItemEnum::MagGlass,
                "saws" => ItemEnum::Saws,
                "no" => ItemEnum::Nothing,
                _ => continue,
            };
            break item_use;
        };

        let mut verify: bool = false;
        for item in &mut *picked_items_vec {
            if item_use == *item {
                verify = true
            }
        }
        if item_use == ItemEnum::Nothing {
            verify = true
        }
        if !verify {
            println!("You have to have the item to use it.");
            continue;
        }
        break item_use;
    };
    match item_use {
        ItemEnum::Cigs => {
            if *player_health == 3 {
                println!(
                    "You light one of the cigs. Your head feels hazy. It doesn't seem to do much."
                );
            } else {
                println!("You light one of the cigs. Your head feels hazy, but you feel power coursing through your veins.");
                *player_health += 1
            }
            let index = picked_items_vec
                .iter()
                .position(|&x| x == ItemEnum::Cigs)
                .unwrap();
            picked_items_vec.remove(index);
        }
        ItemEnum::Saws => {
            println!("Shhk. You slice off the tip of the gun. It'll do 2 damage now.");
            damage = 2;
            let index = picked_items_vec
                .iter()
                .position(|&x| x == ItemEnum::Saws)
                .unwrap();
            picked_items_vec.remove(index);
        }
        ItemEnum::MagGlass => {
            if shell {
                println!("Upon closer inspection, you realize that there's a live round loaded.")
            } else {
                println!("Upon closer inspection, you realize that there's a blank round loaded.")
            }
            let index = picked_items_vec
                .iter()
                .position(|&x| x == ItemEnum::MagGlass)
                .unwrap();
            picked_items_vec.remove(index);
        }
        ItemEnum::Beers => {
            if shell {
                println!("You give the shotgun a pump. A live round drops out.")
            } else {
                println!("You give the shotgun a pump. A blank round drops out.")
            };
            let index = picked_items_vec
                .iter()
                .position(|&x| x == ItemEnum::Beers)
                .unwrap();
            picked_items_vec.remove(index);
            return;
        }
        ItemEnum::Nothing => {}
    }

    println!(
        "You have {player_health} lives remaining. The dealer has {dealer_health} lives remaining.\nShoot self or dealer?" 
    );

    io::stdin().read_line(&mut choice).unwrap();

    match choice.to_lowercase().as_str().trim() {
        "self" => {
            println!("You point the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if shell {
                println!("You shot yourself.");
                *player_health -= 1;
            } else {
                italics!("click");
                println!("Extra turn for you.");
                *turn_owner = !*turn_owner;
            }
        }
        "dealer" => {
            println!("You point the gun towards the dealer.");
            thread::sleep(Duration::from_secs(1));
            if shell {
                println!("You shot the dealer.");
                *dealer_health -= damage;
            } else {
                italics!("click");
            }
        }
        _ => {
            println!("Okay, you it is.");
            thread::sleep(Duration::from_secs(1));
            println!("You point the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if shell {
                println!("You shot yourself.");
                *player_health -= 1;
            } else {
                italics!("click");
                thread::sleep(Duration::from_secs(1));
                println!("Extra turn for you.");
                *turn_owner = !*turn_owner;
            }
        }
    }
    thread::sleep(Duration::from_secs(1));
}

//loading the shotgun shells
fn load_shells(live: u8, blanks: u8) -> Vec<bool> {
    let mut shells: Vec<bool> = Vec::new();
    for _i in 0..blanks {
        shells.push(false);
    }
    for _i in 0..live {
        shells.push(true);
    }
    let mut rng = rand::thread_rng();
    shells.as_mut_slice().shuffle(&mut rng);
    shells
}

//check the lives
fn check_life(player_health: &u8, dealer_health: &u8) {
    if *player_health < 1 {
        println!("You have no lives left. Game over.");
        process::exit(0);
    }
    if *dealer_health < 1 {
        println!("Dealer has no lives left. You win!");
        process::exit(0);
    }
    if *dealer_health > 3 {
        panic!("somethings gone wrong, dealer hp overflowed?")
    }
    if *player_health > 3 {
        panic!("somethings gone wrong, player hp overflowed?")
    }
}
