
use futures::{Future, executor::block_on, select_biased, future::FusedFuture};
use std::time::Duration;
use tokio::{runtime::Runtime, time::{timeout, delay_for, Elapsed, Timeout}};

pub async fn channel_map_and_filter_first_async<T: Send + Clone, TOut, TMap: Fn(T) -> Option<TOut>>(channel: &mut broadcaster::BroadcastChannel<T>, map_and_filter: TMap) -> Option<TOut> {
    loop {
        let msg = channel.recv().await;
        if let Some(msg) = msg {
            let mapped = map_and_filter(msg);
            if mapped.is_some() { return mapped; }
        }
    }
}

pub fn block_on_with_timeout<F: Future>(f: F, t: Duration) -> Result<F::Output, fractal_core::FractalCoreError> {
    let mut runtime = Runtime::new().unwrap();
    let r = runtime.block_on(async {
        timeout(t, f).await
    });
    r.map_err(|_| fractal_core::FractalCoreError::Timeout)
}
