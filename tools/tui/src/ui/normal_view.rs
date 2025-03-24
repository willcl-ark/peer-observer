#![cfg_attr(feature = "strict", deny(warnings))]

use std::rc::Rc;

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};
use ringbuffer::RingBuffer;

use crate::{
    app::App,
    models::{AppMode, EventType},
};

pub fn render_normal_view(f: &mut Frame, app: &App, main_chunks: Rc<[Rect]>) {
    let filtered_messages = app.filtered_messages();

    // Create list items with color styling based on message type
    let items: Vec<ListItem> = filtered_messages
        .iter()
        .map(|msg| {
            // Determine message color and style based on message type and direction
            let (message_color, prefix_style) = match msg.event_type {
                event_type if event_type.contains(EventType::MESSAGE) => {
                    // For P2P messages, color based on direction
                    if msg.direction.contains(crate::models::Direction::INBOUND) {
                        (Color::Green, Style::default().fg(Color::Green))
                    } else if msg.direction.contains(crate::models::Direction::OUTBOUND) {
                        (Color::Cyan, Style::default().fg(Color::Cyan))
                    } else {
                        (Color::White, Style::default().fg(Color::White))
                    }
                }
                event_type if event_type.contains(EventType::CONNECTION) => {
                    (Color::Magenta, Style::default().fg(Color::Magenta))
                }
                event_type if event_type.contains(EventType::ADDRMAN) => {
                    (Color::Yellow, Style::default().fg(Color::Yellow))
                }
                event_type if event_type.contains(EventType::MEMPOOL) => {
                    (Color::LightBlue, Style::default().fg(Color::LightBlue))
                }
                event_type if event_type.contains(EventType::VALIDATION) => {
                    (Color::LightRed, Style::default().fg(Color::LightRed))
                }
                _ => (Color::White, Style::default().fg(Color::White)),
            };

            let formatted = msg.format(app.show_timestamps());
            let (prefix, rest) = if let Some(bracket_end) = formatted.rfind(']') {
                (
                    formatted[..=bracket_end].to_string(),
                    formatted[bracket_end + 1..].to_string(),
                )
            } else {
                (formatted.clone(), String::new())
            };

            // Create the line with styled components
            let text = vec![Line::from(vec![
                Span::styled(prefix, prefix_style),
                Span::styled(rest, Style::default().fg(message_color)),
            ])];

            ListItem::new(text)
        })
        .collect();

    // Main title showing current mode
    let mode_indicator = match app.mode() {
        AppMode::Normal => Span::styled(
            " NORMAL ",
            Style::default().bg(Color::Blue).fg(Color::White),
        ),
        AppMode::Command => Span::styled(
            " COMMAND ",
            Style::default().bg(Color::Yellow).fg(Color::Black),
        ),
        AppMode::Search => Span::styled(
            " SEARCH ",
            Style::default().bg(Color::Green).fg(Color::Black),
        ),
        AppMode::Statistics => Span::styled(
            " STATISTICS ",
            Style::default().bg(Color::Magenta).fg(Color::White),
        ),
        AppMode::Detail => Span::styled(
            " DETAIL ",
            Style::default().bg(Color::Cyan).fg(Color::Black),
        ),
    };

    let autoscroll_indicator = if app.autoscroll() {
        Span::styled(
            " [AUTOSCROLL] ",
            Style::default()
                .add_modifier(Modifier::ITALIC)
                .fg(Color::Green),
        )
    } else {
        Span::raw("")
    };

    let timestamp_indicator = if app.show_timestamps() {
        Span::styled(
            " [TIMESTAMPS] ",
            Style::default()
                .add_modifier(Modifier::ITALIC)
                .fg(Color::Blue),
        )
    } else {
        Span::raw("")
    };

    // Messages List widget
    let messages = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(
                    format!(
                        "Peer Observer Observer {}{}{}",
                        mode_indicator, autoscroll_indicator, timestamp_indicator
                    ),
                    Style::default().add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    // Render
    // Use app's scroll_offset for positioning, but ensure it's valid
    let selected_index = if filtered_messages.is_empty() {
        0
    } else {
        // Clamp to valid range
        let max_index = filtered_messages.len() - 1;
        app.scroll_offset().min(max_index as u16) as usize
    };

    f.render_stateful_widget(
        messages,
        main_chunks[0],
        &mut ratatui::widgets::ListState::default().with_selected(
            if filtered_messages.is_empty() {
                None
            } else {
                Some(selected_index)
            },
        ),
    );

    let filter_status = app.get_filter_status();
    let status_text = format!(
        "{} | {} in filter | {} in buffer | buffer capacity: {} | Total observed: {}",
        filter_status,
        filtered_messages.len(),
        app.messages().len(),
        app.max_messages(),
        app.total_messages_observed(),
    );

    // Render status bar
    let status = Paragraph::new(status_text)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(status, main_chunks[1]);

    // Show command input or keyboard shortcuts based on mode
    match app.mode() {
        AppMode::Command => {
            let command_input = Paragraph::new(format!(": {}", app.command_input()))
                .block(Block::default().borders(Borders::TOP))
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(command_input, main_chunks[2]);
        }
        AppMode::Search => {
            let search_input = Paragraph::new(format!("/{}", app.command_input()))
                .block(Block::default().borders(Borders::TOP))
                .style(Style::default().fg(Color::Green));
            f.render_widget(search_input, main_chunks[2]);
        }
        AppMode::Normal => {
            let help_text = "p=p2p_msg c=connection a=addrman m=mempool v=validation | i/o/b=P2P inbound/outbound/both | /=search | t=timestamps | r=scroll s=stats G=bottom | ENTER=view detail | :buffer N | 0=clear | q=quit";
            let help = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::TOP))
                .alignment(Alignment::Center);
            f.render_widget(help, main_chunks[2]);
        }
        AppMode::Statistics => {
            // This mode should never reach here since we handle it separately,
            // but included for completeness
            let help_text = "You are in statistics mode. Press 's' to return to normal view.";
            let help = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::TOP))
                .alignment(Alignment::Center);
            f.render_widget(help, main_chunks[2]);
        }
        AppMode::Detail => {
            // This mode should never reach here since we handle it separately,
            // but included for completeness
            let help_text = "You are in detail view mode. Press Esc to return to normal view.";
            let help = Paragraph::new(help_text)
                .block(Block::default().borders(Borders::TOP))
                .alignment(Alignment::Center);
            f.render_widget(help, main_chunks[2]);
        }
    }
}
