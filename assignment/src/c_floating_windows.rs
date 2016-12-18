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

use wm_common::Manager;
use wm_common::error::FloatWMError;
use a_fullscreen_wm::FocusManager;
use b_tiling_wm::{TileManager, VerticalLayout};

/// The public type.
pub type WMName = FloatWM;

/// FloatWM as described in the assignment. Will implement WindowManager, TilingSupport
/// and FloatSupport
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FloatWM{
    /// The Manager for the current focus
    pub focus_manager: FocusManager,
    /// The manager to manage the tiles
    pub tile_manager: TileManager<VerticalLayout>,
    /// FloatManager to manage the floating windows
    pub float_manager: FloatManager,
}

impl WindowManager for FloatWM {
    /// The Error type is the specific FloatWMError
    type Error = FloatWMError;

    fn new(screen: Screen) -> FloatWM {
        FloatWM {
            focus_manager: FocusManager::new(),
            tile_manager: TileManager::new(screen, VerticalLayout{}),
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
        let mut windows: Vec<(Window, Geometry)> = Vec::new();
        windows.extend(self.tile_manager.get_window_layout());
        windows.extend(self.float_manager.get_window_layout());
        WindowLayout {
            focused_window: self.get_focused_window(),
            windows: windows,
        }
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        self.focus_manager.focus_window(window)
            .map_err(|error| error.to_float_error())
            .and_then(|_| {
                match window {
                    None => Ok(()),
                    Some(w) => {
                        if self.float_manager.is_managed(w){
                            self.float_manager.focus_window(w)
                        } else {
                            Ok(())
                        }
                    }
                }
        })
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focus_manager.cycle_focus(dir)
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        self.tile_manager.get_window_info(window)
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

impl TilingSupport for FloatWM {
    fn get_master_window(&self) -> Option<Window> {
        self.tile_manager.get_master_window()
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error>{
        self.tile_manager.swap_with_master(window, &mut self.focus_manager)
            .map_err(|error| error.to_float_error())
    }

    fn swap_windows(&mut self, dir: PrevOrNext){
        self.tile_manager.swap_windows(dir, &mut self.focus_manager)
    }
}

impl FloatSupport for FloatWM {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.float_manager.get_windows()
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error>{
        if self.float_manager.is_managed(window) {
            self.float_manager.get_window_info(window).and_then(|window_with_info| {
                self.float_manager.remove_window(window).and_then(|_| {
                    self.tile_manager.add_window(WindowWithInfo{
                        window: window_with_info.window,
                        geometry: window_with_info.geometry,
                        float_or_tile: FloatOrTile::Tile,
                        fullscreen: window_with_info.fullscreen,
                    })
                        .map_err(|error| error.to_float_error())
                })
            })
        } else if self.tile_manager.is_managed(window) {
            self.tile_manager.get_original_window_info(window)
                .map_err(|error| error.to_float_error())
                .and_then(|window_with_info| {
                    self.tile_manager.remove_window(window)
                        .map_err(|error| error.to_float_error())
                        .and_then(|_| {
                            self.float_manager.add_window(WindowWithInfo{
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
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error>{
        self.float_manager.set_window_geometry(window, new_geometry).map_err(|error| {
            if self.tile_manager.is_managed(window) {
                FloatWMError::NotFloatingWindow(window)
            } else {
                error
            }
        })
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
    fn get_windows(&self) -> Vec<Window> {
        self.floaters.iter().map(|w| w.window).collect()
    }
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

    fn focus_window(&mut self, window: Window) -> Result<(), FloatWMError> {
        self.get_window_info(window).and_then(|window_with_info| {
            self.remove_window(window).and_then(|_| {
                self.add_window(window_with_info)
            })
        })
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, FloatWMError> {
        match self.floaters.iter().position(|w| w.window == window) {
            None => Err(FloatWMError::UnknownWindow(window)),
            Some(i) => {
                Ok(self.floaters[i])
            }
        }
    }

    fn get_window_layout(&self) -> Vec<(Window, Geometry)> {
        self.floaters.iter()
            .map(|window_with_info| (window_with_info.window, window_with_info.geometry))
            .collect()
    }

    fn get_screen(&self) -> Screen {
        self.screen
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.screen = screen
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), FloatWMError>{
        match self.floaters.iter().position(|w| w.window == window) {
            None => Err(FloatWMError::UnknownWindow(window)),
            Some(i) => {
                self.floaters[i].geometry = new_geometry;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;
    use wm_common::tests::float_support;

    // We have to import `FloatWM` from the super module.
    use super::FloatWM;
    use b_tiling_wm::VerticalLayout;
    // We have to repeat the imports we did in the super module.
    use cplwm_api::wm::WindowManager;
    use cplwm_api::types::*;

    // We define a static variable for the screen we will use in the tests.
    // You can just as well define it as a local variable in your tests.
    static SCREEN: Screen = Screen {
        width: 800,
        height: 600,
    };


    #[test]
    fn test_empty_tiling_wm(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use common test
        window_manager::test_empty_wm(wm, SCREEN);
    }

    #[test]
    fn test_adding_and_removing_some_windows(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use common test
        window_manager::test_adding_and_removing_windows(wm);
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use common test
        window_manager::test_focus_and_unfocus_window(wm);
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use common test
        window_manager::test_cycle_focus_none_and_one_window(wm);
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use common test
        window_manager::test_cycle_focus_multiple_windows(wm);
    }

    #[test]
    fn test_get_window_info(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use common test
        window_manager::test_get_window_info(wm);
    }

    #[test]
    fn test_resize_screen(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use common test
        window_manager::test_resize_screen(wm, SCREEN);
    }

    #[test]
    fn test_get_master_window(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        tiling_support::test_master_tile(wm);
    }

    #[test]
    fn test_swap_with_master_window(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        tiling_support::test_swap_with_master(wm);
    }


    #[test]
    fn test_swap_windows(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        tiling_support::test_swap_windows(wm, VerticalLayout{});
    }

    #[test]
    fn test_tiling_layout(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        tiling_support::test_get_window_info(wm, VerticalLayout{});
    }

    #[test]
    fn test_get_floating_windows(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        float_support::test_get_floating_windows(wm);
    }

    #[test]
    fn test_toggle_floating(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        float_support::test_toggle_floating(wm);
    }

    #[test]
    fn test_set_window_geometry(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        float_support::test_set_window_geometry(wm);
    }

    #[test]
    fn test_window_layout_order(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        float_support::test_window_layout_order(wm);
    }

    #[test]
    fn test_focus_floating_window_order(){
        // Initialize test with a new window manager
        let wm = FloatWM::new(SCREEN);
        // use the common test
        float_support::test_focus_floating_window_order(wm);
    }
}
