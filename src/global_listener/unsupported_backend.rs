#![cfg(not(target_os = "windows"))]

use crate::global_listener::GlobalInputAction;
use std::sync::mpsc::{Sender, SyncSender};

// On non-Windows platforms, the listener is not supported.
pub fn start_listener(_tx: Sender<GlobalInputAction>, init_tx: SyncSender<Result<(), &'static str>>) {
    // Signal that this platform is unsupported.
    let _ = init_tx.send(Err("Global listener is not supported on this platform."));
}
