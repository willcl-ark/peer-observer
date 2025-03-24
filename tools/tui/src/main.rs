#![cfg_attr(feature = "strict", deny(warnings))]

mod app;
mod models;
mod ui;

use std::{
    io::stdout,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::*};
use shared::{
    clap::{self, Parser},
    event_msg,
    event_msg::event_msg::Event as ObserverEvent,
    nats,
    prost::Message,
};

use crate::{
    app::App,
    models::{AppMode, Direction, EventType, LogMessage},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The NATS server address the tool should connect and subscribe to.
    #[arg(short, long, default_value = "127.0.0.1:4222")]
    nats_address: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let app = Arc::new(Mutex::new(App::new()));

    // Start NATS client in a background thread
    let app_clone = app.clone();
    let _nats_thread = thread::spawn(move || {
        let nc = nats::connect(args.nats_address).expect("Failed to connect to NATS");
        let sub = nc
            .subscribe("*")
            .expect("Failed to subscribe to NATS topics");

        for msg in sub.messages() {
            let event_msg = event_msg::EventMsg::decode(msg.data.as_slice())
                .expect("Failed to decode event message");

            if let Some(event) = event_msg.event {
                // Process event into appropriate LogMessage type
                let log_message = match event {
                    ObserverEvent::Msg(msg) => {
                        // Create a P2P message using data directly from the message metadata
                        let direction = if msg.meta.inbound {
                            Direction::INBOUND
                        } else {
                            Direction::OUTBOUND
                        };

                        // Extract message content safely using Display formatting, not Debug
                        let content = match &msg.msg {
                            Some(m) => format!("{}", m),
                            None => "unknown message".to_string(),
                        };

                        // Get the command/message type directly from metadata
                        let msg_type = Some(msg.meta.command.clone());

                        // Get the message size directly from metadata
                        let msg_size = Some(msg.meta.size);

                        // Format connection type
                        // Note: We're converting the enum value to a string for display purposes
                        let conn_type = format!("{:?}", msg.meta.conn_type);

                        LogMessage::new_p2p(
                            content,
                            direction,
                            msg.meta.peer_id,
                            conn_type,
                            msg_type,
                            msg_size,
                        )
                    }
                    ObserverEvent::Conn(c) => {
                        let content = match &c.event {
                            Some(e) => format!("{}", e),
                            None => "unknown connection event".to_string(),
                        };
                        LogMessage::new_event(content, EventType::CONNECTION)
                    }
                    ObserverEvent::Addrman(a) => {
                        let content = match &a.event {
                            Some(e) => format!("{}", e),
                            None => "unknown addrman event".to_string(),
                        };
                        LogMessage::new_event(content, EventType::ADDRMAN)
                    }
                    ObserverEvent::Mempool(m) => {
                        let content = match &m.event {
                            Some(e) => format!("{}", e),
                            None => "unknown mempool event".to_string(),
                        };
                        LogMessage::new_event(content, EventType::MEMPOOL)
                    }
                    ObserverEvent::Validation(v) => {
                        let content = match &v.event {
                            Some(e) => format!("{}", e),
                            None => "unknown validation event".to_string(),
                        };
                        LogMessage::new_event(content, EventType::VALIDATION)
                    }
                };

                // Add to the app messages ring buffer
                let mut app = app_clone.lock().unwrap();
                app.increment_total_messages();

                // Record statistics for this message
                app.record_statistics(&log_message);

                app.push_message(log_message);
                if app.autoscroll() {
                    app.scroll_to_bottom();
                }
            }
        }
    });

    // Start UI event loop
    let mut should_quit = false;
    while !should_quit {
        terminal.draw(|f| ui::ui(f, &app))?;

        // keyboard input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    let mut app = app.lock().unwrap();

                    match app.mode() {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') => should_quit = true,
                            KeyCode::Char(':') => {
                                app.set_mode(AppMode::Command);
                                app.clear_command_input();
                            }
                            KeyCode::Char('/') => {
                                app.set_mode(AppMode::Search);
                                app.clear_command_input();
                            }
                            KeyCode::Enter => app.open_detail_view(),
                            KeyCode::Char('p') => app.toggle_event_type_filter(EventType::MESSAGE),
                            KeyCode::Char('c') => {
                                app.toggle_event_type_filter(EventType::CONNECTION)
                            }
                            KeyCode::Char('a') => app.toggle_event_type_filter(EventType::ADDRMAN),
                            KeyCode::Char('m') => app.toggle_event_type_filter(EventType::MEMPOOL),
                            KeyCode::Char('v') => {
                                app.toggle_event_type_filter(EventType::VALIDATION)
                            }
                            KeyCode::Char('i') => app.toggle_direction_filter(Direction::INBOUND),
                            KeyCode::Char('o') => app.toggle_direction_filter(Direction::OUTBOUND),
                            KeyCode::Char('b') => {
                                if app.direction_filter() == Direction::BOTH {
                                    app.set_direction_filter(Direction::NONE);
                                } else {
                                    app.set_direction_filter(Direction::BOTH);
                                }
                            }
                            KeyCode::Char('0') => app.clear_all_filters(),
                            KeyCode::Char('r') => app.toggle_autoscroll(),
                            KeyCode::Char('s') => app.toggle_statistics_mode(),
                            KeyCode::Char('t') => app.toggle_timestamps(),
                            KeyCode::Char('G') => app.scroll_to_bottom(),
                            KeyCode::Up | KeyCode::Char('k') => app.manual_scroll(-1),
                            KeyCode::Down | KeyCode::Char('j') => app.manual_scroll(1),
                            KeyCode::PageUp => app.manual_scroll(-10),
                            KeyCode::PageDown => app.manual_scroll(10),
                            KeyCode::Home => {
                                let offset = app.scroll_offset();
                                app.manual_scroll(-(offset as i32));
                            }
                            KeyCode::End => app.scroll_to_bottom(),
                            _ => {}
                        },
                        AppMode::Command => match key.code {
                            KeyCode::Esc => {
                                app.set_mode(AppMode::Normal);
                                app.clear_command_input();
                            }
                            KeyCode::Enter => {
                                app.process_command();
                                app.set_mode(AppMode::Normal);
                            }
                            KeyCode::Backspace => {
                                app.pop_command_input();
                            }
                            KeyCode::Char(c) => {
                                app.push_command_input(c);
                            }
                            _ => {}
                        },
                        AppMode::Search => match key.code {
                            KeyCode::Esc => {
                                app.set_mode(AppMode::Normal);
                                app.clear_command_input();
                                app.set_text_filter(None);
                            }
                            KeyCode::Enter => {
                                // Apply the text filter
                                let search_text = app.command_input().to_string();
                                if search_text.is_empty() {
                                    app.set_text_filter(None);
                                } else {
                                    app.set_text_filter(Some(search_text));
                                }
                                app.set_mode(AppMode::Normal);
                            }
                            KeyCode::Backspace => {
                                app.pop_command_input();
                            }
                            KeyCode::Char(c) => {
                                app.push_command_input(c);
                            }
                            _ => {}
                        },
                        AppMode::Statistics => match key.code {
                            KeyCode::Char('s') | KeyCode::Char('q') | KeyCode::Esc => {
                                app.toggle_statistics_mode()
                            }
                            _ => {}
                        },
                        AppMode::Detail => match key.code {
                            KeyCode::Esc | KeyCode::Char('q') => app.close_detail_view(),
                            _ => {}
                        },
                    }
                }
            }
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
