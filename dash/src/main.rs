// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

mod components;
mod data;
mod theme;
mod views;

use iocraft::prelude::*;
use std::collections::BTreeMap;
use veneer::Error;

use crate::data::PoolData;
use crate::theme::Theme;
use crate::views::PoolView;

#[derive(Clone, Debug, Default)]
struct DashData {
    pools: BTreeMap<String, PoolData>,
}

impl DashData {
    async fn fetch() -> Result<Self, Error> {
        let z = veneer::open()?;

        let mut pools = BTreeMap::default();
        for pool in z.pools()? {
            let root = pool.root_vdev()?;
            let stats = root.stats()?;

            let data = PoolData {
                name: pool.name(),
                state: stats.state,
                size: stats.space,
                alloc: stats.alloc,
                read: stats.bytes[1],
                write: stats.bytes[2],
                _wat: format!("{:?}", stats),
            };

            pools.insert(pool.name(), data);
        }

        Ok(DashData { pools })
    }
}

enum DashState {
    Init,
    Loading,
    Loaded(Result<DashData, Error>),
}

#[component]
fn Dash(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut theme = hooks.use_state(|| Theme::default());

    let mut state = hooks.use_state(|| DashState::Init);
    hooks.use_future(async move {
        loop {
            if !matches!(*state.read(), DashState::Loading) {
                state.set(DashState::Loading);
                state.set(DashState::Loaded(DashData::fetch().await));
            }
            smol::Timer::after(std::time::Duration::from_secs(1)).await;
        }
    });

    let mut exit = hooks.use_state(|| false);
    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => {
                match code {
                    KeyCode::Char('q') => exit.set(true),
                    KeyCode::Char('t') => theme.set(theme.get().toggle()),
                    _ => {}
                }
            }
            _ => {}
        }
    });
    if exit.get() {
        system.exit();
    }

    element! {
        ContextProvider(value: Context::owned(theme.get())) {
            View(
                width,
                height,
                background_color: theme.get().palette().background,
                flex_direction: FlexDirection::Column,
            ) {
                #(match &*state.read() {
                    DashState::Loaded(Ok(data)) => element! {
                        Fragment {
                            #(data.pools.iter().map(|(name,data)| {
                                element! {
                                    PoolView(key: name.clone(), data: data.clone())
                                }
                            }))
                        }
                    }.into_any(),
                    DashState::Loaded(Err(err)) => element! {
                        View(
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            width: 100pct,
                            height: 100pct,
                            padding: 2,
                        ) {
                            Text(content: "Error!", weight: Weight::Bold, color: Color::Red)
                            Text(content: format!("{:#}", err))
                        }
                    }.into_any(),
                    _ => element! {
                        Text(content: "loading")
                    }.into_any(),
                })
            }
        }
    }
}

fn main() {
    smol::block_on(element!(Dash).fullscreen()).unwrap();
}
