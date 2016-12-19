//! Minimising Windows
//!
//! Extend your window manager with support for (un)minimising windows. i.e.
//! the ability to temporarily hide windows and to reveal them again later.
//! See the documentation of the [`MinimiseSupport`] trait for the precise
//! requirements.
//!
//! Either make a copy of the tiling window manager with support for floating
//! windows you developed in the previous assignment and let it implement the
//! [`MinimiseSupport`] trait as well, or implement this trait by building a
//! wrapper around the previous window manager. Note that this window manager
//! must still implement all the traits from previous assignments.
//!
//! [`MinimiseSupport`]: ../../cplwm_api/wm/trait.MinimiseSupport.html
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
use cplwm_api::wm::{WindowManager, TilingSupport, FloatSupport, MinimiseSupport};

use wm_common::Manager;
use wm_common::error::FloatWMError;
use a_fullscreen_wm::FocusManager;
use b_tiling_wm::VerticalLayout;
use c_floating_windows::FloatOrTileManager;
use std::collections::HashMap;



/// the public type
pub type WMName = ();

/// struct for MinimizeWM = {Focus + TileOrFloat + Minimize}
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct MinimizeWM{
    /// focus manager
    pub focus_manager: FocusManager,
    /// tileOrFloat manager (visible windows)
    pub float_or_tile_manager: FloatOrTileManager<VerticalLayout>,
    /// manager for handling minimized windows
    pub minimize_manager: MinimizeManager
}

impl WindowManager for MinimizeWM {
    type Error = FloatWMError;

    fn new(screen: Screen) -> MinimizeWM {
        MinimizeWM {
            focus_manager: FocusManager::new(),
            float_or_tile_manager: FloatOrTileManager::new(screen, VerticalLayout{}),
            minimize_manager: MinimizeManager::new(),
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
                self.float_or_tile_manager.add_window(window_with_info)
            })
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        match self.focus_manager.remove_window(window) {
            Err(error) => Err(error.to_float_error()),
            Ok(_) => self.float_or_tile_manager.remove_window(window)
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        WindowLayout {
            focused_window: self.get_focused_window(),
            windows: self.float_or_tile_manager.get_window_layout(),
        }
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        self.focus_manager.focus_window(window)
            .map_err(|error| error.to_float_error())
            .and_then(|_| self.float_or_tile_manager.focus_window(window))
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focus_manager.cycle_focus(dir)
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

impl TilingSupport for MinimizeWM {
    fn get_master_window(&self) -> Option<Window> {
        self.float_or_tile_manager.get_master_window()
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error>{
        self.float_or_tile_manager.swap_with_master(window, &mut self.focus_manager)
    }

    fn swap_windows(&mut self, dir: PrevOrNext){
        self.float_or_tile_manager.swap_windows(dir, &self.focus_manager)
    }
}

impl FloatSupport for MinimizeWM {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.float_or_tile_manager.get_floating_windows()
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error>{
        self.float_or_tile_manager.toggle_floating(window)
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error>{
        self.float_or_tile_manager.set_window_geometry(window, new_geometry)
    }
}

impl MinimiseSupport for MinimizeWM {
    fn get_minimised_windows(&self) -> Vec<Window> {
        self.minimize_manager.get_windows()
    }
    fn toggle_minimised(&mut self, window: Window) -> Result<(), Self::Error>{
        unimplemented!()
    }
}


/// Manager to manage the minimized windows
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct MinimizeManager {
    /// Map to keep the window and it's last info
    pub minis: HashMap<Window, WindowWithInfo>,
}

impl Manager for MinimizeManager {
    fn get_windows(&self) -> Vec<Window> {
        self.minis.keys().map(|w| *w).collect()
    }
}

impl MinimizeManager {
    /// create empty MinimizeManager
    pub fn new() -> MinimizeManager {
        MinimizeManager{
            minis: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;
    use wm_common::tests::float_support;
    use wm_common::tests::float_and_tile_support;
    use super::MinimizeWM;
    use b_tiling_wm::VerticalLayout;
    use cplwm_api::wm::WindowManager;
    use cplwm_api::types::*;

    static SCREEN: Screen = Screen {
        width: 800,
        height: 600,
    };

    #[test]
    fn test_empty_tiling_wm(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use common test
        window_manager::test_empty_wm(wm, SCREEN);
    }

    #[test]
    fn test_adding_and_removing_some_windows(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use common test
        window_manager::test_adding_and_removing_windows(wm);
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use common test
        window_manager::test_focus_and_unfocus_window(wm);
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use common test
        window_manager::test_cycle_focus_none_and_one_window(wm);
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use common test
        window_manager::test_cycle_focus_multiple_windows(wm);
    }

    #[test]
    fn test_get_window_info(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use common test
        window_manager::test_get_window_info(wm);
    }

    #[test]
    fn test_resize_screen(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use common test
        window_manager::test_resize_screen(wm, SCREEN);
    }

    #[test]
    fn test_get_master_window(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        tiling_support::test_master_tile(wm);
    }

    #[test]
    fn test_swap_with_master_window(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        tiling_support::test_swap_with_master(wm);
    }


    #[test]
    fn test_swap_windows(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        tiling_support::test_swap_windows(wm, VerticalLayout{});
    }

    #[test]
    fn test_tiling_layout(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        tiling_support::test_get_window_info(wm, VerticalLayout{});
    }

    #[test]
    fn test_get_floating_windows(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_support::test_get_floating_windows(wm);
    }

    #[test]
    fn test_toggle_floating(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_support::test_toggle_floating(wm);
    }

    #[test]
    fn test_set_window_geometry(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_support::test_set_window_geometry(wm);
    }

    #[test]
    fn test_window_layout_order(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_support::test_window_layout_order(wm);
    }

    #[test]
    fn test_focus_floating_window_order(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_support::test_focus_floating_window_order(wm);
    }

    #[test]
    fn test_swapping_master_with_floating_window_no_tiles(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_and_tile_support::test_swapping_master_with_floating_window_no_tiles(wm);
    }

    #[test]
    fn test_swapping_master_with_floating_window(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_and_tile_support::test_swapping_master_with_floating_window(wm);
    }

    #[test]
    fn test_swap_windows_on_floating(){
        // Initialize test with a new window manager
        let wm = MinimizeWM::new(SCREEN);
        // use the common test
        float_and_tile_support::test_swap_windows_on_floating(wm);
    }
}
