// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

// Copyright (c) 2023-2026, Rob Norris <robn@despairlabs.com>

use crate::Theme;
use iocraft::{
    component,
    components::{Text, TextWrap},
    element,
    hooks::UseContext,
    AnyElement, Color, Hooks, Props,
};

#[derive(Default, Props)]
pub(crate) struct LabelProps {
    pub content: String,
    pub color: Option<Color>,
}

#[component]
pub(crate) fn Label(props: &LabelProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();

    element! {
        Text(
            content: &props.content,
            color: props.color.unwrap_or_else(|| palette.text),
            wrap: TextWrap::NoWrap,
        )
    }
}
