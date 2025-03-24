#![cfg_attr(feature = "strict", deny(warnings))]

use humansize::{BINARY, format_size};
use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
};
use ringbuffer::RingBuffer;
use std::rc::Rc;

use crate::{
    app::App,
    models::{Direction, EventType},
};

pub fn render_statistics_view(f: &mut Frame, app: &App, chunks: Rc<[Rect]>) {
    // layout for the stats view
    let stat_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[0]);

    // Left side - Event types and directions
    let left_chunks = Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(stat_chunks[0]);

    let mut event_type_items = vec![];
    let mut event_types: Vec<_> = app.statistics().event_type_counts.iter().collect();
    event_types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

    for (event_type, count) in event_types {
        let event_name = match *event_type {
            EventType::MESSAGE => "P2P Message",
            EventType::CONNECTION => "Connection",
            EventType::ADDRMAN => "Address Manager",
            EventType::MEMPOOL => "Mempool",
            EventType::VALIDATION => "Validation",
            _ => "Unknown",
        };

        // Choose color based on event type
        let event_color = match *event_type {
            EventType::MESSAGE => Color::Green,
            EventType::CONNECTION => Color::Magenta,
            EventType::ADDRMAN => Color::Yellow,
            EventType::MEMPOOL => Color::LightBlue,
            EventType::VALIDATION => Color::LightRed,
            _ => Color::White,
        };

        // Format with count and percentage
        let total = app.total_messages_observed() as f64;
        let percentage = if total > 0.0 {
            (*count as f64 / total) * 100.0
        } else {
            0.0
        };

        let event_fixed = format!("{:<16}", format!("{}", event_name));
        let count_text = format!("{} ({:.1}%)", count, percentage);
        let text = Line::from(vec![
            Span::styled(event_fixed, Style::default().fg(event_color)),
            Span::raw(count_text),
        ]);
        event_type_items.push(ListItem::new(text));
    }

    let mut direction_items = vec![];
    let mut directions: Vec<_> = app.statistics().direction_counts.iter().collect();
    directions.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

    let p2p_total = app.statistics().direction_counts.values().sum::<usize>() as f64;

    for (direction, count) in directions {
        if *direction == Direction::NONE {
            continue; // Skip NONE direction
        }

        let direction_name = direction.name();
        let direction_color = if direction.contains(Direction::INBOUND) {
            Color::Green
        } else if direction.contains(Direction::OUTBOUND) {
            Color::Cyan
        } else {
            Color::White
        };

        // Calculate percentage of P2P messages
        let percentage = if p2p_total > 0.0 {
            (*count as f64 / p2p_total) * 100.0
        } else {
            0.0
        };

        let dir_fixed = format!("{:<16}", format!("{}", direction_name));
        let count_text = format!("{} ({:.1}%)", count, percentage);

        let text = Line::from(vec![
            Span::styled(dir_fixed, Style::default().fg(direction_color)),
            Span::raw(count_text),
        ]);

        direction_items.push(ListItem::new(text));
    }

    // Event Type Stats List
    let event_stats = List::new(event_type_items)
        .block(
            Block::default()
                .title("Event Type")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    // Direction Stats List
    let direction_stats = List::new(direction_items)
        .block(
            Block::default()
                .title("Message Direction")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(event_stats, left_chunks[0]);
    f.render_widget(direction_stats, left_chunks[1]);

    // Right side - P2P message types
    let mut p2p_items = vec![];

    // Sort P2P message types by count
    let mut p2p_types: Vec<_> = app.statistics().p2p_message_counts.iter().collect();
    p2p_types.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending

    let total_bytes: u64 = app.statistics().p2p_message_bytes.values().sum();

    for (message_type, count) in p2p_types {
        let count_percentage = if p2p_total > 0.0 {
            (*count as f64 / p2p_total) * 100.0
        } else {
            0.0
        };

        let bytes = app
            .statistics()
            .p2p_message_bytes
            .get(message_type)
            .unwrap_or(&0);
        let bytes_percentage = if total_bytes > 0 {
            (*bytes as f64 / total_bytes as f64) * 100.0
        } else {
            0.0
        };

        let size_str = format_size(*bytes, BINARY);
        let msg_type_fixed = format!("{:<16}", format!("{}", message_type));
        let count_text = format!("{:>8} ({:.1}%)", count, count_percentage);
        let size_text = format!("{:>11} ({:.1}%)", size_str, bytes_percentage);

        let text = Line::from(vec![
            Span::styled(msg_type_fixed, Style::default().fg(Color::Blue)),
            Span::raw(format!("{:<20} | ", count_text)),
            Span::styled(size_text, Style::default().fg(Color::Green)),
        ]);

        p2p_items.push(ListItem::new(text));
    }

    // P2P Type Stats List
    let p2p_stats = List::new(p2p_items)
        .block(
            Block::default()
                .title("P2P Message Type")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(p2p_stats, stat_chunks[1]);

    let mut total_bytes = 0;
    for msg in app.messages().iter() {
        if let Some(size) = msg.msg_size {
            total_bytes += size;
        }
    }

    let stats_count = format!(
        "Total messages: {} | Unique P2P message types: {} | Total bytes: {}",
        app.total_messages_observed(),
        app.statistics().p2p_message_counts.len(),
        format_size(total_bytes, BINARY)
    );

    let stats_status = Paragraph::new(stats_count)
        .alignment(Alignment::Left)
        .block(Block::default().borders(Borders::NONE));

    f.render_widget(stats_status, chunks[1]);

    let help_text = "Press 's' to return to normal view | 'q' to quit";
    let help = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center);

    f.render_widget(help, chunks[2]);
}
