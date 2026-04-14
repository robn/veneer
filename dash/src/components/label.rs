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
