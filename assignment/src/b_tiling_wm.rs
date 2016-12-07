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
//!
//! COMPLETED: ?
//!
//! COMMENTS:
//!
//! ...
//!

// Add imports here
use std::error;
use std::fmt;
use cplwm_api::types::{FloatOrTile, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::WindowManager;

use a_fullscreen_wm::{FullscreenWM, FullscreenWMError};


/// **TODO**: You are free to choose the name for your window manager. As we
/// will use automated tests when grading your assignment, indicate here the
/// name of your window manager data type so we can just use `WMName` instead
/// of having to manually figure out your window manager name.
///
/// Replace `()` with the name of your window manager data type.
pub type WMName = TilingWM;


/// The TilingWM as described in the assignment. Will implement the
/// WindowManager and the TilingSupport
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct TilingWM {
    /// The fullscreen window manager this tiling window manager is wrapped
    /// around.
    pub fullscreen_wm: FullscreenWM
}

/// The errors this window manager can return.
#[derive(Debug)]
pub enum TilingWMError {
    /// Supporting all errors a fullscreen window manager can return.
    FullscreenWMError(FullscreenWMError),
}

impl fmt::Display for TilingWMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TilingWMError::FullscreenWMError(ref error) => error.fmt(f)
        }
    }
}

impl error::Error for TilingWMError {
    fn description(&self) -> &'static str {
        match *self {
            // instead of copying the code for the FullscreenWMErrors, this should
            // delegate to the error, but I got some problems with lifetime requirements
            // I could not solve directly, so I am postponing solving this.
            TilingWMError::FullscreenWMError(ref error) => match *error {
                FullscreenWMError::UnknownWindow(_) => "Unknown window",
                FullscreenWMError::AlReadyManagedWindow(_) => "Already managed window",
            }
        }
    }
}

impl WindowManager for TilingWM {
    /// The Error type is TilingWMError
    type Error = TilingWMError;

    /// constructor with given screen
    fn new(screen: Screen) -> TilingWM {
        TilingWM {
            fullscreen_wm: FullscreenWM::new(screen),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        self.fullscreen_wm.get_windows()
    }
    fn get_focused_window(&self) -> Option<Window> {
        self.fullscreen_wm.get_focused_window()
    }
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        match self.fullscreen_wm.add_window(window_with_info){
            Ok(_) => Ok(()),
            Err(error) => Err(TilingWMError::FullscreenWMError(error))
        }
    }
    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        match self.fullscreen_wm.remove_window(window) {
            Ok(_) => Ok(()),
            Err(error) => Err(TilingWMError::FullscreenWMError(error))
        }
    }
    fn get_window_layout(&self) -> WindowLayout {
        self.fullscreen_wm.get_window_layout()
    }
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        match self.fullscreen_wm.focus_window(window) {
            Ok(_) => Ok(()),
            Err(error) => Err(TilingWMError::FullscreenWMError(error))
        }
    }
    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.fullscreen_wm.cycle_focus(dir)
    }
    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        match self.fullscreen_wm.get_window_info(window){
            Ok(window_with_info) => Ok(window_with_info),
            Err(error) => Err(TilingWMError::FullscreenWMError(error))
        }
    }
    fn get_screen(&self) -> Screen {
        self.fullscreen_wm.get_screen()
    }
    fn resize_screen(&mut self, screen: Screen) {
        self.fullscreen_wm.resize_screen(screen)
    }

}
