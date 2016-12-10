//! Optional: Different Tiling Layout
//!
//! Come up with a different tiling layout algorithm than the one you have
//! already implemented. If you are uninspired, feel free to look for one on
//! the [internet], but *don't forget to mention where you found it*. The
//! layout algorithm *may not be trivial*, e.g., not just adding tiles by
//! splitting the screen horizontally, and must be at least as complex as, but
//! different enough from the original layout algorithm you already had to
//! implement.
//!
//! Make a copy of your tiling window manager that implements the tiling
//! layout algorithm. This window manager has to implement the
//! [`WindowManager`] trait, but *not necessarily* the [`TilingSupport`]
//! trait, as not every layout has a master tile. Feel free to add additional
//! methods to your window manager that can be used to manipulate its layout.
//! You are not required to let this window manager implement all the previous
//! traits.
//!
//! [internet]: http://xmonad.org/xmonad-docs/xmonad-contrib/XMonad-Doc-Extending.html
//! [`WindowManager`]: ../../cplwm_api/wm/trait.WindowManager.html
//! [`TilingSupport`]: ../../cplwm_api/wm/trait.TilingSupport.html
//!
//! # Status
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! A spiral layout where every time the next tile has half the area of the previous tile
//!
//! Source layout: http://xmonad.org/xmonad-docs/xmonad-contrib/XMonad-Layout-Spiral.html

use std::os::raw::{c_int};
use cplwm_api::types::{Geometry, Screen};
pub use cplwm_api::types::FloatOrTile::*;

use layouter::Layouter;
use b_tiling_wm::TilingWM;

/// Type alias for automated tests
pub type WMName = TilingWM<SpiralLayouter>;

/// The struct for a simple tiled layouter with gaps
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct SpiralLayouter {
}

impl Layouter for SpiralLayouter {
    fn get_geom(&self, i: usize, screen: Screen, nb_windows: usize) -> Geometry {
        self.get_geom_for_window(screen.to_geometry(), 0, i, nb_windows)
    }

    fn new() -> SpiralLayouter {
        SpiralLayouter {
        }
    }
}

impl SpiralLayouter {
    /// Get the geometry for the window with index i (if there are nb_windows) if rest_screen is the geometry of the screen that is left after tiling nb_windows_tiled.
    fn get_geom_for_window(&self, rest_screen: Geometry, nb_windows_tiled: usize, i: usize, nb_windows: usize) -> Geometry {
        // If this is the last window to be tiled, we don't need to split it anymore
        if nb_windows_tiled + 1 == nb_windows {
            return rest_screen;
        }

        // Calculate split based on which direction of the spiral
        let (geom_for_next_tile, new_rest_screen) = match (nb_windows_tiled) % 4 {
            0 => {
                self.split_vertically(rest_screen)
            },
            1 => {
                self.split_horizontally(rest_screen)
            },
            2 => {
                let (a, b) = self.split_vertically(rest_screen);
                (b, a)
            },
            3 => {
                let (a, b) = self.split_horizontally(rest_screen);
                (b, a)
            },
            // unreachable branch but Rust doesn't know this
            _ => { (rest_screen, rest_screen) }
        };

        if nb_windows_tiled == i {
            geom_for_next_tile
        } else {
            self.get_geom_for_window(new_rest_screen, nb_windows_tiled + 1, i, nb_windows)
        }
    }

    fn split_vertically(&self, geom: Geometry) -> (Geometry, Geometry) {
        (Geometry {
            x: geom.x,
            y: geom.y,
            width: geom.width / 2,
            height: geom.height,
        },Geometry {
            x: geom.x + (geom.width / 2) as c_int,
            y: geom.y,
            width: geom.width / 2,
            height: geom.height,
        })
    }

    fn split_horizontally(&self, geom: Geometry) -> (Geometry, Geometry) {
        (Geometry {
            x: geom.x,
            y: geom.y,
            width: geom.width,
            height: geom.height / 2,
        },Geometry {
            x: geom.x,
            y: geom.y + (geom.height / 2) as c_int,
            width: geom.width,
            height: geom.height / 2,
        })
    }
}

#[cfg(test)]
#[allow(unused_mut)]
#[allow(unused_variables)]
mod tests {
    pub use super::*;
    pub use b_tiling_wm::TilingWM;

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

            let master_geom = Geometry {
                x: 0, y: 0,
                width: screen.width/2,
                height: screen.height,
            };
            let slave_1_of_1 = Geometry {
                x: (screen.width/2) as c_int,
                y: 0,
                width: screen.width/2,
                height: screen.height,
            };

            let slave_1_of_2 = Geometry {
                x: (screen.width/2) as c_int,
                y: 0,
                width: screen.width/2,
                height: screen.height/2,
            };
            let slave_2_of_2 = Geometry {
                x: (screen.width/2) as c_int,
                y: (screen.height/2) as c_int,
                width: screen.width/2,
                height: screen.height/2,
            };

            let slave_1_of_5 = Geometry {
                x: 400,
                y: 0,
                width: 400,
                height: 300,
            };
            let slave_2_of_5 = Geometry {
                x: 600,
                y: 300,
                width: 200,
                height: 300,
            };
            let slave_3_of_5 = Geometry {
                x: 400,
                y: 450,
                width: 200,
                height: 150,
            };
            let slave_4_of_5 = Geometry {
                x: 400,
                y: 300,
                width: 100,
                height: 150,
            };
            let slave_5_of_5 = Geometry {
                x: 500,
                y: 300,
                width: 100,
                height: 150,
            };

            let mut wm: WMName = TilingWM::new(screen);
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
                expect!(wl.windows).to(be_equal_to(vec![(1, master_geom),(2, slave_1_of_1)]));
            }

            it "should add 3 windows correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                let wl = wm.get_window_layout();
                let windows = vec![(1, master_geom),
                                   (2, slave_1_of_2),
                                   (3, slave_2_of_2)];

                expect!(wm.is_managed(3)).to(be_true());
                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2, 3]));
                expect!(wl.focused_window).to(be_equal_to(Some(3)));
                expect!(wl.windows).to(be_equal_to(windows));
            }

            it "should add 6 windows correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(4, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(5, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(6, some_geom)).unwrap();

                let wl = wm.get_window_layout();
                let windows = vec![(1, master_geom),
                                   (2, slave_1_of_5),
                                   (3, slave_2_of_5),
                                   (4, slave_3_of_5),
                                   (5, slave_4_of_5),
                                   (6, slave_5_of_5)];

                expect!(wm.get_windows()).to(be_equal_to(vec![1, 2, 3, 4, 5, 6]));
                expect!(wl.windows).to(be_equal_to(windows));
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
                expect!(wl.windows).to(be_equal_to(vec![(1, master_geom),(3, slave_1_of_1)]));
            }

            it "should do be in initial state if we remove all windows" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();

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

            it "should work for the slave window" {
                let info = wm.get_window_info(2).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: slave_1_of_1,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should work for a master window" {
                let info = wm.get_window_info(1).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 1,
                    geometry: master_geom,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
            }

            it "should work if there is no focused window" {
                wm.focus_window(None).unwrap();

                let info = wm.get_window_info(2).unwrap();

                expect!(info).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: slave_1_of_1,
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

            it "should work with 3 windows" {
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                expect!(wm.get_window_info(1).unwrap()).to(be_equal_to(WindowWithInfo {
                    window: 1,
                    geometry: master_geom,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
                expect!(wm.get_window_info(2).unwrap()).to(be_equal_to(WindowWithInfo {
                    window: 2,
                    geometry: slave_1_of_2,
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: false,
                }));
                expect!(wm.get_window_info(3).unwrap()).to(be_equal_to(WindowWithInfo {
                    window: 3,
                    geometry: slave_2_of_2,
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

                let master_geom = Geometry {
                    x: 0, y: 0,
                    width: new_screen.width/2,
                    height: new_screen.height,
                };
                let slave_1_of_2 = Geometry {
                    x: (new_screen.width/2) as c_int,
                    y: 0,
                    width: new_screen.width/2,
                    height: new_screen.height/2,
                };
                let slave_2_of_2 = Geometry {
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

            it "should use the new screen in get_window_layout if there is 1 window" {
                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();

                expect!(wl.windows.first().unwrap().1).to(be_equal_to(new_screen.to_geometry()));
            }

            it "should use the new screen in get_window_layout if there are more windows" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();
                let expected = vec![(1, master_geom),
                                    (2, slave_1_of_2),
                                    (3,slave_2_of_2)];
                expect!(wl.windows).to(be_equal_to(expected));
            }
        }

        describe! tiling_support {
            describe! get_master_window {
                it "should return the master window if there is one" {
                    wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

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
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                }

                it "should be able to swap with master" {
                    wm.swap_with_master(2).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(2)));
                    let windows = vec![(2, master_geom),
                                       (1, slave_1_of_2),
                                       (3, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should focus the master tile if it is already the master window" {
                    wm.swap_with_master(1).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(1)));
                    let windows = vec![(1, master_geom),
                                       (2, slave_1_of_2),
                                       (3, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should error if the window is not managed by the wm" {
                    expect!(wm.swap_with_master(4)).to(be_err());
                }
            }

            describe! swap_windows {
                before_each {
                    wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                    wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();
                }

                it "should be able to swap the focused window with another window in forward direction" {
                    wm.focus_window(Some(2)).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    let windows = vec![(1, master_geom),
                                       (3, slave_1_of_2),
                                       (2, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should be able to swap the focused window with another window in backward direction" {
                    wm.focus_window(Some(2)).unwrap();

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    let windows = vec![(2, master_geom),
                                       (1, slave_1_of_2),
                                       (3, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should cycle the swap in forward direction" {
                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(3, master_geom),
                                       (2, slave_1_of_2),
                                       (1, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should cycle the swap in backward direction" {
                    wm.focus_window(Some(1)).unwrap();

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
                    let windows = vec![(3, master_geom),
                                       (2, slave_1_of_2),
                                       (1, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if there is no focused window" {
                    wm.focus_window(None).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(None));
                    let windows = vec![(1, master_geom),
                                       (2, slave_1_of_2),
                                       (3, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if calling swap in 2 opposite directions" {
                    wm.swap_windows(Prev);
                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(1, master_geom),
                                       (2, slave_1_of_2),
                                       (3, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if calling the swap twice and cycling in between" {
                    wm.swap_windows(Next);
                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(1, master_geom),
                                       (2, slave_1_of_2),
                                       (3, slave_2_of_2)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if calling with only one window" {
                    wm.remove_window(1).unwrap();
                    wm.remove_window(2).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(3, screen_geom)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }
            }
        }
    }
}
