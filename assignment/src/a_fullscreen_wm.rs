//! Fullscreen Window Manager
//!
//! Implement the [`WindowManager`] trait by writing a simple window manager
//! that displays every window fullscreen. When a new window is added, the
//! last window that was visible will become invisible.
//!
//! [`WindowManager`]: ../../cplwm_api/wm/trait.WindowManager.html
//!
//! Now have a look at the source code of this file, it contains a tutorial to
//! help you write the fullscreen window manager.
//!
//! You are free to remove the documentation in this file that is only part of
//! the tutorial or no longer matches the code after your changes.
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

// Import some other used types.
use std::collections::{BTreeMap, VecDeque};

// Import some types and the WindowManager trait from the cplwm_api crate
// (defined in the api folder).
use cplwm_api::types::{FloatOrTile, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::WindowManager;
use wm_common::error::StandardError;

/// You are free to choose the name for your window manager. As we will use
/// automated tests when grading your assignment, indicate here the name of
/// your window manager data type so we can just use `WMName` instead of
/// having to manually figure out your window manager name.
pub type WMName = FullscreenWM;


/// The FullscreenWM struct
///
/// # Example Representation
///
/// The fullscreen window manager that we are implementing is very simple: it
/// just needs to keep track of all the windows that were added and remember
/// which one is focused. It is not even necessary to remember the geometries
/// of the windows, as they will all be resized to the size of the screen.
///
/// To understand the `#derive[(..)]` line before the struct, read the
/// [Supertraits] section of the `WindowManager` trait.
///
/// [Supertraits]: ../../cplwm_api/wm/trait.WindowManager.html#supertraits
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FullscreenWM {
    /// A vector deque of windows, the first one is the next one to be focused, the last one is
    /// the previous one to be focused.
    pub windows: VecDeque<Window>,
    /// Currently focused window.
    pub focused_window: Option<Window>,
    /// We need to know which size the fullscreen window must be.
    pub screen: Screen,
    /// A BTreeMap to map windows to the given window info
    pub window_to_info: BTreeMap<Window, WindowWithInfo>,
}



// Now we start implementing our window manager
impl WindowManager for FullscreenWM {
    /// We use `StandardError` as our `Error` type.
    type Error = StandardError;

    /// The constructor is straightforward.
    ///
    /// Track the given screen, no window is initially focused, and add empty Deque and TreeMap
    fn new(screen: Screen) -> FullscreenWM {
        FullscreenWM {
            windows: VecDeque::new(),
            focused_window: None,
            screen: screen,
            window_to_info: BTreeMap::new(),
        }
    }

    /// The `windows` field contains all the windows we manage.
    fn get_windows(&self) -> Vec<Window> {
        let mut windows: Vec<Window> = self.windows.iter().map(|w| *w).collect();
        match self.focused_window {
            Some(w) => windows.push(w),
            None => {}
        }
        return windows;
    }

    /// Returns the currently focused window
    fn get_focused_window(&self) -> Option<Window> {
        self.focused_window
    }

    /// Add a new window. Focus on the window and push the previous window at the back of the Deque,
    /// it there is one. Add the given WindowWithInfo to the window_to_info BTreeMap for future use.
    /// Returns an AlReadyManagedWindow error when the given window is already managed by this
    /// window manager.
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        if !self.is_managed(window_with_info.window) {
            match self.focused_window {
                Some(w) => self.windows.push_back(w),
                None => {}
            }
            self.focused_window = Some(window_with_info.window);
            self.window_to_info.insert(window_with_info.window, window_with_info);
            Ok(())
        } else {
            Err(StandardError::AlReadyManagedWindow(window_with_info.window))
        }
    }

    /// Remove a window. If the window is focused, simple focus the previous window. Otherwise
    /// / remove the window from the Deque. Remove any additional information of the window in the
    /// BTreeMap. Returns an UnknowWindow error when the given window is not managed by this
    /// window manager
    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        match self.focused_window {
            Some(w) => {
                if w == window {
                    self.focused_window = self.windows.pop_back();
                    self.window_to_info.remove(&w);
                    return Ok(());
                }
            }
            None => {}
        }
        match self.windows.iter().position(|w| *w == window) {
            None => Err(StandardError::UnknownWindow(window)),
            Some(i) => {
                let removed_window = self.windows.remove(i);
                // after looking up the index of a window, finding it, and then removing said
                // window based on the index, we are 100% there is a window to remove and this
                // function call will always return a valid Ok.
                self.window_to_info.remove(&removed_window.unwrap());
                Ok(())
            }
        }
    }

    /// Now the most important part: calculating the `WindowLayout`.
    ///
    /// First we build a `Geometry` for a fullscreen window using the
    /// `to_geometry` method: it has the same width and height as the screen.
    ///
    /// Then we look at the focused_window
    ///
    /// * When the `Option` contains `Some(w)`, we know that there was at
    ///   least one window, and `w`, being the last window in the `Vec` should
    ///   be focused. As the other windows will not be visible, the `windows`
    ///   field of `WindowLayout` can just be a `Vec` with one element: the
    ///   one window along with the fullscreen `Geometry`.
    ///
    /// * When the `Option` is `None`, we know that there are no windows, so
    ///   we can just return an empty `WindowLayout`.
    ///
    fn get_window_layout(&self) -> WindowLayout {
        let fullscreen_geometry = self.screen.to_geometry();
        match self.focused_window {
            // If there is at least one window.
            Some(w) => {
                WindowLayout {
                    // The last window is focused ...
                    focused_window: Some(w),
                    // ... and should fill the screen. The other windows are
                    // simply hidden.
                    windows: vec![(w, fullscreen_geometry)],
                }
            }
            // Otherwise, return an empty WindowLayout
            None => WindowLayout::new(),
        }
    }

    /// Puts the given window in focused_window. If None is given, None is focused.
    /// Returns an UnknownWindow error when the given window si not managed by this window manager
    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        match self.focused_window {
            /// if there is a focused window, put it at the back of the Deque and unfocus it
            Some(w) => {
                self.windows.push_back(w);
                self.focused_window = None;
            }
            None => {}
        };
        match window {
            /// if there is no window to focus, than we are done
            None => Ok(()),
            Some(window_value) => {
                match self.windows.iter().position(|w| *w == window_value) {
                    None => Err(StandardError::UnknownWindow(window_value)),
                    Some(i) => {
                        self.windows.remove(i);
                        self.focused_window = window;
                        Ok(())
                    }
                }
            }
        }
    }

    /// When cycling to Next, the window at the front of the deque is focused and the currently
    /// focused window is put at the back of the deque.
    /// When cycling to Prev, the window at the back of the deque is focused and the currently
    /// focused window is put at the front of the deque
    fn cycle_focus(&mut self, dir: PrevOrNext) {
        match dir {
            PrevOrNext::Next => {
                self.focused_window.and_then(|w| {
                    self.windows.push_back(w);
                    Some(w)
                });
                self.focused_window = self.windows.pop_front()
            }
            PrevOrNext::Prev => {
                self.focused_window.and_then(|w| {
                    self.windows.push_front(w);
                    Some(w)
                });
                self.focused_window = self.windows.pop_back()
            }
        }
    }

    /// Returns a window's info in this window manager. The info is adapted to this window manager
    /// (the geometry is set to full screen and fullscreen is set to true)
    /// Returns an UnknownWindow error when the given window si not managed by this window manager
    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        let window_info = self.window_to_info.get(&window);
        match window_info {
            None => Err(StandardError::UnknownWindow(window)),
            Some(w) => {
                Ok(WindowWithInfo {
                    window: w.window,
                    geometry: self.screen.to_geometry(),
                    float_or_tile: FloatOrTile::Tile,
                    fullscreen: true,
                })
            }
        }
    }

    /// Returns the screen
    fn get_screen(&self) -> Screen {
        self.screen
    }

    /// Replace the current screen with the new screen
    fn resize_screen(&mut self, screen: Screen) {
        self.screen = screen
    }
}


#[cfg(test)]
mod tests {

    // We have to import `FullscreenWM` from the super module.
    use super::FullscreenWM;
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
    fn test_adding_and_removing_some_windows() {
        // Let's make a new `FullscreenWM` with `SCREEN` as screen.
        let mut wm = FullscreenWM::new(SCREEN);

        // Initially the window layout should be empty.
        assert_eq!(WindowLayout::new(), wm.get_window_layout());
        // `assert_eq!` is a macro that will check that the second argument,
        // the actual value, matches first value, the expected value.

        // Let's add a window
        wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
        // Because `add_window` returns a `Result`, we use `unwrap`, which
        // tries to extract the `Ok` value from the result, but will panic
        // (crash) when it is an `Err`. You must be very careful when using
        // `unwrap` in your code. Here we can use it because we know for sure
        // that an `Err` won't be returned, and even if that were the case,
        // the panic will simply cause the test to fail.

        // The window should now be managed by the WM
        assert!(wm.is_managed(1));
        // and be present in the `Vec` of windows.
        assert_eq!(vec![1], wm.get_windows());
        // According to the window layout
        let wl1 = wm.get_window_layout();
        // it should be focused
        assert_eq!(Some(1), wl1.focused_window);
        // and fullscreen.
        assert_eq!(vec![(1, SCREEN_GEOM)], wl1.windows);

        // Let's add another window.
        wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();
        // It should now be managed by the WM.
        assert!(wm.is_managed(2));
        // The `Vec` of windows should now contain both windows 1 and 2.
        assert_eq!(vec![1, 2], wm.get_windows());
        // According to the window layout
        let wl2 = wm.get_window_layout();
        // window 2 should be focused
        assert_eq!(Some(2), wl2.focused_window);
        // and fullscreen.
        assert_eq!(vec![(2, SCREEN_GEOM)], wl2.windows);

        // Now let's remove window 2
        wm.remove_window(2).unwrap();
        // It should no longer be managed by the WM.
        assert!(!wm.is_managed(2));
        // The `Vec` of windows should now just contain window 1.
        assert_eq!(vec![1], wm.get_windows());
        // According to the window layout
        let wl3 = wm.get_window_layout();
        // window 1 should be focused again
        assert_eq!(Some(1), wl3.focused_window);
        // and fullscreen.
        assert_eq!(vec![(1, SCREEN_GEOM)], wl3.windows);


        // To run these tests, run the command `cargo test` in the `solution`
        // directory.
        //
        // To learn more about testing, check the Testing chapter of the Rust
        // Book: https://doc.rust-lang.org/book/testing.html
    }

    /// Test the focus(Some(window)) and the focus(None) functionality
    #[test]
    fn test_focus_and_unfocus_window() {
        // Initialize test with a new window manager
        let mut wm = FullscreenWM::new(SCREEN);
        // Assert the initial focused_window window is None
        assert_eq!(None, wm.get_window_layout().focused_window);

        // Add one window
        wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
        // Assert it is focused after adding
        assert_eq!(Some(1), wm.get_window_layout().focused_window);

        // Unfocus all windows
        wm.focus_window(None).unwrap();
        // Assert the window is unfocused
        assert_eq!(None, wm.get_window_layout().focused_window);


        // Focus window 1 again
        wm.focus_window(Some(1)).unwrap();
        // Assert window 1 is correctly focused
        assert_eq!(Some(1), wm.get_window_layout().focused_window);

    }

    /// Test cycle_focus in a window manager with no windows and with one window
    #[test]
    fn test_cycle_focus_none_and_one_window() {
        // Initialize test with a new window manager
        let mut wm = FullscreenWM::new(SCREEN);
        // Assert the initial focused_window window is None
        assert_eq!(None, wm.get_window_layout().focused_window);

        // Cycle does nothing when there are no windows
        wm.cycle_focus(PrevOrNext::Next);
        assert_eq!(None, wm.get_window_layout().focused_window);
        wm.cycle_focus(PrevOrNext::Prev);
        assert_eq!(None, wm.get_window_layout().focused_window);

        // Add one window
        wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
        // Assert it is focused after adding
        assert_eq!(Some(1), wm.get_window_layout().focused_window);

        // When there is only one window, focus it if it no window is focused, otherswise do nothing
        wm.cycle_focus(PrevOrNext::Next);
        assert_eq!(Some(1), wm.get_window_layout().focused_window);
        wm.cycle_focus(PrevOrNext::Prev);
        assert_eq!(Some(1), wm.get_window_layout().focused_window);
        wm.focus_window(None).unwrap();
        wm.cycle_focus(PrevOrNext::Next);
        assert_eq!(Some(1), wm.get_window_layout().focused_window);
        wm.focus_window(None).unwrap();
        wm.cycle_focus(PrevOrNext::Prev);
        assert_eq!(Some(1), wm.get_window_layout().focused_window);
    }

    /// Test cycle_focus in a window manager with multiple windows
    #[test]
    fn test_cycle_focus_multiple_windows() {
        // Initialize test with a new window manager, add 3 windows and
        // assert the last one added is focused
        let mut wm = FullscreenWM::new(SCREEN);
        wm.add_window(WindowWithInfo::new_tiled(1, SOME_GEOM)).unwrap();
        wm.add_window(WindowWithInfo::new_tiled(2, SOME_GEOM)).unwrap();
        wm.add_window(WindowWithInfo::new_tiled(3, SOME_GEOM)).unwrap();
        assert_eq!(Some(3), wm.get_window_layout().focused_window);

        // Cycle back should focus on the previous window
        wm.cycle_focus(PrevOrNext::Prev);
        assert_eq!(Some(2), wm.get_window_layout().focused_window);

        // Going back and forth shouldn't cange the focused window
        wm.cycle_focus(PrevOrNext::Next);
        assert_eq!(Some(3), wm.get_window_layout().focused_window);

        // Cycle forth should focus on the next window
        wm.cycle_focus(PrevOrNext::Next);
        assert_eq!(Some(1), wm.get_window_layout().focused_window);

        // When no window is focused, any window may become focused
        wm.focus_window(None).unwrap();
        wm.cycle_focus(PrevOrNext::Prev);
        assert_eq!(Some(1), wm.get_window_layout().focused_window);

    }
}
