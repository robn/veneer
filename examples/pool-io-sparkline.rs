// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::error::Error;
use std::iter;
use std::time::{Duration, Instant};
use veneer::zfs::{self, Pool};

fn get_stats(pool: &Pool) -> Result<(u64, u64), Box<dyn Error>> {
    let vs = pool.root_vdev()?.stats()?;
    Ok((vs.bytes[1], vs.bytes[2]))
}

struct PoolState {
    read_last: u64,
    write_last: u64,
    read_history: Vec<u64>,
    write_history: Vec<u64>,
}

fn render_pool<B: Backend>(frame: &mut Frame<B>, rect: Rect, pool: &Pool, state: &PoolState) {
    let block = Block::default().title(pool.name()).borders(Borders::ALL);
    let inner = block.inner(rect);
    frame.render_widget(block, rect);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)].as_ref())
        .split(inner);
    let sparkline_r = Sparkline::default()
        .data(&state.read_history)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(sparkline_r, rows[0]);
    let sparkline_w = Sparkline::default()
        .data(&state.write_history)
        .style(Style::default().fg(Color::Green));
    frame.render_widget(sparkline_w, rows[1]);
}

fn main() -> Result<(), Box<dyn Error>> {
    // connect to zfs and get initial stats
    let z = zfs::open()?;

    let mut pool_state: Vec<(Pool, PoolState)> = vec![];

    for pool in z.pools()? {
        let (r, w) = get_stats(&pool)?;
        let read_v: Vec<u64> = iter::repeat(0).take(200).collect();
        let write_v: Vec<u64> = iter::repeat(0).take(200).collect();
        pool_state.push((
            pool,
            PoolState {
                read_last: r,
                write_last: w,
                read_history: read_v,
                write_history: write_v,
            },
        ));
    }

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick = Duration::from_millis(1000);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| {
            let rows = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    iter::repeat(Constraint::Length(4))
                        .take(pool_state.len())
                        .chain(iter::once(Constraint::Min(0)))
                        .collect::<Vec<_>>(),
                )
                .split(f.size());
            pool_state
                .iter()
                .zip(0..pool_state.len())
                .for_each(|((pool, state), row)| render_pool(f, rows[row], pool, state));
        })?;

        let timeout = tick
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }
        if last_tick.elapsed() >= tick {
            for (pool, state) in pool_state.iter_mut() {
                let (r, w) = get_stats(&pool)?;
                let (dr, dw) = (r - state.read_last, w - state.write_last);
                state.read_history.pop();
                state.read_history.insert(0, dr);
                state.write_history.pop();
                state.write_history.insert(0, dw);
                state.read_last = r;
                state.write_last = w;
            }

            last_tick = Instant::now();
        }
    }

    // restore terminal
    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
