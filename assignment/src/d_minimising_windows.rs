//! Minimising Windows
//!
//! Extend your window manager with support for (un)minimising windows. i.e.
//! the ability to temporarily hide windows and to reveal them again later.
//! See the documentation of the [`MinimiseSupport`] trait for the precise
//! requirements.
//!
//! Either make a copy of the tiling window manager with support for floating
//! windows you developed in the previous assignment and let it implement the
//! [`MinimiseSupport`] trait as well, or implement this trait by building a
//! wrapper around the previous window manager. Note that this window manager
//! must still implement all the traits from previous assignments.
//!
//! [`MinimiseSupport`]: ../../cplwm_api/wm/trait.MinimiseSupport.html
//!
//! # Status
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! A lot of tests were copied and adapted from the c_floating_windows
//!

use cplwm_api::types::{Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo, GapSize};
use cplwm_api::wm::{FloatSupport, GapSupport, MinimiseSupport, TilingSupport, WindowManager};
use std::collections::HashMap;

use c_floating_windows::WMName as FloatWM;

/// Type alias for automated tests
pub type WMName = MinimisingWM<FloatWM>;

/// Main struct of the window manager
/// This WM can minimise windows and uses the WrappedWM for all unminimised windows
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct MinimisingWM<WrappedWM: WindowManager> {
    /// A vector of minimised windows
    pub minimised_windows: Vec<Window>,
    /// The wrapped window manager that takes care of the unminimised windows
    pub wrapped_wm: WrappedWM,
    /// The window_with_info's for the minimised windows
    pub infos: HashMap<Window, WindowWithInfo>,
}

impl<WrappedWM: WindowManager> WindowManager for MinimisingWM<WrappedWM> {
    /// We use the Error from the WrappedWM as our Error type.
    type Error = WrappedWM::Error;

    fn new(screen: Screen) -> MinimisingWM<WrappedWM> {
        MinimisingWM {
            minimised_windows: Vec::new(),
            wrapped_wm: WrappedWM::new(screen),
            infos: HashMap::new(),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        let mut windows = self.wrapped_wm.get_windows();
        windows.extend(self.get_minimised_windows());

        windows
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        if !self.is_minimised(window_with_info.window) {
            self.wrapped_wm.add_window(window_with_info)
        } else {
            Ok(())
        }
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        if !self.is_minimised(window) {
            self.wrapped_wm.remove_window(window)
        } else {
            self.infos.remove(&window);

            self.minimised_windows
                .iter()
                .position(|w| *w == window)
                .map(|i| {
                    self.minimised_windows.remove(i);
                });

            Ok(())
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        self.wrapped_wm.get_window_layout()
    }

    /// If the window is minimized, it's first unminimized
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        if let Some(w) = window {
            if self.is_minimised(w) {
                try!(self.toggle_minimised(w));
            }
        }

        self.wrapped_wm.focus_window(window)
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.wrapped_wm.cycle_focus(dir)
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        if self.is_minimised(window) {
            // if the window is minimised, it's part of the infos hashmap
            Ok(self.infos.get(&window).map(|info| *info).unwrap())
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
        self.wrapped_wm.get_focused_window()
    }

    fn is_managed(&self, window: Window) -> bool {
        self.is_minimised(window) || self.wrapped_wm.is_managed(window)
    }
}

impl<WrappedWM: TilingSupport> TilingSupport for MinimisingWM<WrappedWM> {
    fn get_master_window(&self) -> Option<Window> {
        self.wrapped_wm.get_master_window()
    }

    /// If the passed window is a minimised window, it will first be unminimized
    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error> {
        if self.is_minimised(window) {
            try!(self.toggle_minimised(window));
        }

        self.wrapped_wm.swap_with_master(window)
    }

    fn swap_windows(&mut self, dir: PrevOrNext) {
        self.wrapped_wm.swap_windows(dir)
    }
}

impl<WrappedWM: FloatSupport> FloatSupport for MinimisingWM<WrappedWM> {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.wrapped_wm.get_floating_windows()
    }

    /// If the passed window is a minimised window, it will first be unminimized
    /// If the window was not floating before it was minimised, it will now float at
    /// the location that it was tiled before it was minimised.
    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error> {
        if self.is_minimised(window) {
            try!(self.toggle_minimised(window));
        }

        self.wrapped_wm.toggle_floating(window)
    }

    /// If the passed window is a minimised window, it will remember the geometry for when this window becomes unminimised again
    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error> {
        if self.is_minimised(window) {
            // self.infos always contains an entry for minimised windows
            let mut wi = self.infos.get(&window).unwrap().clone();
            wi.geometry = new_geometry;
            self.infos.insert(window, wi);
            Ok(())
        } else {
            self.wrapped_wm.set_window_geometry(window, new_geometry)
        }
    }
}

impl<WrappedWM: WindowManager> MinimiseSupport for MinimisingWM<WrappedWM> {
    fn get_minimised_windows(&self) -> Vec<Window> {
        self.minimised_windows.clone()
    }
    fn toggle_minimised(&mut self, window: Window) -> Result<(), Self::Error> {
        let wi = try!(self.get_window_info(window));
        let was_minimised = self.is_minimised(window);

        if was_minimised {
            try!(self.remove_window(window));
            self.add_window(wi)
        } else {
            try!(self.wrapped_wm.remove_window(window));

            self.infos.insert(window, wi);
            self.minimised_windows.push(window);
            Ok(())
        }
    }

    fn is_minimised(&self, window: Window) -> bool {
        self.infos.contains_key(&window)
    }
}

impl<WrappedWM: GapSupport> GapSupport for MinimisingWM<WrappedWM> {
    fn get_gap(&self) -> GapSize {
        self.wrapped_wm.get_gap()
    }

    fn set_gap(&mut self, gapsize: GapSize) {
        self.wrapped_wm.set_gap(gapsize)
    }
}

#[cfg(test)]
#[allow(unused_mut)]
#[allow(unused_variables)]
mod tests {
    pub use super::*;
    pub use c_floating_windows::WMName as FloatingWM;

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

            let mut wm: MinimisingWM<FloatingWM> = MinimisingWM::new(screen);
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
                    wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();

                    let master = wm.get_master_window();

                    expect!(master).to(be_equal_to(Some(1)));
                }

                it "should return none if there is no master window" {
                    wm.add_window(WindowWithInfo::new_float(5, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();
                    wm.toggle_minimised(6).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(7, some_geom)).unwrap();
                    wm.toggle_minimised(7).unwrap();

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
                    expect!(wm.get_window_layout().windows.iter().find(|&&(w, _geom)| w == 3).unwrap().1).to(be_equal_to(right_lower_quarter));
                }

                it "should be able to toggle floating off for minimised windows" {
                    wm.toggle_floating(6).unwrap();

                    expect!(wm.is_floating(6)).to(be_false());
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
