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
//! ...
//!

use cplwm_api::types::{PrevOrNext, FloatOrTile, Geometry, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::{WindowManager};

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
        match self.windows.iter().position(|w| *w == window) {
            None => Err(UnknownWindow(window)),
            Some(i) => {
                self.windows.remove(i);

                if self.windows.len() == 0 {
                    // if there is no window left, no window has focus.
                    self.focused_index = None;
                } else if let Some(j) = self.focused_index {
                    if i <= j {
                        // Update the index of the focused window to keep the same window in focus
                        self.cycle_focus(PrevOrNext::Prev);
                    }
                }

                Ok(())
            }
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        let fullscreen_geometry = self.screen.to_geometry();

        // Only the focused window can be visible
        match self.focused_index {
            Some(i) => {
                let w = self.windows[i];
                WindowLayout {
                    focused_window: Some(w),
                    windows: vec![(w, fullscreen_geometry)],
                }
            }
            None => WindowLayout::new(),
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
            },
            Some(i) => {
                match dir {
                    PrevOrNext::Prev => Some((i + self.windows.len() - 1) % self.windows.len()),
                    PrevOrNext::Next => Some((i + 1) % self.windows.len()),
                }
            }
        }
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        match self.windows.iter().position(|w| *w == window) {
            None => {
                // Return error if the window is not managed by us
                return Err(UnknownWindow(window))
            }
            Some(i) => {
                // If it's in focus, return fullscreen window info
                if let Some(j) = self.focused_index {
                    if i == j {
                        return Ok(WindowWithInfo {
                            window: window,
                            geometry: self.screen.to_geometry(),
                            float_or_tile: FloatOrTile::Tile,
                            fullscreen: true
                        });
                    }
                }

                // Otherwise return "hidden" window info
                Ok(WindowWithInfo {
                    window: window,
                    geometry: Geometry { x: 0, y: 0, height: 0, width: 0 },
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false
                })
            }
        }
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
                x: (screen.width/2) as i32,
                y: 0,
                width: screen.width/2,
                height: screen.height,
            };

            let right_upper_quarter = Geometry {
                x: (screen.width/2) as i32,
                y: 0,
                width: screen.width/2,
                height: screen.height/2,
            };
            let right_lower_quarter = Geometry {
                x: (screen.width/2) as i32,
                y: (screen.height/2) as i32,
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

                expect!(wm.is_managed(3)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2, 3]));
                expect!(wl.focused_window).to(be_equal_to(Some(3)));
                expect!(wl.windows).to(be_equal_to(vec![(1, left_half),(2, right_upper_quarter),(3, right_lower_quarter)]));
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
                expect!(wm.get_window_layout().windows.len()).to(be_equal_to(0));
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
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

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
                    x: (new_screen.width/2) as i32,
                    y: 0,
                    width: new_screen.width/2,
                    height: new_screen.height/2,
                };
                let right_lower_quarter = Geometry {
                    x: (new_screen.width/2) as i32,
                    y: (new_screen.height/2) as i32,
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

                let wl = wm.get_window_layout();

                expect(wl.windows).to(be_equal_to(vec![(1, left_half),(2, right_upper_quarter),(3,right_lower_quarter)]));
            }
        }
    }
}
