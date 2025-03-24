#![cfg_attr(feature = "strict", deny(warnings))]

use std::rc::Rc;

use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

use crate::app::App;

pub fn render_detail_view(f: &mut Frame, app: &App, main_chunks: Rc<[Rect]>) {
    if let Some(message) = app.selected_message() {
        let mut details = Vec::new();

        // header with timestamp
        details.push(Line::from(vec![Span::styled(
            "Message Details",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        details.push(Line::from(""));

        // timestamp
        details.push(Line::from(vec![
            Span::styled("Timestamp: ", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(
                message
                    .timestamp
                    .strftime("%Y-%m-%d %H:%M:%S.%3f")
                    .to_string(),
            ),
        ]));

        // event type
        details.push(Line::from(vec![
            Span::styled(
                "Event Type: ",
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{:?}", message.event_type)),
        ]));

        if message
            .event_type
            .contains(crate::models::EventType::MESSAGE)
        {
            details.push(Line::from(vec![
                Span::styled("Direction: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{:?}", message.direction)),
            ]));

            if let Some(peer_id) = message.peer_id {
                details.push(Line::from(vec![
                    Span::styled("Peer ID: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(peer_id.to_string()),
                ]));
            }

            if let Some(conn_type) = &message.conn_type {
                details.push(Line::from(vec![
                    Span::styled(
                        "Connection Type: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(conn_type.clone()),
                ]));
            }

            if let Some(msg_type) = &message.p2p_msg_type {
                details.push(Line::from(vec![
                    Span::styled(
                        "Message Type: ",
                        Style::default().add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(msg_type.clone()),
                ]));
            }

            if let Some(size) = message.msg_size {
                details.push(Line::from(vec![
                    Span::styled("Size: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(format!("{} bytes", size)),
                ]));
            }
        }

        details.push(Line::from(""));
        details.push(Line::from(vec![Span::styled(
            "Content:",
            Style::default().add_modifier(Modifier::BOLD),
        )]));
        details.push(Line::from(""));

        let content_lines = message.content.lines().collect::<Vec<_>>();
        if content_lines.is_empty() {
            details.push(Line::from("[Empty content]"));
        } else {
            for line in content_lines {
                details.push(Line::from(line));
            }
        }

        let detail_paragraph = Paragraph::new(details)
            .block(
                Block::default()
                    .title("Message Detail View (Press Esc to return)")
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(detail_paragraph, main_chunks[0]);

        // Help text
        let help_text = "Press Esc to return to normal view";
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::TOP))
            .alignment(Alignment::Center);
        f.render_widget(help, main_chunks[2]);
    }
}
