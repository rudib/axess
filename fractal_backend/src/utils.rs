
use futures::{Future, executor::block_on, select_biased, future::FusedFuture};
use std::time::Duration;
use tokio::{runtime::Runtime, time::{timeout, delay_for, Elapsed, Timeout}};
use crate::{FractalResult, FractalCoreError};


/// Tries to find and map the matching message from the broadcast channel. With timeout support.
pub async fn filter_first<T: Send + Clone, TOut, TMap: Fn(T) -> Option<TOut>>(channel: &mut broadcaster::BroadcastChannel<T>, map_and_filter: TMap, timeout: Duration) -> FractalResult<TOut> {
        
    async fn first_message<T: Send + Clone, TOut, TMap: Fn(T) -> Option<TOut>>(channel: &mut broadcaster::BroadcastChannel<T>, map_and_filter: TMap) -> FractalResult<TOut> {
        loop {
            let msg = channel.recv().await;
            if let Some(msg) = msg {
                let mapped = map_and_filter(msg);
                if mapped.is_some() { 
                    return Ok(mapped.unwrap());
                }
            } else {
                return Err(FractalCoreError::Unknown);
            }
        }
    }

    let with_timeout = tokio::time::timeout(timeout, first_message(channel, map_and_filter));

    match with_timeout.await {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(FractalCoreError::Timeout)
    }
}

/*
pub fn block_on_with_timeout<F: Future>(f: F, t: Duration) -> Result<F::Output, FractalCoreError> {
    let mut runtime = Runtime::new().unwrap();
    let r = runtime.block_on(async {
        timeout(t, f).await
    });
    r.map_err(|_| FractalCoreError::Timeout)
}
*/