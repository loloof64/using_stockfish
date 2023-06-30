/*
Credits goes to ealmloff
https://discord.com/channels/899851952891002890/943190605067079712/1095725883131691050
*/

use dioxus::prelude::ScopeState;

pub fn use_component_lifecycle<C: FnOnce() + 'static, D: FnOnce() + 'static>(
    cx: &ScopeState,
    create: C,
    destroy: D,
) -> &LifeCycle<D> {
    cx.use_hook(|| {
        cx.spawn(async move {
            // This will be run once the component is mounted
            std::future::ready::<()>(()).await;
            create();
        });
        LifeCycle {
            ondestroy: Some(destroy),
        }
    })
}

pub struct LifeCycle<D: FnOnce()> {
    ondestroy: Option<D>,
}

impl<D: FnOnce()> Drop for LifeCycle<D> {
    fn drop(&mut self) {
        let f = self.ondestroy.take().unwrap();
        f();
    }
}