// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

mod components;
mod data;
mod theme;
mod views;

use crate::theme::Theme;
use crate::views::{DebugView, PoolsView};
use iocraft::{
    component,
    components::{ContextProvider, View},
    element,
    hooks::{UseContext, UseState, UseTerminalEvents, UseTerminalSize},
    AnyElement, Context, ElementExt, FlexDirection, Hooks, KeyCode, KeyEvent, KeyEventKind,
    SystemContext, TerminalEvent,
};

#[component]
fn Dash(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut theme = hooks.use_state(Theme::default);

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
                PoolsView
                DebugView
            }
        }
    }
}

fn main() {
    data::start();
    smol::block_on(element!(Dash).fullscreen()).unwrap();
}
