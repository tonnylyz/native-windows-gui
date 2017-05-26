/*!
    Low level events functions
*/
/*
    Copyright (C) 2016  Gabriel Dubé

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use std::mem;
use std::ptr;
use std::hash::Hash;
use std::any::TypeId;

use winapi::{HWND, HMENU, UINT, WPARAM, LPARAM, UINT_PTR, DWORD_PTR, LRESULT, DWORD, WORD};
use winapi::{WM_MOVE, WM_SIZING, WM_SIZE, WM_EXITSIZEMOVE, WM_PAINT, WM_UNICHAR, WM_CHAR,
  WM_CLOSE, WM_LBUTTONUP, WM_RBUTTONUP, WM_MBUTTONUP, WM_LBUTTONDOWN, WM_RBUTTONDOWN,
  WM_MBUTTONDOWN, WM_KEYDOWN, WM_KEYUP, BN_CLICKED, BN_DBLCLK, BN_SETFOCUS, BN_KILLFOCUS};

use ui::UiInner;
use events::{Event, EventArgs};
use controls::{ControlType, AnyHandle, Timer};
use low::defs::{NWG_DESTROY, CBN_SELCHANGE, CBN_KILLFOCUS, CBN_SETFOCUS};

/// A magic number to identify the NWG subclass that dispatches events
const EVENTS_DISPATCH_ID: UINT_PTR = 2465;


// Definition of common system events
pub const Destroyed: Event = Event::System(NWG_DESTROY, &system_event_unpack_no_args);
pub const Paint: Event = Event::System(WM_PAINT, &system_event_unpack_no_args);
pub const Close: Event = Event::System(WM_CLOSE, &system_event_unpack_no_args);
pub const Moved: Event = Event::System(WM_MOVE, &unpack_move);
pub const KeyDown: Event = Event::System(WM_KEYDOWN, &unpack_key);
pub const KeyUp: Event = Event::System(WM_KEYUP, &unpack_key);
pub const Resized: Event = Event::SystemGroup(&[WM_SIZING, WM_SIZE, WM_EXITSIZEMOVE], &unpack_size);
pub const Char: Event = Event::SystemGroup(&[WM_UNICHAR, WM_CHAR], &unpack_char);
pub const MouseUp: Event = Event::SystemGroup(&[WM_LBUTTONUP, WM_RBUTTONUP, WM_MBUTTONUP], &unpack_mouseclick);
pub const MouseDown: Event = Event::SystemGroup(&[WM_LBUTTONDOWN, WM_RBUTTONDOWN, WM_MBUTTONDOWN], &unpack_mouseclick);

// Button events
pub const BtnClick: Event = Event::Command(BN_CLICKED, &command_event_unpack_no_args);
pub const BtnDoubleClick: Event = Event::Command(BN_DBLCLK, &command_event_unpack_no_args);
pub const BtnFocus: Event = Event::CommandGroup(&[BN_SETFOCUS, BN_KILLFOCUS], &unpack_btn_focus);

// Combobox events
pub const CbnFocus: Event = Event::CommandGroup(&[CBN_SETFOCUS, CBN_KILLFOCUS], &unpack_cbn_focus);
pub const CbnSelectionChanged: Event = Event::Command(CBN_SELCHANGE, &unpack_cbn_sel_change);

// Event unpackers for events that have no arguments
pub fn system_event_unpack_no_args(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> Option<EventArgs> { Some(EventArgs::None) }
pub fn command_event_unpack_no_args(hwnd: HWND, ncode: WORD) -> Option<EventArgs> { Some(EventArgs::None) }
pub fn notify_event_unpack_no_args(hwnd: HWND) -> EventArgs { EventArgs::None }

// Event unpackers for the events defined above
fn unpack_move(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> Option<EventArgs> {
    use winapi::{LOWORD, HIWORD};
    
    let (x, y) = (LOWORD(l as u32), HIWORD(l as u32));
    Some(EventArgs::Position(x as i32, y as i32))
}

fn unpack_size(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> Option<EventArgs> {
    use winapi::RECT;
    use user32::GetClientRect;

    let mut r: RECT = mem::uninitialized();

    GetClientRect(hwnd, &mut r);
    let w: u32 = (r.right-r.left) as u32;
    let h: u32 = (r.bottom-r.top) as u32;

    Some(EventArgs::Size(w, h))
}

fn unpack_char(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> Option<EventArgs> {
    use winapi::UNICODE_NOCHAR;

    if w == UNICODE_NOCHAR { 
      return None; 
    } 

    if let Some(c) = ::std::char::from_u32(w as u32) {
      Some( EventArgs::Char( c ) )
    } else {
      None
    }
}

fn unpack_mouseclick(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> Option<EventArgs> {
  use defs::MouseButton;
  use winapi::{GET_X_LPARAM, GET_Y_LPARAM};

  let btn = match msg {
    WM_LBUTTONUP | WM_LBUTTONDOWN => MouseButton::Left,
    WM_RBUTTONUP | WM_RBUTTONDOWN => MouseButton::Right,
    WM_MBUTTONUP | WM_MBUTTONDOWN => MouseButton::Middle,
    _ => MouseButton::Left
  };

  let x = GET_X_LPARAM(l) as i32; 
  let y = GET_Y_LPARAM(l) as i32;

  Some(EventArgs::MouseClick{btn: btn, pos: (x, y)})
}

fn unpack_key(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> Option<EventArgs> {
   Some(EventArgs::Key(w as u32))
}

fn unpack_btn_focus(hwnd: HWND, ncode: WORD) -> Option<EventArgs> {
    Some(EventArgs::Focus(ncode==BN_SETFOCUS))
}

fn unpack_cbn_focus(hwnd: HWND, ncode: WORD) -> Option<EventArgs> {
    Some(EventArgs::Focus(ncode==CBN_SETFOCUS))
}

fn unpack_cbn_sel_change(hwnd: HWND, ncode: WORD) -> Option<EventArgs> {
  unimplemented!()
}



// WARNING! This WHOLE section (from parse_listbox_command to parse_command) will be replaced with the events overhaul in NWG BETA2

fn parse_listbox_command(id: u64, ncode: u32) -> Option<(u64, Event, EventArgs)> {
  use low::defs::{LBN_SELCHANGE, LBN_DBLCLK, LBN_SETFOCUS, LBN_KILLFOCUS};

  match ncode {
    LBN_SELCHANGE => Some((id, Event::SelectionChanged, EventArgs::None)),
    LBN_DBLCLK => Some((id, Event::DoubleClick, EventArgs::None)),
    LBN_SETFOCUS | LBN_KILLFOCUS => Some((id, Event::Focus, EventArgs::Focus(ncode==LBN_SETFOCUS))),
    _ => None
  }
}

fn parse_edit_command(id: u64, ncode: u32) -> Option<(u64, Event, EventArgs)> {
  use low::defs::{EN_SETFOCUS, EN_KILLFOCUS, EN_UPDATE, EN_MAXTEXT};
  match ncode {
    EN_UPDATE => Some((id, Event::ValueChanged, EventArgs::None)),
    EN_MAXTEXT => Some((id, Event::LimitReached, EventArgs::None)),
    EN_SETFOCUS | EN_KILLFOCUS => Some((id, Event::Focus, EventArgs::Focus(ncode==EN_SETFOCUS))),
    _ => None
  }
}

fn parse_static_command(id: u64, ncode: u32) -> Option<(u64, Event, EventArgs)> {
  use low::defs::{STN_CLICKED, STN_DBLCLK};
  match ncode {
    STN_CLICKED => Some((id, Event::Click, EventArgs::None)),
    STN_DBLCLK => Some((id, Event::DoubleClick, EventArgs::None)),
    _ => None
  }
}

fn parse_datepicker_command(id: u64, ncode: u32) -> Option<(u64, Event, EventArgs)> {
  use winapi::DTN_CLOSEUP;
  match ncode {
    DTN_CLOSEUP => {  // DTN_DATETIMECHANGE is sent twice so instead we catch DTN_CLOSEUP ¯\_(ツ)_/¯
      Some((id, Event::DateChanged, EventArgs::None))
    },
    _ => None
  }
}

/**
  Parse the common controls notification passed through the `WM_COMMAND` message.
*/
#[inline(always)]
fn parse_notify(id: u64, control_type: ControlType, w: WPARAM) -> Option<(u64, Event, EventArgs)> {
  match control_type {
    ControlType::DatePicker => parse_datepicker_command(id, w as u32),
    _ => None
  }
}

/**
  Parse the common controls notification passed through the `WM_COMMAND` message.
*/
#[inline(always)]
fn parse_command(id: u64, control_type: ControlType, w: WPARAM) -> Option<(u64, Event, EventArgs)> {
  use winapi::HIWORD;

  let ncode = HIWORD(w as DWORD) as u32;
  match control_type {
    ControlType::ListBox => parse_listbox_command(id, ncode),
    ControlType::TextInput | ControlType::TextBox => parse_edit_command(id, ncode),
    ControlType::Label => parse_static_command(id, ncode),
    ControlType::DatePicker => parse_datepicker_command(id, ncode),
    _ => None
  }
}

/**
  Proc that dispatches the NWG events
*/
#[allow(unused_variables)]
unsafe extern "system" fn process_events<ID: Hash+Clone+'static>(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM, id: UINT_PTR, data: DWORD_PTR) -> LRESULT {
  use comctl32::DefSubclassProc;
  use user32::GetClientRect;
  use winapi::{WM_KEYDOWN, WM_KEYUP, WM_UNICHAR, WM_CHAR, UNICODE_NOCHAR, WM_MENUCOMMAND, WM_CLOSE, WM_LBUTTONUP, WM_LBUTTONDOWN, 
    WM_RBUTTONUP, WM_RBUTTONDOWN, WM_MBUTTONUP, WM_MBUTTONDOWN, WM_COMMAND, WM_TIMER, WM_MOVE, WM_SIZING, WM_EXITSIZEMOVE, WM_SIZE,
    WM_PAINT, WM_NOTIFY, c_int, LOWORD, HIWORD, RECT, NMHDR};
  use low::menu_helper::get_menu_id;
  use low::defs::{NWG_CUSTOM_MIN, NWG_CUSTOM_MAX};

  let inner: &mut UiInner<ID> = mem::transmute(data);
  let inner_id: u64;

  let callback_data = match msg {
    WM_COMMAND => {
      if l == 0 { 
        None 
      } else {
        // Somehow, WM_COMMAND messages get sent while freeing and so inner_id_from_handle can fail...
        let nhandle: HWND = mem::transmute(l);
        if let Some(id) = inner.inner_id_from_handle( &AnyHandle::HWND(nhandle) ) {
          let control_type = (&mut *inner.controls.get(&id).expect("Could not find a control with with the specified type ID").as_ptr()).control_type();
          parse_command(id, control_type, w)
        } else {
          None
        }
      }
    },
    WM_NOTIFY => {
      // WM_NOTIFY is the new WM_COMMAND for the new windows controls
      let nmdr: &NMHDR = mem::transmute(l);
      if let Some(id) = inner.inner_id_from_handle( &AnyHandle::HWND(nmdr.hwndFrom) ) {
        let control_type = (&mut *inner.controls.get(&id).expect("Could not find a control with with the specified type ID").as_ptr()).control_type();
        parse_notify(id, control_type, nmdr.code as WPARAM)
      } else {
        None
      }
    },
    WM_MENUCOMMAND => {
      let parent_menu: HMENU = mem::transmute(l);
      let handle = AnyHandle::HMENU_ITEM(parent_menu, get_menu_id(parent_menu, w as c_int));

      // Custom controls might have their own way to handle the message
      if let Some(inner_id) = inner.inner_id_from_handle( &handle ) { 
        Some( (inner_id, Event::Triggered, EventArgs::None) )
      } else {
        None
      }  
    },
    WM_TIMER => {
      let handle = AnyHandle::Custom(TypeId::of::<Timer>(), w as usize);

      // Here I assume WM_TIMER will only be sent by built-in timers. Using a user event might be a better idea.
      // Custom controls might have their own way to handle the message
      if let Some(inner_id) = inner.inner_id_from_handle( &handle ) {
        let timer: &mut Box<Timer> = mem::transmute( inner.controls.get(&inner_id).unwrap().as_ptr() );
        Some( (inner_id, Event::Tick, EventArgs::Tick(timer.elapsed())) )
      } else {
        None
      }
    }
    _ => { None }
  };

  if let Some((inner_id, evt, params)) = callback_data {
    inner.trigger(inner_id, evt, params);
  }

  // Trigger a raw event 
  if msg < NWG_CUSTOM_MIN || msg > NWG_CUSTOM_MAX {
    if let Some(inner_id) = inner.inner_id_from_handle( &AnyHandle::HWND(hwnd) ) {
      inner.trigger(inner_id, Event::Any, EventArgs::Raw(msg, w as usize, l as usize));
    }
  }

  DefSubclassProc(hwnd, msg, w, l)
}

/**
    Add a subclass that dispatches the system event to the application callbacks to a window control.
*/
pub fn hook_window_events<ID: Hash+Clone+'static>(uiinner: &mut UiInner<ID>, handle: HWND) { unsafe {
  use comctl32::SetWindowSubclass;

  // While definitely questionable in term of safety, the reference to the UiInner is actually (always)
  // a raw pointer belonging to a Ui. Also, when the Ui goes out of scope, every window control
  // gets destroyed BEFORE the UiInner, this guarantees that uinner lives long enough.
  let ui_inner_raw: *mut UiInner<ID> = uiinner as *mut UiInner<ID>;
  SetWindowSubclass(handle, Some(process_events::<ID>), EVENTS_DISPATCH_ID, mem::transmute(ui_inner_raw));
}}

/**
  Remove a subclass and free the associated data
*/
pub fn unhook_window_events<ID: Hash+Clone+'static>(handle: HWND) { unsafe {
  use comctl32::{RemoveWindowSubclass, GetWindowSubclass};
  use winapi::{TRUE, DWORD_PTR};

  let mut data: DWORD_PTR = 0;
  if GetWindowSubclass(handle, Some(process_events::<ID>), EVENTS_DISPATCH_ID, &mut data) == TRUE {
    RemoveWindowSubclass(handle, Some(process_events::<ID>), EVENTS_DISPATCH_ID);
  }
}}

/**
  Check if a window is hooked by nwg. If it is, return its ID, if not return None
*/
pub unsafe fn window_id<ID: Clone+Hash>(handle: HWND, inner_ref: *mut UiInner<ID>) -> Option<u64> {
  use comctl32::GetWindowSubclass;
  use winapi::{TRUE, DWORD_PTR};

  let mut data: DWORD_PTR = 0;
  if GetWindowSubclass(handle, Some(process_events::<ID>), EVENTS_DISPATCH_ID, &mut data) == TRUE {
    let data: *mut UiInner<ID> = mem::transmute(data);
    if data == inner_ref {
      (&*data).inner_id_from_handle( &AnyHandle::HWND(handle) )
    } else {
      None
    }
  } else {
    None
  }
}

/**
    Dispatch the messages waiting the the system message queue to the associated Uis. This includes NWG custom messages.

    Return once a quit event was received.
*/
#[inline(always)]
pub unsafe fn dispatch_events() {
  use winapi::MSG;
  use user32::{GetMessageW, TranslateMessage, DispatchMessageW};

  let mut msg: MSG = mem::uninitialized();
  while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
      TranslateMessage(&msg); 
      DispatchMessageW(&msg); 
      // TODO dispatch events sent from other thread / other processes ( after first stable release )
  }
}

/**
    Send a WM_QUIT to the system queue. Breaks the dispatch_events loop.
*/
#[inline(always)]
pub unsafe fn exit() {
  use user32::PostMessageW;
  use winapi::WM_QUIT;

  PostMessageW(ptr::null_mut(), WM_QUIT, 0, 0);
}