/*!
    Errors and exceptions that can be raise by nwg
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

use std::fmt;

/**
    Error class that regroups errors generated by the system
*/
#[derive(Clone, PartialEq)]
pub enum SystemError {
    SystemClassCreation,
    WindowCreationFail,
    UiCreation,
    FontCreation,
    ImageCreation,
    TreeItemCreation,
    ComInstanceCreation(String),
    ComError(String),
}

impl SystemError {
    fn translate(&self) -> String {
        use low::other_helper::get_system_error;

        let (code, code_txt) = unsafe{ get_system_error() };
        let tr = match self {
            &SystemError::SystemClassCreation => format!("Failed to create a system class for a control"),
            &SystemError::WindowCreationFail => format!("Failed to create a system window for a control"),
            &SystemError::UiCreation => format!("The system could not initialize the Ui"),
            &SystemError::FontCreation => format!("Failed to create a system font"),
            &SystemError::ImageCreation => format!("Failed to create a system image"),
            &SystemError::TreeItemCreation => format!("Failed to create a tree view item"),
            &SystemError::ComInstanceCreation(ref name) => format!("Failed to create a COM instance for {}", name),
            &SystemError::ComError(ref details) => format!("An error ocurred while executing a COM method, {}", details),
        };

        format!("{}.\nID {:?} - {}", tr, code, code_txt)
    }
}

impl fmt::Debug for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.translate())
    }
}

/**
    Error class that regroup errors generated by NWG
*/
#[derive(Clone, PartialEq)]
pub enum Error {
    KeyExists,
    KeyNotFound,
    BadType,
    BadUi(String),
    BadParent(String),
    BadResource(String),
    BorrowError,
    ControlRequired,
    ControlOrResourceRequired,
    ControlInUse,
    ResourceInUse,
    Unimplemented,
    System(SystemError),
    UserError(String)
}

impl Error {
    fn translate(&self) -> String {

        match self {
            &Error::KeyExists => format!("The same key already exists in the UI"),
            &Error::KeyNotFound => format!("The key was not found in the ui"),
            &Error::BadUi(ref r) => format!("Ui error: {}", r),
            &Error::BadType => format!("The key exists in the Ui, but the type requested did not match the type of the underlying object"),
            &Error::BadParent(ref r) => format!("Could not make sense of the requested parent: {}", r),
            &Error::BadResource(ref r) => format!("Could not make sense of the requested resource: {}", r),
            &Error::BorrowError => format!("The Ui element was already borrowed"),
            &Error::ControlRequired => format!("The key passed to the command must identify a control"),
            &Error::ControlOrResourceRequired => format!("The key passed to the command must identify a control or a resource", ),
            &Error::ControlInUse => format!("Impossible to modify the control, it is currently in use."),
            &Error::ResourceInUse => format!("Impossible to modify the resource, it is currently in use."),
            &Error::Unimplemented => format!("Feature not yet implemented"),
            &Error::System(ref e) => format!("A system error was raised: {:?}", e),
            &Error::UserError(ref e) => format!("{}", e),
        }

    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.translate())
    }
}