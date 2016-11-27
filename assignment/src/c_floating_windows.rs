//! Floating Windows
//!
//! Extend your window manager with support for floating windows, i.e. windows
//! that do not tile but that you move around and resize with the mouse. These
//! windows will *float* above the tiles, e.g. dialogs, popups, video players,
//! etc. See the documentation of the [`FloatSupport`] trait for the precise
//! requirements.
//!
//! Either make a copy of the tiling window manager you developed in the
//! previous assignment and let it implement the [`FloatSupport`] trait as
//! well, or implement the [`FloatSupport`] trait by building a wrapper around
//! your tiling window manager. This way you won't have to copy paste code.
//! Note that this window manager must still implement the [`TilingSupport`]
//! trait.
//!
//! [`FloatSupport`]: ../../cplwm_api/wm/trait.FloatSupport.html
//! [`TilingSupport`]: ../../cplwm_api/wm/trait.TilingSupport.html
//!
//! # Status
//!
//! **TODO**: Replace the question mark below with YES, NO, or PARTIAL to
//! indicate the status of this assignment. If you want to tell something
//! about this assignment to the grader, e.g., you have a bug you can't fix,
//! or you want to explain your approach, write it down after the comments
//! section.
//!
//! COMPLETED: ?
//!
//! COMMENTS:
//!
//! A lot of tests were copied and adapted from the b_tiling_wm
//!

use std::os::raw::{c_int, c_uint};
use cplwm_api::types::{Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::types::PrevOrNext::*;
use cplwm_api::types::FloatOrTile;
pub use cplwm_api::types::FloatOrTile::*;
use cplwm_api::wm::{FloatSupport, TilingSupport, WindowManager};
use std::collections::HashMap;

use error::WMError;
use error::WMError::*;
use b_tiling_wm::TilingWM;

/// Type alias for automated tests
pub type WMName = FloatingWM;

/// Main struct of the window manager
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FloatingWM {
    /// A vector of floating windows
    pub floating_windows: Vec<Window>,
    /// The index of the focused window.
    /// If the index is larger than the nb of floating windows, the focused window is tiled.
    pub focused_index: Option<usize>,
    /// The wrapped window manager that takes care of the tiled windows
    pub tiling_wm: TilingWM,
    /// The window_with_info's for the managed windows
    pub infos: HashMap<Window, WindowWithInfo>,
}

impl WindowManager for FloatingWM {
    /// We use `WMError` as our `Error` type.
    type Error = WMError;

    fn new(screen: Screen) -> FloatingWM {
        FloatingWM {
            floating_windows: Vec::new(),
            focused_index: None,
            tiling_wm: TilingWM::new(screen),
            infos: HashMap::new(),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        // Return a clone so external users can't access the original Vector
        let mut windows = self.tiling_wm.get_windows();
        windows.extend(self.floating_windows.clone());

        windows
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        if !self.is_managed(window_with_info.window) {
            match window_with_info.float_or_tile {
                Float => {
                    self.floating_windows.push(window_with_info.window);
                }
                Tile => {
                    self.tiling_wm.add_window(window_with_info);
                }
            }
            // Add the window info to the wm
            self.infos.insert(window_with_info.window.clone(), window_with_info);

            // Focus on this new window
            self.focus_window(Some(window_with_info.window));
        }

        Ok(())
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        // Remove the window info from the wm
        self.infos.remove(&window);

        if self.tiling_wm.is_managed(window) {
            self.tiling_wm.remove_window(window)
        } else {
            self.floating_windows
                .iter()
                .position(|w| *w == window)
                .ok_or(UnknownWindow(window))
                .map(|i| {
                    self.floating_windows.remove(i);

                    // if there is no window left, no window has focus.
                    if self.get_windows().len() == 0 {
                        self.focus_window(None);
                    } else if let Some(j) = self.focused_index {
                        if i <= j {
                            // Update the index of the focused window to keep the same window in focus
                            self.cycle_focus(Prev);
                        }
                    }
                })
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        if self.get_windows().len() == 0 {
            WindowLayout::new()
        } else {
            let focused_window = self.get_focused_window();

            let mut windows: Vec<(Window, Geometry)> = self.tiling_wm
                .get_window_layout()
                .windows;
            windows.extend(self.floating_windows
                .iter()
                .filter(|w| Some(**w) != focused_window)
                .map(|w| (*w, self.get_geom(&w))));
            // Put the focused window on top (if it's floating)
            focused_window.map(|w| if self.is_floating(w) {
                windows.push((w, self.get_geom(&w)))
            });

            WindowLayout {
                focused_window: focused_window,
                windows: windows,
            }
        }
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        match window {
            None => {
                self.focused_index = None;
                self.tiling_wm.focus_window(None);
            }
            Some(w) => {
                if self.tiling_wm.is_managed(w) {
                    let result = self.tiling_wm.focus_window(Some(w));
                    self.focused_index = self.tiling_wm.focused_index.map(|i| self.floating_windows.len() + i);
                    return result;
                } else if self.is_managed(w) {
                    // Set focused index to the position of the window passed along
                    self.focused_index = self.floating_windows.iter().position(|w2| *w2 == w);
                } else {
                    return Err(UnknownWindow(w));
                }
            }
        }

        Ok(())
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        // If no focused window, set focused_index to 0 (unless there are no windows)
        // If focused window, cycle the focus

        self.focused_index = self.focused_index
            .or_else(|| self.floating_windows.first().map(|_w| 0))
            .map(|i| self.cycle_index(i, dir));
        if let Some(i) = self.focused_index {
            if i >= self.floating_windows.len() {
                self.tiling_wm.focused_index = Some(i - self.floating_windows.len());
            }
        }
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        return self.tiling_wm
            .get_window_info(window)
            .or_else(|_e| {
                self.infos
                    .get(&window)
                    .ok_or(UnknownWindow(window))
                    .map(|wi| wi.clone())
            });
    }

    fn get_screen(&self) -> Screen {
        self.tiling_wm.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.tiling_wm.resize_screen(screen)
    }

    fn get_focused_window(&self) -> Option<Window> {
        self.focused_index
            .and_then(|i| if i < self.floating_windows.len() {
                Some(i)
            } else {
                None
            })
            .map(|i| self.floating_windows[i])
            .or_else(|| self.tiling_wm.get_focused_window())
    }

    fn is_managed(&self, window: Window) -> bool {
        self.floating_windows.contains(&window) || self.tiling_wm.is_managed(window)
    }
}

impl TilingSupport for FloatingWM {
    fn get_master_window(&self) -> Option<Window> {
        self.tiling_wm.get_master_window()
    }

    /// If the passed window is a floating window, it will first be tiled before the swap happens
    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error> {
        if !self.is_managed(window) {
            return Err(UnknownWindow(window));
        }
        if self.floating_windows.contains(&window) {
            self.float_or_tile_window(&window, Tile);
        }

        self.tiling_wm.swap_with_master(window);

        Ok(())
    }

    /// If the focused window is a floating window, it will first be tiled
    fn swap_windows(&mut self, dir: PrevOrNext) {
        if let Some(i) = self.focused_index {
            if i >= self.floating_windows.len() {
                self.tiling_wm.swap_windows(dir)
            }
        }
    }
}

impl FloatSupport for FloatingWM {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.floating_windows.clone()
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error> {
        if !self.is_managed(window) {
            return Err(UnknownWindow(window));
        }

        let float_or_tile = if self.is_floating(window) {
            Tile
        } else {
            Float
        };
        self.float_or_tile_window(&window, float_or_tile);

        Ok(())
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error> {
        if !self.is_managed(window) {
            return Err(UnknownWindow(window));
        }

        let mut wi = self.infos.get(&window).unwrap().clone();
        wi.geometry = new_geometry;
        self.infos.insert(window, wi);

        Ok(())
    }
}

impl FloatingWM {
    /// Get the requested geometry for this window
    fn get_geom(&self, window: &Window) -> Geometry {
        self.infos.get(window).unwrap().geometry
    }

    /// Return the 'next' index in the direction of dir
    fn cycle_index(&self, i: usize, dir: PrevOrNext) -> usize {
        let nb_windows = self.get_windows().len();
        match dir {
            Prev => (i + nb_windows - 1) % nb_windows,
            Next => (i + 1) % nb_windows,
        }
    }

    /// Float or tile the window.
    /// Will panic if the window is not managed by this wm
    fn float_or_tile_window(&mut self, window: &Window, float_or_tile: FloatOrTile) {
        let mut wi = self.infos.get(&window).unwrap().clone();
        wi.float_or_tile = float_or_tile;
        self.remove_window(*window);
        self.add_window(wi);
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
#[allow(unused_mut)]
#[allow(unused_variables)]
mod tests {
    pub use super::*;

    pub use std::os::raw::{c_int, c_uint};
    pub use cplwm_api::wm::*;
    pub use cplwm_api::types::*;
    pub use cplwm_api::types::PrevOrNext::*;

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

            let mut wm = FloatingWM::new(screen);
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
        }

        describe! focus_window {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();
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
        }

        describe! cycle_focus {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();
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
        }

        describe! tiling_support {
            describe! get_master_window {
                it "should return the master window if there is one" {
                    wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();

                    let master = wm.get_master_window();

                    expect!(master).to(be_equal_to(Some(1)));
                }

                it "should return none if there is no master window" {
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

                it "should be able to swap with master" {
                    wm.swap_with_master(2);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_window_layout().windows).to(be_equal_to(vec![(2, left_half),
                                                                                (1, right_upper_quarter),
                                                                                (3, right_lower_quarter),
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
            }

            describe! swap_windows {
                before_each {
                    wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_float(5, floating_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
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
            }
        }

        describe! floating_support {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(4, floating_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
            }

            it "should return the floating windows" {
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
            }
        }
    }
}
