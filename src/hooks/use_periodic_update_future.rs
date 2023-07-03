// source
// https://gist.githubusercontent.com/valyagolev/9ddd2805df88e125dc95ab5d106f7cb2/raw/2c0201365e24edba2ac301966364931a48c52370/use_periodic_update_future.rs

use std::{future::Future, mem::ManuallyDrop, sync::Arc, time::Duration};

use dioxus::prelude::Scope;
use tokio::sync::{
    oneshot::{self, Sender},
    RwLock
};

pub struct PeriodicUpdateSub {
    sender: ManuallyDrop<Sender<()>>,
}

impl Drop for PeriodicUpdateSub {
    fn drop(&mut self) {
        let sender = unsafe { ManuallyDrop::take(&mut self.sender) };
        sender.send(()).unwrap();
    }
}

#[allow(dead_code)]
pub fn use_periodic_update(cx: Scope, interval: Duration) {
    cx.use_hook(|| {
        let update = cx.schedule_update();

        let (sender, mut receiver) = oneshot::channel();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = &mut receiver => break,
                    _ = tokio::time::sleep(interval) => {
                        update();
                    }
                }
            }
        });

        PeriodicUpdateSub {
            sender: ManuallyDrop::new(sender),
        }
    });
}

#[allow(dead_code)]
pub fn use_periodic_update_future<'a, T, F>(
    cx: &'a Scope,
    interval: Duration,
    future_fabric: impl Send + Sync + 'static + Fn() -> F,
)
where
    T: Send + Sync + 'static,
    F: Send + Future<Output = T>,
{
    cx.use_hook(|| {
        let value = Arc::new(RwLock::new(None));

        let update = cx.schedule_update();

        let (sender, mut receiver) = oneshot::channel();

        {
            let value = value.clone();
            tokio::spawn(async move {
                loop {
                    let val = Some(future_fabric().await);
                    *value.write().await = val;

                    tokio::select! {
                        _ = &mut receiver => break,
                        _ = tokio::time::sleep(interval) => {
                            update();
                        }
                    }
                }
            });
        }

        (
            value,
            PeriodicUpdateSub {
                sender: ManuallyDrop::new(sender),
            },
        )
    });
}