use std::{
    io::{self, stdout},
    sync::Mutex,
};

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use once_cell::sync::Lazy;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style, Stylize},
    terminal::{Frame, Terminal},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};

use crate::{cleanup, GameInfo};

pub static TERMINAL: Lazy<Mutex<Terminal<CrosstermBackend<io::Stdout>>>> = Lazy::new(|| {
    let stdout = io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let terminal: Terminal<CrosstermBackend<io::Stdout>> = Terminal::new(backend).unwrap();

    terminal.into()
});
static TOP_MESSAGES_STRING: Lazy<Mutex<String>> = Lazy::new(|| {
    let top_messages_vec: String = String::new();
    top_messages_vec.into()
});
static STAT_MESSAGES_VEC: Lazy<Mutex<Vec<String>>> = Lazy::new(|| {
    let stat_messages_vec: Vec<String> = Vec::new();
    stat_messages_vec.into()
});
static PLAYER_INV: Lazy<Mutex<Vec<String>>> = Lazy::new(|| {
    let stat_messages_vec: Vec<String> = Vec::new();
    stat_messages_vec.into()
});
static DEALER_INV: Lazy<Mutex<Vec<String>>> = Lazy::new(|| {
    let stat_messages_vec: Vec<String> = Vec::new();
    stat_messages_vec.into()
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

fn ui(
    f: &mut Frame,
    top_message: &str,
    stat_message: &mut [String],
    player_message: &mut [String],
    dealer_message: &mut [String],
) {
    let chunks: std::rc::Rc<[ratatui::prelude::Rect]> = Layout::vertical(
        [
            Constraint::Percentage(50),
            Constraint::Percentage(20),
            Constraint::Percentage(30),
        ]
        .as_ref(),
    )
    .direction(Direction::Vertical)
    .split(f.size());
    let rect_stat = Rect::new(
        chunks[2].x,
        chunks[2].y,
        chunks[2].width / 3,
        chunks[2].height,
    );
    let rect_dealer_inv = Rect::new(
        chunks[2].width / 3,
        chunks[2].y,
        chunks[2].width / 3,
        chunks[2].height,
    );
    let rect_player_inv = Rect::new(
        chunks[2].width * 2 / 3,
        chunks[2].y,
        chunks[2].width / 3,
        chunks[2].height,
    );

    //we need to display only {height} lines, because that is how much the terminal can display.
    let height: usize = chunks[0].height as usize - 1;

    //this gathers all the text into a vec of strings
    let string_array: Vec<&str> = top_message.split('\n').collect();

    // we need vec[vec.len()-height..vec.len()]

    let string_array = &string_array[string_array.len().saturating_sub(height)..string_array.len()];

    let mut final_text = String::new();
    for item in string_array {
        final_text.push_str(item);
        final_text.push('\n');
    }

    let top_messages = Paragraph::new(final_text)
        .block(Block::bordered())
        .style(Style::new().white().on_black());

    let bot_messages = List::new(stat_message.to_owned())
        .block(Block::bordered().title("Game Information:"))
        .style(Style::new().white().on_black());

    let player_inv = List::new(player_message.to_owned())
        .block(Block::bordered().title("Player Inventory:"))
        .style(Style::new().white().on_black());
    let dealer_inv = List::new(dealer_message.to_owned())
        .block(Block::bordered().title("Dealer Inventory:"))
        .style(Style::new().white().on_black());

    f.render_widget(top_messages, chunks[0]);

    f.render_widget(bot_messages, rect_stat);
    f.render_widget(player_inv, rect_dealer_inv);
    f.render_widget(dealer_inv, rect_player_inv);
}

pub fn message_top_func(given_message: &str) {
    TOP_MESSAGES_STRING
        .try_lock()
        .unwrap()
        .push_str(given_message);
    TOP_MESSAGES_STRING.try_lock().unwrap().push('\n');
    TERMINAL.try_lock().unwrap().autoresize().unwrap();
    //this is the problematic part

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

pub fn message_stats_func(game_info: &mut GameInfo) {
    STAT_MESSAGES_VEC.try_lock().unwrap().clear();
    DEALER_INV.try_lock().unwrap().clear();
    PLAYER_INV.try_lock().unwrap().clear();

    let turn_owner = match game_info.turn_owner {
        crate::TargetEnum::Player => "Player",
        crate::TargetEnum::Dealer => "Dealer",
    };

    let double_or_nothing = if game_info.double_or_nothing {
        "\nDouble or Nothing is enabled!"
    } else {
        ""
    };
    let perfect = if game_info.perfect {
        "\nPerfect Dealer is enabled!"
    } else {
        ""
    };
    let debug_info = if game_info.debug {
        &format!("\n!!!DEBUG MODE ENABLED!!!\nWill print extra information\nShells vec: {:?}", game_info.shells_vector)
    } else {
        ""
    };
    STAT_MESSAGES_VEC.try_lock().unwrap().push(format!(
        "Turn {}. {}'s turn. \n\nDealer Health: {} \nPlayer Health: {}{}{}{}",
        game_info.current_turn,
        turn_owner,
        game_info.dealer_health,
        game_info.player_health,
        double_or_nothing,
        perfect,
        debug_info,
    ));
    let mut dealer_inventory = String::new();
    let mut player_inventory = String::new();
    for item in game_info.dealer_stored_items {
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

// this section handles the item selection prompts

pub fn key_event(selected_index: &mut usize, length: usize) -> bool {
    let mut result = false;
    if let Event::Key(key) = event::read().unwrap() {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && (key.code == crossterm::event::KeyCode::Char('c'))
        {
            cleanup();
        }
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
            KeyCode::Esc => panic!("panic"),

            _ => {}
        }
    }
    result
}
pub fn dialogue<T: std::string::ToString>(options: &[T], title: &str) -> usize {
    let stdout = stdout();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut selected_index = 0;

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(50),
                            Constraint::Percentage(20),
                            Constraint::Fill(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let items: Vec<ListItem> = options
                    .iter()
                    .map(|i| ListItem::new(i.to_string()))
                    .collect();

                let list = List::new(items)
                    .block(Block::bordered().title(title))
                    .style(Style::new().white().on_black())
                    .highlight_style(Style::new().add_modifier(Modifier::REVERSED))
                    .highlight_symbol(">> ")
                    .repeat_highlight_symbol(true);

                let mut test: ListState = ListState::default();
                test.select(Some(selected_index));
                f.render_stateful_widget(list, chunks[1], &mut test);
            })
            .unwrap();
        if key_event(&mut selected_index, options.len()) {
            break;
        }
    }

    selected_index
}
