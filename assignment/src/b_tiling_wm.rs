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
//! COMPLETED: YES
//!
//! COMMENTS: /
//!
//!

// Add imports here
use cplwm_api::types::{FloatOrTile, Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::{WindowManager, TilingSupport};

use wm_common::{TilingLayout, Manager, LayoutManager, TilingTrait};
use wm_common::error::StandardError;
use a_fullscreen_wm::FocusManager;
use std::collections::{HashMap,VecDeque};

/// The public type.
pub type WMName = TilingWM;


/// The TilingWM as described in the assignment. Will implement the
/// WindowManager and the TilingSupport
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct TilingWM{
    /// The manager used to manage the current focus
    pub focus_manager: FocusManager,
    /// The managar used to manage the tiles
    pub tile_manager: TileManager<VerticalLayout>,
}

impl WindowManager for TilingWM {
    /// The Error type is StandardError.
    type Error = StandardError;

    /// constructor with given screen
    fn new(screen: Screen) -> TilingWM  {
        TilingWM {
            focus_manager: FocusManager::new(),
            tile_manager: TileManager::new(screen, VerticalLayout{}),
        }
    }

    fn get_windows(&self) -> Vec<Window> {
        self.focus_manager.get_windows()
    }

    fn get_focused_window(&self) -> Option<Window> {
        self.focus_manager.get_focused_window()
    }
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        self.focus_manager.add_window(window_with_info).and_then(|_| {
            self.tile_manager.add_window(window_with_info)
        })
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        self.focus_manager.remove_window(window).and_then(|_| {
            self.tile_manager.remove_window(window)
        })
    }

    fn get_window_layout(&self) -> WindowLayout {
        WindowLayout {
            focused_window: self.get_focused_window(),
            windows: self.tile_manager.get_window_layout(),
        }
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        self.focus_manager.focus_window(window)
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focus_manager.cycle_focus(dir)
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        self.tile_manager.get_window_info(window)
    }

    fn get_screen(&self) -> Screen {
        self.tile_manager.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.tile_manager.resize_screen(screen)
    }
}

impl TilingSupport for TilingWM {
    fn get_master_window(&self) -> Option<Window> {
        self.tile_manager.get_master_window()
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error>{
        self.tile_manager.swap_with_master(window, &mut self.focus_manager)
    }

    fn swap_windows(&mut self, dir: PrevOrNext){
        self.tile_manager.swap_windows(dir, &self.focus_manager)
    }
}

/// A manager for managing the tiling of windows
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct TileManager<TL: TilingLayout> {
    /// VecDeque to keep the order of the tiles. For the simple vertical layout the first tile is
    /// the master tile.
    pub tiles: VecDeque<Window>,
    /// The original WindowInfo of the managed windows
    pub originals: HashMap<Window, WindowWithInfo>,
    /// The layout strategy this Tiling Window Manager uses.
    pub layout: TL,
    /// the screen
    pub screen: Screen,
}

impl<TL> Manager for TileManager<TL> where TL : TilingLayout<Error=StandardError> {
    type Error = StandardError;

    fn get_windows(&self) -> Vec<Window> {
        self.tiles.iter().map(|w| *w).collect()
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), StandardError> {
        if !self.is_managed(window_with_info.window) {
            self.tiles.push_back(window_with_info.window);
            self.originals.insert(window_with_info.window, window_with_info);
            Ok(())
        } else {
            Err(StandardError::AlReadyManagedWindow(window_with_info.window))
        }
    }

    fn remove_window(&mut self, window: Window) -> Result<(), StandardError> {
        match self.tiles.iter().position(|w| *w == window) {
            None => Err(StandardError::UnknownWindow(window)),
            Some(i) => {
                self.tiles.remove(i);
                self.originals.remove(&window);
                Ok(())
            }
        }
    }
}

impl<TL> LayoutManager for TileManager<TL> where TL : TilingLayout<Error=StandardError> {
    fn get_screen(&self) -> Screen {
        self.screen
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.screen = screen
    }

    fn get_window_layout(&self) -> Vec<(Window, Geometry)> {
        self.get_windows().iter()
            // We know for sure the window argument in get_window_geometry is a managed window,
            // because it comes directly from get_windows.
            .map(|window| (*window, self.get_window_geometry(*window).unwrap()))
            .collect()
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, StandardError> {
        self.get_window_geometry(window).and_then(|geometry| {
            Ok(WindowWithInfo {
                window: window,
                geometry: geometry,
                float_or_tile: FloatOrTile::Tile,
                fullscreen: false,
            })
        })
    }

    fn focus_shifted(&mut self, window: Option<Window>) -> Result<(), Self::Error>{
        // When the focus shifts, this LayoutManager does not need to do anything
        Ok(())
    }

}

impl<TL> TilingTrait for TileManager<TL> where TL : TilingLayout<Error=StandardError> {

    /// Return current master window
    fn get_master_window(&self) -> Option<Window> {
        self.layout.get_master_window(&self.tiles)
    }

    /// Swap the window with the master and focus master through the given focus_manager
    fn swap_with_master(&mut self, window: Window, focus_manager: &mut FocusManager) -> Result<(), StandardError>{
        self.layout.swap_with_master(window, &mut self.tiles).and_then(|_| {
            focus_manager.focus_window(Some(window))
        })
    }

    /// Swap currently focused window in the focus_manager with the next or previous tile
    fn swap_windows(&mut self, dir: PrevOrNext, focus_manager: &FocusManager){
        focus_manager.get_focused_window().and_then(|window| {
            self.layout.swap_windows(window, dir, &mut self.tiles);
            Some(())
        });
    }
}


impl<TL> TileManager<TL> where TL : TilingLayout<Error=StandardError>{
    /// A new, empty TileManager
    pub fn new(screen: Screen, layout: TL) -> TileManager<TL> {
        TileManager {
            tiles: VecDeque::new(),
            originals: HashMap::new(),
            layout: layout,
            screen: screen,
        }
    }

    /// Return the original WindowWithInfo of the given window
    pub fn get_original_window_info(&self, window: Window) -> Result<WindowWithInfo, StandardError> {
        self.originals.get(&window).map(|w| *w).ok_or(StandardError::UnknownWindow(window))
    }

    /// Return the current Geometry for the given window
    pub fn get_window_geometry(&self, window: Window) -> Result<Geometry, StandardError>{
        self.layout.get_window_geometry(window, &self.get_screen(), &self.tiles)
    }
}

/// A Layout algorithm for Tiling window managers as described in assigment b.
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct VerticalLayout {}

impl TilingLayout for VerticalLayout {
    type Error = StandardError;

    fn get_master_window(&self, tiles: &VecDeque<Window>) -> Option<Window>{
        return tiles.front().map(|w| *w)
    }

    fn swap_with_master(&self, window: Window, tiles: &mut VecDeque<Window>) -> Result<(), Self::Error>{
        match self.get_master_window(tiles) {
            // There is no master window, so there are no windows, so the window argument can not be
            // known
            None => Err(StandardError::UnknownWindow(window)),
            Some(_) => {
                // search position of the window arg
                match tiles.iter().position(|w| *w == window){
                    // the window argument is not managed by this window manager
                    None => Err(StandardError::UnknownWindow(window)),
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
        let only_master = tiles.len() <= 1;
        let master_tile_width = screen.width / if only_master { 1 } else { 2 };
        match tiles.iter().position(|w| *w == window) {
            None => Err(StandardError::UnknownWindow(window)),
            Some(0) => Ok(Geometry {
                x: 0,
                y: 0,
                width: master_tile_width,
                height: screen.height
            }),
            Some(index) => {
                // side tiles should get the remaining width of the screen.
                let remaining_width = screen.width - master_tile_width;
                let last_index = tiles.len() - 1;
                let side_tile_height = if tiles.len() > 1 { screen.height / (tiles.len() - 1) as u32 } else { 0 };
                if index != last_index {
                    Ok(Geometry {
                        x: (screen.width / 2) as i32,
                        y: (index as i32 - 1) * side_tile_height as i32,
                        width: remaining_width,
                        height: side_tile_height,
                    })
                } else {
                    // the last side tile should get the remaining height of the screen.
                    let remaining_height = (screen.height as i32 - side_tile_height as i32 * (last_index as i32 - 1) ) as u32;
                    Ok(Geometry {
                        x: (screen.width / 2) as i32,
                        y: (index as i32 - 1) * side_tile_height as i32,
                        width: screen.width - (screen.width / 2),
                        height: remaining_height,
                    })
                }
            }
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
mod vertical_layout_tests {
    use super::VerticalLayout;
    use wm_common::TilingLayout;
    use std::collections::VecDeque;
    use cplwm_api::types::*;

    static SCREEN1: Screen = Screen {
        width: 200,
        height: 300,
    };

    static SCREEN2: Screen = Screen {
        width: 301,
        height: 401,
    };

    #[test]
    fn test_vertical_layout_no_window(){
        // Initialize new VerticalLayout strategy
        let layout = VerticalLayout{};
        // Initialize empty tile Deque
        let tiles = VecDeque::new();

        // make sure there is no geometry.
        assert!(layout.get_window_geometry(1, &SCREEN1, &tiles).is_err());
    }

    #[test]
    fn test_vertical_layout_one_window(){
        // Initialize new VerticalLayout strategy
        let layout = VerticalLayout{};
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push one window on the Deque
        tiles.push_back(1);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 0,
            y: 0,
            width: SCREEN1.width,
            height: SCREEN1.height,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());
    }

    #[test]
    fn test_vertical_layout_two_windows(){
        // Initialize new VerticalLayout strategy
        let layout = VerticalLayout{};
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 2 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 0,
            y: 0,
            width: 100,
            height: 300,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 100,
            y: 0,
            width: 100,
            height: 300,
        },layout.get_window_geometry(2, &SCREEN1, &tiles).ok().unwrap());

        // any other window should return an error
        assert!(layout.get_window_geometry(3, &SCREEN1, &tiles).is_err());
    }

    #[test]
    fn test_vertical_layout_multiple_windows_regular_screen(){
        // Initialize new VerticalLayout strategy
        let layout = VerticalLayout{};
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 4 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);
        tiles.push_back(3);
        tiles.push_back(4);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 0,
            y: 0,
            width: 100,
            height: 300,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 100,
            y: 0,
            width: 100,
            height: 100,
        },layout.get_window_geometry(2, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 100,
            y: 100,
            width: 100,
            height: 100,
        },layout.get_window_geometry(3, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 100,
            y: 200,
            width: 100,
            height: 100,
        },layout.get_window_geometry(4, &SCREEN1, &tiles).ok().unwrap());
    }

    // test to see this layout handles tiles which should round the heights correctly
    #[test]
    fn test_vertical_layout_multiple_windows_irregular_screen(){
        // Initialize new VerticalLayout strategy
        let layout = VerticalLayout{};
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 4 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);
        tiles.push_back(3);
        tiles.push_back(4);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 0,
            y: 0,
            width: 150,
            height: 401,
        },layout.get_window_geometry(1, &SCREEN2, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 150,
            y: 0,
            width: 151,
            height: 133,
        },layout.get_window_geometry(2, &SCREEN2, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 150,
            y: 133,
            width: 151,
            height: 133,
        },layout.get_window_geometry(3, &SCREEN2, &tiles).ok().unwrap());

        // last one should get remaining screen space.
        assert_eq!(Geometry{
            x: 150,
            y: 266,
            width: 151,
            height: 135,
        },layout.get_window_geometry(4, &SCREEN2, &tiles).ok().unwrap());
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
}
