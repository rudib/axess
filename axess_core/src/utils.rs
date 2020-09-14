
use std::time::Duration;
use crate::{FractalResult, FractalCoreError};


/// Tries to find and map the matching message from the broadcast channel. With timeout support.
pub async fn filter_first<T: Send + Clone, TOut, TMap: FnMut(T) -> Option<TOut>>(channel: &mut broadcaster::BroadcastChannel<T>, map_and_filter: TMap, timeout: Duration) -> FractalResult<TOut> {
        
    async fn first_message<T: Send + Clone, TOut, TMap: FnMut(T) -> Option<TOut>>(channel: &mut broadcaster::BroadcastChannel<T>, mut map_and_filter: TMap) -> FractalResult<TOut> {
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