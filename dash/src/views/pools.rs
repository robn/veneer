use crate::{components::Panel, views::PoolView};
use anystring::AnyString;
use iocraft::{component, element, AnyElement};
use std::collections::BTreeSet;

#[component]
pub(crate) fn PoolsView() -> impl Into<AnyElement<'static>> {
    let pools: BTreeSet<AnyString> = {
        let mut set = BTreeSet::new();
        set.insert("lucy".into());
        set
    };

    element! {
        Panel(title: "pools") {
            #(pools.iter().map(|name| {
                element! {
                    PoolView(name)
                }
            }))
        }
    }
}
