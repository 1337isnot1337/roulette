use ansi_term::Style;
use core::fmt;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{Clear, ClearType},
};
use dialoguer::FuzzySelect;
use rand::seq::SliceRandom;
use rand::Rng;
use std::io::{self, Write};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Example cleanup function
fn cleanup() {
    print!("\x1B[?25h");
    io::stdout().flush().unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemEnum {
    Cigs,
    Saws,
    MagGlass,
    Beers,
    Handcuffs,
    Nothing,
}
impl fmt::Display for ItemEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            ItemEnum::Cigs => "Cigarettes",
            ItemEnum::Saws => "Saw",
            ItemEnum::MagGlass => "Magnifying Glass",
            ItemEnum::Beers => "Beer",
            ItemEnum::Handcuffs => "Handcuffs",
            ItemEnum::Nothing => "No item",
        };
        write!(f, "{printable}")
    }
}

struct GameInfo {
    shell: bool,
    dealer_health: i8,
    player_health: i8,
    turn_owner: bool,
    player_stored_items: [ItemEnum; 8],
    dealer_stored_items: [ItemEnum; 8],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TargetEnum {
    Player,
    Dealer,
}
impl fmt::Display for TargetEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            TargetEnum::Player => "self",
            TargetEnum::Dealer => "dealer",
        };
        write!(f, "{printable}")
    }
}

macro_rules! italics {
    ($text:expr) => {{
        let styled_text = Style::new().italic().paint($text);
        println!("{}", styled_text);
    }};
}
fn picked_to_stored_dealer(
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
fn main() {
    // atomic boolean to track if sigint was received
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up  handler
    ctrlc::set_handler(move || {
        println!("Ctrl+C received, cleaning up...");
        cleanup();

        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    while running.load(Ordering::SeqCst) {
        clearscreen::clear().expect("Failed to clear screen");
        match play_screen() {
            Selection::Play => {
                let (player_health, dealer_health) = (3i8, 3i8);
                let (mut dealer_stored_items, mut player_stored_items) =
                    ([ItemEnum::Nothing; 8], [ItemEnum::Nothing; 8]);

                // add code for new items
                pick_items(&mut player_stored_items);

                dealer_stored_items =
                    picked_to_stored_dealer(generate_items(4), &mut dealer_stored_items);
                let mut live;
                let mut blanks;
                loop {
                    live = rand::thread_rng().gen_range(1..=4);
                    blanks = rand::thread_rng().gen_range(1..=4);
                    if (live + blanks) > 2 {
                        break;
                    }
                }
                println!("----------------\n{live} lives and {blanks} blanks are loaded into the shotgun.\n----------------\n");
                let shell_vec = load_shells(live, blanks);
                //turn owner is used to switch between turns for player/dealer.
                //true means it is the players turn, false the dealer's turn.
                let mut turn_owner: bool = true;
                let mut turn = 1;
                //if perfect is on, the dealer will make optimal decisions every round (knowing the bullet)
                let perfect = false;

                for shell in &shell_vec {
                    let mut game_info = GameInfo {
                        shell: *shell,
                        dealer_health,
                        player_health,
                        turn_owner,
                        player_stored_items,
                        dealer_stored_items,
                    };
                    //current bullets vec holds the bullets currently loaded
                    let current_bullets_vec: Vec<bool> = shell_vec[turn - 1..].to_vec();
                    println!("{}", Style::new().bold().paint(format!("Turn {turn}\n")));
                    println!(
                        "You have {player_health} lives remaining. The dealer has {dealer_health} lives remaining."
                    );
                    check_life(player_health, dealer_health);
                    if turn_owner {
                        your_turn(&mut game_info);
                    } else {
                        dealer_turn(current_bullets_vec, perfect, &mut game_info);
                    }
                    turn += 1;
                    turn_owner = !turn_owner;
                    thread::sleep(Duration::from_secs(1));
                }
            }
            Selection::Credits => credits(),
            Selection::Help => help(),
        }
    }
}

fn generate_items(len: usize) -> Vec<ItemEnum> {
    let saws: u8 = rand::thread_rng().gen_range(2..=6);
    let beers: u8 = rand::thread_rng().gen_range(2..7);
    let cigs: u8 = rand::thread_rng().gen_range(2..8);
    let mag_glass: u8 = rand::thread_rng().gen_range(2..7);
    let handcuffs: u8 = rand::thread_rng().gen_range(2..5);
    let mut items_vec: Vec<ItemEnum> = Vec::new();
    for _ in 0..saws {
        items_vec.push(ItemEnum::Saws);
    }
    for _ in 0..beers {
        items_vec.push(ItemEnum::Beers);
    }
    for _ in 0..cigs {
        items_vec.push(ItemEnum::Cigs);
    }
    for _ in 0..mag_glass {
        items_vec.push(ItemEnum::MagGlass);
    }
    for _ in 0..handcuffs {
        items_vec.push(ItemEnum::Handcuffs);
    }
    let mut rng = rand::thread_rng();
    for _ in 0..10 {
        items_vec.as_mut_slice().shuffle(&mut rng);
    }

    //yes ik its overkill but this is my code not urs
    let trimmed_vec = items_vec.iter().take(len).copied().collect::<Vec<_>>();

    trimmed_vec
}

fn handle_items(item_enum_type: ItemEnum, items_vec: &mut Vec<ItemEnum>) {
    let index = items_vec.iter().position(|&x| x == item_enum_type).unwrap();
    items_vec.remove(index);
}

fn pick_items(player_stored_items: &mut [ItemEnum; 8]) {
    let mut items_vec = generate_items(8);
    let mut amount_to_pick = 4;
    let mut i = 0;

    while amount_to_pick > 0 {
        println!("You got {}, where are you going to place it?", items_vec[i]);
        let selection = FuzzySelect::new()
            .with_prompt("Store the item")
            .report(false)
            .items(player_stored_items)
            .interact()
            .unwrap();

        player_stored_items[selection] = items_vec[i]; // replace item in player_stored_items with items_vec[i]

        match items_vec[i] {
            ItemEnum::Cigs => {
                handle_items(ItemEnum::Cigs, &mut items_vec);
                amount_to_pick -= 1;
            }
            ItemEnum::Saws => {
                handle_items(ItemEnum::Saws, &mut items_vec);
                amount_to_pick -= 1;
            }
            ItemEnum::MagGlass => {
                handle_items(ItemEnum::MagGlass, &mut items_vec);
                amount_to_pick -= 1;
            }
            ItemEnum::Handcuffs => {
                handle_items(ItemEnum::Handcuffs, &mut items_vec);
                amount_to_pick -= 1;
            }
            ItemEnum::Beers => {
                handle_items(ItemEnum::Beers, &mut items_vec);
                amount_to_pick -= 1;
            }

            ItemEnum::Nothing => todo!(),
        }
        i += 1;
    }
}

fn dealer_item_use(
    item_type: ItemEnum,
    dealer_stored_items: &mut [ItemEnum; 8],
    dealer_health: &mut i8,
    shell: bool,
    damage: &mut u8,
    turn_owner: &mut bool,
) -> bool {
    let mut knowledge_of_shell = false;
    match item_type {
        ItemEnum::Cigs => {
            if *dealer_health == 3 {
                println!(
                "ERROR: THIS CODE SHOULD NOT BE REACHABLE! PLEASE REPORT THIS BUG. The dealer lights one of the cigs."
            );
            } else {
                println!("The dealer lights one of the cigs.");
                *dealer_health += 1;
            }
            remove_no_item(dealer_stored_items, ItemEnum::Cigs);
        }
        ItemEnum::Saws => {
            println!("Shhk. The dealer slices off the tip of the gun. It'll do 2 damage now.");
            *damage = 2;
            remove_no_item(dealer_stored_items, ItemEnum::Saws);
        }
        ItemEnum::MagGlass => {
            println!(
                "The dealer looks down at the gun with an old magnifying glass. You see him smirk."
            );
            knowledge_of_shell = true;

            remove_no_item(dealer_stored_items, ItemEnum::MagGlass);
        }
        ItemEnum::Beers => {
            if shell {
                println!("The dealer gives the shotgun a pump. A live round drops out.");
            } else {
                println!("The dealer gives the shotgun a pump. A blank round drops out.");
            };
            *turn_owner = !*turn_owner;
            remove_no_item(dealer_stored_items, ItemEnum::Beers);
        }
        ItemEnum::Handcuffs => {
            println!("The dealer grabs onto your hand. When he lets go, your hands are cuffed.");
            if *turn_owner {
                *turn_owner = !*turn_owner;
            }
            remove_no_item(dealer_stored_items, ItemEnum::Handcuffs);
        }

        ItemEnum::Nothing => {
            println!("ERROR: THIS CODE SHOULD NOT BE REACHABLE! PLEASE REPORT THIS BUG.")
        }
    }
    knowledge_of_shell
}

fn dealer_turn(current_bullets_vec: Vec<bool>, perfect: bool, game_info: &mut GameInfo) {
    /*
    dealer item impl
     */
    let mut damage = 1;

    // future goal: add logic for having dealer pick certain items
    let mut shell_knowledge = false;
    let mut handcuff_player: bool = false;
    let coinflip: bool = rand::thread_rng().gen();
    'dealer_use_items: loop {
        if game_info.dealer_stored_items.contains(&ItemEnum::Cigs) & { game_info.dealer_health < 3 }
        {
            dealer_item_use(
                ItemEnum::Cigs,
                &mut game_info.dealer_stored_items,
                &mut game_info.dealer_health,
                game_info.shell,
                &mut damage,
                &mut game_info.turn_owner,
            );
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::MagGlass) & !shell_knowledge {
            shell_knowledge = dealer_item_use(
                ItemEnum::MagGlass,
                &mut game_info.dealer_stored_items,
                &mut game_info.dealer_health,
                game_info.shell,
                &mut damage,
                &mut game_info.turn_owner,
            );
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Saws)
            & shell_knowledge
            & game_info.shell
        {
            dealer_item_use(
                ItemEnum::Saws,
                &mut game_info.dealer_stored_items,
                &mut game_info.dealer_health,
                game_info.shell,
                &mut damage,
                &mut game_info.turn_owner,
            );
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Handcuffs) & !handcuff_player {
            dealer_item_use(
                ItemEnum::Handcuffs,
                &mut game_info.dealer_stored_items,
                &mut game_info.dealer_health,
                game_info.shell,
                &mut damage,
                &mut game_info.turn_owner,
            );
            handcuff_player = !handcuff_player;
            continue 'dealer_use_items;
        }
        if game_info.dealer_stored_items.contains(&ItemEnum::Beers) & !shell_knowledge & coinflip {
            dealer_item_use(
                ItemEnum::Beers,
                &mut game_info.dealer_stored_items,
                &mut game_info.dealer_health,
                game_info.shell,
                &mut damage,
                &mut game_info.turn_owner,
            );
            break 'dealer_use_items;
        }
        break;
    }

    let choice: bool = if perfect | shell_knowledge {
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
    if choice {
        println!("The dealer points the gun at your face.");
        thread::sleep(Duration::from_secs(1));
        if game_info.shell {
            turn_screen_red();
            println!("Dealer shot you.");
            game_info.player_health -= 1;
        } else {
            //play_audio("audio/blank.mp3");
            italics!("click");
        }
    } else {
        println!("The dealer points the gun at its face.");
        thread::sleep(Duration::from_secs(1));
        if game_info.shell {
            turn_screen_red();
            println!("Dealer shot themselves.");
            game_info.dealer_health -= 1;
        } else {
            //play_audio("audio/blank.mp3");
            italics!("click");
            println!("Extra turn for dealer.");
            game_info.turn_owner = !game_info.turn_owner;
        }
    }

    thread::sleep(Duration::from_secs(1));
    check_life(game_info.player_health, game_info.dealer_health);
}

fn your_turn(game_info: &mut GameInfo) {
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
                game_info.turn_owner = !game_info.turn_owner;
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Handcuffs);
            }
            ItemEnum::Beers => {
                if game_info.shell {
                    println!("You give the shotgun a pump. A live round drops out.");
                } else {
                    println!("You give the shotgun a pump. A blank round drops out.");
                };
                game_info.turn_owner = !game_info.turn_owner;
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Beers);
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing);
            }
            ItemEnum::Nothing => {
                remove_no_item(&mut game_info.player_stored_items, ItemEnum::Nothing)
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

    resolve_user_choice(
        choice,
        game_info.shell,
        &mut game_info.player_health,
        &mut game_info.dealer_health,
        &mut game_info.turn_owner,
        damage,
    );
    thread::sleep(Duration::from_secs(1));
    check_life(game_info.player_health, game_info.dealer_health);
}

fn resolve_user_choice(
    choice: TargetEnum,
    shell: bool,
    player_health: &mut i8,
    dealer_health: &mut i8,
    turn_owner: &mut bool,
    damage: i8,
) {
    match choice {
        TargetEnum::Player => {
            println!("You point the gun at your face.");
            thread::sleep(Duration::from_secs(1));
            if shell {
                turn_screen_red();

                println!("You shot yourself.");
                *player_health -= 1;
            } else {
                //play_audio("audio/blank.mp3");
                italics!("click");
                thread::sleep(Duration::from_secs(1));
                println!("Extra turn for you.");
                *turn_owner = !*turn_owner;
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
                //play_audio("audio/blank.mp3");
                italics!("click");
            }
        }
    }
}

fn remove_no_item(picked_items_vec: &mut [ItemEnum; 8], item_type: ItemEnum) {
    if let Some(index) = picked_items_vec.iter().position(|&x| x == item_type) {
        picked_items_vec[index] = ItemEnum::Nothing;
    } else {
        println!("Item {:?} not found in the array", item_type);
    }
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
fn check_life(player_health: i8, dealer_health: i8) {
    if player_health < 1 {
        println!("You have no lives left. Game over.");
        print!("\x1B[?25h");
        io::stdout().flush().unwrap();
        process::exit(0);
    }
    if dealer_health < 1 {
        println!("Dealer has no lives left. You win!");
        print!("\x1B[?25h");
        io::stdout().flush().unwrap();
        process::exit(0);
    }
    assert!(
        dealer_health <= 3,
        "somethings gone wrong, dealer hp overflowed?"
    );
    assert!(
        player_health <= 3,
        "somethings gone wrong, player hp overflowed?"
    );
}

/*fn play_audio(path: &str) {
    // Clone path for use in the thread
    let path = path.to_string();

    // Spawn a new thread to play audio asynchronously
    thread::spawn(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = BufReader::new(File::open(&path).unwrap());
        let source = Decoder::new(file).unwrap().convert_samples();

        stream_handle.play_raw(source).unwrap();

        // Calculate duration of the audio file
        let duration = mp3_duration::from_path(Path::new(&path)).unwrap();
        let duration_secs = duration.as_secs() + if duration.subsec_millis() > 0 { 1 } else { 0 };

        // Sleep for the duration of the audio
        thread::sleep(Duration::from_secs(duration_secs));
    });
}
*/

fn turn_screen_red() {
    // Execute crossterm commands to clear screen and set red background
    let mut chunk = String::new();
    let mut space = 9000;
    //play_audio("audio/live.mp3");
    while space > 0 {
        chunk.push(' ');

        space -= 1;
    }

    execute!(
        io::stdout(),
        Clear(ClearType::All),          // Clear the screen
        SetBackgroundColor(Color::Red), // Set background color to red
        Print(chunk),                   // Print a dummy character to fill the screen with red
        ResetColor                      // Reset colors to default after printing
    )
    .expect("Failed to execute crossterm commands");
    thread::sleep(Duration::from_millis(500));

    // Flush stdout to ensure color change is immediate
    io::stdout().flush().expect("Failed to flush stdout");
    clearscreen::clear().expect("Failed to clear screen");
}


fn credits() {
    clearscreen::clear().expect("Failed to clear screen");
    println!("All (I think) code made by 1337isnot1337 (or whatever my current github name is)");
    thread::sleep(Duration::from_secs(5));
    play_screen();

}
fn help() {
    clearscreen::clear().expect("Failed to clear screen");
    /*println!("How does Buckshot Roulette play?

    Interestingly, this isn’t an action game. BR is a tabletop game where you enter the dingiest nightclub ever, see the gun getting loaded with a certain number of rounds in an unknown order, and hope for the best as you face off against the dealer. It’s not as simple as just a game of randomly taking turns, but, as with all things horror, it’s all best left unspoiled. Buckshot Roulette is rather original — as a video game, at least — but, if I’m to compare it to anything else out there, I’d go with saying it’s sure to please fans of Inscryption.
    
    A full playthrough will likely not take you more than 20 minutes — yes, even if you don’t blow yourself up.
    
    It plays just like a real tabletop game would, down to it featuring pretty much no interface clutter. All the information that you get, you get from the game.
    
    I assure you that even though the results of the game are unpredictable due to the random factor involved, the AI doesn’t know more than it should. The dealer is a very unsettling creature, but it plays fair.
    
    Even though this is an indie game through and through, developer Mark Klubnika warns that you must own what they call a “relatively modern” dedicated graphics card that features Vulkan support.
    What are the items in Buckshot Roulette?
    
    Here are all of the items in Buckshot Roulette:
    Handcuffs	Causes the dealer to skip his next turn
    Hand Saw	Doubles the damage of your shotgun. Great combo with the Magnifying glass
    Beer	Ejects the shell that’s in the chamber
    Pills	Begins a subgame of “double or nothing”*
    Cigarettes	heals one life point
    Magnifying Glass	Allows you to examine the shell currently in the chamber
    
    If you take the pills, you’ll begin a game of double or nothing, which will introduce a new set of items. Let’s look into those:
    Double Or Nothing items
    Inverter	Reverses the effect of the shell currently in the chamber. A blank will turn into a live round and a live round turns into a blank.
    Adrenaline	The player will steal an item from the dealer’s board and use it immediately
    Burner phone	Will inform you about a random round in your stack
    Expired medicine	40% chance of regaining two lives, 60% chance of losing one life
    
    You can get Buckshot Roulette for $2.99 on Steam, or you can get it from itch.io for $1.20 right here.");
    */
    thread::sleep(Duration::from_secs(5));
    play_screen();

}


fn play_screen() -> Selection {
    clearscreen::clear().expect("Failed to clear screen");
    let options_vec: [Selection; 3] = [Selection::Play, Selection::Help, Selection::Credits];
    let selection = FuzzySelect::new()
        .with_prompt("What do you choose?")
        .items(&options_vec)
        .interact()
        .unwrap();

    options_vec[selection]
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Selection {
    Play,
    Help,
    Credits,
}
impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            Selection::Play => "Play",
            Selection::Help => "Help",
            Selection::Credits => "Credits",
        };
        write!(f, "{printable}")
    }
}
