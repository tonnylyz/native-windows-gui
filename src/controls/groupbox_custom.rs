/*!
    Groupbox control definition
*/

use std::any::TypeId;
use std::hash::Hash;

use winapi::{HWND, UINT, WPARAM, LPARAM, LRESULT};

use ui::Ui;
use controls::{Control, ControlT, ControlType, AnyHandle};
use defs::HTextAlign;
use error::Error;

/// System class identifier
const WINDOW_CLASS_NAME: &'static str = "NWG_BUILTIN_GROUPBOX";

/**
    A template that creates a standard groupbox

    Events: None  

    Members:  
    • `text`: The text of the groupbox  
    • `position`: The start position of groupbox  
    • `size`: The start size of the groupbox  
    • `visible`: If the groupbox should be visible to the user   
    • `disabled`: If the user can or can't click on the groupbox  
    • `parent`: The groupbox parent  
    • `font`: The groupbox font. If None, use the system default  
*/
#[derive(Clone)]
pub struct GroupBoxT<ID: Hash+Clone, S: Clone+Into<String>> {
    pub text: S,
    pub position: (i32, i32),
    pub size: (u32, u32),
    pub visible: bool,
    pub disabled: bool,
    pub align: HTextAlign,
    pub parent: ID,
    pub font: Option<ID>,
}

impl<S: Clone+Into<String>, ID: Hash+Clone> ControlT<ID> for GroupBoxT<ID, S> {
    fn type_id(&self) -> TypeId { TypeId::of::<GroupBox>() }

    fn build(&self, _: &Ui<ID>) -> Result<Box<Control>, Error> {
        unsafe{
            // Build the window handle
            if let Err(e) = build_sysclass() { return Err(e); }
            match build_window(&self) {
                Ok(h) => { 
                    Ok( Box::new(GroupBox{handle: h}) as Box<Control> ) 
                },
                Err(e) => Err(e)
            }
        } // unsafe
    }
}
/**
    A groupbox
*/
pub struct GroupBox {
    handle: HWND
}

impl GroupBox {
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
    pub fn get_font<ID: Hash+Clone>(&self, ui: &Ui<ID>) -> Option<ID> { unsafe{ ::low::window_helper::get_window_font(self.handle, ui) } }
    pub fn set_font<ID: Hash+Clone>(&self, ui: &Ui<ID>, f: Option<&ID>) -> Result<(), Error> { unsafe{ ::low::window_helper::set_window_font(self.handle, ui, f) } }
    pub fn update(&self) { unsafe{ ::low::window_helper::update(self.handle); } }
    pub fn focus(&self) { unsafe{ ::user32::SetFocus(self.handle); } }
}

impl Control for GroupBox {

    fn handle(&self) -> AnyHandle {
        AnyHandle::HWND(self.handle)
    }

    fn control_type(&self) -> ControlType { 
        ControlType::GroupBox 
    }

    fn children(&self) -> Vec<AnyHandle> {
        use low::window_helper::list_window_children;
        unsafe{ list_window_children(self.handle) }
    }

    fn free(&mut self) {
        use user32::DestroyWindow;
        unsafe{ DestroyWindow(self.handle) };
    }

}


/*
    Private unsafe control methods
*/

#[allow(unused_variables)]
unsafe extern "system" fn window_sysproc(hwnd: HWND, msg: UINT, w: WPARAM, l: LPARAM) -> LRESULT {
    use winapi::{WM_CREATE, WM_PAINT};
    use user32::{DefWindowProcW};

    let handled = match msg {
        WM_CREATE => true,
        WM_PAINT => {
            false
        },
        _ => false
    };

    if handled {
        0
    } else {
        DefWindowProcW(hwnd, msg, w, l)
    }
}

#[inline(always)]
unsafe fn build_sysclass() -> Result<(), Error> {
    use low::window_helper::{SysclassParams, build_sysclass};
    let params = SysclassParams { 
        class_name: WINDOW_CLASS_NAME,
        sysproc: Some(window_sysproc),
        background: None, style: None
    };
    
    if let Err(e) = build_sysclass(params) {
        Err(Error::System(e))
    } else {
        Ok(())
    }
}

#[inline(always)]
unsafe fn build_window<ID: Hash+Clone, S: Clone+Into<String>>(t: &GroupBoxT<ID, S>) -> Result<HWND, Error> {
    use low::window_helper::{WindowParams, build_window};
    use winapi::{DWORD, WS_VISIBLE, WS_DISABLED};

    let flags: DWORD = 
    if t.visible    { WS_VISIBLE }   else { 0 } |
    if t.disabled   { WS_DISABLED }  else { 0 };

    let params = WindowParams {
        title: "",
        class_name: WINDOW_CLASS_NAME,
        position: t.position.clone(),
        size: t.size.clone(),
        flags: flags,
        ex_flags: None,
        parent: ::std::ptr::null_mut()
    };

    match build_window(params) {
        Ok(h) => Ok(h),
        Err(e) => Err(Error::System(e))
    }
}