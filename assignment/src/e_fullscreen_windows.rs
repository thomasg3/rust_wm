//! Optional: Fullscreen Windows
//!
//! Extend your window manager with support for fullscreen windows, i.e. the
//! ability to temporarily make a window take up the whole screen, thereby
//! obscuring all other windows. See the documentation of the
//! [`FullscreenSupport`] trait for the precise requirements. Don't confuse
//! this with the first assignment, in which you built a window manager that
//! displayed all windows fullscreen.
//!
//! Like in the previous assignments, either make a copy of, or define a
//! wrapper around your previous window manager to implement the
//! [`FullscreenSupport`] trait as well. Note that this window manager must
//! still implement all the traits from previous assignments.
//!
//! [`FullscreenSupport`]: ../../cplwm_api/wm/trait.FullscreenSupport.html
//!
//! # Status
//!
//! COMPLETED: NO
//!
//! COMMENTS:
//! It would however be fairly easy. FullscreenWM would then become:
//! FocusManager + MinimiseManager<FloatOrTileManager<LayoutManager>> + FullscreenManager.
//! The FullscreenManager would keep track of the fullscreen window.
//!

// Add imports here


/// Replace `()` with the name of your window manager data type.
pub type WMName = ();
