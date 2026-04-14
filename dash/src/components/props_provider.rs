// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use iocraft::{
    component,
    components::{Fragment, Text, View},
    element,
    hooks::{UseFuture, UseRef, UseState},
    AlignItems, AnyElement, Color, Component, Element, ElementKey, FlexDirection, Hooks,
    JustifyContent, Weight,
};
use veneer::Error;

use crate::loader::Loader;

enum LoadState {
    Init,
    Loading,
    Loaded,
    Error(Error),
}

#[component]
pub(crate) fn PropsProvider<C: Component>(mut hooks: Hooks) -> impl Into<AnyElement<'static>>
where
    C::Props<'static>: Loader + Clone + Unpin + Sync + Send + 'static,
{
    let mut state = hooks.use_state(|| LoadState::Init);
    let mut data = hooks.use_ref(|| None);

    hooks.use_future(async move {
        loop {
            if !matches!(*state.read(), LoadState::Loading) {
                state.set(LoadState::Loading);
                state.set(C::Props::load().await.map_or_else(
                    |e| LoadState::Error(e),
                    |d| {
                        data.set(Some(d));
                        LoadState::Loaded
                    },
                ));
            }
            smol::Timer::after(std::time::Duration::from_secs(1)).await;
        }
    });

    element! {
        Fragment {
            #(match &*state.read() {
                LoadState::Loaded => Element::<C> {
                    key: ElementKey::new(()),
                    props: data.read().as_ref().unwrap().clone()
                }.into_any(),
                LoadState::Error(err) => element! {
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
