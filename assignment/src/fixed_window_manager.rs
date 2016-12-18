//! This module adds some missing methods to properly implement all WindowManagers
use cplwm_api::types::{Window, WindowWithInfo};
use cplwm_api::wm::WindowManager;

/// The RealWindowInfo allows wrappers to access the interesting WindowWithInfo (not just the tiled one)
pub trait RealWindowInfo: WindowManager {
    /// Get real window info. Equal to get_window_info unless the window is tiled
    /// In that case, the geometry should equal the geometry of the window after toggle_floating()
    fn get_real_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error>;
}
