/*!
    Button control definition
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

use std::hash::Hash;
use std::any::TypeId;

use winapi::{HWND, HFONT};

use ui::Ui;
use controls::{Control, ControlT, ControlType, AnyHandle};
use error::Error;
use events::Event;
use std::ffi::OsStr;

/**
    A template that creates a standard button

    Available events:
    Event::Destroyed, Event::Click, Event::DoubleClick, Event::Focus, Event::Moved, Event::Resized, Event::Raw

    Members:
    • `text`: The text of the button
    • `position`: The start position of the button
    • `size`: The start size of the button
    • `visible`: If the button should be visible to the user
    • `disabled`: If the user can or can't click on the button
    • `parent`: The button parent
    • `font`: The button font. If None, use the system default
*/
#[derive(Clone)]
pub struct ButtonT<S: Clone+Into<String>, ID: Hash+Clone> {
    pub text: S,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub disabled: bool,
    pub parent: ID,
    pub font: Option<ID>,
}

impl<S: Clone+Into<String>, ID: Hash+Clone> ControlT<ID> for ButtonT<S, ID> {
    fn resource_type_id(&self) -> TypeId { TypeId::of::<Button>() }

    fn events(&self) -> Vec<Event> {
        vec![Event::Destroyed, Event::Click, Event::DoubleClick, Event::Focus, Event::Moved, Event::Resized, Event::Raw]
    }

    fn build(&self, ui: &Ui<ID>) -> Result<Box<Control>, Error> {
        use low::window_helper::{WindowParams, build_window, set_window_font, handle_of_window, handle_of_font};
        use winapi::{DWORD, WS_VISIBLE, WS_DISABLED, WS_CHILD, BS_NOTIFY, BS_TEXT, BS_BITMAP, LR_DEFAULTCOLOR, LR_DEFAULTSIZE, LR_LOADFROMFILE, IMAGE_BITMAP};

        let flags: DWORD = WS_CHILD | BS_NOTIFY | BS_BITMAP |
        if self.visible    { WS_VISIBLE }   else { 0 } |
        if self.disabled   { WS_DISABLED }  else { 0 };

        // Get the parent handle
        let parent = match handle_of_window(ui, &self.parent, "The parent of a button must be a window-like control.") {
            Ok(h) => h,
            Err(e) => { return Err(e); }
        };

        // Get the font handle (if any)
        let font_handle: Option<HFONT> = match self.font.as_ref() {
            Some(font_id) =>
                match handle_of_font(ui, &font_id, "The font of a button must be a font resource.") {
                    Ok(h) => Some(h),
                    Err(e) => { return Err(e); }
                },
            None => None
        };

        let params = WindowParams {
            title: self.text.clone().into(),
            class_name: "BUTTON",
            position: self.position.clone(),
            size: self.size.clone(),
            flags: flags,
            ex_flags: Some(0),
            parent: parent
        };

        match unsafe{ build_window(params) } {
            Ok(h) => {
                unsafe {
                    use std::os::windows::ffi::OsStrExt;
                    use user32::{LoadImageW, SendMessageW};
                    use kernel32::{GetLastError,GetModuleHandleW};
                    let handle_img = LoadImageW(
                        GetModuleHandleW(std::ptr::null()),
                        OsStr::new(&self.text.clone().into()).encode_wide().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
                        IMAGE_BITMAP,
                        0,
                        0,
                        LR_DEFAULTCOLOR | LR_DEFAULTSIZE);
                    SendMessageW(h, 247/*BM_SETIMAGE*/, IMAGE_BITMAP as u64, handle_img as i64);
                }
                unsafe{ set_window_font(h, font_handle, true); }
                Ok( Box::new(Button{handle: h}) )
            },
            Err(e) => Err(Error::System(e))
        }
    }
}

/**
    A standard button
*/
pub struct Button {
    handle: HWND
}

impl Button {
    pub fn get_text(&self) -> String { unsafe{ ::low::window_helper::get_window_text(self.handle) } }
    pub fn set_text<'a>(&self, text: &'a str) { unsafe{ ::low::window_helper::set_window_text(self.handle, text); } }
    pub fn get_visibility(&self) -> bool { unsafe{ ::low::window_helper::get_window_visibility(self.handle) } }
    pub fn set_visibility(&self, visible: bool) { unsafe{ ::low::window_helper::set_window_visibility(self.handle, visible); }}
    pub fn get_position(&self) -> (i32, i32) { unsafe{ ::low::window_helper::get_window_position(self.handle) } }
    pub fn set_position(&self, x: i32, y: i32) { unsafe{ ::low::window_helper::set_window_position(self.handle, x, y); }}
    pub fn get_size(&self) -> (u32, u32) { unsafe{ ::low::window_helper::get_window_size(self.handle) } }
    pub fn set_size(&self, w: u32, h: u32) { unsafe{ ::low::window_helper::set_window_size(self.handle, w, h, false); } }
    pub fn get_enabled(&self) -> bool { unsafe{ ::low::window_helper::get_window_enabled(self.handle) } }
    pub fn set_enabled(&self, e:bool) { unsafe{ ::low::window_helper::set_window_enabled(self.handle, e); } }
}

impl Control for Button {

    fn handle(&self) -> AnyHandle {
        AnyHandle::HWND(self.handle)
    }

    fn control_type(&self) -> ControlType { 
        ControlType::Button 
    }

    fn free(&mut self) {
        use user32::DestroyWindow;
        unsafe{ DestroyWindow(self.handle) };
    }

}