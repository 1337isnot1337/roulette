use std::{io, sync::Mutex};

use crate::{cleanup, GameInfo, PlayerDealer, STDIN};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use once_cell::sync::Lazy;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    terminal::{Frame, Terminal},
    widgets::{Block, List, ListItem, ListState},
};
// Static lazy-initialized terminal instance
pub static TERMINAL: Lazy<Mutex<Terminal<CrosstermBackend<io::Stdout>>>> = Lazy::new(|| {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(backend).unwrap();
    terminal.into()
});

// Static lazy-initialized string to store top messages
static TOP_MESSAGES_STRING: Lazy<Mutex<String>> = Lazy::new(|| {
    let top_messages_vec: String = String::new();
    top_messages_vec.into()
});

// Static lazy-initialized vector to store stat messages
static STAT_MESSAGES_VEC: Lazy<Mutex<Vec<String>>> = Lazy::new(|| {
    let stat_messages_vec: Vec<String> = Vec::new();
    stat_messages_vec.into()
});

// Static lazy-initialized vector to store player inventory
static PLAYER_INV: Lazy<Mutex<Vec<String>>> = Lazy::new(|| {
    let stat_messages_vec: Vec<String> = Vec::new();
    stat_messages_vec.into()
});

// Static lazy-initialized vector to store dealer inventory
static DEALER_INV: Lazy<Mutex<Vec<String>>> = Lazy::new(|| {
    let stat_messages_vec: Vec<String> = Vec::new();
    stat_messages_vec.into()
});

static LAYOUT: Lazy<Mutex<Layout>> = Lazy::new(|| {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(20),
                Constraint::Fill(1),
            ]
            .as_ref(),
        )
        .into()
});

#[macro_export]
macro_rules! message_top {
    () => {
        message_top_func(format!(""));
    };
    ($($arg:tt)*) => {{
        message_top_func(&format!($($arg)*));
    }};
}
macro_rules! list {
    ($list_var:expr, $title_var:expr) => {{
        let list = List::new($list_var.to_owned())
            .block(Block::bordered().title($title_var))
            .style(Style::new().white().on_black());
        list
    }};
}

// Function to render the UI
fn ui(
    f: &mut Frame,
    top_message: &str,
    stat_message: &mut [String],
    player_message: &mut [String],
    dealer_message: &mut [String],
) {
    // Define layout chunks
    let chunks: std::rc::Rc<[Rect]> = LAYOUT.try_lock().unwrap().split(f.size());
    // Define sub-rectangles for different sections
    let rect_stat = Rect::new(
        chunks[2].x,
        chunks[2].y,
        chunks[2].width / 2,
        chunks[2].height,
    );
    let rect_dealer_inv = Rect::new(
        chunks[2].width / 2,
        chunks[2].y,
        chunks[2].width / 4,
        chunks[2].height,
    );
    let rect_player_inv = Rect::new(
        chunks[2].width * 3 / 4,
        chunks[2].y,
        chunks[2].width / 4,
        chunks[2].height,
    );

    // Limit the number of lines displayed based on terminal height
    let height: usize = chunks[0].height as usize - 1;
    let string_array: Vec<&str> = top_message.split('\n').collect();
    let string_array = &string_array[string_array.len().saturating_sub(height)..string_array.len()];

    // Create widgets for different sections
    let top_messages = list!(string_array, "Top Messages");
    let bot_messages = list!(stat_message, "Top Messages");
    let player_inv = list!(player_message, "Player Inventory");
    let dealer_inv = list!(dealer_message, "Dealer Inventory");

    // Render widgets
    f.render_widget(top_messages, chunks[0]);
    f.render_widget(bot_messages, rect_stat);
    f.render_widget(player_inv, rect_dealer_inv);
    f.render_widget(dealer_inv, rect_player_inv);
}

// Function to handle top messages
pub fn message_top_func(given_message: &str) {
    TOP_MESSAGES_STRING
        .try_lock()
        .unwrap()
        .push_str(&format!("{given_message}\n"));

    // Clear the terminal and redraw the UI
    TERMINAL.try_lock().unwrap().clear().unwrap();
    TERMINAL
        .try_lock()
        .unwrap()
        .draw(|f: &mut Frame| {
            ui(
                f,
                &TOP_MESSAGES_STRING.try_lock().unwrap(),
                &mut STAT_MESSAGES_VEC.try_lock().unwrap(),
                &mut PLAYER_INV.try_lock().unwrap(),
                &mut DEALER_INV.try_lock().unwrap(),
            );
        })
        .unwrap();
}

// Function to handle stat messages and inventory updates
pub fn message_stats_func(game_info: &mut GameInfo) {
    STAT_MESSAGES_VEC.try_lock().unwrap().clear();
    DEALER_INV.try_lock().unwrap().clear();
    PLAYER_INV.try_lock().unwrap().clear();

    // Determine the turn owner
    let turn_owner = match game_info.turn_owner {
        PlayerDealer::Player => "Player",
        PlayerDealer::Dealer => "Dealer",
    };

    // Prepare additional information strings
    let double_or_nothing: &str = if game_info.double_or_nothing {
        "\nDouble or Nothing is enabled!"
    } else {
        ""
    };
    let perfect: &str = if game_info.perfect {
        "\nPerfect Dealer is enabled!"
    } else {
        ""
    };
    let debug_info: &str = if game_info.debug {
        &format!(
            "\n!!!DEBUG MODE ENABLED!!!\nWill print extra information\nShells vec: {:?}\nDealer Shell Know {:?}",
            game_info.shells_vector,
            game_info.dealer_shell_knowledge_vec,
        )
    } else {
        ""
    };

    // Update stat messages
    STAT_MESSAGES_VEC.try_lock().unwrap().push(format!(
        "Turn {}. {turn_owner}'s turn. \n\nDealer Health: {} \nPlayer Health: {} \nShell Index: {} 
        {double_or_nothing}{perfect}{debug_info}",
        game_info.current_turn,
        game_info.dealer_health,
        game_info.player_health,
        game_info.shell_index,
    ));

    // Update dealer and player inventories
    let mut dealer_inventory = String::new();
    let mut player_inventory = String::new();
    for item in game_info.dealer_inventory {
        dealer_inventory.push_str(&format!("\n{item}"));
    }
    for item in game_info.player_inventory {
        player_inventory.push_str(&format!("\n{item}"));
    }

    DEALER_INV
        .try_lock()
        .unwrap()
        .push(dealer_inventory.to_string());
    PLAYER_INV
        .try_lock()
        .unwrap()
        .push(player_inventory.to_string());

    // Redraw the UI
    TERMINAL
        .try_lock()
        .unwrap()
        .draw(|f: &mut Frame| {
            ui(
                f,
                &TOP_MESSAGES_STRING.try_lock().unwrap(),
                &mut STAT_MESSAGES_VEC.try_lock().unwrap(),
                &mut PLAYER_INV.try_lock().unwrap(),
                &mut DEALER_INV.try_lock().unwrap(),
            );
        })
        .unwrap();
}

// Function to handle key events for item selection prompts
pub fn key_event(selected_index: &mut usize, length: usize) -> bool {
    let mut result = false;
    if let Event::Key(key) = STDIN.get().unwrap().lock().unwrap().recv().unwrap() {
        // Handle CTRL+C to cleanup
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && (key.code == crossterm::event::KeyCode::Char('c'))
        {
            cleanup();
        }
        // Handle navigation keys
        match key.code {
            KeyCode::Down => *selected_index = (*selected_index + 1) % length,
            KeyCode::Up => {
                if *selected_index > 0 {
                    *selected_index -= 1;
                } else {
                    *selected_index = length - 1;
                }
            }
            KeyCode::Enter => result = true,
            _ => {}
        }
    }
    result
}

// Function to handle dialogue selection
pub fn dialogue<T: std::string::ToString>(options: &[T], title: &str) -> usize {
    let mut selected_index = 0;
    let list = list!(options.iter().map(|i| ListItem::new(i.to_string())), title)
        .highlight_style(
            Style::new()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::REVERSED),
        )
        .highlight_symbol(">> ");

    let mut liststate: ListState = ListState::default();

    loop {
        TERMINAL
            .try_lock()
            .unwrap()
            .draw(|f: &mut Frame| {
                liststate.select(Some(selected_index));

                let chunks = LAYOUT.try_lock().unwrap().split(f.size());

                f.render_stateful_widget(&list, chunks[1], &mut liststate);
                ui(
                    f,
                    &TOP_MESSAGES_STRING.try_lock().unwrap(),
                    &mut STAT_MESSAGES_VEC.try_lock().unwrap(),
                    &mut PLAYER_INV.try_lock().unwrap(),
                    &mut DEALER_INV.try_lock().unwrap(),
                );
            })
            .unwrap();
        if key_event(&mut selected_index, options.len()) {
            break;
        }
    }
    selected_index
}
