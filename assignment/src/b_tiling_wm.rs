//! Tiling Window Manager
//!
//! Write a more complex window manager that will *tile* its windows. Tiling
//! is described in the first section of the assignment. Your window manager
//! must implement both the [`WindowManager`] trait and the [`TilingSupport`]
//! trait. See the documentation of the [`TilingSupport`] trait for the
//! precise requirements and an explanation of the tiling layout algorithm.
//!
//! [`WindowManager`]: ../../cplwm_api/wm/trait.WindowManager.html
//! [`TilingSupport`]: ../../cplwm_api/wm/trait.TilingSupport.html
//!
//! # Status
//!
//! **TODO**: Replace the question mark below with YES, NO, or PARTIAL to
//! indicate the status of this assignment. If you want to tell something
//! about this assignment to the grader, e.g., you have a bug you can't fix,
//! or you want to explain your approach, write it down after the comments
//! section.
//! **TODO** Add TilingSupport!
//!
//! COMPLETED: ?
//!
//! COMMENTS:
//!
//! A lot of code (+ tests) were copied from assignment a
//!

use cplwm_api::types::{Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::types::PrevOrNext::*;
pub use cplwm_api::types::FloatOrTile::*;
use cplwm_api::wm::WindowManager;

use error::WMError;
use error::WMError::*;

/// Type alias for automated tests
pub type WMName = TilingWM;

/// Main struct of the window manager
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct TilingWM {
    /// A vector of windows, the first one is the master window.
    pub windows: Vec<Window>,
    /// The screen that is managed
    pub screen: Screen,
    /// The index of the focused window (or None if no window is focussed)
    pub focused_index: Option<usize>,
}

fn get_geom(screen: Screen, i: usize, n: usize) -> Geometry {
    if i == 0 {
        // the master window
        get_master_geom(screen, n)
    } else {
        // a slave window
        get_slave_geom(screen, i - 1, n - 1)
    }
}

/// Return the geometry for the master window with n windows on the given screen
fn get_master_geom(screen: Screen, n: usize) -> Geometry {
    if n > 1 {
        // There are slaves
        Geometry {
            x: 0,
            y: 0,
            width: (screen.width / 2) as c_uint,
            height: screen.height,
        }
    } else {
        screen.to_geometry()
    }
}
/// Return the geometry for the i-th slave (of n slaves) on the given screen
fn get_slave_geom(screen: Screen, i: usize, n: usize) -> Geometry {
    let nn = n as c_uint;
    let ii = i as c_uint;

    Geometry {
        x: (screen.width / 2) as c_int,
        y: ((screen.height / nn) * ii) as c_int,
        width: screen.width / 2,
        height: (screen.height / nn) as c_uint,
    }
}

impl WindowManager for TilingWM {
    /// We use `WMError` as our `Error` type.
    type Error = WMError;

    fn new(screen: Screen) -> TilingWM {
        TilingWM {
            windows: Vec::new(),
            screen: screen,
            focused_index: None,
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        // Return a clone so external users can't access the original Vector
        self.windows.clone()
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        if !self.is_managed(window_with_info.window) {
            self.windows.push(window_with_info.window);
            // Focus on this new window
            self.focused_index = Some(self.windows.len() - 1);
        }

        Ok(())
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        self.windows
            .iter()
            .position(|w| *w == window)
            .ok_or(UnknownWindow(window))
            .map(|i| {
                self.windows.remove(i);

                // if there is no window left, no window has focus.
                if self.windows.len() == 0 {
                    self.focused_index = None;
                } else if let Some(j) = self.focused_index {
                    if i <= j {
                        // Update the index of the focused window to keep the same window in focus
                        self.cycle_focus(Prev);
                    }
                }
            })
    }

    fn get_window_layout(&self) -> WindowLayout {
        // Only the focused window can be visible
        if self.windows.len() == 0 {
            WindowLayout::new()
        } else {
            WindowLayout {
                focused_window: self.focused_index.map(|i| self.windows[i]),
                windows: self.windows
                    .iter()
                    .enumerate()
                    .map(|(i, w)| (*w, get_geom(self.screen, i, self.windows.len())))
                    .collect(),
            }
        }
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        match window {
            None => self.focused_index = None,
            Some(w) => {
                if !self.is_managed(w) {
                    return Err(UnknownWindow(w));
                }

                // Set focused index to the position of the window passed along
                self.focused_index = self.windows.iter().position(|w2| *w2 == w);
            }
        }

        Ok(())
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focused_index = match self.focused_index {
            None => {
                // Set focused_index to 0 unless there are no windows
                self.windows.first().map(|_w| 0)
            }
            Some(i) => {
                match dir {
                    Prev => Some((i + self.windows.len() - 1) % self.windows.len()),
                    Next => Some((i + 1) % self.windows.len()),
                }
            }
        }
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        self.windows.iter().position(|w| *w == window)
            // Return error if the window is not managed by us
            .ok_or(UnknownWindow(window))
            .map(|i| get_geom(self.screen, i, self.windows.len()))
            .map(|geom| WindowWithInfo {
                window: window,
                geometry: geom,
                float_or_tile: Tile,
                fullscreen: self.windows.len() == 1
            })
    }

    fn get_screen(&self) -> Screen {
        self.screen
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.screen = screen;
    }

    fn get_focused_window(&self) -> Option<Window> {
        self.focused_index.map(|i| self.windows[i])
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
#[allow(unused_mut)]
#[allow(unused_variables)]
mod tests {
    pub use super::*;

    pub use cplwm_api::wm::WindowManager;
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

            let mut wm = TilingWM::new(screen);
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
                expect!(wl.windows).to(be_equal_to(expected_windows));
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

            it "should not lose focus if we remove another window" {
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

            it "should do be in initial state if we remove all windows" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

                wm.remove_window(1).unwrap();
                wm.remove_window(2).unwrap();

                expect!(wm.is_managed(1)).to(be_false());
                expect!(wm.is_managed(2)).to(be_false());
                expect(wm.get_windows().len()).to(be_equal_to(0));

                expect!(wm.get_window_layout()).to(be_equal_to(WindowLayout::new()));
            }
        }

        describe! focus_window {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
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
                expect!(wm.focus_window(Some(3))).to(be_err());
            }
        }

        describe! cycle_focus {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();
            }

            it "should cycle in forward direction" {
                wm.cycle_focus(Next);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
            }

            it "should work in backward direction" {
                wm.cycle_focus(Prev);

                expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
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
        }

        describe! get_window_info {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
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

                expect(info).to(be_err());
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

            it "should work with 3 windows" {
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
        }

        describe! screen {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

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
                expect(wm.get_screen()).to(be_equal_to(screen));
            }

            it "should return the new screen if one is provided" {
                wm.resize_screen(new_screen);

                expect(wm.get_screen()).to(be_equal_to(new_screen));
            }

            it "should use the new screen in get_window_layout if there is 1 window" {
                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();

                expect(wl.windows.first().unwrap().1).to(be_equal_to(new_screen.to_geometry()));
            }

            it "should use the new screen in get_window_layout if there are more windows" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();
                let expected = vec![(1, left_half),
                                    (2, right_upper_quarter),
                                    (3,right_lower_quarter)];
                expect(wl.windows).to(be_equal_to(expected));
            }
        }
    }
}
