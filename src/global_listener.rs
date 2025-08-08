use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use std::collections::HashMap;
use std::sync::mpsc::{channel, sync_channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::keyboard::{SpecialKey, UnicodeKey};

pub mod unsupported_backend;
pub mod windows_backend;

#[napi]
#[derive(Clone, Debug)]
pub enum GlobalInputAction {
    KeyUp { value: GlobalInputActionType },
    KeyDown { value: GlobalInputActionType },
}

#[napi]
#[derive(Clone, Debug)]
pub enum GlobalInputActionType {
    Raw{ keycode: u32 },
    UnicodeKey{ key: UnicodeKey },
    SpecialKey{ key: SpecialKey },
}

type Subscriber = ThreadsafeFunction<GlobalInputAction>;
type SubscriberId = u64;

#[derive(Clone, Default)]
struct ListenerState {
  subscribers: Arc<Mutex<HashMap<SubscriberId, Subscriber>>>,
  next_id: Arc<Mutex<SubscriberId>>,
}

impl ListenerState {
  fn add_subscriber(&self, subscriber: Subscriber) -> SubscriberId {
    let mut next_id_guard = self.next_id.lock().unwrap();
    let id = *next_id_guard;
    *next_id_guard += 1;

    let mut subs_guard = self.subscribers.lock().unwrap();
    subs_guard.insert(id, subscriber);
    id
  }

  fn remove_subscriber(&self, id: SubscriberId) {
    let mut subs_guard = self.subscribers.lock().unwrap();
    subs_guard.remove(&id);
  }

  fn broadcast(&self, action: GlobalInputAction) {
    let subs_guard = self.subscribers.lock().unwrap();
    for sub in subs_guard.values() {
      sub.call(Ok(action.clone().into()), ThreadsafeFunctionCallMode::Blocking);
    }
  }
}

#[napi]
pub struct GlobalListener {
  state: ListenerState,
  action_tx: Option<Sender<GlobalInputAction>>,
  _os_listener_handle: Option<JoinHandle<()>>,
  _dispatcher_handle: Option<JoinHandle<()>>,
}

#[napi]
impl GlobalListener {
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    let state = ListenerState::default();
    let (action_tx, action_rx) = channel::<GlobalInputAction>();

    let dispatcher_state = state.clone();
    let _dispatcher_handle = Some(thread::spawn(move || {
      while let Ok(action) = action_rx.recv() {
        dispatcher_state.broadcast(action);
      }
    }));

    let os_listener_tx = action_tx.clone();
    let (init_tx, init_rx) = sync_channel(1);

    let _os_listener_handle = Some(thread::spawn(move || {
      #[cfg(target_os = "windows")]
      windows_backend::start_listener(os_listener_tx, init_tx);

      #[cfg(not(target_os = "windows"))]
      unsupported_backend::start_listener(os_listener_tx, init_tx);
    }));

    match init_rx.recv() {
      Ok(Ok(())) => Ok(Self {
        state,
        action_tx: Some(action_tx),
        _os_listener_handle,
        _dispatcher_handle,
      }),
      Ok(Err(err_msg)) => Err(Error::from_reason(err_msg)),
      Err(_) => Err(Error::from_reason(
        "The global listener thread panicked during initialization.",
      )),
    }
  }

  #[napi]
  pub fn subscribe<'a>(
    &'a self,
    env: &'a Env,
    subscriber: ThreadsafeFunction<GlobalInputAction,>,
  ) -> Result<Function<'a, (), ()>> {
    let id = self.state.add_subscriber(subscriber);
    let state_clone = self.state.clone();

    env.create_function_from_closure("unsubscribe", move |_ctx| {
      state_clone.remove_subscriber(id);
      Ok(())
    })
  }

  #[napi]
  pub fn close(&mut self) -> Result<()> {
    if let Some(tx) = self.action_tx.take() {
      drop(tx);
    }
    self._os_listener_handle = None;
    self._dispatcher_handle = None;
    Ok(())
  }
}

impl Drop for GlobalListener {
  fn drop(&mut self) {
    let _ = self.close();
  }
}
