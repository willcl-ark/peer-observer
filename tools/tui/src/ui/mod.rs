#![cfg_attr(feature = "strict", deny(warnings))]

mod detail_view;
mod normal_view;
mod stats_view;

pub use detail_view::render_detail_view;
pub use normal_view::render_normal_view;
pub use stats_view::render_statistics_view;

use std::sync::{Arc, Mutex};

use ratatui::Frame;

use crate::app::App;

pub fn ui(f: &mut Frame, app: &Arc<Mutex<App>>) {
    let app = app.lock().unwrap();

    // Create a layout with space for command input at the bottom
    let main_chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Min(3),
            ratatui::layout::Constraint::Length(1),
            ratatui::layout::Constraint::Length(2),
        ])
        .split(f.area());

    // Render different view based on mode
    match app.mode() {
        crate::models::AppMode::Statistics => render_statistics_view(f, &app, main_chunks),
        crate::models::AppMode::Detail => render_detail_view(f, &app, main_chunks),
        _ => render_normal_view(f, &app, main_chunks),
    };
}
