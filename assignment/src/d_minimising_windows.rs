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
use cplwm_api::types::{Geometry, PrevOrNext, Screen, Window, WindowLayout, WindowWithInfo};
use cplwm_api::wm::{WindowManager, TilingSupport, FloatSupport, MinimiseSupport};

use wm_common::{Manager, LayoutManager, TilingTrait, FloatTrait, FloatAndTileTrait};
use wm_common::error::FloatWMError;
use a_fullscreen_wm::FocusManager;
use b_tiling_wm::VerticalLayout;
use c_floating_windows::FloatOrTileManager;



/// the public type
pub type WMName = MinimiseWM;

/// struct for MinimiseWM = {Focus + TileOrFloat + Minimize}
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct MinimiseWM{
    /// focus manager
    pub focus_manager: FocusManager,
    /// the layout manager
    pub minimise_manager: MinimiseManager<FloatOrTileManager<VerticalLayout>>,
}

impl WindowManager for MinimiseWM {
    type Error = FloatWMError;

    fn new(screen: Screen) -> MinimiseWM {
        MinimiseWM {
            focus_manager: FocusManager::new(),
            minimise_manager: MinimiseManager::new(FloatOrTileManager::new(screen, VerticalLayout{})),
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
                self.minimise_manager.add_window(window_with_info)
            })
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        match self.focus_manager.remove_window(window) {
            Err(error) => Err(error.to_float_error()),
            Ok(_) => self.minimise_manager.remove_window(window)
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        WindowLayout {
            focused_window: self.get_focused_window(),
            windows: self.minimise_manager.get_window_layout(),
        }
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error> {
        match window {
            None => Ok(()),
            Some(w) => self.minimise_manager.maximise_if_minimised(w, &mut self.focus_manager),
        }.and_then(|_| {
            self.focus_manager.focus_window(window)
                .map_err(|error| error.to_float_error())
                .and_then(|_| self.minimise_manager.focus_shifted(window))
        })
    }

    fn cycle_focus(&mut self, dir: PrevOrNext) {
        self.focus_manager.cycle_focus(dir);
        self.minimise_manager.focus_shifted(self.focus_manager.get_focused_window()).is_ok();
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error> {
        self.minimise_manager.get_window_info(window)
    }

    fn get_screen(&self) -> Screen {
        self.minimise_manager.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen) {
        self.minimise_manager.resize_screen(screen);
    }
}

impl TilingSupport for MinimiseWM {
    fn get_master_window(&self) -> Option<Window> {
        self.minimise_manager.get_master_window()
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error>{
        self.minimise_manager.swap_with_master(window, &mut self.focus_manager)
    }

    fn swap_windows(&mut self, dir: PrevOrNext){
        self.minimise_manager.swap_windows(dir, &self.focus_manager)
    }
}

impl FloatSupport for MinimiseWM {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.minimise_manager.get_floating_windows()
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error>{
        self.minimise_manager.toggle_floating(window, &mut self.focus_manager)
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error>{
        self.minimise_manager.set_window_geometry(window, new_geometry)
    }
}

impl MinimiseSupport for MinimiseWM {
    fn get_minimised_windows(&self) -> Vec<Window> {
        self.minimise_manager.get_minimised_windows()
    }

    fn toggle_minimised(&mut self, window: Window) -> Result<(), Self::Error>{
        self.minimise_manager.toggle_minimised(window, &mut self.focus_manager)
    }
}

/// Manager to manage the minimised windows and a LayoutManager
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct MinimiseManager<LM : LayoutManager<Error=FloatWMError> + FloatAndTileTrait> {
    /// The wrapped layout manager
    pub layout_manager: LM,
    /// The helper for the minimised windows
    pub minimise_assistant_manager: MinimiseAssistantManager,
}

impl<LM : LayoutManager<Error=FloatWMError> + FloatAndTileTrait> Manager for MinimiseManager<LM> {
    type Error = FloatWMError;

    fn get_windows(&self) -> Vec<Window> {
        let mut windows = self.layout_manager.get_windows();
        windows.extend(self.minimise_assistant_manager.get_windows());
        windows
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error> {
        self.layout_manager.add_window(window_with_info)
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        self.layout_manager.remove_window(window)
            .or_else(|_| self.minimise_assistant_manager.remove_window(window))
    }
}

impl<LM : LayoutManager<Error=FloatWMError> + FloatAndTileTrait> LayoutManager for MinimiseManager<LM> {
    fn get_window_layout(&self) -> Vec<(Window, Geometry)>{
        self.layout_manager.get_window_layout()
    }

    fn focus_shifted(&mut self, window: Option<Window>) -> Result<(), Self::Error>{
        match window {
            None => Ok(()),
            Some(w) => if self.minimise_assistant_manager.is_managed(w) {
                self.minimise_assistant_manager.get_window_info(w)
                    .and_then(|info| self.minimise_assistant_manager.remove_window(w)
                        .and_then(|_| self.layout_manager.add_window(info)))
                    } else {
                        Ok(())
                    }
        }.and_then(|_| self.layout_manager.focus_shifted(window))
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error>{
        self.layout_manager.get_window_info(window)
            .or_else(|_| self.minimise_assistant_manager.get_window_info(window))
    }

    fn get_screen(&self) -> Screen{
        self.layout_manager.get_screen()
    }

    fn resize_screen(&mut self, screen: Screen){
        self.layout_manager.resize_screen(screen);
    }
}

impl<LM : LayoutManager<Error=FloatWMError> + FloatAndTileTrait> TilingTrait for MinimiseManager<LM> {
    /// get the master
    fn get_master_window(&self) -> Option<Window>{
        self.layout_manager.get_master_window()
    }
    /// swap with the master
    fn swap_with_master(&mut self, window: Window, focus_manager: &mut FocusManager) -> Result<(), Self::Error>{
        self.maximise_if_minimised(window, focus_manager)
            .and_then(|_| {
                self.layout_manager.swap_with_master(window, focus_manager)
            })
    }
    /// swap windows
    fn swap_windows(&mut self, dir: PrevOrNext, focus_manager: &FocusManager){
        self.layout_manager.swap_windows(dir, focus_manager)
    }
}

impl<LM : LayoutManager<Error=FloatWMError> + FloatAndTileTrait> FloatTrait for MinimiseManager<LM> {
        /// change geometry of the floater
        fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error>{
            self.layout_manager.set_window_geometry(window, new_geometry)
        }
}

impl<LM : LayoutManager<Error=FloatWMError> + FloatAndTileTrait> FloatAndTileTrait for MinimiseManager<LM> {
    fn get_floating_windows(&self) -> Vec<Window>{
        self.layout_manager.get_floating_windows()
    }
    /// get all tiled windows
    fn get_tiled_windows(&self) -> Vec<Window>{
        self.layout_manager.get_tiled_windows()
    }
    /// toggle floating on window
    fn toggle_floating(&mut self, window: Window, focus_manager: &mut FocusManager) -> Result<(), Self::Error>{
        self.maximise_if_minimised(window, focus_manager)
            .and_then(|_| self.layout_manager.toggle_floating(window, focus_manager))
    }
}


impl<LM : LayoutManager<Error=FloatWMError> + FloatAndTileTrait> MinimiseManager<LM> {
    fn new(layout_manager: LM) -> MinimiseManager<LM>{
        MinimiseManager {
            layout_manager: layout_manager,
            minimise_assistant_manager: MinimiseAssistantManager::new(),
        }
    }

    fn get_minimised_windows(&self) -> Vec<Window> {
        self.minimise_assistant_manager.get_windows()
    }

    fn maximise_if_minimised(&mut self, window: Window, focus_manager: &mut FocusManager) -> Result<(), FloatWMError> {
        if self.minimise_assistant_manager.is_managed(window) {
            self.toggle_minimised(window, focus_manager)
        } else {
            Ok(())
        }
    }

    fn toggle_minimised(&mut self, window: Window, focus_manager: &mut FocusManager) -> Result<(), FloatWMError>{
        if self.minimise_assistant_manager.is_managed(window) {
            self.minimise_assistant_manager.get_window_info(window).and_then(|info| {
                self.minimise_assistant_manager.remove_window(window)
                    .and_then(|_| self.layout_manager.add_window(info)
                        .and_then(|_| focus_manager.focus_window(Some(window))
                            .map_err(|error| error.to_float_error())))

            })
        } else {
            self.layout_manager.get_window_info(window).and_then(|info| {
                self.layout_manager.remove_window(window)
                    .and_then(|_| self.minimise_assistant_manager.add_window(info)
                        .and_then(|_| {
                            match focus_manager.get_focused_window() {
                                None => Ok(()),
                                Some(w) => if window == w {
                                    focus_manager.focus_window(None)
                                        .map_err(|error| error.to_float_error())
                                } else {
                                    Ok(())
                                }
                            }
                        }))
            })
        }
    }


}




/// Manager to manage the minimised windows
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct MinimiseAssistantManager {
    /// Map to keep the window and it's last info
    pub minis: Vec<WindowWithInfo>,
}

impl Manager for MinimiseAssistantManager {
    type Error = FloatWMError;

    fn get_windows(&self) -> Vec<Window> {
        self.minis.iter().map(|w| w.window).collect()
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), FloatWMError> {
        if self.is_managed(window_with_info.window) {
            Err(FloatWMError::AlReadyManagedWindow(window_with_info.window))
        } else {
            self.minis.push(window_with_info);
            Ok(())
        }
    }

    fn remove_window(&mut self, window: Window) -> Result<(), FloatWMError> {
        self.minis.iter().position(|w| w.window == window)
            .ok_or(FloatWMError::UnknownWindow(window))
            .and_then(|index| {
                self.minis.remove(index);
                Ok(())
            })
    }
}

impl MinimiseAssistantManager {
    /// create empty MinimiseAssistantManager
    pub fn new() -> MinimiseAssistantManager {
        MinimiseAssistantManager{
            minis: Vec::new(),
        }
    }

    /// get specific window_info
    pub fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, FloatWMError> {
        self.minis.iter().position(|w| w.window == window)
            .ok_or(FloatWMError::UnknownWindow(window))
            .and_then(|index| {
                self.minis.get(index)
                    .map(|w| *w)
                    .ok_or(FloatWMError::UnknownWindow(window))
            })
    }

}


#[cfg(test)]
mod tests {
    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;
    use wm_common::tests::float_support;
    use wm_common::tests::float_and_tile_support;
    use wm_common::tests::minimise_support;
    use super::MinimiseWM;
    use b_tiling_wm::VerticalLayout;

    #[test]
    fn test_empty_tiling_wm(){
        window_manager::test_empty_wm::<MinimiseWM>();
    }

    #[test]
    fn test_adding_and_removing_some_windows(){
        window_manager::test_adding_and_removing_windows::<MinimiseWM>();
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        window_manager::test_focus_and_unfocus_window::<MinimiseWM>();
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        window_manager::test_cycle_focus_none_and_one_window::<MinimiseWM>();
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        window_manager::test_cycle_focus_multiple_windows::<MinimiseWM>();
    }

    #[test]
    fn test_get_window_info(){
        window_manager::test_get_window_info::<MinimiseWM>();
    }

    #[test]
    fn test_resize_screen(){
        window_manager::test_resize_screen::<MinimiseWM>();
    }

    #[test]
    fn test_get_master_window(){
        tiling_support::test_master_tile::<MinimiseWM>();
    }

    #[test]
    fn test_swap_with_master_window(){
        tiling_support::test_swap_with_master::<MinimiseWM>();
    }


    #[test]
    fn test_swap_windows(){
        tiling_support::test_swap_windows::<MinimiseWM, VerticalLayout>(VerticalLayout{});
    }

    #[test]
    fn test_tiling_layout(){
        tiling_support::test_get_window_info::<MinimiseWM, VerticalLayout>(VerticalLayout{});
    }

    #[test]
    fn test_get_floating_windows(){
        float_support::test_get_floating_windows::<MinimiseWM>();
    }

    #[test]
    fn test_toggle_floating(){
        float_support::test_toggle_floating::<MinimiseWM>();
    }

    #[test]
    fn test_set_window_geometry(){
        float_support::test_set_window_geometry::<MinimiseWM>();
    }

    #[test]
    fn test_window_layout_order(){
        float_support::test_window_layout_order::<MinimiseWM>();
    }

    #[test]
    fn test_focus_floating_window_order(){
        float_support::test_focus_floating_window_order::<MinimiseWM>();
    }

    #[test]
    fn test_swapping_master_with_floating_window_no_tiles(){
        float_and_tile_support::test_swapping_master_with_floating_window_no_tiles::<MinimiseWM>();
    }

    #[test]
    fn test_swapping_master_with_floating_window(){
        float_and_tile_support::test_swapping_master_with_floating_window::<MinimiseWM>();
    }

    #[test]
    fn test_swap_windows_on_floating(){
        float_and_tile_support::test_swap_windows_on_floating::<MinimiseWM>();
    }

    #[test]
    fn test_swap_windows_with_float_focused(){
        float_and_tile_support::test_swap_windows_with_float_focused::<MinimiseWM>();
    }

    #[test]
    fn test_toggle_floating_focus(){
        float_and_tile_support::test_toggle_floating_focus::<MinimiseWM>();
    }

    #[test]
    fn test_minimise() {
        minimise_support::test_minimise::<MinimiseWM>();
    }

    #[test]
    fn test_minimise_state_after_focus() {
        minimise_support::test_minimise_state_after_focus::<MinimiseWM>();
    }

    #[test]
    fn test_minimise_of_floating_window() {
        minimise_support::test_minimise_of_floating_window::<MinimiseWM>();
    }

    #[test]
    fn test_minimise_of_tiled_window() {
        minimise_support::test_minimise_of_tiled_window::<MinimiseWM>();
    }

    #[test]
    fn test_minimise_order() {
        minimise_support::test_minimise_order::<MinimiseWM>();
    }

    #[test]
    fn test_minimise_state_after_cycle_focus() {
        minimise_support::test_minimise_state_after_cycle_focus::<MinimiseWM>();
    }


}
