//! Fullscreen Window Manager
//!
//! Implement the [`WindowManager`] trait by writing a simple window manager
//! that displays every window fullscreen. When a new window is added, the
//! last window that was visible will become invisible.
//!
//! [`WindowManager`]: ../../cplwm_api/wm/trait.WindowManager.html
//!
//! # Status
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! I used an index to track which is the focussed window.
//!
use cplwm_api::types::{PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::types::PrevOrNext::*;
pub use cplwm_api::types::FloatOrTile::*;
use cplwm_api::wm::WindowManager;

use error::WMError;
use error::WMError::*;

/// Type alias for automated tests
pub type WMName = FullscreenWM;

/// Main struct of the window manager
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FullscreenWM {
    /// A vector of windows, the first one is on the bottom, the last one is on top
    pub windows: Vec<Window>,
    /// The screen that is managed
    pub screen: Screen,
    /// The index of the focused window (or None if no window is focussed)
    pub focused_index: Option<usize>,
}

impl WindowManager for FullscreenWM {
    /// We use `WMError` as our `Error` type.
    type Error = WMError;

    fn new(screen: Screen) -> FullscreenWM {
        FullscreenWM {
            windows: Vec::new(),
            screen: screen,
            focused_index: None,
        }
    }

    fn get_windows(&self) -> Vec<Window> {
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
                };
            })
    }

    fn get_window_layout(&self) -> WindowLayout {
        // Only the focused window can be visible
        self.focused_index.map(|i| self.windows[i]).map_or(WindowLayout::new(), |w| {
            WindowLayout {
                focused_window: Some(w),
                windows: vec![(w, self.screen.to_geometry())],
            }
        })
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
            // Check if it's in focus
            .map(|_i| WindowWithInfo {
                window: window,
                geometry: self.screen.to_geometry(),
                float_or_tile: Tile,
                fullscreen: false
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
#[allow(unused_mut)]
#[allow(unused_variables)]
mod tests {
    pub use super::*;

    pub use cplwm_api::wm::WindowManager;
    pub use cplwm_api::types::*;
    pub use cplwm_api::types::PrevOrNext::*;

    // Import expectest names
    pub use expectest::prelude::*;

    describe! full_screen_wm {
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

            let mut wm = FullscreenWM::new(screen);
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

            it "should add 2 windows correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

                let wl = wm.get_window_layout();

                expect!(wm.is_managed(2)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2]));
                expect!(wl.focused_window).to(be_equal_to(Some(2)));
                expect!(wl.windows).to(be_equal_to(vec![(2, screen_geom)]));
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
                expect!(wl.windows).to(be_equal_to(vec![(3, screen_geom)]));
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
        }

        describe! get_window_info {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
            }

            it "should work for the visible window" {
                let info = wm.get_window_info(2).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: screen_geom,
                    float_or_tile: Tile,
                    fullscreen: false,
                }));
            }

            it "should work for a hidden window" {
                let info = wm.get_window_info(1).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 1,
                    geometry: screen_geom,
                    float_or_tile: Tile,
                    fullscreen: false,
                }));
            }

            it "should work if there is no visible window" {
                wm.focus_window(None).unwrap();

                let info = wm.get_window_info(2).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: screen_geom,
                    float_or_tile: Tile,
                    fullscreen: false,
                }));
            }

            it "should error if the window is not managed by the window manager" {
                let info = wm.get_window_info(3);

                expect(info).to(be_err());
            }
        }

        describe! screen {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

                let new_screen = Screen {
                    width: 200,
                    height: 200
                };
            }
            it "should return the default screen"{
                expect(wm.get_screen()).to(be_equal_to(screen));
            }

            it "should return the new screen if one is provided" {
                wm.resize_screen(new_screen);

                expect(wm.get_screen()).to(be_equal_to(new_screen));
            }

            it "should change the windowlayout of the visible screen if a new screen is provided" {
                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();
                expect(wl.windows.first().unwrap().1).to(be_equal_to(new_screen.to_geometry()));
            }
        }
    }
}
