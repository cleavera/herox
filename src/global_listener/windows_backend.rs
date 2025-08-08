#![cfg(target_os = "windows")]

use crate::global_listener::{GlobalInputAction, GlobalInputActionType};
use crate::keyboard::{unicode, SpecialKey, UnicodeKey};
use once_cell::sync::OnceCell;
use std::sync::mpsc::{Sender, SyncSender};
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
  GetKeyboardState, MapVirtualKeyW, ToUnicode, MAP_VIRTUAL_KEY_TYPE,
};
use windows::Win32::UI::WindowsAndMessaging::{
  CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP
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
    let kbd_ll_hook_struct = unsafe { *(l_param.0 as *const KBDLLHOOKSTRUCT) };

    if w_param_u == WM_KEYDOWN || w_param_u == WM_SYSKEYDOWN {
      handle_keydown(kbd_ll_hook_struct.vkCode);
    }

    if w_param_u == WM_KEYUP || w_param_u == WM_SYSKEYUP {
      handle_keyup(kbd_ll_hook_struct.vkCode);
    }
  }

  unsafe { CallNextHookEx(HHOOK(std::ptr::null_mut()), n_code, w_param, l_param) }
}

pub fn handle_keydown(key_code: u32) {
  if let Some(tx) = ACTION_TX.get() {
    let _ = tx.send(GlobalInputAction::KeyDown {
      value: key_code.into(),
    });
  }
}

pub fn handle_keyup(key_code: u32) {
  if let Some(tx) = ACTION_TX.get() {
    let _ = tx.send(GlobalInputAction::KeyUp {
      value: key_code.into(),
    });
  }
}

impl From<u32> for GlobalInputActionType {
  fn from(value: u32) -> Self {
    if let Ok(result) = SpecialKey::try_from(value) {
      return GlobalInputActionType::SpecialKey { key: result };
    }

    if let Ok(result) = UnicodeKey::try_from(value) {
      return GlobalInputActionType::UnicodeKey { key: result };
    }

    GlobalInputActionType::Raw { keycode: value }
  }
}

impl TryFrom<u32> for SpecialKey {
  type Error = ();

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    match value {
      0x08 => Ok(SpecialKey::Backspace),
      0x09 => Ok(SpecialKey::Tab),
      0x10 => Ok(SpecialKey::Shift),
      0x11 => Ok(SpecialKey::Control),
      0x12 => Ok(SpecialKey::Alt),
      0x13 => Ok(SpecialKey::Pause),
      0x1B => Ok(SpecialKey::Escape),
      0x20 => Ok(SpecialKey::Space),
      0x21 => Ok(SpecialKey::PageUp),
      0x22 => Ok(SpecialKey::PageDown),
      0x23 => Ok(SpecialKey::End),
      0x24 => Ok(SpecialKey::Home),
      0x2D => Ok(SpecialKey::Insert),
      0x2E => Ok(SpecialKey::Delete),
      0xA0 => Ok(SpecialKey::LShift),
      0xA1 => Ok(SpecialKey::RShift),
      0xA2 => Ok(SpecialKey::LControl),
      0xA3 => Ok(SpecialKey::RControl),
      _ => Err(()),
    }
  }
}

impl TryFrom<u32> for UnicodeKey {
  type Error = ();

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    let mut keyboard_state: [u8; 256] = [0; 256];
    unsafe { GetKeyboardState(&mut keyboard_state).unwrap() };
    let mut buffer: [u16; 2] = [0; 2];
    let scan_code = unsafe { MapVirtualKeyW(value, MAP_VIRTUAL_KEY_TYPE(0)) };

    let chars_copied =
      unsafe { ToUnicode(value, scan_code, Some(&keyboard_state), &mut buffer, 0) };
    if chars_copied == 0 {
      return Err(());
    }

    Ok(unicode(char::from_u32(buffer[0] as u32).ok_or(())?.into()))
  }
}
