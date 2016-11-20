//! Fullscreen Window Manager
//!
//! Implement the [`WindowManager`] trait by writing a simple window manager
//! that displays every window fullscreen. When a new window is added, the
//! last window that was visible will become invisible.
//!
//! [`WindowManager`]: ../../cplwm_api/wm/trait.WindowManager.html
//!
//! Now have a look at the source code of this file, it contains a tutorial to
//! help you write the fullscreen window manager.
//!
//! You are free to remove the documentation in this file that is only part of
//! the tutorial or no longer matches the code after your changes.
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
//! ...
//!

// Because not all methods are implemented yet, some arguments are unused,
// which generates warnings. The annotation below disables this warning.
// Remove this annotation when you have implemented all methods, so you get
// warned about variables that you did not use by mistake.
#![allow(unused_variables)]

// We import std::error and std::format so we can say error::Error instead of
// std::error::Error, etc.
use std::error;
use std::fmt;

// Import some types and the WindowManager trait from the cplwm_api crate
// (defined in the api folder).
use cplwm_api::types::{PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::WindowManager;

/// You are free to choose the name for your window manager. As we will use
/// automated tests when grading your assignment, indicate here the name of
/// your window manager data type so we can just use `WMName` instead of
/// having to manually figure out your window manager name.
pub type WMName = FullscreenWM;


/// The FullscreenWM struct
///
/// The first thing to do when writing a window manager, is to define a struct
/// (or enum) that will contain the state of the window manager, e.g. the
/// managed windows along with their geometries, the focused window, etc.
///
/// Depending on the layout and the functionality the window manager provides,
/// this can vary from simple `Vec`s to trees, hashmaps, etc. You can have a
/// look at the [collections](https://doc.rust-lang.org/std/collections/) Rust
/// provides.
///
/// Remember that you are free to add additional dependencies to your project,
/// e.g., for another type of data structure. But this is certainly not
/// required. For more information, see the Hints & Tricks section of the
/// assignment.
///
/// # Example Representation
///
/// The fullscreen window manager that we are implementing is very simple: it
/// just needs to keep track of all the windows that were added and remember
/// which one is focused. It is not even necessary to remember the geometries
/// of the windows, as they will all be resized to the size of the screen.
///
/// A possible data structure to keep track of the windows is a simple `Vec`:
/// the last element in the vector is the window on top, which is also the
/// only window to display. Why not the first element? Because it is easier to
/// add an element to the end of a vector. This is convenient, as adding a new
/// window should also put it on top of the other windows.
///
/// Another thing we need to keep track of is the `Screen`, because we must
/// resize the windows the size of the screen. A `Screen` is passed via the
/// `new` method of the trait and the `resize_screen` method of the trait
/// updates the screen with a new one.
///
/// These two fields are enough to get started, which does not mean that they
/// are enough to correctly implement this window manager. As you will notice
/// in a short while, there is a problem with this representation. Feel free
/// to add/replace/remove fields.
///
/// To understand the `#derive[(..)]` line before the struct, read the
/// [Supertraits] section of the `WindowManager` trait.
///
/// [Supertraits]: ../../cplwm_api/wm/trait.WindowManager.html#supertraits
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FullscreenWM {
    /// A vector of windows, the first one is on the bottom, the last one is
    /// on top, and also the only visible window.
    pub windows: Vec<Window>,
    /// We need to know which size the fullscreen window must be.
    pub screen: Screen,
    /// The index of the focused window (or None if no window is focussed)
    pub focused_index: Option<usize>,
}

/// The errors that this window manager can return.
///
/// For more information about why you need this, read the documentation of
/// the associated [Error] type of the `WindowManager` trait.
///
/// In the code below, we would like to return an error when we are asked to
/// do something with a window that we do not manage, so we define an enum
/// `FullscreenWMError` with one variant: `UnknownWindow`.
///
/// Feel free to add or remove variants from this enum. You may also replace
/// it with a type or struct if you wish to do so.
///
/// [Error]: ../../cplwm_api/wm/trait.WindowManager.html#associatedtype.Error
#[derive(Debug)]
pub enum FullscreenWMError {
    /// This window is not known by the window manager.
    UnknownWindow(Window),
}

// This code is explained in the documentation of the associated [Error] type
// of the `WindowManager` trait.
impl fmt::Display for FullscreenWMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FullscreenWMError::UnknownWindow(ref window) => write!(f, "Unknown window: {}", window),
        }
    }
}

// This code is explained in the documentation of the associated [Error] type
// of the `WindowManager` trait.
impl error::Error for FullscreenWMError {
    fn description(&self) -> &'static str {
        match *self {
            FullscreenWMError::UnknownWindow(_) => "Unknown window",
        }
    }
}

// Now we start implementing our window manager
impl WindowManager for FullscreenWM {
    /// We use `FullscreenWMError` as our `Error` type.
    type Error = FullscreenWMError;

    /// The constructor is straightforward.
    ///
    /// Track the given screen and make a new empty `Vec`.
    fn new(screen: Screen) -> FullscreenWM {
        FullscreenWM {
            windows: Vec::new(),
            screen: screen,
            focused_index: None,
        }
    }

    /// The `windows` field contains all the windows we manage.
    ///
    /// Why do we need a `clone` here?
    fn get_windows(&self) -> Vec<Window> {
        self.windows.clone()
    }

    /// The last window in the list is the focused one.
    ///
    /// Note that the `last` method of `Vec` returns an `Option`.
    // fn get_focused_window(&self) -> Option<Window> {
    //    self.windows.last().map(|w| *w)
    // }
    /// To add a window, just push it onto the end the `windows` `Vec`.
    ///
    /// We could choose to return an error when the window is already managed
    /// by the window manager, but in this case we just do nothing. You are
    /// free to define another error to handle this case.
    ///
    /// Note that we completely ignore the information that comes along with
    /// the info, this *could* lead to issues in later assignments.
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        if !self.is_managed(window_with_info.window) {
            self.windows.push(window_with_info.window);
            self.focused_index = Some(self.windows.len() - 1);
        }
        Ok(())
    }

    /// To remove a window, just remove it from the `windows` `Vec`.
    ///
    /// First we look up the position (or index) of the window in `windows`,
    /// and then remove it unless the window does not occur in the `Vec`, in
    /// which case we return an error.
    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        match self.windows.iter().position(|w| *w == window) {
            None => Err(FullscreenWMError::UnknownWindow(window)),
            Some(i) => {
                self.windows.remove(i);

                if self.windows.len() == 0 {
                    self.focused_index = None;
                } else if let Some(j) = self.focused_index {
                    if i <= j {
                        self.cycle_focus(PrevOrNext::Prev);
                    }
                }

                Ok(())
            }
        }
    }

    /// Now the most important part: calculating the `WindowLayout`.
    ///
    /// First we build a `Geometry` for a fullscreen window using the
    /// `to_geometry` method: it has the same width and height as the screen.
    ///
    /// Then we look at the last window, remember that the `last()` method of
    /// `Vec` returns an `Option`.
    ///
    /// * When the `Option` contains `Some(w)`, we know that there was at
    ///   least one window, and `w`, being the last window in the `Vec` should
    ///   be focused. As the other windows will not be visible, the `windows`
    ///   field of `WindowLayout` can just be a `Vec` with one element: the
    ///   one window along with the fullscreen `Geometry`.
    ///
    /// * When the `Option` is `None`, we know that there are no windows, so
    ///   we can just return an empty `WindowLayout`.
    ///
    fn get_window_layout(&self) -> WindowLayout {
        let fullscreen_geometry = self.screen.to_geometry();
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

    // Before you continue any further, first have a look at the bottom of
    // this file, where we show you how to write unit tests.

    /// Try this yourself
    ///
    /// Don't forget that when the argument is `None`, i.e. no window should
    /// be focused, `get_focused_window()` must return `None` afterwards. The
    /// `focused_window` field of the `WindowLayout` must also be `None`.
    ///
    /// You will probably have to change the code above (method
    /// implementations as well as the `FullscreenWM` struct) to achieve this.
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        match window {
            None => self.focused_index = None,
            Some(w) => {
                if !self.is_managed(w) {
                    return Err(FullscreenWMError::UnknownWindow(w));
                }

                self.focused_index = self.windows.iter().position(|w2| *w2 == w);
            }
        }

        Ok(())
    }

    /// Try this yourself
    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focused_index = match self.focused_index {
            None => self.windows.first().map(|w| 0),
            Some(i) => {
                match dir {
                    PrevOrNext::Prev => Some((i + self.windows.len() - 1) % self.windows.len()),
                    PrevOrNext::Next => Some((i + 1) % self.windows.len()),
                }
            }
        }
    }

    /// Try this yourself
    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        unimplemented!()
    }

    /// Try this yourself
    fn get_screen(&self) -> Screen {
        unimplemented!()
    }

    /// Try this yourself
    fn resize_screen(&mut self, screen: Screen) {
        unimplemented!()
    }
}

// Here we define a submodule, called `tests`, that will contain the unit
// tests of this module.
//
// The `#[cfg(test)]` annotation means that this code is only compiled when
// we're testing the code.
#[cfg(test)]
#[allow(unused_must_use)]
#[allow(unused_mut)]
mod tests {

    // We have to import `FullscreenWM` from the super module.
    pub use super::FullscreenWM;
    // We have to repeat the imports we did in the super module.
    pub use cplwm_api::wm::WindowManager;
    pub use cplwm_api::types::*;
    pub use cplwm_api::types::PrevOrNext::*;

    // Import expectest names
    pub use expectest::prelude::*;

    // We define a static variable for the screen we will use in the tests.
    pub static SCREEN: Screen = Screen {
        width: 800,
        height: 600,
    };

    // We define a static variable for the geometry of a fullscreen window.
    // Note that it matches the dimensions of `SCREEN`.
    pub static SCREEN_GEOM: Geometry = Geometry {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    };

    // We define a static variable for some random geometry that we will use
    // when adding windows to a window manager.
    pub static SOME_GEOM: Geometry = Geometry {
        x: 10,
        y: 10,
        width: 100,
        height: 100,
    };

    describe! full_screen_wm {
        before_each {
            // Let's make a new `FullscreenWM` with `SCREEN` as screen.
            let mut wm = FullscreenWM::new(SCREEN);
        }

        it "should have an empty window layout initially" {
            assert_eq!(WindowLayout::new(), wm.get_window_layout());
        }

        it "should add a window correctly" {
            wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();

            // The window should now be managed by the WM
            assert!(wm.is_managed(1));
            // and be present in the `Vec` of windows.
            assert_eq!(vec![1], wm.get_windows());
            // According to the window layout
            let wl1 = wm.get_window_layout();
            // it should be focused
            assert_eq!(Some(1), wl1.focused_window);
            // and fullscreen.
            assert_eq!(vec![(1, SCREEN_GEOM)], wl1.windows);
        }

        it "should add 2 windows correctly" {
            wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();

            wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();

            // It should now be managed by the WM.
            assert!(wm.is_managed(2));
            // The `Vec` of windows should now contain both windows 1 and 2.
            assert_eq!(vec![1, 2], wm.get_windows());
            // According to the window layout
            let wl2 = wm.get_window_layout();
            // window 2 should be focused
            assert_eq!(Some(2), wl2.focused_window);
            // and fullscreen.
            assert_eq!(vec![(2, SCREEN_GEOM)], wl2.windows);
        }

        describe! remove_window {
            it "should remove a window correctly" {
                wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();

                wm.remove_window(2).unwrap();

                // It should no longer be managed by the WM.
                assert!(!wm.is_managed(2));
                // The `Vec` of windows should now just contain window 1.
                assert_eq!(vec![1], wm.get_windows());
                // According to the window layout
                let wl3 = wm.get_window_layout();
                // window 1 should be focused again
                assert_eq!(Some(1), wl3.focused_window);
                // and fullscreen.
                assert_eq!(vec![(1, SCREEN_GEOM)], wl3.windows);
            }

            it "should not lose focus if we remove another window" {
                wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).unwrap();

                wm.remove_window(2).unwrap();

                assert!(!wm.is_managed(2));
                assert_eq!(vec![1, 3], wm.get_windows());
                let wl3 = wm.get_window_layout();
                assert_eq!(Some(3), wl3.focused_window);
                assert_eq!(vec![(3, SCREEN_GEOM)], wl3.windows);
            }

            it "should do be in initial state if we remove all windows" {
                wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();

                wm.remove_window(1).unwrap();
                wm.remove_window(2).unwrap();

                assert!(!wm.is_managed(1));
                assert!(!wm.is_managed(2));
                expect(wm.get_windows().len()).to(be_equal_to(0));
                let wl = wm.get_window_layout();
                assert_eq!(None, wl.focused_window);
                expect(wl.windows.len()).to(be_equal_to(0));
            }
        }

        describe! focus_window {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();
            }

            it "should focus the correct window" {
                wm.focus_window(Some(1)).unwrap();

                assert_eq!(Some(1), wm.get_window_layout().focused_window);
            }

            it "should keep the focus if already focussed" {
                wm.focus_window(Some(2)).unwrap();

                assert_eq!(Some(2), wm.get_window_layout().focused_window);
            }

            it "should lose the focus if passed no window" {
                wm.focus_window(None).unwrap();

                assert_eq!(None, wm.get_window_layout().focused_window);
                assert_eq!(0, wm.get_window_layout().windows.len());
            }

            it "should throw error on unknown window" {
                expect!(wm.focus_window(Some(3))).to(be_err());
            }
        }

        describe! cycle_focus {
            before_each {
                wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).unwrap();
                wm.add_window(WindowWithInfo::new_tiled(4, SOME_GEOM)).unwrap();
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
    }
}
