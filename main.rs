use ansi_term::Style;
use rand::seq::SliceRandom;
use rand::{random, Rng};
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
    let mut player_health: u8 = 3;
    let mut dealer_health: u8 = 3;
    play(&mut dealer_health, &mut player_health);
}
fn dealer_turn(
    shell: bool,
    dealer_health: &mut u8,
    player_health: &mut u8,
    turn_owner: &mut bool,
    perfect: bool,
) {
    let choice: bool = if perfect {
        shell
    } else {
        random()
    };
    //true means dealer shoots you, false means dealer shoots itself
    match choice {
        true => {
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
        false => {
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

fn play(dealer_health: &mut u8, player_health: &mut u8) {
    loop {
        let live: u8 = rand::thread_rng().gen_range(2..=5);
        let blanks: u8 = rand::thread_rng().gen_range(2..=3);
        println!("----------------\n{live} lives and {blanks} blanks are loaded into the shotgun.\n----------------");
        let shell_vec = load_shells(live, blanks);
        //turn owner is used to switch between turns for player/dealer.
        //true means it is the players turn, false the dealer's turn.
        let mut turn_owner: bool = true;
        let mut turn = 1;
        //if perfect is on, the dealer will make optimal decisions every round.
        let perfect = false;

        for shell in shell_vec {
            println!("\nRound {turn}.");
            if turn_owner {
                your_turn(shell, dealer_health, player_health, &mut turn_owner);
                check_life(player_health, dealer_health);
            } else {
                dealer_turn(
                    shell,
                    dealer_health,
                    player_health,
                    &mut turn_owner,
                    perfect,
                );
                check_life(player_health, dealer_health);
            }
            turn += 1;
            turn_owner = !turn_owner;
            thread::sleep(Duration::from_secs(1));
        }
    }
}

fn your_turn(shell: bool, dealer_health: &mut u8, player_health: &mut u8, turn_owner: &mut bool) {
    let mut choice = String::new();
    println!(
        "You have {player_health} lives remaining. The dealer has {dealer_health} lives remaining."
    );
    println!("Shoot Self or Dealer?");
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
                *dealer_health -= 1;
            } else {
                italics!("click");
            }
        }
        _ => {
            println!("Okay, you it is.");
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
}
