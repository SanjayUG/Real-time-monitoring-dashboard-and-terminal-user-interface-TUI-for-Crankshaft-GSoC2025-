use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tokio::{
    sync::mpsc,
    time::sleep,
};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
struct Job {
    id: u64,
    name: String,
    status: String, // e.g., "Running", "Completed", "Failed"
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Initialize jobs
    let mut jobs = vec![
        Job {
            id: 1,
            name: "Job A".to_string(),
            status: "Running".to_string(),
        },
        Job {
            id: 2,
            name: "Job B".to_string(),
            status: "Completed".to_string(),
        },
    ];

    // Channel for updating jobs asynchronously
    let (tx, mut rx) = mpsc::channel(1);

    // Spawn async job updater
    let mut jobs_clone = jobs.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(2)).await;
            for job in jobs_clone.iter_mut() {
                job.status = if job.status == "Running" {
                    "Completed".to_string()
                } else {
                    "Running".to_string()
                };
            }
            tx.send(jobs_clone.clone()).await.ok();
        }
    });

    // Main loop
    loop {
        // Update job statuses if received
        if let Ok(updated_jobs) = rx.try_recv() {
            jobs = updated_jobs;
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Length(3), // Header
                        Constraint::Min(10),   // Job list
                        Constraint::Length(3), // Footer
                    ]
                    .as_ref(),
                )
                .split(f.size());

            // Header
            let header = Paragraph::new("Crankshaft Monitoring Dashboard")
                .style(Style::default().fg(Color::LightCyan))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(header, chunks[0]);

            // Job list
            let job_list = Paragraph::new(
                jobs.iter()
                    .map(|job| Spans::from(vec![
                        Span::styled(format!("{}: ", job.id), Style::default().fg(Color::Yellow)),
                        Span::raw(format!("{} - {}", job.name, job.status)),
                    ]))
                    .collect::<Vec<_>>(),
            )
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(job_list, chunks[1]);

            // Footer
            let footer = Paragraph::new("Press `q` to quit")
                .style(Style::default().fg(Color::Gray))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[2]);
        })?;

        // Handle user input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}
