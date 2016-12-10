//! This module provides the Layouter interface for layouting tiled windows
use rustc_serialize::{Decodable, Encodable};
use std::fmt::Debug;

use cplwm_api::types::{Geometry, Screen, GapSize};

/// A trait to layout tiling windows
/// One must implement get_geom OR (get_master_geom AND get_slave_geom)
pub trait Layouter: Encodable + Decodable + Debug + Clone  {
    /// Return the geometry for the window at position i
    /// in the given screen and with the given number of windows
    fn get_geom(&self, i: usize, screen: Screen, nb_windows: usize) -> Geometry {
        if i == 0 {
            // the master window
            self.get_master_geom(screen, nb_windows)
        } else {
            // a slave window
            self.get_slave_geom(i - 1, screen, nb_windows)
        }
    }

    /// Return the geometry for the master window
    fn get_master_geom(&self, screen: Screen, nb_windows: usize) -> Geometry {
        self.get_geom(0, screen, nb_windows)
    }
    /// Return the geometry for the i-th slave
    fn get_slave_geom(&self, i: usize, screen: Screen, nb_windows: usize) -> Geometry {
        self.get_geom(i + 1, screen, nb_windows)
    }

    /// Create a new instance of the layouter
    fn new() -> Self;
}


/// GapSupport in the Layouter. Similar to GapSupport for WindowManager but the upper trait bound is not enforced.
pub trait GapSupport {
    /// Return the current gap size.
    ///
    /// Initially 0.
    fn get_gap(&self) -> GapSize;

    /// Set the gap size.
    ///
    /// **Invariant**: after setting `set_gap(g)` with some gap size `g`,
    /// `get_gap() == g`.
    fn set_gap(&mut self, GapSize);
}
