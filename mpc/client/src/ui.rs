use std::time::Instant;

use ratatui::{
    crossterm::event::{self, KeyEvent},
    prelude::{
        symbols, Alignment, Backend, Color, Constraint, Frame, Layout, Line, Modifier, Rect, Span,
        Style, Terminal, Widget,
    },
    widgets::{block, Block, Gauge, LineGauge, List, ListItem, Paragraph},
};
use throbber_widgets_tui::ThrobberState;
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::types::Status;

pub enum Event {
    Input(KeyEvent),
    Tick,
    NewStatus(Status),
    Resize,
}

enum UiState {
    Idle,
    Downloading(String, u8, Instant),
    DownloadEnded,
    Contributing(Instant),
    ContributionEnded,
    Uploading(String, u8, Instant),
    Successful,
    Failed(String),
}

fn ui(f: &mut Frame, state: &UiState, throbber_state: &mut ThrobberState) {
    let area = f.size();

    let block = Block::new().title(
        block::Title::from("Contribution Steps (press `q` to exit)").alignment(Alignment::Center),
    );
    f.render_widget(block, area);

    let vertical = Layout::vertical([Constraint::Length(2), Constraint::Length(4)]).margin(1);
    let horizontal = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]);
    let [progress_area, main] = vertical.areas(area);
    let [list_area, gauge_area] = horizontal.areas(main);
    let chunks = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints(
            [
                ratatui::layout::Constraint::Min(10),
                ratatui::layout::Constraint::Percentage(100),
            ]
            .as_ref(),
        )
        .split(list_area);

    // total progress
    let steps_done = match state {
        UiState::Idle => 0,
        UiState::Downloading(_, _, _) => 0,
        UiState::DownloadEnded => 1,
        UiState::Contributing(_) => 1,
        UiState::ContributionEnded => 2,
        UiState::Uploading(_, _, _) => 2,
        UiState::Successful => 3,
        UiState::Failed(_) => 3,
    };
    let num_steps = 3;
    #[allow(clippy::cast_precision_loss)]
    let progress = LineGauge::default()
        .filled_style(Style::default().fg(Color::Blue))
        .label(format!("{steps_done}/{num_steps}"))
        .ratio(steps_done as f64 / num_steps as f64);
    f.render_widget(progress, progress_area);

    match state {
        UiState::Idle => {
            // Set full with state
            let full = throbber_widgets_tui::Throbber::default()
                .label("Awaiting orders...")
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::White))
                .throbber_style(
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::CLOCK)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_stateful_widget(full, chunks[0], throbber_state);
        }
        UiState::Downloading(name, progress, started_at) => {
            // in progress download
            let item = ListItem::new(Line::from(vec![
                Span::raw(symbols::DOT),
                Span::styled(
                    format!(" downloading {:>2}", name),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" ({}s)", started_at.elapsed().as_secs())),
            ]));

            let list = List::new(vec![item]);
            f.render_widget(list, list_area);

            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(Color::Cyan))
                .ratio(*progress as f64 / 100.0);
            if gauge_area.top().saturating_add(0 as u16) > area.bottom() {
                return;
            }
            f.render_widget(
                gauge,
                Rect {
                    x: gauge_area.left(),
                    y: gauge_area.top().saturating_add(0 as u16),
                    width: gauge_area.width,
                    height: 1,
                },
            );
        }
        UiState::Contributing(_) => {
            let full = throbber_widgets_tui::Throbber::default()
                .label("Your contribution is being computed, please be patient...")
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::White))
                .throbber_style(
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::CLOCK)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_stateful_widget(full, chunks[0], throbber_state);
        }
        UiState::Uploading(name, _, started_at) => {
            let item = ListItem::new(Line::from(vec![
                Span::raw(symbols::DOT),
                Span::styled(
                    format!(" uploading {:>2}", name),
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" ({}s)", started_at.elapsed().as_secs())),
            ]));
            f.render_widget(List::new(vec![item]), list_area);
        }
        UiState::Successful => {
            // Set full with state
            let full = throbber_widgets_tui::Throbber::default()
                .label("Contribution successfully upload...")
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan))
                .throbber_style(
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::CLOCK)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_stateful_widget(full, chunks[0], throbber_state);
        }
        UiState::Failed(error) => {
            // Set full with state
            let full = throbber_widgets_tui::Throbber::default()
                .label(format!("Failed to contribute: {}", error))
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Red))
                .throbber_style(
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::CLOCK)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_stateful_widget(full, chunks[0], throbber_state);
        }
        UiState::DownloadEnded => {
            // Set full with state
            let full = throbber_widgets_tui::Throbber::default()
                .label("Initializing contribution...")
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::White))
                .throbber_style(
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::CLOCK)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_stateful_widget(full, chunks[0], throbber_state);
        }
        UiState::ContributionEnded => {
            // Set full with state
            let full = throbber_widgets_tui::Throbber::default()
                .label("Initializing upload...")
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::White))
                .throbber_style(
                    ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD),
                )
                .throbber_set(throbber_widgets_tui::CLOCK)
                .use_type(throbber_widgets_tui::WhichUse::Spin);
            f.render_stateful_widget(full, chunks[0], throbber_state);
        }
    }
}

pub async fn run_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    mut rx: mpsc::UnboundedReceiver<Event>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut state = UiState::Idle;
    // let mut download_name = "".into();
    // let mut download_started_at = Instant::now();
    // let mut upload_name = "".into();
    // let mut upload_started_at = Instant::now();
    let mut throbber_state = ThrobberState::default();
    let mut redraw = false;
    let mut start_time = Instant::now();
    loop {
        if redraw {
            throbber_state.calc_next();
            terminal.draw(|f| ui(f, &state, &mut throbber_state))?;
            redraw = false;
        }
        match rx.try_recv() {
            Ok(e) => match e {
                Event::Input(event) => {
                    if event.code == event::KeyCode::Char('q') {
                        break;
                    }
                }
                Event::Resize => {
                    terminal.autoresize()?;
                }
                Event::Tick => {
                    redraw = true;
                }
                Event::NewStatus(new_status) => {
                    state = match (new_status, state) {
                        (Status::Idle, _) => UiState::Idle,
                        (Status::DownloadStarted(name), _) => {
                            start_time = Instant::now();
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Started downloading"),
                                    Span::styled(
                                        format!("checkpoint {}", &name),
                                        Style::default().add_modifier(Modifier::BOLD),
                                    ),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::Downloading(name, 0, Instant::now())
                        }
                        (
                            Status::Downloading(name, progress),
                            UiState::Downloading(_, _, started_at),
                        ) => UiState::Downloading(name, progress, started_at),
                        (Status::DownloadEnded(_), UiState::Downloading(name, _, started_at)) => {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Finished "),
                                    Span::styled(
                                        format!("downloading checkpoint {}", &name),
                                        Style::default().add_modifier(Modifier::BOLD),
                                    ),
                                    Span::from(format!(" in {}s", started_at.elapsed().as_secs())),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::DownloadEnded
                        }
                        (Status::ContributionStarted, _) => {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Started "),
                                    Span::styled(
                                        "contribution computation...",
                                        Style::default().add_modifier(Modifier::BOLD),
                                    ),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::Contributing(Instant::now())
                        }
                        (Status::ContributionEnded, UiState::Contributing(started_at)) => {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Finished "),
                                    Span::styled(
                                        "contribution computation",
                                        Style::default().add_modifier(Modifier::BOLD),
                                    ),
                                    Span::from(format!(" in {}s", started_at.elapsed().as_secs())),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::ContributionEnded
                        }
                        (Status::UploadStarted(name), _) => {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Started uploading"),
                                    Span::styled(
                                        format!("contribution {}", &name),
                                        Style::default().add_modifier(Modifier::BOLD),
                                    ),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::Uploading(name, 0, Instant::now())
                        }
                        (
                            Status::UploadEnded(_),
                            UiState::Uploading(name, progress, started_at),
                        ) => {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Finished "),
                                    Span::styled(
                                        format!("uploading contribution {}", &name),
                                        Style::default().add_modifier(Modifier::BOLD),
                                    ),
                                    Span::from(format!(" in {}s", started_at.elapsed().as_secs())),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::Uploading(name, progress, started_at)
                        }
                        (Status::Successful, _) => {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Done, "),
                                    Span::styled(
                                        "successfully contributed",
                                        Style::default()
                                            .add_modifier(Modifier::BOLD)
                                            .fg(Color::Green),
                                    ),
                                    Span::from(format!(" in {}s", start_time.elapsed().as_secs())),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::Successful
                        }
                        (Status::Failed(err), _) => {
                            terminal.insert_before(1, |buf| {
                                Paragraph::new(Line::from(vec![
                                    Span::from("Done "),
                                    Span::styled(
                                        format!("contribution failed: {}", err),
                                        Style::default()
                                            .add_modifier(Modifier::BOLD)
                                            .fg(Color::Red),
                                    ),
                                    Span::from(format!(" in {}s", start_time.elapsed().as_secs())),
                                ]))
                                .render(buf.area, buf);
                            })?;
                            UiState::Failed(err)
                        }
                        (_, s) => s,
                    };
                }
            },
            Err(TryRecvError::Empty) => {}
            _ => panic!("impossible"),
        };
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    Ok(())
}
