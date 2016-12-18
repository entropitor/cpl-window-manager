//! Optional: Gaps
//!
//! Extend your window manager with support for gaps, i.e. the ability to add
//! some space between the different tiles. See the documentation of the
//! [`GapSupport`] trait for the precise requirements.
//!
//! Make a copy of your tiling window manager from assignment B and let it
//! implement the [`GapSupport`] trait. You are not required to let this
//! window manager implement all the previous traits.
//!
//! [`GapSupport`]: ../../cplwm_api/wm/trait.GapSupport.html
//!
//! # Status
//!
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! The tests were copied from assignment b except for the geometries
//!

use std::os::raw::{c_int};
use cplwm_api::types::{GapSize, Geometry, Screen};
pub use cplwm_api::types::FloatOrTile::*;
use cplwm_api::wm::GapSupport;

use layouter::Layouter;
use layouter::GapSupport as GenericGapSupport;
use b_tiling_wm::{TilingWM, SimpleLayouter};

/// Type alias for automated tests
pub type WMName = TilingWM<GappedLayouter<SimpleLayouter>>;

/// The struct for a simple tiled layouter with gaps
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct GappedLayouter<WrappedLayouter: Layouter> {
    /// The size of the gap
    pub gap_size: GapSize,
    /// The wrapped layouter
    pub wrapped_layouter: WrappedLayouter
}

impl<WrappedLayouter: Layouter> Layouter for GappedLayouter<WrappedLayouter> {
    fn get_geom(&self, i: usize, screen: Screen, nb_windows: usize) -> Geometry {
        let geom = self.wrapped_layouter.get_geom(i, screen, nb_windows);

        let signed_gap = self.gap_size as c_int;

        Geometry {
            x: geom.x + signed_gap,
            y: geom.y + signed_gap,
            width: geom.width - 2 * self.gap_size,
            height: geom.height - 2 * self.gap_size,
        }
    }

    fn new() -> GappedLayouter<WrappedLayouter> {
        GappedLayouter {
            gap_size: 0,
            wrapped_layouter: WrappedLayouter::new()
        }
    }
}

impl<WrappedLayouter: Layouter>  GenericGapSupport for GappedLayouter<WrappedLayouter> {
    fn get_gap(&self) -> GapSize {
        self.gap_size
    }

    fn set_gap(&mut self, gapsize: GapSize) {
        self.gap_size = gapsize;
    }
}

impl<MyLayouter: GenericGapSupport + Layouter> GapSupport for TilingWM<MyLayouter> {
    fn get_gap(&self) -> GapSize {
        self.layouter.get_gap()
    }

    fn set_gap(&mut self, gapsize: GapSize) {
        self.layouter.set_gap(gapsize)
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
            let screen_geom = Geometry {
                x: 10,
                y: 10,
                width: 780,
                height: 580,
            };

            let some_geom: Geometry = Geometry {
                x: 10,
                y: 10,
                width: 100,
                height: 100,
            };

            let left_half = Geometry {
                x: 10,
                y: 10,
                width: 380,
                height: 580,
            };
            let right_half = Geometry {
                x: 410,
                y: 10,
                width: 380,
                height: 580,
            };

            let right_upper_quarter = Geometry {
                x: 410,
                y: 10,
                width: 380,
                height: 280,
            };
            let right_lower_quarter = Geometry {
                x: 410,
                y: 310,
                width: 380,
                height: 280,
            };

            let mut wm: WMName = TilingWM::new(screen);
            wm.set_gap(10);
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
                let new_screen_geom = Geometry {
                    x: 10,
                    y: 10,
                    width: 180,
                    height: 180,
                };

                let left_half = Geometry {
                    x: 10,
                    y: 10,
                    width: 80,
                    height: 180,
                };
                let right_upper_quarter = Geometry {
                    x: 110,
                    y: 10,
                    width: 80,
                    height: 80,
                };
                let right_lower_quarter = Geometry {
                    x: 110,
                    y: 110,
                    width: 80,
                    height: 80,
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

                expect!(wl.windows.first().unwrap().1).to(be_equal_to(new_screen_geom));
            }

            it "should use the new screen in get_window_layout if there are more windows" {
                wm.add_window(WindowWithInfo::new_tiled(2, some_geom)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, some_geom)).unwrap();

                wm.resize_screen(new_screen);

                let wl = wm.get_window_layout();
                let expected = vec![(1, left_half),
                                    (2, right_upper_quarter),
                                    (3,right_lower_quarter)];
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
                    let windows = vec![(2, left_half),
                                       (1, right_upper_quarter),
                                       (3, right_lower_quarter)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should focus the master tile if it is already the master window" {
                    wm.swap_with_master(1).unwrap();

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
                    expect!(wm.get_master_window()).to(be_equal_to(Some(1)));
                    let windows = vec![(1, left_half),
                                       (2, right_upper_quarter),
                                       (3, right_lower_quarter)];
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

                it "should be able to swap the focussed window with another window in forward direction" {
                    wm.focus_window(Some(2)).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    let windows = vec![(1, left_half),
                                       (3, right_upper_quarter),
                                       (2, right_lower_quarter)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should be able to swap the focussed window with another window in backward direction" {
                    wm.focus_window(Some(2)).unwrap();

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(2)));
                    let windows = vec![(2, left_half),
                                       (1, right_upper_quarter),
                                       (3, right_lower_quarter)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should cycle the swap in forward direction" {
                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(3, left_half),
                                       (2, right_upper_quarter),
                                       (1, right_lower_quarter)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "should cycle the swap in backward direction" {
                    wm.focus_window(Some(1)).unwrap();

                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(1)));
                    let windows = vec![(3, left_half),
                                       (2, right_upper_quarter),
                                       (1, right_lower_quarter)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if there is no focused window" {
                    wm.focus_window(None).unwrap();

                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(None));
                    let windows = vec![(1, left_half),
                                       (2, right_upper_quarter),
                                       (3, right_lower_quarter)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if calling swap in 2 opposite directions" {
                    wm.swap_windows(Prev);
                    wm.swap_windows(Next);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(1, left_half),
                                       (2, right_upper_quarter),
                                       (3, right_lower_quarter)];
                    expect!(wm.get_window_layout().windows).to(be_equal_to(windows));
                }

                it "shouldn't do anything if calling the swap twice and cycling in between" {
                    wm.swap_windows(Next);
                    wm.swap_windows(Prev);

                    expect!(wm.get_focused_window()).to(be_equal_to(Some(3)));
                    let windows = vec![(1, left_half),
                                       (2, right_upper_quarter),
                                       (3, right_lower_quarter)];
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

        describe! gap_support {
            it "should be able to retrieve the gap" {
                expect!(wm.get_gap()).to(be_equal_to(10));
            }

            it "should be able to set a new gap" {
                wm.set_gap(20);

                expect!(wm.get_gap()).to(be_equal_to(20));
            }

            it "should use the new gap in the layout" {
                wm.add_window(WindowWithInfo::new_tiled(1, some_geom)).unwrap();

                wm.set_gap(20);

                expect!(wm.get_window_info(1).unwrap().geometry).to(be_equal_to(Geometry {
                    x: 20,
                    y: 20,
                    width: 760,
                    height: 560,
                }));
            }
        }
    }
}
