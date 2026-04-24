use crate::{components::Panel, data, views::PoolView};
use iocraft::{
    component,
    components::ContextProvider,
    element,
    hooks::{UseFuture, UseState},
    AnyElement, Context, Hooks,
};

#[component]
pub(crate) fn PoolsView(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut pools = hooks.use_state(|| data::get().pools);
    hooks.use_future(async move {
        loop {
            pools.set(data::get().pools);
            smol::Timer::after(std::time::Duration::from_secs(1)).await;
        }
    });

    element! {
        Panel(title: "pools") {
            #(pools.read().iter().map(|name| {
                element! {
                    ContextProvider(value: Context::owned(name.clone())) {
                        PoolView
                    }
                }
            }))
        }
    }
}
