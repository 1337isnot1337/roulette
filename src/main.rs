use crate::local_ratatui::message_top_func;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dealer::picked_to_stored;
use local_ratatui::{dialogue, message_stats_func, TERMINAL};
use player::pick_items;
use rand::{seq::SliceRandom, Rng};
use rodio::{Decoder, Source};
use std::{
    env,
    fs::File,
    io::{self, BufReader, Write},
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::channel,
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
mod dealer;
mod local_ratatui;
mod player;
use once_cell::sync::Lazy;
use rodio::{OutputStream, OutputStreamHandle};
use std::{
    fmt, mem,
    sync::{mpsc::Receiver, OnceLock},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Selection {
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

static GAME_BEGUN: Lazy<Mutex<bool>> = Lazy::new(|| {
    let begun = false;
    begun.into()
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemEnum {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameInfo {
    pub dealer_health: i8,
    pub player_health: i8,
    pub turn_owner: PlayerDealer,
    pub player_inventory: [ItemEnum; 8],
    pub dealer_inventory: [ItemEnum; 8],
    pub perfect: bool,
    pub double_or_nothing: bool,
    pub debug: bool,
    pub shells_vector: Vec<bool>,
    pub current_turn: i32,
    pub shell_index: usize,
    pub dealer_shell_knowledge_vec: Vec<Option<bool>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerDealer {
    Player,
    Dealer,
}
impl fmt::Display for PlayerDealer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let printable = match *self {
            PlayerDealer::Player => "self",
            PlayerDealer::Dealer => "dealer",
        };
        write!(f, "{printable}")
    }
}
pub static ON_OR_OFF_AUDIO: Lazy<Mutex<bool>> = Lazy::new(|| {
    let thisbool = false;
    thisbool.into()
});
pub static PREVIOUS_INDEX: Lazy<Mutex<usize>> = Lazy::new(|| {
    let prev_index: usize = 0;
    prev_index.into()
});
pub static STDIN: OnceLock<Mutex<Receiver<Event>>> = OnceLock::new();
pub static AUDIO_HANDLE: Lazy<OutputStreamHandle> = Lazy::new(|| {
    let (stream, stream_handle) = OutputStream::try_default().unwrap();
    mem::forget(stream);
    stream_handle
});

fn main() {
    enable_raw_mode().unwrap();
    let mut terminal = TERMINAL.try_lock().unwrap();
    execute!(
        terminal.backend_mut(),
        EnterAlternateScreen,
        EnableMouseCapture
    )
    .unwrap();
    drop(terminal);
    // atomic boolean to track if sigint was received
    let running = Arc::new(AtomicBool::new(true));
    let _r = running.clone();

    let (input_sender, input) = channel::<Event>();
    STDIN.set(Mutex::new(input)).unwrap();
    let _handle = thread::spawn(move || loop {
        let event = event::read().unwrap();
        if let Event::Key(KeyEvent {
            code: KeyCode::Char('c'),
            modifiers,
            ..
        }) = event
        {
            if modifiers.contains(KeyModifiers::CONTROL) {
                cleanup();
            }
        };
        input_sender.send(event).unwrap();
    });

    while running.load(Ordering::SeqCst) {
        gameplay();
    }
}

fn gameplay() {
    let args: Vec<String> = env::args().collect::<Vec<String>>()[1..].to_vec();

    let dealer_health: i8 = 3;
    let player_health: i8 = 3;
    let turn_owner: PlayerDealer = PlayerDealer::Player;
    let player_inventory: [ItemEnum; 8] = [ItemEnum::Nothing; 8];
    let dealer_inventory: [ItemEnum; 8] = [ItemEnum::Nothing; 8];
    let mut perfect: bool = false;
    let mut double_or_nothing: bool = false;
    let mut debug: bool = false;

    let mut invalid_args = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--perfect" => perfect = true,
            "--double" => double_or_nothing = true,
            "--debug" => debug = true,
            "--no-audio" => *ON_OR_OFF_AUDIO.try_lock().unwrap() = true,
            _ => invalid_args.push(arg),
        }
    }
    play_audio("music/music_main_techno_techno.ogg");
    if !invalid_args.is_empty() {
        let mut error_string: String = "The following args were not recognized: ".to_owned();
        for arg in invalid_args {
            error_string.push_str(&format!("{}, ", &arg));
        }
        panic!("{error_string}");
    }

    let shells_vector: Vec<bool> = vec![];
    let current_turn: i32 = 1;
    let shell_index = 0;
    let dealer_shell_knowledge_vec: Vec<Option<bool>> = Vec::new();

    let mut game_info: GameInfo = GameInfo {
        dealer_health,
        player_health,
        turn_owner,
        player_inventory,
        dealer_inventory,
        perfect,
        double_or_nothing,
        debug,
        shells_vector,
        current_turn,
        shell_index,
        dealer_shell_knowledge_vec,
    };
    message_stats_func(&mut game_info);

    enable_raw_mode().unwrap();

    match play_screen() {
        Selection::Play => {
            *GAME_BEGUN.try_lock().unwrap() = true;
            loop {
                play(&mut game_info);
            }
        }
        Selection::Credits => credits(),
        Selection::Help => help(),
    }
}

fn play(game_info: &mut GameInfo) {
    (game_info.dealer_inventory, game_info.player_inventory) =
        ([ItemEnum::Nothing; 8], [ItemEnum::Nothing; 8]);

    // this block ensures some of the variables used once are dropped fast
    {
        pick_items(game_info);

        game_info.dealer_inventory = picked_to_stored(generate_items(4, game_info), game_info);
        let mut lives: i8;
        let mut blanks: i8;
        loop {
            lives = rand::thread_rng().gen_range(1..=4);
            blanks = rand::thread_rng().gen_range(1..=4);
            if (lives + blanks) > 2 && ((lives - blanks).abs() < 3) {
                break;
            }
        }

        let mut live_plural: &str = "live";
        if lives > 1 {
            live_plural = "lives";
        }
        let mut blank_plural: &str = "shell";
        if blanks > 1 {
            blank_plural = "shells";
        }
        message_top!("----------------\n {lives} {live_plural} and {blanks} blank {blank_plural} are loaded into the shotgun.\n----------------\n");
        game_info.shells_vector = load_shells(lives, blanks);
        game_info.shell_index = 0;
        game_info.dealer_shell_knowledge_vec.clear();
        for _ in 0..(lives + blanks) {
            game_info.dealer_shell_knowledge_vec.push(None);
        }
    }

    game_info.turn_owner = PlayerDealer::Player;
    game_info.current_turn = 1;
    let mut player_extraturn: bool;
    let mut dealer_extraturn: bool;
    let mut empty_due_to_beer: bool;
    while game_info.shells_vector.len() != game_info.shell_index {
        check_life(game_info);

        match game_info.turn_owner {
            PlayerDealer::Player => {
                (player_extraturn, empty_due_to_beer) = player::turn(game_info);
                if empty_due_to_beer {
                    game_info.shell_index = 0;
                    message_top!(
                        "All shells have been used, loading new shells and generating new items."
                    );
                    play(game_info);
                }
                if !player_extraturn {
                    game_info.turn_owner = PlayerDealer::Dealer;
                };
            }

            PlayerDealer::Dealer => {
                dealer_extraturn = dealer::turn(game_info);

                if !dealer_extraturn {
                    game_info.turn_owner = PlayerDealer::Player;
                };
            }
        }
        game_info.current_turn += 1;

        thread::sleep(Duration::from_secs(1));
    }

    message_top!("All shells have been used, loading new shells and generating new items.");

    message_stats_func(game_info);
}

fn generate_items(len: usize, game_info: &mut GameInfo) -> Vec<ItemEnum> {
    let mut items_vec: Vec<ItemEnum> = Vec::new();
    let saws: u8 = rand::thread_rng().gen_range(4..6);
    let beers: u8 = rand::thread_rng().gen_range(4..7);
    let cigs: u8 = rand::thread_rng().gen_range(4..8);
    let mag_glass: u8 = rand::thread_rng().gen_range(4..7);
    let handcuffs: u8 = rand::thread_rng().gen_range(4..5);
    if game_info.double_or_nothing {
        let adren: u8 = rand::thread_rng().gen_range(4..6);
        let burn_pho: u8 = rand::thread_rng().gen_range(4..7);
        let invert: u8 = rand::thread_rng().gen_range(4..7);
        let exp_med: u8 = rand::thread_rng().gen_range(4..8);
        if game_info.double_or_nothing {
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
    for _ in 0..1000 {
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
    for _ in 0..40 {
        let mut rng = rand::thread_rng();
        items_vec.as_mut_slice().shuffle(&mut rng);
    }

    //yes ik its overkill and stupid asf but this is my code not urs
    items_vec.iter().take(len).copied().collect::<Vec<_>>()
}

fn remove_item(picked_items_vec: &mut [ItemEnum; 8], index: usize) {
    picked_items_vec[index] = ItemEnum::Nothing;
}

//loading the shotgun shells
fn load_shells(lives: i8, blanks: i8) -> Vec<bool> {
    let mut shells: Vec<bool> = Vec::new();
    for _i in 0..blanks {
        shells.push(false);
        play_audio("load_single_shell.ogg");
        thread::sleep(Duration::from_millis(300));
    }
    for _i in 0..lives {
        shells.push(true);
        play_audio("load_single_shell.ogg");
        thread::sleep(Duration::from_millis(300));
    }
    //yes ik its overkill and stupid asf but this is my code not urs (part 2)
    for _ in 0..10 {
        let mut rng = rand::thread_rng();
        shells.as_mut_slice().shuffle(&mut rng);
    }

    shells
}

//check the lives
fn check_life(game_info: &mut GameInfo) {
    if game_info.player_health < 1 || game_info.dealer_health < 1 {
        if game_info.player_health < 1 {
            message_top!("\n\nYou have no lives left. Game over. \n\nPlay Again? \n");
        }
        if game_info.dealer_health < 1 {
            message_top!(
                "\n\nDealer has no lives left. You win!\n\nStart a new game, if you wish. \n\n"
            );
            play_audio("winner.ogg");
        }
        message_top!("\n\nPlay Again?");
        if dialogue(&["Continue", "Quit Game"], "Continue?", None, false) == 0 {
            game_info.player_health = 3;
            game_info.dealer_health = 3;
            game_info.current_turn = 1;
            game_info.shells_vector.clear();
            gameplay();
        } else {
            cleanup();
        }
    }
}

fn play_audio(path: &'static str) {
    let audio_avail = ON_OR_OFF_AUDIO.try_lock().unwrap();
    if !*audio_avail {
        let path: String = format!("audio/{path}");

        let _handle: thread::JoinHandle<()> = thread::spawn(move || {
            let file: BufReader<File> = BufReader::new(match File::open(&path) {
                Ok(t) => t,
                Err(e) => {
                    panic!(
                        "There was an error in audio playing, {e}. The relevent file is at {path}"
                    )
                }
            });

            let source: Decoder<BufReader<File>> = Decoder::new(file).unwrap();
            AUDIO_HANDLE.play_raw(source.convert_samples()).unwrap();
        });
    }
}

fn turn_screen_red() {
    // Execute crossterm commands to clear screen and set red background
    let mut chunk = String::new();
    play_audio("temp_gunshot_live.wav");
    for _ in 0..18000 {
        chunk.push(' ');
    }

    thread::sleep(Duration::from_millis(400));

    // Flush stdout to ensure color change is immediate
    io::stdout().flush().expect("Failed to flush stdout");
    //clearscreen::clear().expect("Failed to clear screen");
}

fn credits() {
    let contents = include_str!("../txt_files/credits.txt");
    message_top!("{contents}");
    message_top!("\n\nSelect continue to continue...");
    dialogue(&[&"Continue"], "Pick a choice:", None, false);
    message_top!("Continuing...");
}
fn help() {
    let contents = include_str!("../txt_files/help.txt");
    message_top!("{contents}");
    message_top!("\n\nSelect continue to continue...");
    dialogue(&[&"Continue"], "Pick a choice:", None, false);
    message_top!("Continuing...");
}

fn play_screen() -> Selection {
    let options_vec: [Selection; 3] = [Selection::Play, Selection::Help, Selection::Credits];
    message_top!("Welcome to the game. \nWhat do you wish to do?");
    let selection = dialogue(&options_vec, "Pick a choice:", None, false);
    options_vec[selection]
}

fn cleanup() {
    disable_raw_mode().unwrap();
    let mut terminal = TERMINAL.try_lock().unwrap();
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )
    .unwrap();
    terminal.show_cursor().unwrap();
    print!("\x1B[?25h");
    io::stdout().flush().unwrap();
    clearscreen::clear().expect("Failed to clear screen");
    process::exit(0);
}
