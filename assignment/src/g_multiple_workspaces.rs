//! Optional: Multiple Workspaces
//!
//! Extend your window manager with support for multiple workspaces. The
//! concept of workspaces is described in the first section of the assignment.
//! See the documentation of the [`MultiWorkspaceSupport`] trait for the precise
//! requirements.
//!
//! *Unlike* the previous assignments, you are not allowed to make a copy of
//! your previous window manager. You *have* to define a wrapper implementing
//! the [`MultiWorkspaceSupport`] trait. This wrapper can take any existing
//! window manager and uses it to create the different workspaces. This
//! wrapper must also implement all the traits you have implemented in the
//! other assignments, you can forward them to the window manager of the
//! current workspace.
//!
//! [`MultiWorkspaceSupport`]: ../../cplwm_api/wm/trait.MultiWorkspaceSupport.html
//!
//! # Status
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! Most methods only operate on the current workspace. swap_with_master will first move the window to the current workspace. (I find it hard to believe it would be used in practice but there is currently no other way to move a window after the window has been added to the WM).
//!

use cplwm_api::types::{Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo, WorkspaceIndex, MAX_WORKSPACE_INDEX, GapSize};
use cplwm_api::wm::{FloatSupport, FullscreenSupport, GapSupport, MinimiseSupport, TilingSupport, WindowManager};

use e_fullscreen_windows::WMName as FullscreenWM;
use fixed_window_manager::RealWindowInfo;

/// Type alias for automated tests
pub type WMName = WorkspaceWM<FullscreenWM>;

/// Main struct of the window manager
/// This WM has multiple workspaces. Each workspace uses a different WM
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct WorkspaceWM<WrappedWM: RealWindowInfo> {
    /// The index of the current workspace
    pub current_workspace: WorkspaceIndex,
    /// The list of wrapped window managers
    pub wrapped_wms: Vec<WrappedWM>,
}

impl<WrappedWM: RealWindowInfo> WorkspaceWM<WrappedWM> {
    /// Get the wm for the current workspace
    fn get_current_wm(&self) -> &WrappedWM {
        &self.wrapped_wms[self.current_workspace]
    }

    /// Get the wm for the current workspace
    fn get_current_mutable_wm(&mut self) -> &mut WrappedWM {
        &mut self.wrapped_wms[self.current_workspace]
    }

    /// Get the index of the wm for the given window
    /// (or the current_workspace index if the window is not managed by this WM)
    fn get_index_for_window(&self, window: Window) -> WorkspaceIndex {
        if self.is_managed(window) {
            let ref wms = self.wrapped_wms;
            wms
                .into_iter()
                .position(|wm| wm.is_managed(window))
                // we can unwrap because it is managed by some wm
                .unwrap()
        } else {
            self.current_workspace
        }
    }

    /// Gets the WM that manages the given window,
    /// or the current workspace in case the window is not managed by this WM
    fn get_wm_for_window(&self, window: Window) -> &WrappedWM {
        let pos = self.get_index_for_window(window);
        &self.wrapped_wms[pos]
    }

    /// Gets the WM that manages the given window,
    /// or the current workspace in case the window is not managed by this WM
    fn get_mutable_wm_for_window(&mut self, window: Window) -> &mut WrappedWM {
        let pos = self.get_index_for_window(window);
        &mut self.wrapped_wms[pos]
    }

    /// Gets the WM that manages the given window and switches the focus to it
    fn get_mutable_wm_for_window_and_switch(&mut self, window: Window) -> &mut WrappedWM {
        let pos = self.get_index_for_window(window);
        self.current_workspace = pos;
        &mut self.wrapped_wms[pos]
    }

    /// Moves the given window to the current workspace
    fn move_window_to_current_workspace(&mut self, window: Window) -> Result<(), WrappedWM::Error> {
        let info = try!(self.get_wm_for_window(window)
            .get_real_window_info(window));

        try!(self.get_mutable_wm_for_window(window)
            .remove_window(window));

        let current_wm = self.get_current_mutable_wm();

        current_wm.add_window(info)
    }
}

impl<WrappedWM: RealWindowInfo> WindowManager for WorkspaceWM<WrappedWM> {
    /// We use the Error from the WrappedWM as our Error type.
    type Error = WrappedWM::Error;

    fn new(screen: Screen) -> WorkspaceWM<WrappedWM> {
        WorkspaceWM {
            current_workspace: 0,
            wrapped_wms: (0..MAX_WORKSPACE_INDEX).map(|_| WrappedWM::new(screen)).collect(),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        let mut windows = vec![];

        let ref wms = self.wrapped_wms;
        for wm in wms {
            windows.extend(wm.get_windows());
        }

        windows
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        self.get_current_mutable_wm()
            .add_window(window_with_info)
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        self.get_mutable_wm_for_window(window)
            .remove_window(window)
    }

    fn get_window_layout(&self) -> WindowLayout {
        self.get_current_wm()
            .get_window_layout()
    }

    /// Will switch workspace if the window is not in the current workspace
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        match window {
            None => {
                self.get_current_mutable_wm().focus_window(window)
            },
            Some(w) => {
                self.get_mutable_wm_for_window_and_switch(w)
                    .focus_window(window)
            },
        }
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.get_current_mutable_wm()
            .cycle_focus(dir)
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        self.get_wm_for_window(window)
            .get_window_info(window)
    }

    fn get_screen(&self) -> Screen {
        self.get_current_wm().get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        let ref mut wms = self.wrapped_wms;
        for wm in wms {
            wm.resize_screen(screen)
        };
    }

    fn get_focused_window(&self) -> Option<Window> {
        self.get_current_wm()
            .get_focused_window()
    }

    fn is_managed(&self, window: Window) -> bool {
        self.wrapped_wms
            .iter()
            .any(|wm| wm.is_managed(window))
    }
}

impl<WrappedWM: TilingSupport+RealWindowInfo> TilingSupport for WorkspaceWM<WrappedWM> {
    fn get_master_window(&self) -> Option<Window> {
        self.get_current_wm()
            .get_master_window()
    }

    /// If the window is not on the current workspace
    /// it will move it to this workspace
    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error> {
        if !self.get_current_wm().is_managed(window) {
            try!(self.move_window_to_current_workspace(window));
        }

        self.get_current_mutable_wm()
            .swap_with_master(window)
    }

    fn swap_windows(&mut self, dir: PrevOrNext) {
        self.get_current_mutable_wm()
            .swap_windows(dir)
    }
}

impl<WrappedWM: FloatSupport+RealWindowInfo> FloatSupport for WorkspaceWM<WrappedWM> {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.get_current_wm()
            .get_floating_windows()
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error> {
        self.get_current_mutable_wm()
            .toggle_floating(window)
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error> {
        self.get_mutable_wm_for_window(window)
            .set_window_geometry(window, new_geometry)
    }
}

impl<WrappedWM: MinimiseSupport+RealWindowInfo> MinimiseSupport for WorkspaceWM<WrappedWM> {
    fn get_minimised_windows(&self) -> Vec<Window> {
        self.get_current_wm()
            .get_minimised_windows()
    }

    fn toggle_minimised(&mut self, window: Window) -> Result<(), Self::Error> {
        self.get_current_mutable_wm()
            .toggle_minimised(window)
    }
}

impl<WrappedWM: FullscreenSupport+RealWindowInfo> FullscreenSupport for WorkspaceWM<WrappedWM> {
    fn get_fullscreen_window(&self) -> Option<Window> {
        self.get_current_wm()
            .get_fullscreen_window()
    }

    fn toggle_fullscreen(&mut self, window: Window) -> Result<(), Self::Error> {
        self.get_mutable_wm_for_window_and_switch(window)
            .toggle_fullscreen(window)
    }
}

impl<WrappedWM: GapSupport+RealWindowInfo> GapSupport for WorkspaceWM<WrappedWM> {
    fn get_gap(&self) -> GapSize {
        self.get_current_wm()
            .get_gap()
    }

    fn set_gap(&mut self, gap_size: GapSize) {
        let ref mut wms = self.wrapped_wms;
        for wm in wms {
            wm.set_gap(gap_size)
        };
    }
}
