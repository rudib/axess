
use futures::executor::block_on;

// todo: TIMEOUT!!!

pub fn channel_map_and_filter_first<T: Send + Clone, TOut, TMap: Fn(T) -> Option<TOut>>(channel: &mut broadcaster::BroadcastChannel<T>, map_and_filter: TMap) -> Option<TOut> {
    loop {
        let msg = block_on(channel.recv()).unwrap();
        let mapped = map_and_filter(msg);
        if mapped.is_some() { return mapped; }
    }
}