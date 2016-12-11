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
use cplwm_api::types::{FloatOrTile, Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::{WindowManager, TilingSupport};

use wm_common::TilingLayout;
use a_fullscreen_wm::{FullscreenWM, FullscreenWMError};
use std::collections::VecDeque;

/// **TODO**: You are free to choose the name for your window manager. As we
/// will use automated tests when grading your assignment, indicate here the
/// name of your window manager data type so we can just use `WMName` instead
/// of having to manually figure out your window manager name.
///
/// Replace `()` with the name of your window manager data type.
pub type WMName = TilingWM<VerticalLayout>;


/// The TilingWM as described in the assignment. Will implement the
/// WindowManager and the TilingSupport
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct TilingWM<T : TilingLayout>{
    /// The fullscreen window manager this tiling window manager is wrapped
    /// around.
    pub fullscreen_wm: FullscreenWM,
    /// VecDeque to keep the order of the tiles. For the simple vertical layout the first tile is
    /// the master tile.
    pub tiles: VecDeque<Window>,
    /// The layout strategy this Tiling Window Manager uses.
    pub layout: T,
}



impl WindowManager for TilingWM<VerticalLayout> {
    /// The Error type is FullscreenWMError, since the errors are exactly the same.
    type Error = FullscreenWMError;

    /// constructor with given screen
    fn new(screen: Screen) -> TilingWM<VerticalLayout>  {
        TilingWM {
            fullscreen_wm: FullscreenWM::new(screen),
            tiles: VecDeque::new(),
            layout: VerticalLayout{},
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        self.fullscreen_wm.get_windows()
    }
    fn get_focused_window(&self) -> Option<Window> {
        self.fullscreen_wm.get_focused_window()
    }
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        self.fullscreen_wm.add_window(window_with_info).and_then(|_| {
            // No check on whether this window is already added, because the underlying
            // FullscreenWM checks this for us. So when the add_window returns Ok, it is
            // ok to add this window.
            self.tiles.push_back(window_with_info.window);
            Ok(())
        })
    }
    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        self.fullscreen_wm.remove_window(window).and_then(|_| {
            // If remove_window succeeded in the underlying fullscreen_wm, we know for
            // certain the window is/was present in this window manager
            match self.tiles.iter().position(|w| *w == window) {
                None => Err(FullscreenWMError::UnknownWindow(window)),
                Some(i) => {
                    self.tiles.remove(i);
                    Ok(())
                }
            }
        })
    }
    fn get_window_layout(&self) -> WindowLayout {
        let layout = VerticalLayout{};
        WindowLayout {
            focused_window: self.get_focused_window(),
            windows: self.get_windows().iter()
                // We know for sure the window argument in get_window_geometry is a managed window,
                // because it comes directly from get_windows.
                .map(|window| (*window, layout.get_window_geometry(*window, &self.get_screen(), &self.tiles).unwrap()))
                .collect(),
        }
    }
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        self.fullscreen_wm.focus_window(window)
    }
    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.fullscreen_wm.cycle_focus(dir)
    }
    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        let layout = VerticalLayout{};
        layout.get_window_geometry(window, &self.get_screen(), &self.tiles).and_then(|geometry| {
            Ok(WindowWithInfo {
                window: window,
                geometry: geometry,
                float_or_tile: FloatOrTile::Tile,
                fullscreen: false,
            })
        })
    }
    fn get_screen(&self) -> Screen {
        self.fullscreen_wm.get_screen()
    }
    fn resize_screen(&mut self, screen: Screen) {
        self.fullscreen_wm.resize_screen(screen)
    }
}

impl TilingSupport for TilingWM<VerticalLayout> {

    fn get_master_window(&self) -> Option<Window> {
        self.layout.get_master_window(&self.tiles)
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error>{
        self.layout.swap_with_master(window, &mut self.tiles).and_then(|_| {
            self.focus_window(Some(window))
        })
    }

    fn swap_windows(&mut self, dir: PrevOrNext){
        self.get_focused_window().and_then(|window| {
            self.layout.swap_windows(window, dir, &mut self.tiles);
            Some(())
        });
    }
}

/// A Layout algorithm for Tiling window managers as described in assigment b.
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct VerticalLayout {}

impl TilingLayout for VerticalLayout {
    type Error = FullscreenWMError;

    fn get_master_window(&self, tiles: &VecDeque<Window>) -> Option<Window>{
        return tiles.front().map(|w| *w)
    }

    fn swap_with_master(&self, window: Window, tiles: &mut VecDeque<Window>) -> Result<(), Self::Error>{
        match self.get_master_window(tiles) {
            // There is no master window, so there are no windows, so the window argument can not be
            // known
            None => Err(FullscreenWMError::UnknownWindow(window)),
            Some(_) => {
                // search position of the window arg
                match tiles.iter().position(|w| *w == window){
                    // the window argument is not managed by this window manager
                    None => Err(FullscreenWMError::UnknownWindow(window)),
                    Some(index) => {
                        tiles.swap_remove_front(index);
                        tiles.push_front(window);
                        Ok(())
                    }
                }
            }
        }
    }

    fn swap_windows(&self, window:Window, dir: PrevOrNext, tiles: &mut VecDeque<Window>){
        tiles.iter().position(|w| *w == window).and_then(|index| {
            let n = tiles.len() as i32;
            let neighbour = (neighbour_of(&(index as i32), dir) + n) % n;
            tiles.swap(index, neighbour as usize);
            Some(())
        });
    }


    fn get_window_geometry(&self, window: Window, screen: &Screen, tiles: &VecDeque<Window>) -> Result<Geometry, Self::Error>{
        let side_tile_height = if tiles.len() > 1 { screen.height / (tiles.len() - 1) as u32 } else { 0 };
        let only_master = side_tile_height == 0;
        match tiles.iter().position(|w| *w == window) {
            None => Err(FullscreenWMError::UnknownWindow(window)),
            Some(0) => Ok(Geometry {
                x: 0,
                y: 0,
                width: screen.width / if only_master { 1 } else { 2 },
                height: screen.height
            }),
            Some(index) => Ok(Geometry {
                                x: (screen.width / 2) as i32,
                                y: (index as i32 - 1) * side_tile_height as i32,
                                width: (screen.width / 2),
                                height: side_tile_height,
                            })
        }
    }
}

fn neighbour_of(&index : &i32, dir: PrevOrNext) -> i32{
    match dir {
        PrevOrNext::Prev => index - 1,
        PrevOrNext::Next => index + 1
    }
}

#[cfg(test)]
mod tests {

    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;

    // We have to import `TilingWM` from the super module.
    use super::TilingWM;
    use super::VerticalLayout;
    // We have to repeat the imports we did in the super module.
    use cplwm_api::wm::WindowManager;
    use cplwm_api::types::*;

    // We define a static variable for the screen we will use in the tests.
    // You can just as well define it as a local variable in your tests.
    static SCREEN: Screen = Screen {
        width: 800,
        height: 600,
    };

    // We define a static variable for the geometry of a fullscreen window.
    // Note that it matches the dimensions of `SCREEN`.
    static SCREEN_GEOM: Geometry = Geometry {
        x: 0,
        y: 0,
        width: 800,
        height: 600,
    };

    // We define a static variable for some random geometry that we will use
    // when adding windows to a window manager.
    static SOME_GEOM: Geometry = Geometry {
        x: 10,
        y: 10,
        width: 100,
        height: 100,
    };


    #[test]
    fn test_empty_tiling_wm(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use common test
        window_manager::test_empty_wm(wm, SCREEN);
    }

    #[test]
    fn test_adding_and_removing_some_windows(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use common test
        window_manager::test_adding_and_removing_windows(wm);
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use common test
        window_manager::test_focus_and_unfocus_window(wm);
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use common test
        window_manager::test_cycle_focus_none_and_one_window(wm);
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use common test
        window_manager::test_cycle_focus_multiple_windows(wm);
    }

    #[test]
    fn test_get_window_info(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use common test
        window_manager::test_get_window_info(wm);
    }

    #[test]
    fn test_resize_screen(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use common test
        window_manager::test_resize_screen(wm, SCREEN);
    }

    #[test]
    fn test_get_master_window(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use the common test
        tiling_support::test_master_tile(wm);
    }

    #[test]
    fn test_swap_with_master_window(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use the common test
        tiling_support::test_swap_with_master(wm);
    }


    #[test]
    fn test_swap_windows(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use the common test
        tiling_support::test_swap_windows(wm, VerticalLayout{});
    }

    #[test]
    fn test_tiling_layout(){
        // Initialize test with a new window manager
        let wm = TilingWM::new(SCREEN);
        // use the common test
        tiling_support::test_get_window_info(wm, VerticalLayout{});
    }

    //TODO: test for the VerticalLayout


}
