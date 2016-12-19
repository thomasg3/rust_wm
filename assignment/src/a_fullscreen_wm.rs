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
use wm_common::Manager;
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
    /// The FocusManager which manages the current focus and keeps al the windows
    pub focus_manager: FocusManager,
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
            focus_manager: FocusManager::new(),
            screen: screen,
            window_to_info: BTreeMap::new(),
        }
    }

    /// The `windows` field contains all the windows we manage.
    fn get_windows(&self) -> Vec<Window> {
        self.focus_manager.get_windows()
    }

    /// Returns the currently focused window
    fn get_focused_window(&self) -> Option<Window> {
        self.focus_manager.get_focused_window()
    }

    /// Add a new window. Focus on the window and push the previous window at the back of the Deque,
    /// it there is one. Add the given WindowWithInfo to the window_to_info BTreeMap for future use.
    /// Returns an AlReadyManagedWindow error when the given window is already managed by this
    /// window manager.
    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        self.focus_manager.add_window(window_with_info).and_then(|_| {
            self.window_to_info.insert(window_with_info.window, window_with_info);
            Ok(())
        })
    }

    /// Remove a window. If the window is focused, simple focus the previous window. Otherwise
    /// / remove the window from the Deque. Remove any additional information of the window in the
    /// BTreeMap. Returns an UnknowWindow error when the given window is not managed by this
    /// window manager
    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        self.focus_manager.remove_window(window).and_then(|_| {
            self.window_to_info.remove(&window);
            Ok(())
        })
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
        match self.get_focused_window() {
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
        self.focus_manager.focus_window(window)
    }

    /// When cycling to Next, the window at the front of the deque is focused and the currently
    /// focused window is put at the back of the deque.
    /// When cycling to Prev, the window at the back of the deque is focused and the currently
    /// focused window is put at the front of the deque
    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focus_manager.cycle_focus(dir)
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

/// A manager who is solely occupied with managing which window is focused
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct FocusManager {
    /// A vector deque of windows, the first one is the next one to be focused, the last one is
    /// the previous one to be focused.
    pub windows: VecDeque<Window>,
    /// Currently focused window.
    pub focused_window: Option<Window>,
}

impl Manager for FocusManager {
    type Error = StandardError;

    fn get_windows(&self) -> Vec<Window> {
        let mut windows: Vec<Window> = self.windows.iter().map(|w| *w).collect();
        match self.focused_window {
            Some(w) => windows.push(w),
            None => {}
        }
        return windows;
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), StandardError> {
        if !self.is_managed(window_with_info.window) {
            match self.focused_window {
                Some(w) => self.windows.push_back(w),
                None => {}
            }
            self.focused_window = Some(window_with_info.window);
            Ok(())
        } else {
            Err(StandardError::AlReadyManagedWindow(window_with_info.window))
        }
    }

    fn remove_window(&mut self, window: Window) -> Result<(), StandardError> {
        match self.focused_window {
            Some(w) => {
                if w == window {
                    self.focused_window = self.windows.pop_back();
                    return Ok(());
                }
            }
            None => {}
        }
        match self.windows.iter().position(|w| *w == window) {
            None => Err(StandardError::UnknownWindow(window)),
            Some(i) => {
                self.windows.remove(i);
                Ok(())
            }
        }
    }
}

impl FocusManager {
    /// A new, empty FocusManager
    pub fn new() -> FocusManager {
        FocusManager {
            windows: VecDeque::new(),
            focused_window: None,
        }
    }

    /// The currently focused window
    pub fn get_focused_window(&self) -> Option<Window> {
        self.focused_window
    }



    /// focus anohter window
    pub fn focus_window(&mut self, window: Option<Window>) -> Result<(), StandardError> {
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

    /// cycle focus
    pub fn cycle_focus(&mut self, dir: PrevOrNext) {
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
}


#[cfg(test)]
mod tests {
    use wm_common::tests::window_manager;
    use super::FullscreenWM;

    #[test]
    fn test_empty_tiling_wm(){
        window_manager::test_empty_wm::<FullscreenWM>();
    }

    #[test]
    fn test_adding_and_removing_some_windows(){
        window_manager::test_adding_and_removing_windows::<FullscreenWM>();
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        window_manager::test_focus_and_unfocus_window::<FullscreenWM>();
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        window_manager::test_cycle_focus_none_and_one_window::<FullscreenWM>();
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        window_manager::test_cycle_focus_multiple_windows::<FullscreenWM>();
    }

    #[test]
    fn test_get_window_info(){
        window_manager::test_get_window_info::<FullscreenWM>();
    }

    #[test]
    fn test_resize_screen(){
        window_manager::test_resize_screen::<FullscreenWM>();
    }
}
