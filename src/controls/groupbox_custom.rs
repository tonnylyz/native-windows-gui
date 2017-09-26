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

    fn build(&self, ui: &Ui<ID>) -> Result<Box<Control>, Error> {
        unsafe{
            // Build the window handle
            if let Err(e) = build_sysclass() { return Err(e); }
            match build_window(ui, &self) {
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
    use std::mem;
    use winapi::{UINT, WM_CREATE, WM_PAINT, PAINTSTRUCT, RECT, COLOR_WINDOW};
    use user32::{DefWindowProcW, DrawEdge, BeginPaint, EndPaint, FillRect, GetClientRect};
    
    let mut ps: PAINTSTRUCT = mem::uninitialized();
    const EDGE_RAISED: UINT = 0x0002 | 0x0004;
    const BF_RECT: UINT = 0x0001 | 0x0002 | 0x0004 | 0x0008 | 0x4000;

    let handled = match msg {
        WM_CREATE => true,
        WM_PAINT => {
            let hdc = BeginPaint(hwnd, &mut ps); 
            let mut rect: RECT = mem::zeroed();
            GetClientRect(hwnd, &mut rect);
            FillRect(hdc, &ps.rcPaint, mem::transmute(COLOR_WINDOW as usize));
            DrawEdge(hdc, &mut rect, EDGE_RAISED, BF_RECT);
            EndPaint(hwnd, &ps); 
            true
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
unsafe fn build_window<ID: Hash+Clone, S: Clone+Into<String>>(ui: &Ui<ID>, t: &GroupBoxT<ID, S>) -> Result<HWND, Error> {
    use low::window_helper::{WindowParams, build_window, handle_of_window};
    use winapi::{DWORD, WS_VISIBLE, WS_DISABLED, WS_CHILD};

    let flags: DWORD = WS_CHILD | 
    if t.visible    { WS_VISIBLE }   else { 0 } |
    if t.disabled   { WS_DISABLED }  else { 0 };

    // Get the parent handle
    let parent = match handle_of_window(ui, &t.parent, "The parent of a groupbox must be a window-like control.") {
        Ok(h) => h,
        Err(e) => { return Err(e); }
    };

    let params = WindowParams {
        title: "",
        class_name: WINDOW_CLASS_NAME,
        position: t.position.clone(),
        size: t.size.clone(),
        flags: flags,
        ex_flags: None,
        parent: parent
    };

    match build_window(params) {
        Ok(h) => {
            add_label_children(ui, t, h)?;
            Ok(h)
        },
        Err(e) => Err(Error::System(e))
    }
}

#[inline(always)]
unsafe fn add_label_children<ID: Hash+Clone, S: Clone+Into<String>>(ui: &Ui<ID>, t: &GroupBoxT<ID, S>, handle: HWND) -> Result<(), Error> {
    use low::window_helper::{WindowParams, build_window, handle_of_font, set_window_font_raw};
    use low::defs::{SS_NOTIFY, SS_NOPREFIX, SS_CENTER};
    use winapi::{WS_CHILD, WS_VISIBLE, HFONT};

     let params = WindowParams {
        title: t.text.clone().into(),
        class_name: "STATIC",
        position: (0,0),
        size: (0, 0),
        flags: WS_CHILD | WS_VISIBLE | SS_NOTIFY | SS_NOPREFIX | SS_CENTER,
        ex_flags: Some(0),
        parent: handle
    };

    let font_handle: Option<HFONT> = match t.font.as_ref() {
        Some(font_id) => 
            match handle_of_font(ui, &font_id, "The font of a label must be a font resource.") {
                Ok(h) => Some(h),
                Err(e) => { return Err(e); }
            },
        None => None
    };

    match build_window(params) {
        Ok(h) => {
            set_window_font_raw(h, font_handle, true);
            Ok(())
        },
        Err(e) => Err(Error::System(e))
    }
}
