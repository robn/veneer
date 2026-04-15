// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

mod components;
mod data;
mod loader;
mod theme;
mod views;

use anystring::AnyString;
use iocraft::prelude::*;
use std::collections::BTreeMap;
use veneer::Error;

use crate::components::PropsProvider;
use crate::data::PoolData;
use crate::loader::Loader;
use crate::theme::Theme;
use crate::views::PoolView;

#[derive(Default, Clone, Props)]
struct DashPoolViewProps {
    pools: BTreeMap<AnyString, PoolData>,
}

#[component]
fn DashPoolView(props: &DashPoolViewProps) -> impl Into<AnyElement<'static>> {
    element! {
        Fragment {
            #(props.pools.iter().map(|(name, data)| {
                element! {
                    PoolView(key: name.to_c_string(), data: data.clone())
                }
            }))
        }
    }
}

impl Loader for DashPoolViewProps {
    async fn load() -> Result<Self, Error> {
        let z = veneer::open()?;

        let mut pools = BTreeMap::default();
        for pool in z.pools()? {
            let root = pool.root_vdev()?;
            let stats = root.stats()?;

            let data = PoolData {
                name: pool.name().into(),
                state: stats.state,
                size: stats.space,
                alloc: stats.alloc,
                read: stats.bytes[1],
                write: stats.bytes[2],
                _wat: format!("{:?}", stats),
            };

            pools.insert(pool.name().into(), data);
        }

        Ok(Self { pools })
    }
}

#[component]
fn Dash(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut theme = hooks.use_state(|| Theme::default());

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
                PropsProvider<DashPoolView>
            }
        }
    }
}

fn main() {
    smol::block_on(element!(Dash).fullscreen()).unwrap();
}
