//! Floating Windows
//!
//! Extend your window manager with support for floating windows, i.e. windows
//! that do not tile but that you move around and resize with the mouse. These
//! windows will *float* above the tiles, e.g. dialogs, popups, video players,
//! etc. See the documentation of the [`FloatSupport`] trait for the precise
//! requirements.
//!
//! Either make a copy of the tiling window manager you developed in the
//! previous assignment and let it implement the [`FloatSupport`] trait as
//! well, or implement the [`FloatSupport`] trait by building a wrapper around
//! your tiling window manager. This way you won't have to copy paste code.
//! Note that this window manager must still implement the [`TilingSupport`]
//! trait.
//!
//! [`FloatSupport`]: ../../cplwm_api/wm/trait.FloatSupport.html
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
use cplwm_api::types::{FloatOrTile, Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::{WindowManager, TilingSupport, FloatSupport};

use wm_common::error::{StandardError, FloatWMError};
use a_fullscreen_wm::FullscreenWM;
use b_tiling_wm::{TilingWM, VerticalLayout};

/// The public type.
pub type WMName = FloatWM;

/// FloatWM as described in the assignment. Will implement WindowManager, TilingSupport
/// and FloatSupport
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FloatWM{
    /// FullscreenWM this FloatWM uses as a focus manager
    pub focus_manager: FullscreenWM,
    /// Tiling WM this FloatWM uses as a tile manager
    pub tile_manager: TilingWM,
    /// FloatManager to manage the floating windows
    pub float_manager: FloatManager,
}

/// A Manager to manage floating windows only.
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FloatManager {
    /// The screen this FloatManager is operating in
    pub screen: Screen,
    /// Vec with all the floating windows
    pub floaters: Vec<WindowWithInfo>,
}

impl FloatManager {
    fn new(screen: Screen) -> FloatManager{
        FloatManager {
            screen: screen,
            floaters: Vec::new(),
        }
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo)  -> Result<(), FloatWMError> {
        if self.get_windows().contains(&window_with_info.window) {
            Err(FloatWMError::AlReadyManagedWindow(window_with_info.window))
        } else {
            self.floaters.push(window_with_info);
            Ok(())
        }
    }

    fn remove_window(&mut self, window: Window) -> Result<(), FloatWMError> {
        match self.floaters.iter().position(|w| w.window == window) {
            None => Err(FloatWMError::UnknownWindow(window)),
            Some(i) => {
                self.floaters.remove(i);
                Ok(())
            }
        }
    }

    fn get_windows(&self) -> Vec<Window>{
        self.floaters.iter().map(|w| w.window).collect()
    }

    fn get_screen(&self) -> Screen {
        self.screen
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.screen = screen
    }

}


impl WindowManager for FloatWM {
    /// The Error type is the specific FloatWMError
    type Error = FloatWMError;

    fn new(screen: Screen) -> FloatWM {
        FloatWM {
            focus_manager: FullscreenWM::new(screen),
            tile_manager: TilingWM::new(screen),
            float_manager: FloatManager::new(screen),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        self.focus_manager.get_windows()
    }

    fn get_focused_window(&self) -> Option<Window> {
        self.focus_manager.get_focused_window()
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        self.focus_manager.add_window(window_with_info)
            .map_err(|error| error.to_float_error())
            .and_then(|_| {
                match window_with_info.float_or_tile {
                    FloatOrTile::Tile => self.tile_manager.add_window(window_with_info)
                        .map_err(|error| error.to_float_error()),
                    FloatOrTile::Float => self.float_manager.add_window(window_with_info),
                }
            })
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        match self.focus_manager.remove_window(window) {
            Err(error) => Err(error.to_float_error()),
            Ok(_) => self.tile_manager.remove_window(window).or_else(|_| {
                self.float_manager.remove_window(window)
            })
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        unimplemented!();
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        self.focus_manager.focus_window(window).map_err(|error| error.to_float_error())
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focus_manager.cycle_focus(dir)
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        unimplemented!();
    }

    fn get_screen(&self) -> Screen {
        self.float_manager.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.focus_manager.resize_screen(screen);
        self.tile_manager.resize_screen(screen);
        self.float_manager.resize_screen(screen);
    }
}

impl TilingSupport for FloatWM {
    fn get_master_window(&self) -> Option<Window> {
        self.tile_manager.get_master_window()
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error>{
        self.tile_manager.swap_with_master(window)
            .map_err(|error| error.to_float_error())
            .and_then(|_| {
                self.focus_window(Some(window))
            })
    }

    fn swap_windows(&mut self, dir: PrevOrNext){
        self.get_focused_window().and_then(|window| {
            self.tile_manager.swap_windows(dir);
            Some(())
        });
    }
}
