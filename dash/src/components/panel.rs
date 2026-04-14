use crate::Theme;
use iocraft::{
    component,
    components::{BorderStyle, Text, TextWrap, View},
    element,
    hooks::UseContext,
    AnyElement, FlexDirection, Hooks, Props,
};

#[derive(Default, Props)]
pub(crate) struct PanelProps<'a> {
    pub children: Vec<AnyElement<'a>>,
    pub title: String,
}

#[component]
pub(crate) fn Panel(
    props: &mut PanelProps<'static>,
    hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let palette = hooks.use_context::<Theme>().palette();

    element! {
        View(
            background_color: palette.background,
            border_style: BorderStyle::Single,
            border_color: palette.border,
            flex_direction: FlexDirection::Column,
            padding_left: 1,
            padding_right: 1,
        ) {
            View(margin_top: -1) {
                Text(
                    content: format!(" {} ", &props.title),
                    color: palette.border,
                    wrap: TextWrap::NoWrap,
                )
            }
            #(std::mem::take(&mut props.children))
        }
    }
}
