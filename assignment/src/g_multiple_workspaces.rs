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
//! Most methods only operate on the current workspace. swap_with_master will first move the window to the current workspace.
//! (I find it hard to believe it would be used in practice but there is currently no other way to move a window after the window has been added to the WM).
//!
//! A lot of tests were copied from e_fullscreen_windows
//!

use cplwm_api::types::{GapSize, Geometry, MAX_WORKSPACE_INDEX, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo, WorkspaceIndex};
use cplwm_api::wm::{FloatSupport, FullscreenSupport, GapSupport, MinimiseSupport, MultiWorkspaceSupport, TilingSupport, WindowManager};

use e_fullscreen_windows::WMName as FullscreenWM;
use error::MultiWMError;
use error::MultiWMError::*;
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
    type Error = MultiWMError<WrappedWM::Error>;

    fn new(screen: Screen) -> WorkspaceWM<WrappedWM> {
        WorkspaceWM {
            current_workspace: 0,
            wrapped_wms: (0..(MAX_WORKSPACE_INDEX+1)).map(|_| WrappedWM::new(screen)).collect(),
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
        Ok(self.get_current_mutable_wm()
            .add_window(window_with_info)?)
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        Ok(self.get_mutable_wm_for_window(window)
            .remove_window(window)?)
    }

    fn get_window_layout(&self) -> WindowLayout {
        self.get_current_wm()
            .get_window_layout()
    }

    /// Will switch workspace if the window is not in the current workspace
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        Ok(match window {
            None => self.get_current_mutable_wm().focus_window(window),
            Some(w) => {
                self.get_mutable_wm_for_window_and_switch(w)
                    .focus_window(window)
            }
        }?)
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.get_current_mutable_wm()
            .cycle_focus(dir)
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        Ok(self.get_wm_for_window(window)
            .get_window_info(window)?)
    }

    fn get_screen(&self) -> Screen {
        self.get_current_wm().get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        let ref mut wms = self.wrapped_wms;
        for wm in wms {
            wm.resize_screen(screen)
        }
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

impl<WrappedWM: TilingSupport + RealWindowInfo> TilingSupport for WorkspaceWM<WrappedWM> {
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

        Ok(self.get_current_mutable_wm()
            .swap_with_master(window)?)
    }

    fn swap_windows(&mut self, dir: PrevOrNext) {
        self.get_current_mutable_wm()
            .swap_windows(dir)
    }
}

impl<WrappedWM: FloatSupport + RealWindowInfo> FloatSupport for WorkspaceWM<WrappedWM> {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.get_current_wm()
            .get_floating_windows()
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error> {
        Ok(self.get_current_mutable_wm()
            .toggle_floating(window)?)
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error> {
        Ok(self.get_mutable_wm_for_window(window)
            .set_window_geometry(window, new_geometry)?)
    }
}

impl<WrappedWM: MinimiseSupport + RealWindowInfo> MinimiseSupport for WorkspaceWM<WrappedWM> {
    fn get_minimised_windows(&self) -> Vec<Window> {
        self.get_current_wm()
            .get_minimised_windows()
    }

    fn toggle_minimised(&mut self, window: Window) -> Result<(), Self::Error> {
        Ok(self.get_current_mutable_wm()
            .toggle_minimised(window)?)
    }
}

impl<WrappedWM: FullscreenSupport + RealWindowInfo> FullscreenSupport for WorkspaceWM<WrappedWM> {
    fn get_fullscreen_window(&self) -> Option<Window> {
        self.get_current_wm()
            .get_fullscreen_window()
    }

    fn toggle_fullscreen(&mut self, window: Window) -> Result<(), Self::Error> {
        Ok(self.get_mutable_wm_for_window_and_switch(window)
            .toggle_fullscreen(window)?)
    }
}

impl<WrappedWM: GapSupport + RealWindowInfo> GapSupport for WorkspaceWM<WrappedWM> {
    fn get_gap(&self) -> GapSize {
        self.get_current_wm()
            .get_gap()
    }

    fn set_gap(&mut self, gap_size: GapSize) {
        let ref mut wms = self.wrapped_wms;
        for wm in wms {
            wm.set_gap(gap_size)
        }
    }
}

impl<WrappedWM: RealWindowInfo> MultiWorkspaceSupport<WrappedWM> for WorkspaceWM<WrappedWM> {
    fn get_current_workspace_index(&self) -> WorkspaceIndex {
        self.current_workspace
    }

    fn get_workspace(&self, index: WorkspaceIndex) -> Result<&WrappedWM, Self::Error> {
        // No test for index < 0 because of type limits and warnings should be avoided
        if index > MAX_WORKSPACE_INDEX {
            Err(UnknownWorkspace(index))
        } else {
            Ok(&self.wrapped_wms[index])
        }
    }

    fn get_workspace_mut(&mut self, index: WorkspaceIndex) -> Result<&mut WrappedWM, Self::Error> {
        if index > MAX_WORKSPACE_INDEX {
            Err(UnknownWorkspace(index))
        } else {
            Ok(&mut self.wrapped_wms[index])
        }
    }

    /// Switch to the workspace at the given index.
    ///
    /// If `index == get_current_workspace_index()`, do nothing.
    ///
    /// **Invariant**: the window layout after switching to another workspace
    /// and then switching back to the original workspace should be the same
    /// as before.
    ///
    /// This function *should* return an appropriate error when `0 <= index <=
    /// MAX_WORKSPACE_INDEX` is not true.
    fn switch_workspace(&mut self, index: WorkspaceIndex) -> Result<(), Self::Error> {
        if index > MAX_WORKSPACE_INDEX {
            Err(UnknownWorkspace(index))
        } else {
            self.current_workspace = index;
            Ok(())
        }
    }
}

#[cfg(test)]
#[allow(unused_mut)]
#[allow(unused_variables)]
mod tests {
    pub use super::*;
    pub use fixed_window_manager::RealWindowInfo;

    pub use std::os::raw::{c_int, c_uint};
    pub use cplwm_api::wm::*;
    pub use cplwm_api::types::*;
    pub use cplwm_api::types::PrevOrNext::*;
    pub use cplwm_api::types::FloatOrTile::*;

    // Import expectest names
    pub use expectest::prelude::*;

    describe! wm {
        before_each {
            let screen: Screen = Screen {
                width: 800,
                height: 600,
            };
            let screen_geom = screen.to_geometry();

            let some_geom: Geometry = Geometry {
                x: 10,
                y: 10,
                width: 100,
                height: 100,
            };

            let left_half = Geometry {
                x: 0, y: 0,
                width: screen.width/2,
                height: screen.height,
            };
            let right_half = Geometry {
                x: (screen.width/2) as c_int,
                y: 0,
                width: screen.width/2,
                height: screen.height,
            };

            let right_upper_quarter = Geometry {
                x: (screen.width/2) as c_int,
                y: 0,
                width: screen.width/2,
                height: screen.height/2,
            };
            let right_lower_quarter = Geometry {
                x: (screen.width/2) as c_int,
                y: (screen.height/2) as c_int,
                width: screen.width/2,
                height: screen.height/2,
            };

            let floating_geom: Geometry = Geometry {
                x: 20,
                y: 40,
                width: 200,
                height: 20,
            };

            let mut wm = WMName::new(screen);
        }

        it "should have an empty window layout initially" {
            expect!(wm.get_window_layout()).to(be_equal_to(WindowLayout::new()));
        }

        describe! add_window {
            it "should add a window correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "shouldn't add a window twice" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "shouldn't add a floating window twice" {
                wm.add_window(WindowWithInfo::new_float(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, some_geom)]));
            }

            it "shouldn't add a floating window twice if passed as tiled" {
                wm.add_window(WindowWithInfo::new_float(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, some_geom)]));
            }

            it "shouldn't add a tiled window twice if passed as float" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "shouldn't add a minimised window twice" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.toggle_minimised(1).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(None));
                expect!(wl.windows).to(be_equal_to(vec![]));
            }

            it "should add a fullscreen window correctly" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_fullscreen(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![2, 3, 1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "shouldn't add a fullscreen window twice" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_fullscreen(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_fullscreen(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![2, 1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "shouldn't add a window twice if passed 2nd time as fullscreen" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_fullscreen(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![2, 1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(2, left_half),(1, right_half)]));
            }

            it "shouldn't add a window twice if passed 1st time as fullscreen" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_fullscreen(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![2, 1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "should add 2 windows correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(2)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2]));
                expect!(wl.focused_window).to(be_equal_to(Some(2)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),(2, right_half)]));
            }

            it "should add 3 windows correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                let wl = wm.get_window_layout();
                let windows = vec![(1, left_half),
                                   (2, right_upper_quarter),
                                   (3, right_lower_quarter)];

                expect!(wm.is_managed(3)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2, 3]));
                expect!(wl.focused_window).to(be_equal_to(Some(3)));
                expect!(wl.windows).to(be_equal_to(windows));
            }

            it "should add floating windows correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

                wm.add_window(WindowWithInfo::new_float(3, floating_geom)).unwrap();

                let wl = wm.get_window_layout();
                expect!(wm.is_managed(3)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2, 3]));
                expect!(wl.focused_window).to(be_equal_to(Some(3)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),
                                                        (2, right_half),
                                                        (3, floating_geom)]));
            }

            it "should add a minimised window correctly" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(3, floating_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.toggle_minimised(1).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![2,3,1]));
                expect!(wl.focused_window).to(be_equal_to(Some(2)));
                expect!(wl.windows).to(be_equal_to(vec![(2, screen_geom),(3, floating_geom)]));
            }

            it "should show floating windows above tiled windows" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(3, floating_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

                let wl = wm.get_window_layout();
                expect!(wm.is_managed(3)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2, 3]));
                expect!(wl.focused_window).to(be_equal_to(Some(2)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),
                                                        (2, right_half),
                                                        (3, floating_geom)]));
            }

            it "should add work for a fullscreen window and a tiled window" {
                wm.add_window(WindowWithInfo::new_fullscreen(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(1)).to(be_true());
                expect!(wm.is_managed(2)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2]));
                expect!(wl.focused_window).to(be_equal_to(Some(2)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),(2, right_half)]));
            }
        }

        describe! remove_window {
            it "should remove a window correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

                wm.remove_window(2).unwrap();

                let wl = wm.get_window_layout();
                expect!(wm.is_managed(2)).to(be_false());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "should remove a fullscreen correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_fullscreen(2, some_geom)).unwrap();

                wm.remove_window(2).unwrap();

                let wl = wm.get_window_layout();
                expect!(wm.is_managed(2)).to(be_false());
                expect!(wm.get_windows()).to(be_equal_to(vec![1,3]));
                expect!(wl.focused_window).to(be_equal_to(Some(3)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),(3, right_half)]));
            }

            it "should remove a window correctly if there is a fullscreen window" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.toggle_fullscreen(1).unwrap();

                wm.remove_window(2).unwrap();

                let wl = wm.get_window_layout();
                expect!(wm.is_managed(2)).to(be_false());
                expect!(wm.get_windows()).to(be_equal_to(vec![3, 1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
            }

            it "should not lose focus if we remove another tiled window" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.remove_window(2).unwrap();

                expect!(wm.is_managed(2)).to(be_false());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 3]));
                let wl = wm.get_window_layout();
                expect!(wl.focused_window).to(be_equal_to(Some(3)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),(3, right_half)]));
            }

            it "should cycle focus if we remove the focused tiled window" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.remove_window(3).unwrap();

                expect!(wm.is_managed(3)).to(be_false());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2]));
                let wl = wm.get_window_layout();
                expect!(wl.focused_window).to(be_equal_to(Some(2)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),(2, right_half)]));
            }

            it "should not lose focus if we remove another float window" {
                wm.add_window(WindowWithInfo::new_float(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.remove_window(1).unwrap();

                expect!(wm.is_managed(1)).to(be_false());
                expect!(wm.get_windows()).to(be_equal_to(vec![2, 3]));
                let wl = wm.get_window_layout();
                expect!(wl.focused_window).to(be_equal_to(Some(3)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                expect!(wl.windows).to(be_equal_to(vec![(2, left_half),(3, right_half)]));
            }

            it "should do be in initial state if we remove all windows" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(2, some_geom)).unwrap();

                wm.remove_window(1).unwrap();
                wm.remove_window(2).unwrap();

                expect!(wm.is_managed(1)).to(be_false());
                expect!(wm.is_managed(2)).to(be_false());
                expect!(wm.get_windows().len()).to(be_equal_to(0));

                expect!(wm.get_window_layout()).to(be_equal_to(WindowLayout::new()));
            }

            it "should remove a floating window correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.toggle_minimised(2).unwrap();

                wm.remove_window(2).unwrap();

                let wl = wm.get_window_layout();
                expect!(wm.is_managed(2)).to(be_false());
                expect!(wm.get_windows()).to(be_equal_to(vec![1]));
                expect!(wl.focused_window).to(be_equal_to(Some(1)));
                expect!(wl.windows).to(be_equal_to(vec![(1, screen_geom)]));
                expect!(wm.get_minimised_windows().contains(&2)).to(be_false());
            }
        }

        describe! focus_window {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();

                wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                wm.toggle_minimised(6).unwrap();
            }

            it "should focus the correct window" {
                wm.focus_window(Some(1)).unwrap();

                expect!(wm.get_window_layout().focused_window).to(be_equal_to(Some(1)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
            }

            it "should keep the focus if already focussed" {
                wm.focus_window(Some(2)).unwrap();

                expect!(wm.get_window_layout().focused_window).to(be_equal_to(Some(2)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
            }

            it "should lose the focus if passed no window" {
                wm.focus_window(None).unwrap();

                expect!(wm.get_window_layout().focused_window).to(be_equal_to(None));
                expect!(wm.get_focused_window()).to(be_equal_to(None));
            }

            it "should throw error on unknown window" {
                expect!(wm.focus_window(Some(100))).to(be_err());
            }

            it "should bring the focused window to the front" {
                wm.focus_window(Some(3)).unwrap();

                expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(1, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (4, right_lower_quarter),
                                                                            (5, some_geom),
                                                                            (3, some_geom)]));
            }

            it "should unminimize a minimized window before focussing" {
                wm.focus_window(Some(6)).unwrap();

                expect!(wm.is_minimised(6)).to(be_false());
                expect!(wm.get_window_layout().focused_window).to(be_equal_to(Some(6)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(6)));
            }

            it "should keep the focus if it wants to refocus on the fullscreen window" {
                wm.add_window(WindowWithInfo::new_fullscreen(7, some_geom)).unwrap();

                wm.focus_window(Some(7)).unwrap();

                expect!(wm.get_window_layout().focused_window).to(be_equal_to(Some(7)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(7)));
            }

            it "should unfullscreen if it wants to lose the focus on the fullscreen window" {
                wm.add_window(WindowWithInfo::new_fullscreen(7, some_geom)).unwrap();

                wm.focus_window(None).unwrap();

                expect!(wm.get_window_layout().focused_window).to(be_equal_to(None));
                expect!(wm.get_focused_window()).to(be_equal_to(None));
            }

            it "should unfullscreen if it wants to set the focus to a window different from the fullscreen window" {
                wm.add_window(WindowWithInfo::new_fullscreen(7, some_geom)).unwrap();

                wm.focus_window(Some(3)).unwrap();

                expect!(wm.get_window_layout().focused_window).to(be_equal_to(Some(3)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
            }
        }

        describe! cycle_focus {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();

                wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                wm.toggle_minimised(6).unwrap();
            }

            it "should cycle in forward direction" {
                wm.focus_window(Some(1)).unwrap();

                wm.cycle_focus(Next);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
            }

            it "should work in backward direction" {
                wm.cycle_focus(Prev);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
            }

            it "should cycle in backward direction" {
                wm.cycle_focus(Prev);
                wm.cycle_focus(Prev);
                wm.cycle_focus(Prev);
                wm.cycle_focus(Prev);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(4)));
            }

            it "should not do anything if cycling back an forth" {
                wm.cycle_focus(Prev);
                wm.cycle_focus(Next);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(4)));
            }

            it "should not focus on a window if none was selected" {
                wm.focus_window(None).unwrap();

                expect!(wm.get_focused_window()).to(be_equal_to(None));
            }

            it "should not focus on a window if there are none" {
                wm.remove_window(1).unwrap();
                wm.remove_window(2).unwrap();
                wm.remove_window(3).unwrap();
                wm.remove_window(4).unwrap();

                expect!(wm.get_focused_window()).to(be_equal_to(None));
            }

            it "should switch between floating windows and tiled windows" {
                wm.focus_window(Some(3)).unwrap();

                wm.cycle_focus(Next);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
            }

            it "should switch between tiled windows and floating windows" {
                wm.focus_window(Some(4)).unwrap();

                wm.cycle_focus(Next);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
            }

            it "should bring the focused window to the front if a float" {
                wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();

                wm.cycle_focus(Prev);
                expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(1, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (4, right_lower_quarter),
                                                                            (5, some_geom),
                                                                            (3, some_geom)]));
            }

            it "should unfullscreen on calling cycle_focus" {
                wm.add_window(WindowWithInfo::new_fullscreen(7, some_geom)).unwrap();

                wm.cycle_focus(Prev);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(4)));
            }
        }

        describe! get_window_info {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(4, floating_geom)).unwrap();
            }

            it "should work for the slave window" {
                let info = wm.get_window_info(2).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: right_half,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should work for a master window" {
                let info = wm.get_window_info(1).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 1,
                    geometry: left_half,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should work if there is no focused window" {
                wm.focus_window(None).unwrap();

                let info = wm.get_window_info(2).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: right_half,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should error if the window is not managed by the window manager" {
                let info = wm.get_window_info(3);

                expect!(info).to(be_err());
            }

            it "should work with 1 window" {
                wm.remove_window(2).unwrap();

                let info = wm.get_window_info(1).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 1,
                    geometry: screen_geom,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should work with 3 tiled windows" {
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                expect!(wm.get_window_info(1).unwrap()).to(be_equal_to(WindowWithInfo {
                    window: 1,
                    geometry: left_half,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
                expect!(wm.get_window_info(2).unwrap()).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: right_upper_quarter,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
                expect!(wm.get_window_info(3).unwrap()).to(be_equal_to(WindowWithInfo {
                    window: 3,
                    geometry: right_lower_quarter,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should work for the floating window" {
                expect!(wm.get_window_info(4).unwrap()).to(be_equal_to(WindowWithInfo {
                    window: 4,
                    geometry: floating_geom,
                    float_or_tile: Float,
                    fullscreen: false,
                }));
            }

            it "should work with a minimized tiled window" {
                wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                wm.toggle_minimised(6).unwrap();

                let info = wm.get_window_info(6).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 6,
                    geometry: right_lower_quarter,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should work with a minimized floating window" {
                wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                wm.toggle_minimised(6).unwrap();

                let info = wm.get_window_info(6).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 6,
                    geometry: some_geom,
                    float_or_tile: FloatOrTile::Float,
                    fullscreen: false,
                }));
            }

            it "should work with a fullscreen tiled window" {
                wm.add_window(WindowWithInfo::new_fullscreen(6, some_geom)).unwrap();

                let info = wm.get_window_info(6).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 6,
                    geometry: screen_geom,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: true,
                }));
            }

            it "should work with a fullscreen floating window" {
                wm.add_window(WindowWithInfo {
                    window: 6,
                    fullscreen: true,
                    float_or_tile: Float,
                    geometry: some_geom,
                }).unwrap();

                let info = wm.get_window_info(6).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 6,
                    geometry: screen_geom,
                    float_or_tile: Float,
                    fullscreen: true,
                }));
            }
        }

        describe! screen {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();

                let new_screen = Screen {
                    width: 200,
                    height: 200
                };

                let left_half = Geometry {
                    x: 0, y: 0,
                    width: new_screen.width/2,
                    height: new_screen.height,
                };
                let right_upper_quarter = Geometry {
                    x: (new_screen.width/2) as c_int,
                    y: 0,
                    width: new_screen.width/2,
                    height: new_screen.height/2,
                };
                let right_lower_quarter = Geometry {
                    x: (new_screen.width/2) as c_int,
                    y: (new_screen.height/2) as c_int,
                    width: new_screen.width/2,
                    height: new_screen.height/2,
                };
            }

            it "should return the default screen"{
                expect!(wm.get_screen()).to(be_equal_to(screen));
            }

            it "should return the new screen if one is provided" {
                wm.resize_screen(new_screen);

                expect!(wm.get_screen()).to(be_equal_to(new_screen));
            }

            it "should use the new screen in get_window_layout if there is 1 tiled window" {
                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();

                expect!(wl.windows).to(be_equal_to(vec![(1, new_screen.to_geometry()),
                                                        (5, some_geom)]));
            }

            it "should use the new screen in get_window_layout if there are more windows" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),
                                                        (2, right_upper_quarter),
                                                        (3, right_lower_quarter),
                                                        (5, some_geom)]));
            }

            it "should use the new screen in get_window_layout if there is a fullscreen window" {
                wm.add_window(WindowWithInfo::new_fullscreen(2, some_geom)).unwrap();

                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();
                expect!(wl.windows).to(be_equal_to(vec![(2, new_screen.to_geometry())]));
            }
        }

        describe! tiling_support {
            describe! get_master_window {
                it "should return the master window if there is one" {
                    wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();

                    let master = wm.get_master_window();

                    expect!(master).to(be_equal_to(Some(1)));
                }

                it "should return none if there is no master window" {
                    wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();

                    let master = wm.get_master_window();

                    expect!(master).to(be_equal_to(None));
                }
            }

            describe! swap_with_master {
                before_each {
                    wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                }

                it "should be able to swap a float with master" {
                    wm.remove_window(2).unwrap();

                    wm.swap_with_master(5).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(5, left_half),
                                                                                (3, right_upper_quarter),
                                                                                (1, right_lower_quarter)]));
                }

                it "should be able to swap a minimised float with master" {
                    wm.remove_window(2).unwrap();
                    wm.toggle_minimised(5).unwrap();

                    wm.swap_with_master(5).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(5, left_half),
                                                                                (3, right_upper_quarter),
                                                                                (1, right_lower_quarter)]));
                }


                it "should be able to swap a tiled with master" {
                    wm.swap_with_master(2).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half),
                                                                                (1, right_upper_quarter),
                                                                                (3, right_lower_quarter),
                                                                                (5, some_geom)]));
                }

                it "should be able to swap a minimised tiled with master" {
                    wm.toggle_minimised(2).unwrap();

                    wm.swap_with_master(2).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half),
                                                                                (3, right_upper_quarter),
                                                                                (1, right_lower_quarter),
                                                                                (5, some_geom)]));
                }

                it "should focus the master tile if it is already the master window" {
                    wm.swap_with_master(1).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(1)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(1, left_half),
                                                                                (2, right_upper_quarter),
                                                                                (3, right_lower_quarter),
                                                                                (5, some_geom)]));
                }

                it "should error if the window is not managed by the wm" {
                    expect!(wm.swap_with_master(4)).to(be_err());
                }

                it "should unfullscreen if swaping the fullscreen with master" {
                    wm.toggle_fullscreen(3).unwrap();

                    wm.swap_with_master(3).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(3, left_half),
                                                                                (2, right_upper_quarter),
                                                                                (1, right_lower_quarter),
                                                                                (5, some_geom)]));
                }

                it "should unfullscreen if swaping a window with master while other window fullscreen" {
                    wm.toggle_fullscreen(3).unwrap();

                    wm.swap_with_master(2).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half),
                                                                                (1, right_upper_quarter),
                                                                                (3, right_lower_quarter),
                                                                                (5, some_geom)]));
                }
            }

            describe! swap_windows {
                before_each {
                    wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_float(5, floating_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                    wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();
                }

                it "should be able to swap the focussed window with another window in forward direction" {
                    wm.focus_window(Some(2)).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(1, left_half),
                                                                            (3, right_upper_quarter),
                                                                            (2, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "should be able to swap the focussed window with another window in backward direction" {
                    wm.focus_window(Some(2)).unwrap();

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(2, left_half),
                                                                            (1, right_upper_quarter),
                                                                            (3, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "should cycle the swap in forward direction" {
                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(3, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (1, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "should cycle the swap in backward direction" {
                    wm.focus_window(Some(1)).unwrap();

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(3, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (1, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "shouldn't do anything if there is no focused window" {
                    wm.focus_window(None).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(None));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(1, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (3, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "shouldn't do anything if calling swap in 2 opposite directions" {
                    wm.swap_windows(Prev);
                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(1, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (3, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "shouldn't do anything if calling the swap twice and cycling in between" {
                    wm.swap_windows(Next);
                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(1, left_half),
                                       (2, right_upper_quarter),
                                       (3, right_lower_quarter),
                                       (5, floating_geom)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if calling with only one tiled window" {
                    wm.remove_window(1).unwrap();
                    wm.remove_window(2).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(3, screen_geom),
                                                                            (5, floating_geom)]));
                }

                it "should unfullscreen if called with a fullscreen window" {
                    wm.toggle_fullscreen(2).unwrap();

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(1, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (3, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }
            }
        }

        describe! floating_support {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(4, floating_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                wm.toggle_minimised(6).unwrap();
            }

            it "should return the floating windows" {
                wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();

                expect!(wm.get_floating_windows()).to(be_equal_to(vec![4, 5]));
            }

            describe! toggle_floating {
                it "should be able to toggle floating windows on" {
                    wm.toggle_floating(1).unwrap();

                    expect!(wm.is_floating(1)).to(be_true());
                    expect!(wm.get_window_layout().windows.iter().find(|&&(w, _geom)| w == 1).unwrap().1).to(be_equal_to(some_geom));
                }
                it "should be able to toggle floating windows off" {
                    wm.toggle_floating(4).unwrap();

                    expect!(wm.is_floating(4)).to(be_false());
                }

                it "should be able to toggle floating on for minimised windows" {
                    wm.toggle_minimised(3).unwrap();
                    wm.toggle_floating(3).unwrap();

                    expect!(wm.is_floating(3)).to(be_true());
                    expect!(wm.get_window_layout().windows.iter().find(|&&(w, _geom)| w == 3).unwrap().1).to(be_equal_to(some_geom));
                }

                it "should be able to toggle floating off for minimised windows" {
                    wm.toggle_floating(6).unwrap();

                    expect!(wm.is_floating(6)).to(be_false());
                }

                it "should be able to toggle floating on for fullscreen windows" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    wm.toggle_floating(8).unwrap();

                    expect!(wm.is_floating(8)).to(be_true());
                }

                it "should be able to toggle floating off for fullscreen windows" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    wm.toggle_fullscreen(5).unwrap();
                    wm.toggle_floating(5).unwrap();

                    expect!(wm.is_floating(5)).to(be_false());
                }
            }

            describe! set_window_geometry {
                it "should be able to set a new window geometry" {
                    wm.set_window_geometry(5, floating_geom).unwrap();

                    expect!(wm.get_window_layout().windows.iter().find(|&&(w, _geom)| w == 5).unwrap().1).to(be_equal_to(floating_geom));
                }

                it "should error if the window is not managed by the wm" {
                    expect!(wm.set_window_geometry(100, floating_geom)).to(be_err());
                }

                it "should not error if the window is not floating" {
                    expect!(wm.set_window_geometry(1, floating_geom)).to(be_ok());
                }

                it "should be able to set a new window geometry for minimised floating windows" {
                    wm.set_window_geometry(6, floating_geom).unwrap();

                    expect!(wm.get_window_info(6).unwrap().geometry).to(be_equal_to(floating_geom));
                }

                it "should be able to set a new window geometry for minimised tiled windows" {
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();
                    wm.set_window_geometry(7, floating_geom).unwrap();

                    expect!(wm.get_window_info(7).unwrap().geometry).to(be_equal_to(floating_geom));
                }

                it "should be able to set a new window geometry for a fullscreen window" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    wm.set_window_geometry(8, floating_geom).unwrap();

                    wm.toggle_floating(8).unwrap();

                    expect!(wm.get_window_info(8).unwrap().geometry).to(be_equal_to(floating_geom));
                }
            }
        }

        describe! minimise_support {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(5, floating_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
            }
            describe! get_minimised_windows {
                it "should return empty list if there are none" {
                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![]));
                }

                it "should return singleton if there is one" {
                    wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![6]));
                    expect!(wm.is_minimised(6)).to(be_true());
                }

                it "should return them in order of minimising (works for float, tiled)" {
                    wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![6, 7]));
                    expect!(wm.is_minimised(6)).to(be_true());
                    expect!(wm.is_minimised(7)).to(be_true());
                }

                it "should return them in order of minimising (works for tiled, float)" {
                    wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_float(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![6, 7]));
                    expect!(wm.is_minimised(6)).to(be_true());
                    expect!(wm.is_minimised(7)).to(be_true());
                }

                it "should return singleton if there were two but one got unminised" {
                    wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();
                    wm.toggle_minimised(6).unwrap();

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![7]));
                    expect!(wm.is_minimised(6)).to(be_false());
                    expect!(wm.is_minimised(7)).to(be_true());
                }

                it "should include a fullscreen window that got minimised" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    expect!(wm.is_minimised(8)).to(be_false());

                    wm.toggle_minimised(8).unwrap();

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![8]));
                    expect!(wm.is_minimised(8)).to(be_true());
                }
            }

            describe! toggle_minimised {
                it "should be able to toggle minimisation on for a tiled window" {
                    wm.toggle_minimised(3).unwrap();

                    expect!(wm.is_managed(3)).to(be_true());
                    expect!(wm.is_minimised(3)).to(be_true());
                    expect!(wm.get_windows().contains(&3)).to(be_true());
                    let visible_windows: Vec<Window> = wm.get_window_layout().windows.iter().map(|t| t.0).collect();
                    expect!(visible_windows.contains(&3)).to(be_false());
                    expect!(wm.get_window_info(3).unwrap()).to(be_equal_to(WindowWithInfo {
                        window: 3,
                        geometry: right_lower_quarter,
                        float_or_tile: Tile,
                        fullscreen: false,
                    }));
                }

                it "should be able to toggle minimisation off for a tiled window" {
                    wm.toggle_minimised(1).unwrap();
                    wm.toggle_minimised(1).unwrap();

                    expect!(wm.is_managed(1)).to(be_true());
                    expect!(wm.is_minimised(1)).to(be_false());
                    expect!(wm.get_windows().contains(&1)).to(be_true());
                    expect!(wm.get_window_layout().windows.contains(&(1, right_lower_quarter))).to(be_true());
                    expect!(wm.get_window_info(1).unwrap()).to(be_equal_to(WindowWithInfo {
                        window: 1,
                        geometry: right_lower_quarter,
                        float_or_tile: Tile,
                        fullscreen: false,
                    }));
                }

                it "should be able to toggle minimisation on for a floating window" {
                    wm.add_window(WindowWithInfo::new_float(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();

                    expect!(wm.is_managed(7)).to(be_true());
                    expect!(wm.is_minimised(7)).to(be_true());
                    expect!(wm.get_windows().contains(&7)).to(be_true());
                    let visible_windows: Vec<Window> = wm.get_window_layout().windows.iter().map(|t| t.0).collect();
                    expect!(visible_windows.contains(&7)).to(be_false());
                    expect!(wm.get_window_info(7).unwrap()).to(be_equal_to(WindowWithInfo {
                        window: 7,
                        geometry: some_geom,
                        float_or_tile: Float,
                        fullscreen: false,
                    }));
                }

                it "should be able to toggle minimisation off for a floating window" {
                    wm.add_window(WindowWithInfo::new_float(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();
                    wm.toggle_minimised(7).unwrap();

                    expect!(wm.is_managed(7)).to(be_true());
                    expect!(wm.is_minimised(7)).to(be_false());
                    expect!(wm.get_windows().contains(&7)).to(be_true());
                    expect!(wm.get_window_layout().windows.contains(&(7, some_geom))).to(be_true());
                    expect!(wm.get_window_info(7).unwrap()).to(be_equal_to(WindowWithInfo {
                        window: 7,
                        geometry: some_geom,
                        float_or_tile: Float,
                        fullscreen: false,
                    }));
                }

                it "should not unfullscreen if other window is minimised" {
                    wm.toggle_fullscreen(3).unwrap();

                    wm.toggle_minimised(2).unwrap();

                    expect!(wm.is_managed(2)).to(be_true());
                    expect!(wm.is_minimised(2)).to(be_true());
                    expect!(wm.get_windows().contains(&2)).to(be_true());
                    let visible_windows: Vec<Window> = wm.get_window_layout().windows.iter().map(|t| t.0).collect();
                    expect!(visible_windows.contains(&2)).to(be_false());
                    expect!(wm.get_window_info(2).unwrap()).to(be_equal_to(WindowWithInfo {
                        window: 2,
                        geometry: right_half,
                        float_or_tile: Tile,
                        fullscreen: false,
                    }));
                }

                it "should unfullscreen if fullscreen window is minimised" {
                    wm.toggle_fullscreen(3).unwrap();

                    wm.toggle_minimised(3).unwrap();

                    expect!(wm.is_managed(3)).to(be_true());
                    expect!(wm.is_minimised(3)).to(be_true());
                    expect!(wm.get_windows().contains(&3)).to(be_true());
                    let visible_windows: Vec<Window> = wm.get_window_layout().windows.iter().map(|t| t.0).collect();
                    expect!(visible_windows.contains(&3)).to(be_false());
                    expect!(wm.get_window_info(3).unwrap()).to(be_equal_to(WindowWithInfo {
                        window: 3,
                        geometry: right_lower_quarter,
                        float_or_tile: Tile,
                        fullscreen: false,
                    }));
                }

                // it "should not restore window as fullscreen if toggle minimised" {
                //     wm.toggle_fullscreen(3).unwrap();

                //     wm.toggle_minimised(3).unwrap();
                //     wm.toggle_minimised(3).unwrap();

                //     expect!(wm.is_fullscreen(3)).to(be_false());
                // }
            }
        }

        describe! fullscreen_support {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(5, floating_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

            }

            describe! get_fullscreen_window {
                it "should return the fullscreen window" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();

                    expect!(wm.get_fullscreen_window()).to(be_equal_to(Some(8)));
                }

                it "should return none if there is no fullscreen window" {
                    expect!(wm.get_fullscreen_window()).to(be_equal_to(None));
                }

                it "should keep the last fullscreen window as fullscreen" {
                    wm.toggle_fullscreen(1).unwrap();
                    wm.toggle_fullscreen(2).unwrap();

                    expect!(wm.get_fullscreen_window()).to(be_equal_to(Some(2)));
                }
            }

            describe! toggle_fullscreen {
                it "should be able to toggle a fullscreen on" {
                    wm.toggle_fullscreen(2).unwrap();

                    expect!(wm.get_fullscreen_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, screen_geom)]));
                }

                it "should be able to toggle a fullscreen off" {
                    wm.toggle_fullscreen(2).unwrap();
                    wm.toggle_fullscreen(2).unwrap();

                    expect!(wm.get_fullscreen_window()).to(be_equal_to(None));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(1, left_half),
                                                                              (3, right_upper_quarter),
                                                                              (2, right_lower_quarter),
                                                                              (5, floating_geom)]));
                }
            }

            it "should keep the layout if toggling minimise before and after" {
                wm.toggle_fullscreen(2).unwrap();

                expect!(wm.get_fullscreen_window()).to(be_equal_to(Some(2)));
                expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, screen_geom)]));

                wm.toggle_minimised(2).unwrap();

                expect!(wm.get_fullscreen_window()).to(be_equal_to(None));
                expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(1, left_half),(3, right_half), (5, floating_geom)]));

                wm.toggle_minimised(2).unwrap();

                expect!(wm.get_fullscreen_window()).to(be_equal_to(Some(2)));
                expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, screen_geom)]));
            }
        }
    }

    describe! integration_test {
        before_each {
            let screen: Screen = Screen {
                width: 800,
                height: 600,
            };
            let screen_geom = screen.to_geometry();

            let some_geom: Geometry = Geometry {
                x: 10,
                y: 10,
                width: 100,
                height: 100,
            };

            let left_half = Geometry {
                x: 0, y: 0,
                width: screen.width/2,
                height: screen.height,
            };
            let right_half = Geometry {
                x: (screen.width/2) as c_int,
                y: 0,
                width: screen.width/2,
                height: screen.height,
            };
            let right_upper_quarter = Geometry {
                x: (screen.width/2) as c_int,
                y: 0,
                width: screen.width/2,
                height: screen.height/2,
            };
            let right_lower_quarter = Geometry {
                x: (screen.width/2) as c_int,
                y: (screen.height/2) as c_int,
                width: screen.width/2,
                height: screen.height/2,
            };
            let right_upper_sixth = Geometry {
                x: (screen.width/2) as c_int,
                y: 0,
                width: screen.width/2,
                height: screen.height/3,
            };
            let right_middle_sixth = Geometry {
                x: (screen.width/2) as c_int,
                y: (screen.height/3) as c_int,
                width: screen.width/2,
                height: screen.height/3,
            };
            let right_lower_sixth = Geometry {
                x: (screen.width/2) as c_int,
                y: (screen.height*2/3) as c_int,
                width: screen.width/2,
                height: screen.height/3,
            };

            let floating_geom: Geometry = Geometry {
                x: 20,
                y: 40,
                width: 200,
                height: 20,
            };

            let mut wm = WMName::new(screen);
        }

        it "should work with the example from the forum" {
            // Let's walk through the steps:
            // windows = []
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![]));

            // add 1 as a floating window
            wm.add_window(WindowWithInfo::new_float(1, floating_geom)).unwrap();
            // windows = [(1, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(1, floating_geom)]));

            // add 2 as a tiled window
            wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
            // windows = [(2, fullscreen_geometry), (1, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, screen_geom),(1, floating_geom)]));

            // add 3 as a floating window
            wm.add_window(WindowWithInfo::new_float(3, floating_geom)).unwrap();
            // windows = [(2, fullscreen_geometry), (1, float_geometry), (3, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, screen_geom),(1, floating_geom), (3, floating_geom)]));

            // add 4 as a tiled window
            wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();
            // windows = [(2, master_geometry), (4, slave_geometry), (1, float_geometry), (3, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_half), (1, floating_geom), (3, floating_geom)]));

            // add 5 as a floating window
            wm.add_window(WindowWithInfo::new_float(5, floating_geom)).unwrap();
            // windows = [(2, master_geometry), (4, slave_geometry), (1, float_geometry), (3, float_geometry), (5, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_half), (1, floating_geom), (3, floating_geom), (5, floating_geom)]));

            // add 6 as a tiled window
            wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
            // windows = [(2, master_geometry), (4, slave_geometry), (6, slave_geometry), (1, float_geometry), (3, float_geometry), (5, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_quarter), (6, right_lower_quarter), (1, floating_geom), (3, floating_geom), (5, floating_geom)]));

            // toggle_floating(3)
            wm.toggle_floating(3).unwrap();
            // windows = [(2, master_geometry), (4, slave_geometry), (6, slave_geometry), (3, slave_geometry), (1, float_geometry), (5, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_sixth), (6, right_middle_sixth), (3, right_lower_sixth), (1, floating_geom), (5, floating_geom)]));

            // toggle_floating(6)
            wm.toggle_floating(6).unwrap();
            // windows = [(2, master_geometry), (4, slave_geometry), (3, slave_geometry), (1, float_geometry), (5, float_geometry), (6, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_quarter), (3, right_lower_quarter), (1, floating_geom), (5, floating_geom), (6, some_geom)]));

            // toggle_floating(1)
            wm.toggle_floating(1).unwrap();
            // windows = [(2, master_geometry), (4, slave_geometry), (3, slave_geometry), (1, slave_geometry), (5, float_geometry), (6, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_sixth), (3, right_middle_sixth), (1, right_lower_sixth), (5, floating_geom), (6, some_geom)]));

            // focus_window(Some(5))
            wm.focus_window(Some(5)).unwrap();
            // windows = [(2, master_geometry), (4, slave_geometry), (3, slave_geometry), (1, slave_geometry), (6, float_geometry), (5, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_sixth), (3, right_middle_sixth), (1, right_lower_sixth), (6, some_geom), (5, floating_geom)]));
        }

        it "should work with a second example from the forum" {
            wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();
            wm.add_window(WindowWithInfo::new_tiled(5, some_geom)).unwrap();
            wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
            wm.add_window(WindowWithInfo::new_float(1, floating_geom)).unwrap();
            wm.add_window(WindowWithInfo::new_float(2, floating_geom)).unwrap();
            wm.add_window(WindowWithInfo::new_float(3, floating_geom)).unwrap();

            wm.toggle_floating(4).unwrap();
            wm.toggle_floating(5).unwrap();
            wm.toggle_floating(6).unwrap();

            let wl = wm.get_window_layout();
            expect(wl.windows).to(be_equal_to(vec![(1, floating_geom),
                                                   (2, floating_geom),
                                                   (3, floating_geom),
                                                   (4, some_geom),
                                                   (5, some_geom),
                                                   (6, some_geom)]));
            expect(wl.focused_window).to(be_equal_to(Some(3)));

            wm.cycle_focus(Next);

            let wl2 = wm.get_window_layout();
            expect(wl2.windows).to(be_equal_to(vec![(1, floating_geom),
                                                    (2, floating_geom),
                                                    (3, floating_geom),
                                                    (5, some_geom),
                                                    (6, some_geom),
                                                    (4, some_geom)]));
            expect(wl2.focused_window).to(be_equal_to(Some(4)));

            wm.cycle_focus(Prev);

            let wl3 = wm.get_window_layout();
            expect(wl3.windows).to(be_equal_to(vec![(1, floating_geom),
                                                    (2, floating_geom),
                                                    (5, some_geom),
                                                    (6, some_geom),
                                                    (4, some_geom),
                                                    (3, floating_geom)]));
            expect(wl3.focused_window).to(be_equal_to(Some(3)));
        }
    }
}
