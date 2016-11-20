//! This module provides the possible errors for our window managers

use std::error;
use std::fmt;

use cplwm_api::types::{Window};

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
