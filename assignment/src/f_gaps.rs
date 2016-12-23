//! Optional: Gaps
//!
//! Extend your window manager with support for gaps, i.e. the ability to add
//! some space between the different tiles. See the documentation of the
//! [`GapSupport`] trait for the precise requirements.
//!
//! Make a copy of your tiling window manager from assignment B and let it
//! implement the [`GapSupport`] trait. You are not required to let this
//! window manager implement all the previous traits.
//!
//! [`GapSupport`]: ../../cplwm_api/wm/trait.GapSupport.html
//!
//! # Status
//!
//! COMPLETED: YES
//!
//! COMMENTS: /
//!

// Add imports here
use std::cmp;
use std::collections::VecDeque;
use cplwm_api::types::{Window, PrevOrNext, Screen, Geometry, GapSize, WindowWithInfo, WindowLayout};
use cplwm_api::wm::{WindowManager, TilingSupport, GapSupport};
use wm_common::{Manager, LayoutManager, TilingTrait, TilingLayout, GapTrait};
use wm_common::error::StandardError;
use a_fullscreen_wm::FocusManager;
use b_tiling_wm::{TileManager, VerticalLayout};


/// The public type.
pub type WMName = TilingWM;


/// The TilingWM as described in the assignment. Will implement the
/// WindowManager and the TilingSupport
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct TilingWM{
    /// The manager used to manage the current focus
    pub focus_manager: FocusManager,
    /// The managar used to manage the tiles
    pub tile_manager: TileManager<GapLayout<VerticalLayout>>,
}

impl WindowManager for TilingWM {
    /// The Error type is StandardError.
    type Error = StandardError;

    /// constructor with given screen
    fn new(screen: Screen) -> TilingWM  {
        TilingWM {
            focus_manager: FocusManager::new(),
            tile_manager: TileManager::new(screen,
                GapLayout{
                    tiling_layout: VerticalLayout{},
                    gap: 0,
                }),
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

impl GapSupport for TilingWM {
    fn get_gap(&self) -> GapSize {
        self.tile_manager.get_gap()
    }
    fn set_gap(&mut self, gap: GapSize){
        self.tile_manager.set_gap(gap)
    }
}

impl<T : GapTrait> TileManager<T> {
    fn get_gap(&self) -> GapSize {
        self.layout.get_gap()
    }

    fn set_gap(&mut self, gap: GapSize) {
        self.layout.set_gap(gap)
    }
}

/// A TIlingLayout which wraps another layout and adds a gap
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct GapLayout<T: TilingLayout> {
    /// size of the gap
    pub gap: GapSize,
    /// the underlying layout strategy
    pub tiling_layout: T,
}

impl<T: TilingLayout> GapTrait for GapLayout<T> {
    fn get_gap(&self) -> GapSize {
        self.gap
    }

    fn set_gap(&mut self, gap: GapSize) {
        self.gap = gap;
    }
}

impl<T: TilingLayout> TilingLayout for GapLayout<T> {
    // use the same type for Error as the wrapped layout
    type Error = T::Error;

    fn get_master_window(&self, tiles: &VecDeque<Window>) -> Option<Window>{
        self.tiling_layout.get_master_window(tiles)
    }
    fn swap_with_master(&self, window: Window, tiles: &mut VecDeque<Window>) -> Result<(), Self::Error>{
        self.tiling_layout.swap_with_master(window, tiles)
    }
    fn swap_windows(&self, window:Window, dir:PrevOrNext, tiles: &mut VecDeque<Window>){
        self.tiling_layout.swap_windows(window, dir, tiles)
    }
    fn get_window_geometry(&self, window: Window, screen: &Screen, tiles: &VecDeque<Window>) -> Result<Geometry, Self::Error>{
        self.tiling_layout.get_window_geometry(window, screen, tiles).and_then(|geometry| {
            Ok(Geometry{
                x: geometry.x + self.gap as i32,
                y: geometry.y + self.gap as i32,
                width: cmp::max(0, geometry.width as i32 - 2 * self.gap as i32) as u32,
                height: cmp::max(0, geometry.height as i32 - 2 * self.gap as i32) as u32,
            })
        })
    }
}


#[cfg(test)]
mod vertical_layout_tests {
    use super::GapLayout;
    use wm_common::TilingLayout;
    use b_tiling_wm::VerticalLayout;
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
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
        // Initialize empty tile Deque
        let tiles = VecDeque::new();

        // make sure there is no geometry.
        assert!(layout.get_window_geometry(1, &SCREEN1, &tiles).is_err());
    }

    #[test]
    fn test_vertical_layout_one_window(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
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
    fn test_vertical_layout_one_window_gapped(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 5
        };
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push one window on the Deque
        tiles.push_back(1);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 5,
            y: 5,
            width: SCREEN1.width - 10,
            height: SCREEN1.height - 10,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());
    }

    #[test]
    fn test_vertical_layout_two_windows(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
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
    fn test_vertical_layout_two_windows_gapped(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 5
        };
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 2 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 5,
            y: 5,
            width: 90,
            height: 290,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 105,
            y: 5,
            width: 90,
            height: 290,
        },layout.get_window_geometry(2, &SCREEN1, &tiles).ok().unwrap());

        // any other window should return an error
        assert!(layout.get_window_geometry(3, &SCREEN1, &tiles).is_err());
    }

    #[test]
    fn test_vertical_layout_multiple_windows_regular_screen(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
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

    #[test]
    fn test_vertical_layout_multiple_windows_regular_screen_gapped(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 5
        };
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 4 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);
        tiles.push_back(3);
        tiles.push_back(4);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 5,
            y: 5,
            width: 90,
            height: 290,
        },layout.get_window_geometry(1, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 105,
            y: 5,
            width: 90,
            height: 90,
        },layout.get_window_geometry(2, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 105,
            y: 105,
            width: 90,
            height: 90,
        },layout.get_window_geometry(3, &SCREEN1, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 105,
            y: 205,
            width: 90,
            height: 90,
        },layout.get_window_geometry(4, &SCREEN1, &tiles).ok().unwrap());
    }

    // test to see this layout handles tiles which should round the heights correctly
    #[test]
    fn test_vertical_layout_multiple_windows_irregular_screen(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
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

    // test to see this layout handles tiles which should round the heights correctly
    #[test]
    fn test_vertical_layout_multiple_windows_irregular_screen_gapped(){
        // Initialize new GapLayout strategy
        let layout = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 5
        };
        // Initialize empty tile Deque
        let mut tiles = VecDeque::new();
        // Push 4 tiles on the Deque, the first one will be the master in this layout.
        tiles.push_back(1);
        tiles.push_back(2);
        tiles.push_back(3);
        tiles.push_back(4);

        // compare to exptected geometry
        assert_eq!(Geometry{
            x: 5,
            y: 5,
            width: 140,
            height: 391,
        },layout.get_window_geometry(1, &SCREEN2, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 155,
            y: 5,
            width: 141,
            height: 123,
        },layout.get_window_geometry(2, &SCREEN2, &tiles).ok().unwrap());

        assert_eq!(Geometry{
            x: 155,
            y: 138,
            width: 141,
            height: 123,
        },layout.get_window_geometry(3, &SCREEN2, &tiles).ok().unwrap());

        // last one should get remaining screen space.
        assert_eq!(Geometry{
            x: 155,
            y: 271,
            width: 141,
            height: 125,
        },layout.get_window_geometry(4, &SCREEN2, &tiles).ok().unwrap());
    }
}


#[cfg(test)]
mod tests {
    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;
    use wm_common::tests::gap_support;
    use super::TilingWM;
    use super::GapLayout;
    use b_tiling_wm::VerticalLayout;

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
        let layout: GapLayout<VerticalLayout> = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
        tiling_support::test_swap_windows::<TilingWM, GapLayout<VerticalLayout>>(layout);
    }

    #[test]
    fn test_tiling_layout(){
        let layout: GapLayout<VerticalLayout> = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
        tiling_support::test_get_window_info::<TilingWM, GapLayout<VerticalLayout>>(layout);
    }

    #[test]
    fn test_set_gap(){
        let layout: GapLayout<VerticalLayout> = GapLayout {
            tiling_layout: VerticalLayout{},
            gap: 0
        };
        gap_support::test_set_gap::<TilingWM, GapLayout<VerticalLayout>>(layout);
    }
}
