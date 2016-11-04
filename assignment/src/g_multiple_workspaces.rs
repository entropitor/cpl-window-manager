//! Optional: Multiple Workspaces
//!
//! Extend your window manager with support for multiple workspaces. The
//! concept of workspaces is described in the first section of the assignment.
//! See the documentation of the [`MultiWorkspaceSupport`] trait for the precise
//! requirements.
//!
//! *Unlike* the previous assignments, you are not allowed to make a copy of
//! your previous window manager. You *have* to define a wrapper implementing
//! the [`MultiWorkspaceSupport`] trait. This wrapper can take any existing
//! window manager and uses it to create the different workspaces. This
//! wrapper must also implement all the traits you have implemented in the
//! other assignments, you can forward them to the window manager of the
//! current workspace.
//!
//! [`MultiWorkspaceSupport`]: ../../cplwm_api/wm/trait.MultiWorkspaceSupport.html
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

// Add imports here


/// **TODO**: You are free to choose the name for your window manager. As we
/// will use automated tests when grading your assignment, indicate here the
/// name of your window manager data type so we can just use `WMName` instead
/// of having to manually figure out your window manager name.
///
/// Replace `()` with the name of your window manager data type.
///
/// Replace the `WM` type parameter with the name of the window manager from
/// assignment E or the window manager of the last assignment you completed
/// (except F). E.g. use `type WMName =
/// MultiWorkspaceWM<e_fullscreen_windows::WMName>`.
pub type WMName = ();
