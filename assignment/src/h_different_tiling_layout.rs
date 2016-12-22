//! Optional: Different Tiling Layout
//!
//! Come up with a different tiling layout algorithm than the one you have
//! already implemented. If you are uninspired, feel free to look for one on
//! the [internet], but *don't forget to mention where you found it*. The
//! layout algorithm *may not be trivial*, e.g., not just adding tiles by
//! splitting the screen horizontally, and must be at least as complex as, but
//! different enough from the original layout algorithm you already had to
//! implement.
//!
//! Make a copy of your tiling window manager that implements the tiling
//! layout algorithm. This window manager has to implement the
//! [`WindowManager`] trait, but *not necessarily* the [`TilingSupport`]
//! trait, as not every layout has a master tile. Feel free to add additional
//! methods to your window manager that can be used to manipulate its layout.
//! You are not required to let this window manager implement all the previous
//! traits.
//!
//! [internet]: http://xmonad.org/xmonad-docs/xmonad-contrib/XMonad-Doc-Extending.html
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
//! This T
//!
//!
//! **TODO**: If you did not come up yourself with this layout, mention its
//! source below.
//!
//! ...

// Add imports here
use cplwm_api::types::{Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::{WindowManager, TilingSupport};

use wm_common::{TilingLayout, Manager, LayoutManager, TilingTrait};
use wm_common::error::StandardError;
use a_fullscreen_wm::FocusManager;
use b_tiling_wm::TileManager;
use std::collections::VecDeque;

/// The public type.
pub type WMName = TilingWM;


/// The TilingWM as described in the assignment. Will implement the
/// WindowManager and the TilingSupport
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct TilingWM{
    /// The manager used to manage the current focus
    pub focus_manager: FocusManager,
    /// The managar used to manage the tiles
    pub tile_manager: TileManager<BasicDockLayout>,
}

impl WindowManager for TilingWM {
    /// The Error type is StandardError.
    type Error = StandardError;

    /// constructor with given screen
    fn new(screen: Screen) -> TilingWM  {
        TilingWM {
            focus_manager: FocusManager::new(),
            tile_manager: TileManager::new(screen, BasicDockLayout{}),
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


/// Basic dock layout
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct BasicDockLayout {
}

impl TilingLayout for BasicDockLayout {
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
        match tiles.iter().position(|w| *w == window) {
            None => Err(StandardError::UnknownWindow(window)),
            Some(0) => self.get_master_window_geometry(screen, tiles).ok_or(StandardError::UnknownWindow(window)),
            Some(index) => {
                match index % 3 {
                    // Bottom dock
                    0 => {
                        let dock_tiles = tiles.iter().enumerate()
                            .filter(|&(i,_)| i % 3 == 0 && i != 0)
                            .map(|(_, t)| *t)
                            .collect();
                        self.get_bottom_dock_geometry(index / 3, screen, dock_tiles).ok_or(StandardError::UnknownWindow(window))
                    },
                    // Left dock
                    1 => {
                        let dock_tiles = tiles.iter().enumerate()
                            .filter(|&(i,_)| i % 3 == 1)
                            .map(|(_, t)| *t)
                            .collect();
                        self.get_left_dock_geometry(index / 3, screen, dock_tiles).ok_or(StandardError::UnknownWindow(window))
                    },
                    // Right dock
                    2 => {
                        let dock_tiles = tiles.iter().enumerate()
                            .filter(|&(i,_)| i % 3 == 2)
                            .map(|(_, t)| *t)
                            .collect();
                        self.get_right_dock_geometry(index / 3, screen, dock_tiles).ok_or(StandardError::UnknownWindow(window))
                    },
                    // an usize % 3 can only return 0, 1, or 2, so anything else would indicate some serious problems.
                    _ => panic!("Math broke =(")
                }
            }
        }
    }
}


impl BasicDockLayout {

    fn get_left_dock_geometry(&self, index: usize, screen: &Screen, dock_tiles: Vec<Window>) -> Option<Geometry> {
        if index >= dock_tiles.len(){
            None
        } else {
            let width_part: u32 = screen.width / 5;
            let height: u32 = (screen.height as i32 / dock_tiles.len() as i32) as u32;
            Some(Geometry {
                x: 0,
                y: index as i32 * height as i32,
                width: width_part,
                height: height,
            })
        }
    }

    fn get_right_dock_geometry(&self, index: usize, screen: &Screen, dock_tiles: Vec<Window>) -> Option<Geometry> {
        let width_part: u32 = screen.width / 5;
        let height: u32 = (screen.height as i32 / dock_tiles.len() as i32) as u32;
        Some(Geometry {
            x: screen.width as i32 - width_part as i32,
            y: index as i32 * height as i32,
            width: width_part,
            height: height,
        })
    }

    fn get_bottom_dock_geometry(&self, index: usize, screen: &Screen, dock_tiles: Vec<Window>) -> Option<Geometry> {
        let height_part: u32 = screen.height / 5;
        let width_part: u32 = screen.width / 5;
        let width: u32 = (screen.width as i32 / dock_tiles.len() as i32) as u32;
        Some(Geometry {
            x: width_part as i32 + (index as i32 - 1) * width as i32,
            y: screen.height as i32 - height_part as i32,
            width: width - 2 * width_part,
            height: height_part,
        })
    }

    fn get_master_window_geometry(&self, screen: &Screen, tiles: &VecDeque<Window>) -> Option<Geometry> {
        let width_part: u32 = screen.width / 5;
        let height_part: u32 = screen.height / 5;
        match tiles.len(){
                0 => None,
                1 => Some(Geometry{
                    x:0,
                    y:0,
                    width: screen.width,
                    height: screen.height,
                }),
                2 => Some(Geometry{
                    x: width_part as i32,
                    y:0,
                    width: screen.width - width_part,
                    height: screen.height,
                }),
                3 => Some(Geometry{
                    x: width_part as i32,
                    y:0,
                    width: screen.width - 2 * width_part,
                    height: screen.height,
                }),
                _ => Some(Geometry{
                    x: width_part as i32,
                    y:0,
                    width: screen.width - 2 * width_part,
                    height: screen.height - height_part,
                }),
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
    use super::BasicDockLayout;
    use wm_common::TilingLayout;
    use std::collections::VecDeque;
    use cplwm_api::types::*;

    static SCREEN1: Screen = Screen {
        width: 500,
        height: 500,
    };


    #[test]
    fn test_basic_dock_layout_no_window(){
        // Initialize new BasicDockLayout strategy
        let layout = BasicDockLayout{};
        // Initialize empty tile Deque
        let tiles = VecDeque::new();

        // make sure there is no geometry.
        assert!(layout.get_window_geometry(1, &SCREEN1, &tiles).is_err());
    }

    #[test]
    fn test_basic_dock_layout_one_window(){
        // Initialize new BasicDockLayout strategy
        let layout = BasicDockLayout{};
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
    fn test_basic_dock_layout_two_windows(){
        // Initialize new BasicDockLayout strategy
        let layout = BasicDockLayout{};
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 2 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 100,
            y: 0,
            width: 400,
            height: 500,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 0,
            y: 0,
            width: 100,
            height: 500,
        },layout.get_window_geometry(2, &SCREEN1, &tiles).ok().unwrap());

        // any other window should return an error
        assert!(layout.get_window_geometry(3, &SCREEN1, &tiles).is_err());
    }

    #[test]
    fn test_basic_dock_layout_three_windows(){
        // Initialize new BasicDockLayout strategy
        let layout = BasicDockLayout{};
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 2 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);
        tiles.push_back(3);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 100,
            y: 0,
            width: 300,
            height: 500,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 0,
            y: 0,
            width: 100,
            height: 500,
        },layout.get_window_geometry(2, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 400,
            y: 0,
            width: 100,
            height: 500,
        },layout.get_window_geometry(3, &SCREEN1, &tiles).ok().unwrap());

        // any other window should return an error
        assert!(layout.get_window_geometry(4, &SCREEN1, &tiles).is_err());
    }

    #[test]
    fn test_basic_dock_layout_four_windows(){
        // Initialize new BasicDockLayout strategy
        let layout = BasicDockLayout{};
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 2 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);
        tiles.push_back(3);
        tiles.push_back(4);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 100,
            y: 0,
            width: 300,
            height: 400,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 0,
            y: 0,
            width: 100,
            height: 500,
        },layout.get_window_geometry(2, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 400,
            y: 0,
            width: 100,
            height: 500,
        },layout.get_window_geometry(3, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 100,
            y: 400,
            width: 300,
            height: 100,
        },layout.get_window_geometry(4, &SCREEN1, &tiles).ok().unwrap());

        // any other window should return an error
        assert!(layout.get_window_geometry(5, &SCREEN1, &tiles).is_err());
    }
}


#[cfg(test)]
mod tests {
    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;
    use super::TilingWM;
    use super::BasicDockLayout;

    #[test]
    fn test_empty_tiling_wm(){
        window_manager::test_empty_wm::<TilingWM>();
    }

    #[test]
    fn test_adding_and_removing_some_windows(){
        window_manager::test_adding_and_removing_windows::<TilingWM>();
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        window_manager::test_focus_and_unfocus_window::<TilingWM>();
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        window_manager::test_cycle_focus_none_and_one_window::<TilingWM>();
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        window_manager::test_cycle_focus_multiple_windows::<TilingWM>();
    }

    #[test]
    fn test_get_window_info(){
        window_manager::test_get_window_info::<TilingWM>();
    }

    #[test]
    fn test_resize_screen(){
        window_manager::test_resize_screen::<TilingWM>();
    }

    #[test]
    fn test_get_master_window(){
        tiling_support::test_master_tile::<TilingWM>();
    }

    #[test]
    fn test_swap_with_master_window(){
        tiling_support::test_swap_with_master::<TilingWM>();
    }


    #[test]
    fn test_swap_windows(){
        tiling_support::test_swap_windows::<TilingWM, BasicDockLayout>(BasicDockLayout{});
    }

    #[test]
    fn test_tiling_layout(){
        tiling_support::test_get_window_info::<TilingWM, BasicDockLayout>(BasicDockLayout{});
    }
}
