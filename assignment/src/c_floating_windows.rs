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
//! COMPLETED: YES
//!
//! COMMENTS:
//!
//! ...
//!

// Add imports here
use cplwm_api::types::{FloatOrTile, Geometry, PrevOrNext, Screen, Window, WindowLayout,
                       WindowWithInfo};
use cplwm_api::wm::{FloatSupport, TilingSupport, WindowManager};

use wm_common::{FloatAndTileTrait, FloatTrait, LayoutManager, Manager, TilingLayout, TilingTrait};
use wm_common::error::{FloatWMError, StandardError};
use a_fullscreen_wm::FocusManager;
use b_tiling_wm::{TileManager, VerticalLayout};

/// The public type.
pub type WMName = FloatWM;

/// FloatWM as described in the assignment. Will implement WindowManager, TilingSupport
/// and FloatSupport. FloatWM is the combination of a FocusManager and FloatOrTileManager
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FloatWM {
    /// The Manager for the current focus
    pub focus_manager: FocusManager,
    /// The manager to manage the tiles and floating windows
    pub float_or_tile_manager: FloatOrTileManager<VerticalLayout>,
}


impl WindowManager for FloatWM {
    /// The Error type is the specific FloatWMError
    type Error = FloatWMError;

    fn new(screen: Screen) -> FloatWM {
        FloatWM {
            focus_manager: FocusManager::new(),
            float_or_tile_manager: FloatOrTileManager::new(screen, VerticalLayout {}),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        self.focus_manager.get_windows()
    }

    fn get_focused_window(&self) -> Option<Window> {
        self.focus_manager.get_focused_window()
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        self.focus_manager
            .add_window(window_with_info)
            .map_err(|error| error.to_float_error())
            .and_then(|_| self.float_or_tile_manager.add_window(window_with_info))
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        match self.focus_manager.remove_window(window) {
            Err(error) => Err(error.to_float_error()),
            Ok(_) => self.float_or_tile_manager.remove_window(window),
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        WindowLayout {
            focused_window: self.get_focused_window(),
            windows: self.float_or_tile_manager.get_window_layout(),
        }
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        self.focus_manager
            .focus_window(window)
            .map_err(|error| error.to_float_error())
            .and_then(|_| self.float_or_tile_manager.focus_shifted(window))
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focus_manager.cycle_focus(dir);
        self.float_or_tile_manager.focus_shifted(self.focus_manager.get_focused_window()).is_ok();
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        self.float_or_tile_manager.get_window_info(window)
    }

    fn get_screen(&self) -> Screen {
        self.float_or_tile_manager.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.float_or_tile_manager.resize_screen(screen);
    }
}

impl TilingSupport for FloatWM {
    fn get_master_window(&self) -> Option<Window> {
        self.float_or_tile_manager.get_master_window()
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error> {
        self.float_or_tile_manager.swap_with_master(window, &mut self.focus_manager)
    }

    fn swap_windows(&mut self, dir: PrevOrNext) {
        self.float_or_tile_manager.swap_windows(dir, &self.focus_manager)
    }
}

impl FloatSupport for FloatWM {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.float_or_tile_manager.get_floating_windows()
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error> {
        self.float_or_tile_manager.toggle_floating(window, &mut self.focus_manager)
    }

    fn set_window_geometry(&mut self,
                           window: Window,
                           new_geometry: Geometry)
                           -> Result<(), Self::Error> {
        self.float_or_tile_manager.set_window_geometry(window, new_geometry)
    }
}


/// Manager for Floating and tiled windows
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FloatOrTileManager<T: TilingLayout> {
    /// The manager to manage the tiles
    pub tile_manager: TileManager<T>,
    /// FloatManager to manage the floating windows
    pub float_manager: FloatManager,
}

impl<T: TilingLayout<Error = StandardError>> Manager for FloatOrTileManager<T> {
    type Error = FloatWMError;

    fn get_windows(&self) -> Vec<Window> {
        let mut windows = self.tile_manager.get_windows();
        windows.extend(self.float_manager.get_windows());
        windows
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), FloatWMError> {
        match window_with_info.float_or_tile {
            FloatOrTile::Tile => {
                self.tile_manager
                    .add_window(window_with_info)
                    .map_err(|error| error.to_float_error())
            }
            FloatOrTile::Float => self.float_manager.add_window(window_with_info),
        }
    }

    fn remove_window(&mut self, window: Window) -> Result<(), FloatWMError> {
        self.tile_manager
            .remove_window(window)
            .or_else(|_| self.float_manager.remove_window(window))
    }
}

impl<T: TilingLayout<Error = StandardError>> LayoutManager for FloatOrTileManager<T> {
    fn get_window_layout(&self) -> Vec<(Window, Geometry)> {
        let mut windows: Vec<(Window, Geometry)> = Vec::new();
        windows.extend(self.tile_manager.get_window_layout());
        windows.extend(self.float_manager.get_window_layout());
        windows
    }

    fn focus_shifted(&mut self, window: Option<Window>) -> Result<(), FloatWMError> {
        match window {
            None => Ok(()),
            Some(w) => {
                if self.float_manager.is_managed(w) {
                    self.float_manager.focus_shifted(Some(w))
                } else {
                    Ok(())
                }
            }
        }
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, FloatWMError> {
        self.tile_manager
            .get_window_info(window)
            .map_err(|error| error.to_float_error())
            .or_else(|_| self.float_manager.get_window_info(window))
    }

    fn get_screen(&self) -> Screen {
        self.float_manager.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.tile_manager.resize_screen(screen);
        self.float_manager.resize_screen(screen);
    }
}

impl<T: TilingLayout<Error = StandardError>> TilingTrait for FloatOrTileManager<T> {
    /// get the master window
    fn get_master_window(&self) -> Option<Window> {
        self.tile_manager.get_master_window()
    }

    /// swap with the master, also floating windows can swap with master
    fn swap_with_master(&mut self,
                        window: Window,
                        mut focus_manager: &mut FocusManager)
                        -> Result<(), FloatWMError> {
        if self.is_floating(window) {
            self.toggle_floating(window, focus_manager).and_then(|_| {
                let master = self.get_master_window().unwrap(); // unwrap is possible since there
                // is bound to be at least one tile after calling toggle_float on a floating window
                // if there is one tile, there is one master
                self.tile_manager
                    .swap_with_master(window, &mut focus_manager)
                    .map_err(|error| error.to_float_error())
                    .and_then(|_| self.toggle_floating(master, focus_manager))
            })
        } else {
            self.tile_manager
                .swap_with_master(window, focus_manager)
                .map_err(|error| error.to_float_error())
        }
    }

    /// swap windows
    fn swap_windows(&mut self, dir: PrevOrNext, focus_manager: &FocusManager) {
        match focus_manager.get_focused_window() {
            None => {}
            Some(w) => {
                if self.is_tiled(w) {
                    self.tile_manager.swap_windows(dir, &focus_manager)
                }
            }
        }
    }
}

impl<T: TilingLayout<Error = StandardError>> FloatTrait for FloatOrTileManager<T> {
    /// set the geometry of floating window
    fn set_window_geometry(&mut self,
                           window: Window,
                           new_geometry: Geometry)
                           -> Result<(), FloatWMError> {
        self.float_manager.set_window_geometry(window, new_geometry).map_err(|error| {
            if self.tile_manager.is_managed(window) {
                FloatWMError::NotFloatingWindow(window)
            } else {
                error
            }
        })
    }
}

impl<T: TilingLayout<Error = StandardError>> FloatAndTileTrait for FloatOrTileManager<T> {
    /// get all floating windows
    fn get_floating_windows(&self) -> Vec<Window> {
        self.float_manager.get_windows()
    }

    /// get all tiled windows
    fn get_tiled_windows(&self) -> Vec<Window> {
        self.tile_manager.get_windows()
    }

    /// toggle floating on window
    fn toggle_floating(&mut self,
                       window: Window,
                       focus_manager: &mut FocusManager)
                       -> Result<(), FloatWMError> {
        focus_manager.focus_window(Some(window))
            .map_err(|error| error.to_float_error())
            .and_then(|_| {
                if self.float_manager.is_managed(window) {
                    self.float_manager.get_window_info(window).and_then(|window_with_info| {
                        self.float_manager.remove_window(window).and_then(|_| {
                            self.tile_manager
                                .add_window(WindowWithInfo {
                                    window: window_with_info.window,
                                    geometry: window_with_info.geometry,
                                    float_or_tile: FloatOrTile::Tile,
                                    fullscreen: window_with_info.fullscreen,
                                })
                                .map_err(|error| error.to_float_error())
                        })
                    })
                } else if self.tile_manager.is_managed(window) {
                    self.tile_manager
                        .get_original_window_info(window)
                        .map_err(|error| error.to_float_error())
                        .and_then(|window_with_info| {
                            self.tile_manager
                                .remove_window(window)
                                .map_err(|error| error.to_float_error())
                                .and_then(|_| {
                                    self.float_manager.add_window(WindowWithInfo {
                                        window: window_with_info.window,
                                        geometry: window_with_info.geometry,
                                        float_or_tile: FloatOrTile::Float,
                                        fullscreen: window_with_info.fullscreen,
                                    })
                                })
                        })
                } else {
                    Err(FloatWMError::UnknownWindow(window))
                }
            })
    }
}

impl<T: TilingLayout<Error = StandardError>> FloatOrTileManager<T> {
    /// creates empty FloatOrTileManager
    pub fn new(screen: Screen, tiling_layout: T) -> FloatOrTileManager<T> {
        FloatOrTileManager {
            tile_manager: TileManager::new(screen, tiling_layout),
            float_manager: FloatManager::new(screen),
        }
    }
}

/// A Manager to manage floating windows only.
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FloatManager {
    /// The screen this FloatManager is operating in
    pub screen: Screen,
    /// Vec with all the floating windows
    pub floaters: Vec<WindowWithInfo>,
}

impl Manager for FloatManager {
    type Error = FloatWMError;

    fn get_windows(&self) -> Vec<Window> {
        self.floaters.iter().map(|w| w.window).collect()
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), FloatWMError> {
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
}

impl LayoutManager for FloatManager {
    fn focus_shifted(&mut self, window: Option<Window>) -> Result<(), FloatWMError> {
        match window {
            None => Ok(()),
            Some(w) => {
                self.get_window_info(w).and_then(|window_with_info| {
                    self.remove_window(w).and_then(|_| self.add_window(window_with_info))
                })
            }
        }
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, FloatWMError> {
        match self.floaters.iter().position(|w| w.window == window) {
            None => Err(FloatWMError::UnknownWindow(window)),
            Some(i) => Ok(self.floaters[i]),
        }
    }

    fn get_window_layout(&self) -> Vec<(Window, Geometry)> {
        self.floaters
            .iter()
            .map(|window_with_info| (window_with_info.window, window_with_info.geometry))
            .collect()
    }

    fn get_screen(&self) -> Screen {
        self.screen
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.screen = screen
    }
}

impl FloatTrait for FloatManager {
    fn set_window_geometry(&mut self,
                           window: Window,
                           new_geometry: Geometry)
                           -> Result<(), FloatWMError> {
        match self.floaters.iter().position(|w| w.window == window) {
            None => Err(FloatWMError::UnknownWindow(window)),
            Some(i) => {
                self.floaters[i].geometry = new_geometry;
                Ok(())
            }
        }
    }
}

impl FloatManager {
    fn new(screen: Screen) -> FloatManager {
        FloatManager {
            screen: screen,
            floaters: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;
    use wm_common::tests::float_support;
    use wm_common::tests::float_and_tile_support;
    use super::FloatWM;
    use b_tiling_wm::VerticalLayout;

    #[test]
    fn test_empty_tiling_wm() {
        window_manager::test_empty_wm::<FloatWM>();
    }

    #[test]
    fn test_adding_and_removing_some_windows() {
        window_manager::test_adding_and_removing_windows::<FloatWM>();
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        window_manager::test_focus_and_unfocus_window::<FloatWM>();
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        window_manager::test_cycle_focus_none_and_one_window::<FloatWM>();
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        window_manager::test_cycle_focus_multiple_windows::<FloatWM>();
    }

    #[test]
    fn test_get_window_info() {
        window_manager::test_get_window_info::<FloatWM>();
    }

    #[test]
    fn test_resize_screen() {
        window_manager::test_resize_screen::<FloatWM>();
    }

    #[test]
    fn test_get_master_window() {
        tiling_support::test_master_tile::<FloatWM>();
    }

    #[test]
    fn test_swap_with_master_window() {
        tiling_support::test_swap_with_master::<FloatWM>();
    }


    #[test]
    fn test_swap_windows() {
        tiling_support::test_swap_windows::<FloatWM, VerticalLayout>(VerticalLayout {});
    }

    #[test]
    fn test_tiling_layout() {
        tiling_support::test_get_window_info::<FloatWM, VerticalLayout>(VerticalLayout {});
    }

    #[test]
    fn test_get_floating_windows() {
        float_support::test_get_floating_windows::<FloatWM>();
    }

    #[test]
    fn test_toggle_floating() {
        float_support::test_toggle_floating::<FloatWM>();
    }

    #[test]
    fn test_set_window_geometry() {
        float_support::test_set_window_geometry::<FloatWM>();
    }

    #[test]
    fn test_window_layout_order() {
        float_support::test_window_layout_order::<FloatWM>();
    }

    #[test]
    fn test_focus_floating_window_order() {
        float_support::test_focus_floating_window_order::<FloatWM>();
    }

    #[test]
    fn test_swapping_master_with_floating_window_no_tiles() {
        float_and_tile_support::test_swapping_master_with_floating_window_no_tiles::<FloatWM>();
    }

    #[test]
    fn test_swapping_master_with_floating_window() {
        float_and_tile_support::test_swapping_master_with_floating_window::<FloatWM>();
    }

    #[test]
    fn test_swap_windows_on_floating() {
        float_and_tile_support::test_swap_windows_on_floating::<FloatWM>();
    }

    #[test]
    fn test_swap_windows_with_float_focused() {
        float_and_tile_support::test_swap_windows_with_float_focused::<FloatWM>();
    }

    #[test]
    fn test_toggle_floating_focus() {
        float_and_tile_support::test_toggle_floating_focus::<FloatWM>();
    }
}
