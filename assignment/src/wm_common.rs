//! Module for all common code for the Window Mangers

use rustc_serialize::{Decodable, Encodable};
use std::fmt::Debug;
use std::collections::VecDeque;

use cplwm_api::types::*;

use a_fullscreen_wm::FocusManager;

/// Trait which defines an interface to a Tiling Layout strategy
pub trait TilingLayout: Encodable + Decodable + Debug + Clone  {
    /// The type of error associated with this TilingLayout
    type Error;

    /// get the master window from the provided Deque of tiles.
    fn get_master_window(&self, tiles: &VecDeque<Window>) -> Option<Window>;
    /// swap the given window with the current window in the master tile.
    /// should return an error when the window is not in the given tiles Deque.
    fn swap_with_master(&self, window: Window, tiles: &mut VecDeque<Window>) -> Result<(), Self::Error>;
    /// swaps the given window with the next or previous window according to this TilingLayout
    fn swap_windows(&self, window:Window, dir:PrevOrNext, tiles: &mut VecDeque<Window>);
    /// get the geometry of a window in this layout from the provided Deque of tiles.
    fn get_window_geometry(&self, window: Window, screen: &Screen, tiles: &VecDeque<Window>) -> Result<Geometry, Self::Error>;
}

/// Trait which all Managers should have
pub trait Manager {
    /// The type of error this Manager can return
    type Error;
    /// Return all windows managed by this specific manager
    fn get_windows(&self) -> Vec<Window>;
    /// Add a window
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error>;
    /// Remove a window
    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error>;

    /// Returns true if window in get_windows() Vec, otherwise false
    fn is_managed(&self, window: Window) -> bool {
        self.get_windows().contains(&window)
    }
}

/// Trait that defines a Manager which has the task of managing the layout
pub trait LayoutManager : Manager {
    /// vector of the windows and their geometry in the right order
    fn get_window_layout(&self) -> Vec<(Window, Geometry)>;
    /// react to window being focused
    fn focus_shifted(&mut self, window: Option<Window>) -> Result<(), Self::Error>;
    /// get specific window_info
    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error>;
    /// get screen
    fn get_screen(&self) -> Screen;
    /// resize screen
    fn resize_screen(&mut self, screen: Screen);
}

/// Trait which describes TilingSupport for Managers
pub trait TilingTrait : LayoutManager {
    /// get the master
    fn get_master_window(&self) -> Option<Window>;
    /// swap with the master
    fn swap_with_master(&mut self, window: Window, focus_manager: &mut FocusManager) -> Result<(), Self::Error>;
    /// swap windows
    fn swap_windows(&mut self, dir: PrevOrNext, focus_manager: &FocusManager);
}

/// Trait which describes FloatSupport for Managers
pub trait FloatTrait : LayoutManager {
    /// change geometry of the floater
    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error>;
}

/// combining TilingTrait and FloatTrait
pub trait FloatAndTileTrait : TilingTrait + FloatTrait {
    /// get all floating windows
    fn get_floating_windows(&self) -> Vec<Window>;
    /// get all tiled windows
    fn get_tiled_windows(&self) -> Vec<Window>;
    /// toggle floating on window
    fn toggle_floating(&mut self, window: Window, focus_manager: &mut FocusManager) -> Result<(), Self::Error>;

    /// true when window in get_floating_windows
    fn is_floating(&self, window: Window) -> bool {
        self.get_floating_windows().contains(&window)
    }
    /// true when a window is tiled (in tiled windows)
    fn is_tiled(&self, window: Window) -> bool {
        self.get_tiled_windows().contains(&window)
    }
}

/// Module for the used error types
pub mod error {
    use cplwm_api::types::*;

    use std::error;
    use std::fmt;

    /// A simple StandardError for all WindowManagers
    #[derive(Debug)]
    pub enum StandardError {
        /// This window is not known by the window manager.
        UnknownWindow(Window),
        /// This window is already managed by the window manager.
        AlReadyManagedWindow(Window),
    }

    // This code is explained in the documentation of the associated [Error] type
    // of the `WindowManager` trait.
    impl fmt::Display for StandardError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                StandardError::UnknownWindow(ref window) => write!(f, "Unknown window: {}", window),
                StandardError::AlReadyManagedWindow(ref window) => {
                    write!(f, "Already managed window: {}", window)
                }
            }
        }
    }

    // This code is explained in the documentation of the associated [Error] type
    // of the `WindowManager` trait.
    impl error::Error for StandardError {
        fn description(&self) -> &'static str {
            match *self {
                StandardError::UnknownWindow(_) => "Unknown window",
                StandardError::AlReadyManagedWindow(_) => "Already managed window",
            }
        }
    }

    impl StandardError {
        /// convert a standard error in a FloatWMError
        pub fn to_float_error(&self) -> FloatWMError{
            match *self {
                StandardError::UnknownWindow(w) => FloatWMError::UnknownWindow(w),
                StandardError::AlReadyManagedWindow(w) => FloatWMError::AlReadyManagedWindow(w),
            }
        }
    }


    /// A simple StandardError for all WindowManagers
    #[derive(Debug)]
    pub enum FloatWMError {
        /// This window is not known by the window manager.
        UnknownWindow(Window),
        /// This window is already managed by the window manager.
        AlReadyManagedWindow(Window),
        /// This window is not floating.
        NotFloatingWindow(Window),
    }

    // This code is explained in the documentation of the associated [Error] type
    // of the `WindowManager` trait.
    impl fmt::Display for FloatWMError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                FloatWMError::UnknownWindow(ref window) => write!(f, "Unknown window: {}", window),
                FloatWMError::AlReadyManagedWindow(ref window) => {
                    write!(f, "Already managed window: {}", window)
                },
                FloatWMError::NotFloatingWindow(ref window) => {
                    write!(f, "Not floating window: {}", window)
                },
            }
        }
    }

    // This code is explained in the documentation of the associated [Error] type
    // of the `WindowManager` trait.
    impl error::Error for FloatWMError {
        fn description(&self) -> &'static str {
            match *self {
                FloatWMError::UnknownWindow(_) => "Unknown window",
                FloatWMError::AlReadyManagedWindow(_) => "Already managed window",
                FloatWMError::NotFloatingWindow(_) => "Not Floating window",
            }
        }
    }

}




/// Module which contains all the actual code to teŒst certain types of WindowManagers
pub mod tests {

    /// Module to test minimize functionality
    pub mod minimise_support {
        use cplwm_api::wm::{MinimiseSupport, FloatSupport, TilingSupport};
        use cplwm_api::types::*;

        static SCREEN: Screen = Screen {
            width: 800,
            height: 600,
        };

        static SOME_GEOM: Geometry = Geometry {
            x: 10,
            y: 10,
            width: 100,
            height: 100,
        };

        /// Test the correct order of the minimised windows
        pub fn test_minimise_order<T: MinimiseSupport>(){
            let mut wm = T::new(SCREEN);

            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_float(3, SOME_GEOM)).is_ok());

            assert_eq!(Vec::<Window>::new(), wm.get_minimised_windows());

            assert!(wm.toggle_minimised(1).is_ok());
            assert_eq!(vec![1], wm.get_minimised_windows());

            assert!(wm.toggle_minimised(3).is_ok());
            assert_eq!(vec![1,3], wm.get_minimised_windows());

            assert!(wm.toggle_minimised(2).is_ok());
            assert_eq!(vec![1,3,2], wm.get_minimised_windows());
        }

        /// Test whether toggle_minimised hides the windows, but keeps them managed, and adds them again later with the correct properties
        pub fn test_minimise<T: MinimiseSupport>(){
            let mut wm = T::new(SCREEN);

            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());

            // assert the correct properties
            assert!(wm.is_managed(1));
            assert!(wm.is_managed(2));
            assert!(!wm.is_minimised(1));
            assert!(!wm.is_minimised(2));
            assert_eq!(2, wm.get_window_layout().windows.len());

            // toggle minize on float
            assert!(wm.toggle_minimised(1).is_ok());

            // assert correct properties
            assert!(wm.is_managed(1));
            assert!(wm.is_managed(2));
            assert!(wm.is_minimised(1));
            assert!(!wm.is_minimised(2));
            assert_eq!(1, wm.get_window_layout().windows.len());

            // toggle minize on float
            assert!(wm.toggle_minimised(1).is_ok());

            // assert correct properties
            assert!(wm.is_managed(1));
            assert!(wm.is_managed(2));
            assert!(!wm.is_minimised(1));
            assert!(!wm.is_minimised(2));
            assert_eq!(2, wm.get_window_layout().windows.len());

            // toggle minize on tile
            assert!(wm.toggle_minimised(2).is_ok());

            // assert correct properties
            assert!(wm.is_managed(1));
            assert!(wm.is_managed(2));
            assert!(!wm.is_minimised(1));
            assert!(wm.is_minimised(2));
            assert_eq!(1, wm.get_window_layout().windows.len());

            // toggle minize on tile
            assert!(wm.toggle_minimised(2).is_ok());

            // assert correct properties
            assert!(wm.is_managed(1));
            assert!(wm.is_managed(2));
            assert!(!wm.is_minimised(1));
            assert!(!wm.is_minimised(2));
            assert_eq!(2, wm.get_window_layout().windows.len());
        }

        ///Test to check whether focusing to a minimised window, unminimises the window and minimising
        /// focused window makes the focus none
        pub fn test_minimise_state_after_focus<T: MinimiseSupport>(){
            let mut wm = T::new(SCREEN);

            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());

            assert_eq!(Some(2), wm.get_focused_window());

            assert!(wm.toggle_minimised(2).is_ok());

            assert_eq!(None, wm.get_focused_window());

            assert!(wm.focus_window(Some(2)).is_ok());

            assert!(!wm.is_minimised(2));
        }

        ///Test to check whether cycle focus will maximize minimised windows
        pub fn test_minimise_state_after_cycle_focus<T: MinimiseSupport>(){
            let mut wm = T::new(SCREEN);

            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());

            assert_eq!(Some(2), wm.get_focused_window());

            assert!(wm.toggle_minimised(1).is_ok());

            wm.cycle_focus(PrevOrNext::Next);

            assert!(!wm.is_minimised(1));
        }

        /// Test to check minimise keeps the window info of a float exactly the same
        pub fn test_minimise_of_floating_window<T: FloatSupport+MinimiseSupport>(){
            let mut wm = T::new(SCREEN);
            let window_with_info = WindowWithInfo::new_float(1, SOME_GEOM);

            assert!(wm.add_window(window_with_info).is_ok());
            assert_eq!(window_with_info, wm.get_window_info(1).unwrap());

            assert!(wm.toggle_minimised(1).is_ok());
            assert_eq!(window_with_info, wm.get_window_info(1).unwrap());

            assert!(wm.toggle_minimised(1).is_ok());
            assert_eq!(window_with_info, wm.get_window_info(1).unwrap());

        }

        /// Test to check minimise keeps a tiled window a tiled window
        pub fn test_minimise_of_tiled_window<T: TilingSupport+MinimiseSupport>(){
            let mut wm = T::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert_eq!(FloatOrTile::Tile, wm.get_window_info(1).unwrap().float_or_tile);

            assert!(wm.toggle_minimised(1).is_ok());
            assert_eq!(FloatOrTile::Tile, wm.get_window_info(1).unwrap().float_or_tile);

            assert!(wm.toggle_minimised(1).is_ok());
            assert_eq!(FloatOrTile::Tile, wm.get_window_info(1).unwrap().float_or_tile);

        }
    }


    /// Module for tests concerning window managers which support aswell as floating as tiled windows
    pub mod float_and_tile_support {
        use cplwm_api::wm::{TilingSupport,FloatSupport};
        use cplwm_api::types::*;

        static SCREEN: Screen = Screen {
            width: 800,
            height: 600,
        };

        static SOME_GEOM_A: Geometry = Geometry {
            x: 10,
            y: 10,
            width: 100,
            height: 100,
        };

        static SOME_GEOM_B: Geometry = Geometry {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        };

        /// toggle_floating on a window should also focus it
        pub fn test_toggle_floating_focus<T: TilingSupport+FloatSupport>(){
            let mut wm = T::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM_A)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM_A)).is_ok());
            assert_eq!(Some(2), wm.get_focused_window());
            assert!(wm.toggle_floating(1).is_ok());
            assert_eq!(Some(1), wm.get_focused_window());
        }

        /// Test: swapping tiled windows back and forth should not do anything if a floating window
        /// is focused
        pub fn test_swap_windows_with_float_focused<T: TilingSupport+FloatSupport>(){
            let mut wm = T::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM_A)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM_A)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM_A)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_float(4, SOME_GEOM_B)).is_ok());

            assert_eq!(Some(4), wm.get_focused_window());
            let initial_layout = wm.get_window_layout();

            wm.swap_windows(PrevOrNext::Prev);
            assert_eq!(initial_layout, wm.get_window_layout());

            wm.swap_windows(PrevOrNext::Next);
            assert_eq!(initial_layout, wm.get_window_layout());
        }

        /// swapping a floating window with a master when there is no master should do nothing
        pub fn test_swapping_master_with_floating_window_no_tiles<T: TilingSupport+FloatSupport>(){
            let mut wm = T::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_float(2, SOME_GEOM_B)).is_ok());

            assert!(wm.swap_with_master(2).is_ok());
            assert!(wm.get_master_window().is_none());
            assert!(wm.is_floating(2));
            assert_eq!(SOME_GEOM_B, wm.get_window_info(2).unwrap().geometry);
        }

        /// test swapping floating window with master tile works as expected: current master -> floating, floating -> master
        pub fn test_swapping_master_with_floating_window<T: TilingSupport+FloatSupport>(){
            let mut wm = T::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM_A)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_float(2, SOME_GEOM_B)).is_ok());

            assert!(wm.swap_with_master(2).is_ok());
            assert_eq!(2, wm.get_master_window().unwrap());
            assert!(wm.is_floating(1));
            assert_eq!(SOME_GEOM_A, wm.get_window_info(1).unwrap().geometry);

        }

        /// test swap_windows does nothing for a floating window
        pub fn test_swap_windows_on_floating<T: TilingSupport+FloatSupport>(){
            let mut wm = T::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM_A)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_float(2, SOME_GEOM_B)).is_ok());

            wm.swap_windows(PrevOrNext::Prev);

            assert_eq!(1, wm.get_master_window().unwrap());
            assert!(wm.is_floating(2));
            assert_eq!(2, wm.get_focused_window().unwrap());

            wm.swap_windows(PrevOrNext::Next);

            assert_eq!(1, wm.get_master_window().unwrap());
            assert!(wm.is_floating(2));
            assert_eq!(2, wm.get_focused_window().unwrap());
        }

    }

    /// Module for all test concerning FloatSupport trait
    pub mod float_support {
        use cplwm_api::wm::FloatSupport;
        use cplwm_api::types::*;

        static SCREEN: Screen = Screen {
            width: 800,
            height: 600,
        };

        static SOME_GEOM: Geometry = Geometry {
            x: 10,
            y: 10,
            width: 100,
            height: 100,
        };

        static SOME_GEOM_A: Geometry = Geometry {
            x: 10,
            y: 20,
            width: 100,
            height: 400,
        };

        static SOME_GEOM_B: Geometry = Geometry {
            x: 0,
            y: 10,
            width: 133,
            height: 100,
        };

        /// Test the get_floating_windows functionality
        pub fn test_get_floating_windows<F: FloatSupport>(){
            let mut wm = F::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_float(2, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_float(3, SOME_GEOM)).is_ok());

            let floaters = wm.get_floating_windows();

            assert!(floaters.contains(&1));
            assert!(floaters.contains(&2));
            assert!(floaters.contains(&3));
            assert!(!floaters.contains(&4));
        }

        /// Test toggle_floating
        pub fn test_toggle_floating<F: FloatSupport>(){
            let mut wm = F::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM_A)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM_B)).is_ok());

            assert_eq!(SOME_GEOM_A, wm.get_window_info(1).unwrap().geometry);

            assert!(wm.is_floating(1));
            assert!(!wm.is_floating(2));

            assert!(wm.toggle_floating(1).is_ok());
            assert!(!wm.is_floating(1));

            assert!(wm.toggle_floating(2).is_ok());
            assert!(wm.is_floating(2));
            assert_eq!(SOME_GEOM_B, wm.get_window_info(2).unwrap().geometry);


            assert!(wm.toggle_floating(1).is_ok());
            assert!(wm.is_floating(1));
            assert_eq!(SOME_GEOM_A, wm.get_window_info(1).unwrap().geometry);


            assert!(wm.toggle_floating(2).is_ok());
            assert!(!wm.is_floating(2));

            assert!(wm.toggle_floating(3).is_err());
        }

        /// Test set_window_geometry
        pub fn test_set_window_geometry<F: FloatSupport>(){
            let mut wm = F::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());

            assert_eq!(SOME_GEOM, wm.get_window_info(1).unwrap().geometry);

            let other_geometry = Geometry {
                x: 20,
                y: 30,
                width: 300,
                height: 200,
            };

            assert!(wm.set_window_geometry(1, other_geometry).is_ok());
            assert_eq!(other_geometry, wm.get_window_info(1).unwrap().geometry);

            assert!(wm.set_window_geometry(2, other_geometry).is_err());

            assert!(wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).is_ok());
            assert!(wm.set_window_geometry(3, other_geometry).is_err());
        }

        /// Test to check floating windows are above tiled windows
        pub fn test_window_layout_order<F: FloatSupport>(){
            let mut wm = F::new(SCREEN);
            assert_eq!(WindowLayout::new(), wm.get_window_layout());

            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).is_ok());

            let window_layout = wm.get_window_layout().windows;
            assert_eq!(3, window_layout[0].0);
            assert_eq!(1, window_layout[1].0);

            assert!(wm.add_window(WindowWithInfo::new_float(2, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(4, SOME_GEOM)).is_ok());

            let window_layout = wm.get_window_layout().windows;
            assert_eq!(3, window_layout[0].0);
            assert_eq!(4, window_layout[1].0);
            assert_eq!(1, window_layout[2].0);
            assert_eq!(2, window_layout[3].0);
        }

        /// Test to check focusing on a floating window puts it on top
        pub fn test_focus_floating_window_order<F: FloatSupport>(){
            let mut wm = F::new(SCREEN);
            assert_eq!(WindowLayout::new(), wm.get_window_layout());

            assert!(wm.add_window(WindowWithInfo::new_float(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_float(2, SOME_GEOM)).is_ok());

            let window_layout = wm.get_window_layout().windows;
            assert_eq!(1, window_layout[0].0);
            assert_eq!(2, window_layout[1].0);

            assert!(wm.focus_window(Some(1)).is_ok());

            let window_layout = wm.get_window_layout().windows;
            assert_eq!(2, window_layout[0].0);
            assert_eq!(1, window_layout[1].0);
        }

    }

    /// Module for all tests concerning the TilingSupport trait.
    pub mod tiling_support {
        use std::collections::VecDeque;
        use wm_common::TilingLayout;
        use cplwm_api::wm::TilingSupport;
        use cplwm_api::types::*;

        static SCREEN: Screen = Screen {
            width: 800,
            height: 600,
        };

        // A random, unimportant Geometry
        static SOME_GEOM: Geometry = Geometry {
            x: 10,
            y: 10,
            width: 100,
            height: 100,
        };

        /// test if there is a master window when there are windows, and no master tile if there
        /// are on windows.
        pub fn test_master_tile<T: TilingSupport>() {
            let mut wm = T::new(SCREEN);
            assert_eq!(None, wm.get_master_window());
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert_eq!(Some(1), wm.get_master_window());
        }

        /// test swap_with_master swaps with the master and focusses the window.
        pub fn test_swap_with_master<T: TilingSupport>(){
            let mut wm = T::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());

            assert!(wm.swap_with_master(2).is_ok());
            assert_eq!(Some(2), wm.get_focused_window());
            assert_eq!(Some(2), wm.get_master_window());

            // if the window to swap is the master tile, nothing should change.
            assert!(wm.swap_with_master(2).is_ok());
            assert_eq!(Some(2), wm.get_focused_window());
            assert_eq!(Some(2), wm.get_master_window());
        }

        /// test swap_windows swaps the windows
        pub fn test_swap_windows<TS: TilingSupport, TL: TilingLayout>(layout: TL){
            let mut wm = TS::new(SCREEN);

            wm.swap_windows(PrevOrNext::Next);
            assert_eq!(None, wm.get_master_window());
            assert_eq!(None, wm.get_focused_window());

            wm.swap_windows(PrevOrNext::Prev);
            assert_eq!(None, wm.get_master_window());
            assert_eq!(None, wm.get_focused_window());

            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());

            wm.swap_windows(PrevOrNext::Next);
            assert_eq!(Some(1), wm.get_master_window());
            assert_eq!(Some(1), wm.get_focused_window());

            wm.swap_windows(PrevOrNext::Prev);
            assert_eq!(Some(1), wm.get_master_window());
            assert_eq!(Some(1), wm.get_focused_window());

            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(4, SOME_GEOM)).is_ok());
            assert!(wm.focus_window(Some(3)).is_ok());

            // we expect the tiles Deque given to the layout initially is [1,2,3,4]
            let mut tiles = VecDeque::<Window>::new();
            tiles.push_back(1);
            tiles.push_back(2);
            tiles.push_back(3);
            tiles.push_back(4);

            wm.swap_windows(PrevOrNext::Next);
            assert_eq!(Some(1), wm.get_master_window());
            assert_eq!(Some(3), wm.get_focused_window());

            // check if the layout changed as expected and therefor the order in the Deque
            layout.swap_windows(3, PrevOrNext::Next, &mut tiles);
            let expected_layout = layout.get_window_geometry(3, &wm.get_screen(), &tiles).ok().unwrap();
            let actual_layout = wm.get_window_info(3).unwrap().geometry;
            assert_eq!(expected_layout, actual_layout);

            wm.swap_windows(PrevOrNext::Prev);
            assert_eq!(Some(1), wm.get_master_window());
            assert_eq!(Some(3), wm.get_focused_window());

            // check if the layout changed as expected and therefor the order in the Deque
            layout.swap_windows(3, PrevOrNext::Prev, &mut tiles);
            let expected_layout = layout.get_window_geometry(3, &wm.get_screen(), &tiles).ok().unwrap();
            let actual_layout = wm.get_window_info(3).unwrap().geometry;
            assert_eq!(expected_layout, actual_layout);
        }

        /// test if get_window_info returns the expected layout for the window
        pub fn test_get_window_info<TS: TilingSupport, TL: TilingLayout>(layout: TL){
            let mut wm = TS::new(SCREEN);
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(4, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(5, SOME_GEOM)).is_ok());

            let mut tiles = VecDeque::<Window>::new();
            tiles.push_back(1);
            tiles.push_back(2);
            tiles.push_back(3);
            tiles.push_back(4);
            tiles.push_back(5);

            for tile in &tiles {
                let expected_layout = layout.get_window_geometry(*tile, &wm.get_screen(), &tiles).ok().unwrap();
                let actual_layout = wm.get_window_info(*tile).unwrap().geometry;
                assert_eq!(expected_layout, actual_layout);
            }

        }

    }


    /// Module for all tests concerning the WindowManager trait.
    pub mod window_manager {
        use cplwm_api::wm::WindowManager;
        use cplwm_api::types::*;

        static SCREEN: Screen = Screen {
            width: 800,
            height: 600,
        };

        // A random, unimportant Geometry
        static SOME_GEOM: Geometry = Geometry {
            x: 10,
            y: 10,
            width: 100,
            height: 100,
        };

        /// test if the given window manager is initialized the right way
        pub fn test_empty_wm<T : WindowManager>(){
            let wm = T::new(SCREEN);
            assert_eq!(WindowLayout::new(), wm.get_window_layout());
            assert_eq!(0, wm.get_windows().len());
            assert_eq!(SCREEN, wm.get_screen())
        }

        /// test for adding and removing windows to the WindowManager
        pub fn test_adding_and_removing_windows<T: WindowManager>() {
            let mut wm = T::new(SCREEN);
            let window_a = WindowWithInfo::new_tiled(1, SOME_GEOM);
            let window_b = WindowWithInfo::new_tiled(2, SOME_GEOM);

            assert!(wm.add_window(window_a).is_ok());
            assert!(wm.is_managed(1));
            assert_eq!(vec![1], wm.get_windows());
            assert_eq!(Some(1), wm.get_focused_window());

            assert!(wm.add_window(window_b).is_ok());
            assert!(wm.is_managed(2));
            //TODO: returned windows should be sorted because the order does not have to be gaurenteed.
            assert_eq!(vec![1,2], wm.get_windows());
            assert_eq!(Some(2), wm.get_focused_window());

            assert!(wm.remove_window(1).is_ok());
            assert!(!wm.is_managed(1));
            assert!(wm.is_managed(2));
            assert_eq!(vec![2], wm.get_windows());
            assert_eq!(Some(2), wm.get_focused_window());

            assert!(wm.add_window(window_a).is_ok());
            assert_eq!(Some(1), wm.get_focused_window());
            assert!(wm.remove_window(1).is_ok());
            assert!(!wm.is_managed(1));
            assert!(wm.is_managed(2));
            assert_eq!(vec![2], wm.get_windows());
            assert_eq!(Some(2), wm.get_focused_window());

            assert!(wm.add_window(window_b).is_err(), "adding window twice should error");
            assert!(wm.remove_window(300).is_err(), "removing unmanaged window should error");
        }

        /// test for the focus functionality in WindowManager
        pub fn test_focus_and_unfocus_window<T: WindowManager>() {
            let mut wm = T::new(SCREEN);
            // Assert the initial focused_window window is None
            assert_eq!(None, wm.get_focused_window());
            assert_eq!(None, wm.get_window_layout().focused_window);

            // Add one window, it should be focused
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);

            // Unfocus all windows, None should be focused again
            assert!(wm.focus_window(None).is_ok());
            assert_eq!(None, wm.get_focused_window());
            assert_eq!(None, wm.get_window_layout().focused_window);

            // Focus window 1 again, it should be focused
            assert!(wm.focus_window(Some(1)).is_ok());
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);

            // Focussing an unmanged window should return an error.
            assert!(wm.focus_window(Some(404)).is_err());
        }

        /// test for cycle_focus in a WindowManager with no windows and with one window
        pub fn test_cycle_focus_none_and_one_window<T: WindowManager>() {
            let mut wm = T::new(SCREEN);
            // Assert the initial focused_window window is None
            assert_eq!(None, wm.get_focused_window());
            assert_eq!(None, wm.get_window_layout().focused_window);

            // Cycle does nothing when there are no windows
            wm.cycle_focus(PrevOrNext::Next);
            assert_eq!(None, wm.get_focused_window());
            assert_eq!(None, wm.get_window_layout().focused_window);
            wm.cycle_focus(PrevOrNext::Prev);
            assert_eq!(None, wm.get_focused_window());
            assert_eq!(None, wm.get_window_layout().focused_window);

            // Add one window, it should be focused
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);

            // When there is only one window, focus it if it no window is focused, otherwise do nothing
            wm.cycle_focus(PrevOrNext::Next);
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);
            wm.cycle_focus(PrevOrNext::Prev);
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);
            assert!(wm.focus_window(None).is_ok());
            wm.cycle_focus(PrevOrNext::Next);
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);
            assert!(wm.focus_window(None).is_ok());
            wm.cycle_focus(PrevOrNext::Prev);
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);
        }

        /// test for cycle_focus in a WindowManager with multiple windows
        pub fn test_cycle_focus_multiple_windows<T: WindowManager>() {
            let mut wm = T::new(SCREEN);
            //make sure the given WindowManager is empty
            assert_eq!(0, wm.get_windows().len());

            // Add 3 window to the initial WindowManager and
            // assert the last one added is focused
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).is_ok());
            assert_eq!(Some(3), wm.get_focused_window());
            assert_eq!(Some(3), wm.get_window_layout().focused_window);

            // Cycle back should focus on the previous window
            wm.cycle_focus(PrevOrNext::Prev);
            assert_eq!(Some(2), wm.get_focused_window());
            assert_eq!(Some(2), wm.get_window_layout().focused_window);

            // Going back and forth shouldn't cange the focused window
            wm.cycle_focus(PrevOrNext::Next);
            assert_eq!(Some(3), wm.get_focused_window());
            assert_eq!(Some(3), wm.get_window_layout().focused_window);

            // Cycle forth should focus on the next window
            wm.cycle_focus(PrevOrNext::Next);
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);

            // When no window is focused, any window may become focused
            assert!(wm.focus_window(None).is_ok());
            wm.cycle_focus(PrevOrNext::Prev);
            assert_eq!(Some(1), wm.get_focused_window());
            assert_eq!(Some(1), wm.get_window_layout().focused_window);
        }

        /// test for resize screen in WindowManager
        pub fn test_resize_screen<T: WindowManager>() {
            let mut wm = T::new(SCREEN);
            let new_screen = Screen {
                width: 1000,
                height: 1000,
            };
            assert_eq!(SCREEN, wm.get_screen());
            wm.resize_screen(new_screen);
            assert_eq!(new_screen, wm.get_screen());
        }

        /// test each window should have window info available
        pub fn test_get_window_info<T: WindowManager>(){
            let mut wm = T::new(SCREEN);
            // Add 3 window to the initial WindowManager
            assert!(wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).is_ok());
            assert!(wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).is_ok());

            for window in wm.get_windows() {
                assert!(wm.get_window_info(window).is_ok(), "should return window_info for each managed window");
            }
            assert!(wm.get_window_info(300).is_err(), "should error for unmanaged window");
        }
    }
}
