//! Optional: Fullscreen Windows
//!
//! Extend your window manager with support for fullscreen windows, i.e. the
//! ability to temporarily make a window take up the whole screen, thereby
//! obscuring all other windows. See the documentation of the
//! [`FullscreenSupport`] trait for the precise requirements. Don't confuse
//! this with the first assignment, in which you built a window manager that
//! displayed all windows fullscreen.
//!
//! Like in the previous assignments, either make a copy of, or define a
//! wrapper around your previous window manager to implement the
//! [`FullscreenSupport`] trait as well. Note that this window manager must
//! still implement all the traits from previous assignments.
//!
//! [`FullscreenSupport`]: ../../cplwm_api/wm/trait.FullscreenSupport.html
//!
//! # Status
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! A lot of tests were copied and adapted from the d_minimising_windows
//! A fullscreen window always has the focus and is the only one rendered. Floating windows are not visible above fullscreen windows
//!

use cplwm_api::types::{Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::{FullscreenSupport, MinimiseSupport, FloatSupport, TilingSupport, WindowManager};

use d_minimising_windows::WMName as MinimisingWM;

/// Type alias for automated tests
pub type WMName = FullscreenWM<MinimisingWM>;

/// Main struct of the window manager
/// This WM can make a window fullscreen and uses the WrappedWM for all other windows
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FullscreenWM<WrappedWM: WindowManager> {
    /// The WindowWithInfo for the fullscreen window
    pub fullscreen_window: Option<WindowWithInfo>,
    /// The wrapped window manager that takes care of all the other windows
    pub wrapped_wm: WrappedWM,
}

impl<WrappedWM: WindowManager> FullscreenWM<WrappedWM> {
    fn un_fullscreen(&mut self) {
        self.get_fullscreen_window()
            .map(|w| self.toggle_fullscreen(w));

        self.fullscreen_window = None;
    }

    fn is_fullscreen(&self, window: Window) -> bool {
        self.get_fullscreen_window()
            .map(|w| w == window)
            .unwrap_or(false)
    }
}

impl<WrappedWM: WindowManager> WindowManager for FullscreenWM<WrappedWM> {
    /// We use the Error from the WrappedWM as our Error type.
    type Error = WrappedWM::Error;

    fn new(screen: Screen) -> FullscreenWM<WrappedWM> {
        FullscreenWM {
            fullscreen_window: None,
            wrapped_wm: WrappedWM::new(screen),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        let mut windows = self.wrapped_wm.get_windows();

        self.get_fullscreen_window()
            .map(|w| windows.push(w));

        windows
    }

    /// Removes the current fullscreen window if any
    /// (unless the window was already added)
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        if self.is_managed(window_with_info.window) {
            return Ok(())
        }

        self.un_fullscreen();

        if window_with_info.fullscreen {
            self.fullscreen_window = Some(window_with_info);
            Ok(())
        } else {
            self.wrapped_wm.add_window(window_with_info)
        }
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        if self.is_fullscreen(window) {
            self.fullscreen_window = None;
            return Ok(());
        }

        self.wrapped_wm.remove_window(window)
    }

    /// If there is a fullscreen window, it has focus
    fn get_window_layout(&self) -> WindowLayout {
        self.get_fullscreen_window()
            .map(|w| WindowLayout {
                windows: vec![(w, self.get_screen().to_geometry())],
                focused_window: Some(w)
            })
            .unwrap_or_else(|| self.wrapped_wm.get_window_layout())
    }

    /// If the new focused window is not the fullscreen window, the fullscreen window will be un-fullscreened
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        // If the focus is a new window, remove the fullscreenness
        if window.map(|w| self.is_fullscreen(w)).unwrap_or(false) {
            // Wants to focus the fullscreen window
            Ok(())
        } else {
            self.un_fullscreen();
            self.wrapped_wm.focus_window(window)
        }
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.un_fullscreen();

        self.wrapped_wm.cycle_focus(dir)
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        if self.is_fullscreen(window) {
            Ok(self.fullscreen_window
               .map(|info| WindowWithInfo {
                   window: info.window,
                   geometry: self.get_screen().to_geometry(),
                   float_or_tile: info.float_or_tile,
                   fullscreen: true,
               })
               // we know we can unwrap because we just checked if the window is fullscreen
               .unwrap())
        } else {
            self.wrapped_wm.get_window_info(window)
        }
    }

    fn get_screen(&self) -> Screen {
        self.wrapped_wm.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.wrapped_wm.resize_screen(screen)
    }

    fn get_focused_window(&self) -> Option<Window> {
        self.get_fullscreen_window()
            .or_else(|| self.wrapped_wm.get_focused_window())
    }

    fn is_managed(&self, window: Window) -> bool {
        self.is_fullscreen(window) || self.wrapped_wm.is_managed(window)
    }
}

impl<WrappedWM: TilingSupport> TilingSupport for FullscreenWM<WrappedWM> {
    fn get_master_window(&self) -> Option<Window> {
        self.wrapped_wm.get_master_window()
    }

    /// This will unfullscreen
    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error> {
        self.un_fullscreen();

        self.wrapped_wm.swap_with_master(window)
    }

    /// This will unfullscreen
    fn swap_windows(&mut self, dir: PrevOrNext) {
        self.un_fullscreen();

        self.wrapped_wm.swap_windows(dir)
    }
}

impl<WrappedWM: FloatSupport> FloatSupport for FullscreenWM<WrappedWM> {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.wrapped_wm.get_floating_windows()
    }

    /// If the passed window is fullscreen, it will un-fullscreen it first
    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error> {
        if self.is_fullscreen(window) {
            self.un_fullscreen();
        }

        self.wrapped_wm.toggle_floating(window)
    }

    /// If the passed window is a fullscreen window, it will remember the geometry for when this window becomes unfullscreen again
    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error> {
        if self.is_fullscreen(window) {
            let mut wi = self.fullscreen_window.unwrap().clone();
            wi.geometry = new_geometry;
            self.fullscreen_window = Some(wi);
            Ok(())
        } else {
            self.wrapped_wm.set_window_geometry(window, new_geometry)
        }
    }
}

impl<WrappedWM: MinimiseSupport> MinimiseSupport for FullscreenWM<WrappedWM> {
    fn get_minimised_windows(&self) -> Vec<Window> {
        self.wrapped_wm.get_minimised_windows()
    }

    /// If the passed window is fullscreen, it will first un-fullscreen it.
    fn toggle_minimised(&mut self, window: Window) -> Result<(), Self::Error> {
        if self.is_fullscreen(window) {
            // let wi = try!(self.get_window_info(window));
            // try!(self.wrapped_wm.add_window(wi));

            // self.fullscreen_window = None;
            self.un_fullscreen();
        }

        self.wrapped_wm.toggle_minimised(window)
    }

    fn is_minimised(&self, window: Window) -> bool {
        self.wrapped_wm.is_minimised(window)
    }
}

impl<WrappedWM: WindowManager> FullscreenSupport for FullscreenWM<WrappedWM> {
    fn get_fullscreen_window(&self) -> Option<Window> {
        self.fullscreen_window
            .map(|info| info.window)
    }

    fn toggle_fullscreen(&mut self, window: Window) -> Result<(), Self::Error> {
        if self.is_fullscreen(window) {
            // We know there is a fullscreen window so we can unwrap
            let wi = self.fullscreen_window.unwrap();
            self.fullscreen_window = None;

            self.wrapped_wm.add_window(wi)
        } else {
            let wi = try!(self.wrapped_wm.get_window_info(window));
            self.fullscreen_window = Some(wi);

            self.wrapped_wm.remove_window(window)
        }
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
#[allow(unused_mut)]
#[allow(unused_variables)]
mod tests {
    pub use super::*;
    pub use d_minimising_windows::WMName as MinimisingWM;

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

            let mut wm: FullscreenWM<MinimisingWM> = FullscreenWM::new(screen);
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
                wm.toggle_minimised(1);
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
                expect!(wm.is_fullscreen(1)).to(be_true());
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
                expect!(wm.is_fullscreen(1)).to(be_true());
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
                expect!(wm.is_fullscreen(1)).to(be_false());
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
                expect!(wm.is_fullscreen(1)).to(be_true());
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
                wm.toggle_minimised(1);

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
                wm.toggle_fullscreen(1);

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
                wm.toggle_minimised(2);

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
                wm.toggle_minimised(6);
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

                expect!(wm.is_fullscreen(7)).to(be_true());
                expect!(wm.get_window_layout().focused_window).to(be_equal_to(Some(7)));
                expect!(wm.get_focused_window()).to(be_equal_to(Some(7)));
            }

            it "should unfullscreen if it wants to lose the focus on the fullscreen window" {
                wm.add_window(WindowWithInfo::new_fullscreen(7, some_geom)).unwrap();

                wm.focus_window(None).unwrap();

                expect!(wm.is_fullscreen(7)).to(be_false());
                expect!(wm.get_window_layout().focused_window).to(be_equal_to(None));
                expect!(wm.get_focused_window()).to(be_equal_to(None));
            }

            it "should unfullscreen if it wants to set the focus to a window different from the fullscreen window" {
                wm.add_window(WindowWithInfo::new_fullscreen(7, some_geom)).unwrap();

                wm.focus_window(Some(3)).unwrap();

                expect!(wm.is_fullscreen(7)).to(be_false());
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
                wm.toggle_minimised(6);
            }

            it "should cycle in forward direction" {
                wm.focus_window(Some(1));

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
                wm.focus_window(None);

                expect!(wm.get_focused_window()).to(be_equal_to(None));
            }

            it "should not focus on a window if there are none" {
                wm.remove_window(1);
                wm.remove_window(2);
                wm.remove_window(3);
                wm.remove_window(4);

                expect!(wm.get_focused_window()).to(be_equal_to(None));
            }

            it "should switch between floating windows and tiled windows" {
                wm.focus_window(Some(3));

                wm.cycle_focus(Next);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
            }

            it "should switch between tiled windows and floating windows" {
                wm.focus_window(Some(4));

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

                expect!(wm.is_fullscreen(7)).to(be_false());
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
                wm.focus_window(None);

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
                wm.remove_window(2);

                let info = wm.get_window_info(1).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 1,
                    geometry: screen_geom,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: true,
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
                wm.toggle_minimised(6);

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
                wm.toggle_minimised(6);

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
                    wm.toggle_minimised(6);
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7);
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();

                    let master = wm.get_master_window();

                    expect!(master).to(be_equal_to(Some(1)));
                }

                it "should return none if there is no master window" {
                    wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                    wm.toggle_minimised(6);
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7);
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
                    wm.remove_window(2);

                    wm.swap_with_master(5);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(5, left_half),
                                                                                (3, right_upper_quarter),
                                                                                (1, right_lower_quarter)]));
                }

                it "should be able to swap a minimised float with master" {
                    wm.remove_window(2);
                    wm.toggle_minimised(5);

                    wm.swap_with_master(5);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(5)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(5, left_half),
                                                                                (3, right_upper_quarter),
                                                                                (1, right_lower_quarter)]));
                }


                it "should be able to swap a tiled with master" {
                    wm.swap_with_master(2);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half),
                                                                                (1, right_upper_quarter),
                                                                                (3, right_lower_quarter),
                                                                                (5, some_geom)]));
                }

                it "should be able to swap a minimised tiled with master" {
                    wm.toggle_minimised(2);

                    wm.swap_with_master(2);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half),
                                                                                (3, right_upper_quarter),
                                                                                (1, right_lower_quarter),
                                                                                (5, some_geom)]));
                }

                it "should focus the master tile if it is already the master window" {
                    wm.swap_with_master(1);

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
                    wm.toggle_fullscreen(3);

                    wm.swap_with_master(3);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(3, left_half),
                                                                                (2, right_upper_quarter),
                                                                                (1, right_lower_quarter),
                                                                                (5, some_geom)]));
                }

                it "should unfullscreen if swaping a window with master while other window fullscreen" {
                    wm.toggle_fullscreen(3);

                    wm.swap_with_master(2);

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
                    wm.toggle_minimised(6);
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7);
                }

                it "should be able to swap the focussed window with another window in forward direction" {
                    wm.focus_window(Some(2));

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(1, left_half),
                                                                            (3, right_upper_quarter),
                                                                            (2, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "should be able to swap the focussed window with another window in backward direction" {
                    wm.focus_window(Some(2));

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
                    wm.focus_window(Some(1));

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(3, left_half),
                                                                            (2, right_upper_quarter),
                                                                            (1, right_lower_quarter),
                                                                            (5, floating_geom)]));
                }

                it "shouldn't do anything if there is no focused window" {
                    wm.focus_window(None);

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
                    wm.remove_window(1);
                    wm.remove_window(2);

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to([(3, screen_geom),
                                                                            (5, floating_geom)]));
                }

                it "should unfullscreen if called with a fullscreen window" {
                    wm.toggle_fullscreen(2);

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
                wm.toggle_minimised(6);
            }

            it "should return the floating windows" {
                wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();

                expect!(wm.get_floating_windows()).to(be_equal_to(vec![4, 5]));
            }

            describe! toggle_floating {
                it "should be able to toggle floating windows on" {
                    wm.toggle_floating(1);

                    expect!(wm.is_floating(1)).to(be_true());
                    expect!(wm.get_window_layout().windows.iter().find(|&&(w, _geom)| w == 1).unwrap().1).to(be_equal_to(some_geom));
                }
                it "should be able to toggle floating windows off" {
                    wm.toggle_floating(4);

                    expect!(wm.is_floating(4)).to(be_false());
                }

                it "should be able to toggle floating on for minimised windows" {
                    wm.toggle_minimised(3);
                    wm.toggle_floating(3);

                    expect!(wm.is_floating(3)).to(be_true());
                    expect!(wm.get_window_layout().windows.iter().find(|&&(w, _geom)| w == 3).unwrap().1).to(be_equal_to(right_lower_quarter));
                }

                it "should be able to toggle floating off for minimised windows" {
                    wm.toggle_floating(6);

                    expect!(wm.is_floating(6)).to(be_false());
                }

                it "should be able to toggle floating on for fullscreen windows" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    wm.toggle_floating(8);

                    expect!(wm.is_floating(8)).to(be_true());
                    expect!(wm.is_fullscreen(8)).to(be_false());
                }

                it "should be able to toggle floating off for fullscreen windows" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    wm.toggle_fullscreen(5);
                    wm.toggle_floating(5);

                    expect!(wm.is_floating(5)).to(be_false());
                    expect!(wm.is_fullscreen(5)).to(be_false());
                    expect!(wm.is_fullscreen(8)).to(be_false());
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
                    wm.toggle_minimised(7);
                    wm.set_window_geometry(7, floating_geom).unwrap();

                    expect!(wm.get_window_info(7).unwrap().geometry).to(be_equal_to(floating_geom));
                }

                it "should be able to set a new window geometry for a fullscreen window" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    wm.set_window_geometry(8, floating_geom).unwrap();

                    wm.toggle_floating(8);

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
                    wm.toggle_minimised(6);

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![6]));
                    expect!(wm.is_minimised(6)).to(be_true());
                }

                it "should return them in order of minimising (works for float, tiled)" {
                    wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                    wm.toggle_minimised(6);
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7);

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![6, 7]));
                    expect!(wm.is_minimised(6)).to(be_true());
                    expect!(wm.is_minimised(7)).to(be_true());
                }

                it "should return them in order of minimising (works for tiled, float)" {
                    wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                    wm.toggle_minimised(6);
                    wm.add_window(WindowWithInfo::new_float(7, some_geom)).unwrap();
                    wm.toggle_minimised(7);

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![6, 7]));
                    expect!(wm.is_minimised(6)).to(be_true());
                    expect!(wm.is_minimised(7)).to(be_true());
                }

                it "should return singleton if there were two but one got unminised" {
                    wm.add_window(WindowWithInfo::new_float(6, some_geom)).unwrap();
                    wm.toggle_minimised(6);
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7);
                    wm.toggle_minimised(6);

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![7]));
                    expect!(wm.is_minimised(6)).to(be_false());
                    expect!(wm.is_minimised(7)).to(be_true());
                }

                it "should include a fullscreen window that got minimised" {
                    wm.add_window(WindowWithInfo::new_fullscreen(8, some_geom)).unwrap();
                    expect!(wm.is_minimised(8)).to(be_false());

                    wm.toggle_minimised(8);

                    expect!(wm.get_minimised_windows()).to(be_equal_to(vec![8]));
                    expect!(wm.is_minimised(8)).to(be_true());
                }
            }

            describe! toggle_minimised {
                it "should be able to toggle minimisation on for a tiled window" {
                    wm.toggle_minimised(3);

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
                    wm.toggle_minimised(1);
                    wm.toggle_minimised(1);

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
                    wm.toggle_minimised(7);

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
                    wm.toggle_minimised(7);
                    wm.toggle_minimised(7);

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
                    wm.toggle_fullscreen(3);

                    wm.toggle_minimised(2);

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
                    expect!(wm.is_fullscreen(3)).to(be_true());
                }

                it "should unfullscreen if fullscreen window is minimised" {
                    wm.toggle_fullscreen(3);

                    wm.toggle_minimised(3);

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
                    expect!(wm.is_fullscreen(3)).to(be_false());
                }

                it "should not restore window as fullscreen if toggle minimised" {
                    wm.toggle_fullscreen(3);

                    wm.toggle_minimised(3);
                    wm.toggle_minimised(3);

                    expect!(wm.is_fullscreen(3)).to(be_false());
                }
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
                    wm.toggle_fullscreen(1);
                    wm.toggle_fullscreen(2);

                    expect!(wm.get_fullscreen_window()).to(be_equal_to(Some(2)));
                }
            }

            describe! toggle_fullscreen {
                it "should be able to toggle a fullscreen on" {
                    wm.toggle_fullscreen(2);

                    expect!(wm.get_fullscreen_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, screen_geom)]));
                }

                it "should be able to toggle a fullscreen off" {
                    wm.toggle_fullscreen(2);
                    wm.toggle_fullscreen(2);

                    expect!(wm.get_fullscreen_window()).to(be_equal_to(None));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(1, left_half),
                                                                              (3, right_upper_quarter),
                                                                              (2, right_lower_quarter),
                                                                              (5, floating_geom)]));
                }
            }
        }
    }

    describe! integration_test {
        it "should work with the example from the forum" {
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

            let mut wm: FullscreenWM<MinimisingWM> = FullscreenWM::new(screen);

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
            wm.toggle_floating(3);
            // windows = [(2, master_geometry), (4, slave_geometry), (6, slave_geometry), (3, slave_geometry), (1, float_geometry), (5, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_sixth), (6, right_middle_sixth), (3, right_lower_sixth), (1, floating_geom), (5, floating_geom)]));

            // toggle_floating(6)
            wm.toggle_floating(6);
            // windows = [(2, master_geometry), (4, slave_geometry), (3, slave_geometry), (1, float_geometry), (5, float_geometry), (6, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_quarter), (3, right_lower_quarter), (1, floating_geom), (5, floating_geom), (6, some_geom)]));

            // toggle_floating(1)
            wm.toggle_floating(1);
            // windows = [(2, master_geometry), (4, slave_geometry), (3, slave_geometry), (1, slave_geometry), (5, float_geometry), (6, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_sixth), (3, right_middle_sixth), (1, right_lower_sixth), (5, floating_geom), (6, some_geom)]));

            // focus_window(Some(5))
            wm.focus_window(Some(5));
            // windows = [(2, master_geometry), (4, slave_geometry), (3, slave_geometry), (1, slave_geometry), (6, float_geometry), (5, float_geometry)]
            expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half), (4, right_upper_sixth), (3, right_middle_sixth), (1, right_lower_sixth), (6, some_geom), (5, floating_geom)]));
        }
    }
}
