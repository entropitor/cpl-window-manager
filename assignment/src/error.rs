//! This module provides the possible errors for our window managers

use std::error;
use std::error::Error;
use std::fmt;
use std::convert::From;

use cplwm_api::types::{Window, WorkspaceIndex};

/// The errors that a window manager can return.
///
/// [Error]: ../../cplwm_api/wm/trait.WindowManager.html#associatedtype.Error
#[derive(Debug)]
pub enum WMError {
    /// This window is not known by the window manager.
    UnknownWindow(Window),
}

impl fmt::Display for WMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WMError::UnknownWindow(ref window) => write!(f, "Unknown window: {}", window),
        }
    }
}

impl error::Error for WMError {
    fn description(&self) -> &'static str {
        match *self {
            WMError::UnknownWindow(_) => "Unknown window",
        }
    }
}

/// The extra errors that a multi-workspace window manager can return.
///
/// [Error]: ../../cplwm_api/wm/trait.WindowManager.html#associatedtype.Error
#[derive(Debug)]
pub enum MultiWMError<MyError: Error + 'static> {
    /// This workspace index is unknown
    UnknownWorkspace(WorkspaceIndex),
    /// An error from the wrapped WM
    WrappedError(MyError),
}

impl<MyError: Error + 'static> fmt::Display for MultiWMError<MyError> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MultiWMError::UnknownWorkspace(ref index) => write!(f, "Unknown index: {}", index),
            MultiWMError::WrappedError(ref error) => fmt::Display::fmt(&error, f),
        }
    }
}

impl<MyError: Error + 'static> error::Error for MultiWMError<MyError> {
    fn description(&self) -> &'static str {
        match *self {
            MultiWMError::UnknownWorkspace(_) => "Unknown index",
            MultiWMError::WrappedError(_) => "An error occurred in the wrapped workspace",
        }
    }
}

impl<MyError: Error + 'static> From<MyError> for MultiWMError<MyError> {
    fn from(error: MyError) -> MultiWMError<MyError> {
        MultiWMError::WrappedError(error)
    }
}
