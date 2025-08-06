#![cfg(not(target_os = "windows"))]

use crate::global_listener::GlobalInputAction;
use std::sync::mpsc::{Sender, SyncSender};

pub fn start_listener(
  _tx: Sender<GlobalInputAction>,
  init_tx: SyncSender<Result<(), &'static str>>,
) {
  let _ = init_tx.send(Err("Global listener is not supported on this platform."));
}
