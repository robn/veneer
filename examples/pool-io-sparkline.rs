// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023, Rob Norris <robn@despairlabs.com>

// XXX convert to high-level api, when we have one

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::collections::BTreeMap;
use std::error::Error;
use std::ffi::CStr;
use std::iter;
use std::time::{Duration, Instant};
use veneer::ioc;

// XXX this and other structures like it in fs/zfs.h can be extended with
//     new versions, but not reduced. so we need to initialise to zero, and
//     make sure we don't overrun, but its ok to come up short

// vdev_stat_t
#[repr(C)]
#[derive(Debug, Default)]
struct VdevStats {
    timestamp: u64, // hrtime_t
    state: u64,     // vdev_state_t
    aux: u64,       // vdev_aux_t
    alloc: u64,
    space: u64,
    dspace: u64,
    rsize: u64,
    esize: u64,
    ops: [u64; 6],   // VS_ZIO_TYPES
    bytes: [u64; 6], // VS_ZIO_TYPES
    read_errors: u64,
    write_errors: u64,
    checksum_errors: u64,
    initialize_errors: u64,
    self_healed: u64,
    scan_removing: u64,
    scan_processed: u64,
    fragmentation: u64,
    initialize_bytes_done: u64,
    initialize_bytes_est: u64,
    initialize_state: u64,       // vdev_initializing_state_t
    initialize_action_time: u64, // time_t
    checkpoint_space: u64,
    resilver_deferred: u64,
    slow_ios: u64,
    trim_errors: u64,
    trim_notsup: u64,
    trim_bytes_done: u64,
    trim_bytes_est: u64,
    trim_state: u64,       // vdev_trim_state_t
    trim_action_time: u64, // time_t
    rebuild_processed: u64,
    configured_ashift: u64,
    logical_ashift: u64,
    physical_ashift: u64,
    noalloc: u64,
    pspace: u64,
}

impl From<&[u64]> for VdevStats {
    fn from(s: &[u64]) -> Self {
        let count = std::cmp::min(
            s.len(),
            std::mem::size_of::<VdevStats>() / std::mem::size_of::<u64>(),
        );
        let mut vs = VdevStats::default();
        unsafe {
            std::ptr::copy_nonoverlapping(
                s.as_ptr(),
                std::ptr::addr_of_mut!(vs) as *mut u64,
                count,
            );
        }
        vs
    }
}

fn get_stats(ioc: &mut ioc::Handle, pool: &CStr) -> Result<(u64, u64), Box<dyn Error>> {
    let stats = ioc.pool_stats(pool)?;

    let (r, w) = stats
        .get("vdev_tree")
        .and_then(|p| p.as_list())
        .and_then(|l| l.get("vdev_stats"))
        .and_then(|p| p.as_u64_slice())
        .map(|s| VdevStats::from(s))
        .map(|vs| (vs.bytes[1], vs.bytes[2]))
        .unwrap_or_default();

    Ok((r, w))
}

struct PoolState {
    read_last: u64,
    write_last: u64,
    read_history: Vec<u64>,
    write_history: Vec<u64>,
}

fn render_pool<B: Backend>(frame: &mut Frame<B>, rect: Rect, pool: &CStr, state: &PoolState) {
    let block = Block::default()
        .title(pool.to_string_lossy().to_string())
        .borders(Borders::ALL);
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
    let mut ioc = ioc::Handle::open()?;

    let mut pool_state: BTreeMap<&CStr, PoolState> = BTreeMap::new();

    let config = ioc.pool_configs()?;
    for pool in config.keys() {
        let (r, w) = get_stats(&mut ioc, pool)?;
        let read_v: Vec<u64> = iter::repeat(0).take(200).collect();
        let write_v: Vec<u64> = iter::repeat(0).take(200).collect();
        pool_state.insert(
            pool,
            PoolState {
                read_last: r,
                write_last: w,
                read_history: read_v,
                write_history: write_v,
            },
        );
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
            for (&pool, state) in pool_state.iter_mut() {
                let (r, w) = get_stats(&mut ioc, pool)?;
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
