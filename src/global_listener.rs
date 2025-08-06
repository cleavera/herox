use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

#[napi]
#[derive(Clone, Copy, Debug)]
pub enum GlobalInputAction {
    Keyboard,
}

// A thread-safe function that can be called from any thread to notify JS.
type Subscriber = ThreadsafeFunction<GlobalInputAction>;
// A unique ID to track and remove subscribers.
type SubscriberId = u64;

// The shared state that will be accessed by the listener and dispatcher threads.
#[derive(Clone, Default)]
struct ListenerState {
    subscribers: Arc<Mutex<HashMap<SubscriberId, Subscriber>>>,
    next_id: Arc<Mutex<SubscriberId>>,
}

#[napi]
pub struct GlobalListener {
    // Sends actions from the OS listener thread to the dispatcher thread.
    action_tx: Option<Sender<GlobalInputAction>>,
    // We hold the handles to ensure threads are cleaned up when the listener is dropped.
    _os_listener_handle: Option<JoinHandle<()>>,
    _dispatcher_handle: Option<JoinHandle<()>>,
}

#[napi]
impl GlobalListener {
    #[napi(constructor)]
    pub fn new() -> Self {
        // 1. Create a new mpsc channel (sender/receiver).
        // 2. Create a new ListenerState.
        // 3. Spawn the dispatcher thread, giving it the receiver and state.
        // 4. Spawn the OS listener thread, giving it the sender.
        // 5. Return a new `GlobalListener` instance containing the sender and thread handles.
        todo!()
    }

    /// Subscribes to global input events.
    ///
    /// Returns a function that, when called, will unsubscribe the listener.
    #[napi]
    pub fn subscribe(
        &self,
        _env: Env,
        #[napi(ts_arg_type = "(action: GlobalInputAction) => void")] _subscriber: ThreadsafeFunction<GlobalInputAction>,
    ) -> Result<Function<()>> {
        // 1. Get a new unique ID.
        // 2. Store the `subscriber` (ThreadsafeFunction) in the shared state (HashMap).
        // 3. Return a new JS function that, when called, removes the subscriber from the HashMap using its ID.
        todo!()
    }

    /// Closes the listener and shuts down the background threads gracefully.
    #[napi]
    pub fn close(&mut self) -> Result<()> {
        // 1. Take ownership of the `action_tx` sender.
        // 2. Drop the sender. This closes the channel, causing the `recv()` calls in the
        //    dispatcher and OS listener threads to return an `Err`, gracefully exiting their loops.
        // 3. Set the thread handles to `None`.
        todo!()
    }
}

