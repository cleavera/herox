#![cfg(target_os = "windows")]

use crate::global_listener::GlobalInputAction;
use once_cell::sync::OnceCell;
use std::sync::mpsc::{Sender, SyncSender};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
  CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK,
  KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

static ACTION_TX: OnceCell<Sender<GlobalInputAction>> = OnceCell::new();

pub fn start_listener(
  tx: Sender<GlobalInputAction>,
  init_tx: SyncSender<Result<(), &'static str>>,
) {
  if ACTION_TX.set(tx).is_err() {
    let _ = init_tx.send(Err("A GlobalListener is already active for this process."));
    return;
  }

  let hook_handle =
    unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), None, 0) };

  if hook_handle.is_err() {
    let _ = init_tx.send(Err("Failed to set the low-level keyboard hook."));
    return;
  }

  if init_tx.send(Ok(())).is_err() {
    unsafe {
      let _ = UnhookWindowsHookEx(hook_handle.unwrap());
    }
    return;
  }

  let mut msg = MSG::default();
  while unsafe { GetMessageW(&mut msg, None, 0, 0) }.as_bool() {
    unsafe { DispatchMessageW(&msg) };
  }

  unsafe {
    let _ = UnhookWindowsHookEx(hook_handle.unwrap());
  }
}

extern "system" fn low_level_keyboard_proc(
  n_code: i32,
  w_param: WPARAM,
  l_param: LPARAM,
) -> LRESULT {
  if n_code >= 0 {
    let w_param_u = w_param.0 as u32;
    if w_param_u == WM_KEYDOWN || w_param_u == WM_SYSKEYDOWN {
      if let Some(tx) = ACTION_TX.get() {
        let _ = tx.send(GlobalInputAction::keyboard);
      }
    }
  }

  unsafe { CallNextHookEx(HHOOK(0), n_code, w_param, l_param) }
}
