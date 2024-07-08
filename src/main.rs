use ansi_term::Style;
use core::fmt;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor},
    terminal::{Clear, ClearType},
};
use dealer::picked_to_stored;
use dialoguer::FuzzySelect;
use once_cell::sync::Lazy;
use player::pick_items;
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

mod dealer;
mod player;

static AUDIO_HANDLE: Lazy<OutputStreamHandle> = Lazy::new(|| {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    mem::forget(stream);
    stream_handle
});

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemEnum {
    Cigs,
    Saws,
    MagGlass,
    Beers,
    Handcuffs,
    Adren,
    BurnPho,
    Invert,
    ExpMed,
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
            ItemEnum::Adren => "Adrenaline",
            ItemEnum::BurnPho => "Burner Phone",
            ItemEnum::Invert => "Inverter",
            ItemEnum::ExpMed => "Expired Medicine",
            ItemEnum::Nothing => "No item",
        };
        write!(f, "{printable}")
    }
}

struct GameInfo {
    dealer_health: i8,
    player_health: i8,
    turn_owner: TargetEnum,
    player_inventory: [ItemEnum; 8],
    dealer_stored_items: [ItemEnum; 8],
    perfect: bool,
    double_or_nothing: bool,
    shells_vector: Vec<bool>,
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
    play_audio("music/music_main_techno_techno.ogg");

    while running.load(Ordering::SeqCst) {
        let mut perfect = false;
        let mut double_or_nothing = false;
        let args: Vec<String> = env::args().collect();
        if args.contains(&"--perfect".to_string()) {
            perfect = true;
        }
        if args.contains(&"--double".to_string()) {
            double_or_nothing = true;
        }

        clearscreen::clear().expect("Failed to clear screen");
        let (player_health, dealer_health) = (3i8, 3i8);
        match play_screen() {
            Selection::Play => {
                loop {
                    let (mut dealer_stored_items, mut player_inventory) =
                        ([ItemEnum::Nothing; 8], [ItemEnum::Nothing; 8]);

                    // add code for new items
                    pick_items(&mut player_inventory, double_or_nothing);

                    dealer_stored_items = picked_to_stored(
                        generate_items(4, double_or_nothing),
                        &mut dealer_stored_items,
                    );
                    let mut live: u8;
                    let mut blanks: u8;
                    loop {
                        live = rand::thread_rng().gen_range(1..=4);
                        blanks = rand::thread_rng().gen_range(1..=4);
                        if (live + blanks) > 2 {
                            break;
                        }
                    }
                    let mut live_plural: &str = "live";
                    if live > 1 {
                        live_plural = "lives";
                    }
                    let mut blank_plural: &str = "blank";
                    if blanks > 1 {
                        blank_plural = "blanks";
                    }
                    println!("----------------\nThere are {live} {live_plural} and {blanks} {blank_plural}.\n----------------\n");
                    let shell_vec: Vec<bool> = load_shells(live, blanks);
                    dbg!(&shell_vec);
                    //turn owner is used to switch between turns for player/dealer.
                    //true means it is the players turn, false the dealer's turn.
                    let mut turn_owner: TargetEnum = TargetEnum::Player;
                    let mut turn: usize = 1;
                    let mut player_extraturn: bool;
                    let mut dealer_extraturn: bool;

                    let mut game_info: GameInfo = GameInfo {
                        dealer_health,
                        player_health,
                        turn_owner,
                        player_inventory,
                        dealer_stored_items,
                        perfect,
                        double_or_nothing,
                        shells_vector: (*shell_vec).to_vec(),
                    };

                    'shell_iter: for _ in &shell_vec {
                        //current bullets vec holds the bullets currently loaded
                        let current_bullets_vec: Vec<bool> = shell_vec[turn - 1..].to_vec();
                        println!("{}", Style::new().bold().paint(format!("Turn {turn}\n")));
                        println!(
                            "You have {0} lives remaining. The dealer has {1} lives remaining.",
                            game_info.player_health, game_info.dealer_health
                        );
                        check_life(game_info.player_health, game_info.dealer_health);
                        match turn_owner {
                            TargetEnum::Player => {
                                player_extraturn = player::turn(&mut game_info);
                                if !player_extraturn {
                                    
                                    break 'shell_iter;
                                };
                            }
                            TargetEnum::Dealer => {
                                dealer_extraturn =
                                    dealer::turn(current_bullets_vec, &mut game_info);
                                if !dealer_extraturn {
                                    break 'shell_iter;
                                };
                            }
                        }
                        turn += 1;

                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
            Selection::Credits => credits(),
            Selection::Help => help(),
        }
    }
}

fn generate_items(len: usize, double_or_nothing: bool) -> Vec<ItemEnum> {
    let mut items_vec: Vec<ItemEnum> = Vec::new();
    let saws: u8 = rand::thread_rng().gen_range(2..=6);
    let beers: u8 = rand::thread_rng().gen_range(2..7);
    let cigs: u8 = rand::thread_rng().gen_range(2..8);
    let mag_glass: u8 = rand::thread_rng().gen_range(2..7);
    let handcuffs: u8 = rand::thread_rng().gen_range(2..5);
    if double_or_nothing {
        let adren: u8 = rand::thread_rng().gen_range(2..6);
        let burn_pho: u8 = rand::thread_rng().gen_range(2..7);
        let invert: u8 = rand::thread_rng().gen_range(2..8);
        let exp_med: u8 = rand::thread_rng().gen_range(2..8);
        if double_or_nothing {
            for _ in 0..adren {
                items_vec.push(ItemEnum::Adren);
            }
            for _ in 0..burn_pho {
                items_vec.push(ItemEnum::BurnPho);
            }
            for _ in 0..invert {
                items_vec.push(ItemEnum::Invert);
            }
            for _ in 0..exp_med {
                items_vec.push(ItemEnum::ExpMed);
            }
        }
    }

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
    for _ in 0..10 {
        let mut rng = rand::thread_rng();
        items_vec.as_mut_slice().shuffle(&mut rng);
    }

    //yes ik its overkill but this is my code not urs
    let trimmed_vec: Vec<ItemEnum> = items_vec.iter().take(len).copied().collect::<Vec<_>>();

    trimmed_vec
}

fn remove_no_item(picked_items_vec: &mut [ItemEnum; 8], item_type: ItemEnum) {
    if let Some(index) = picked_items_vec.iter().position(|&x| x == item_type) {
        picked_items_vec[index] = ItemEnum::Nothing;
    } else {
        dbg!(picked_items_vec);
        println!("{item_type}");
        panic!("Item {item_type:?} not found in the array. ");
    }
}

//loading the shotgun shells
fn load_shells(live: u8, blanks: u8) -> Vec<bool> {
    let mut shells: Vec<bool> = Vec::new();
    for _i in 0..blanks {
        shells.push(false);
        play_audio("load_single_shell.ogg");
        thread::sleep(Duration::from_millis(500));
    }
    for _i in 0..live {
        shells.push(true);
        play_audio("load_single_shell.ogg");
        thread::sleep(Duration::from_millis(500));
    }
    let mut rng = rand::thread_rng();
    shells.as_mut_slice().shuffle(&mut rng);
    shells
}

//check the lives
fn check_life(player_health: i8, dealer_health: i8) {
    if player_health < 1 {
        println!("You have no lives left. Game over.");
        cleanup();
    }
    if dealer_health < 1 {
        println!("Dealer has no lives left. You win!");
        cleanup();
    }
}

fn play_audio(path: &'static str) {
    let path: String = format!("audio/{path}");

    let _handle: thread::JoinHandle<()> = thread::spawn(move || {
        let file: BufReader<File> = BufReader::new(match File::open(path) {
            Ok(t) => t,
            Err(e) => panic!("{e}"),
        });

        let source: Decoder<BufReader<File>> = Decoder::new(file).unwrap();
        AUDIO_HANDLE.play_raw(source.convert_samples()).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(5));
    });
}

fn turn_screen_red() {
    // Execute crossterm commands to clear screen and set red background
    let mut chunk = String::new();
    play_audio("temp_gunshot_live.wav");
    for _ in 0..18000 {
        chunk.push(' ');
    }

    execute!(
        io::stdout(),
        //Clear(ClearType::All),          // Clear the screen
        SetBackgroundColor(Color::Red), // Set background color to red
        Print(chunk),                   // Print a dummy character to fill the screen with red
        ResetColor                      // Reset colors to default after printing
    )
    .expect("Failed to execute crossterm commands");
    thread::sleep(Duration::from_millis(500));

    // Flush stdout to ensure color change is immediate
    io::stdout().flush().expect("Failed to flush stdout");
    //clearscreen::clear().expect("Failed to clear screen");
}

fn credits() {
    clearscreen::clear().expect("Failed to clear screen");
    let contents = fs::read_to_string("credits.txt")
        .expect("The help.txt file is missing or in the wrong area!");
    println!("{contents}");
    println!("Press enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    println!("Continuing...");
}
fn help() {
    clearscreen::clear().expect("Failed to clear screen");
    let contents =
        fs::read_to_string("help.txt").expect("The help.txt file is missing or in the wrong area!");
    println!("{contents}");
    println!("Press enter to continue...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    println!("Continuing...");
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

fn cleanup() {
    print!("\x1B[?25h");
    io::stdout().flush().unwrap();
    process::exit(0);
}

fn italics(text: &str) {
    let styled_text = Style::new().italic().paint(text);
    println!("{styled_text}");
}
