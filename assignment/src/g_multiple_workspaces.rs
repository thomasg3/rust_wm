//! Optional: Multiple Workspaces
//!
//! Extend your window manager with support for multiple workspaces. The
//! concept of workspaces is described in the first section of the assignment.
//! See the documentation of the [`MultiWorkspaceSupport`] trait for the precise
//! requirements.
//!
//! *Unlike* the previous assignments, you are not allowed to make a copy of
//! your previous window manager. You *have* to define a wrapper implementing
//! the [`MultiWorkspaceSupport`] trait. This wrapper can take any existing
//! window manager and uses it to create the different workspaces. This
//! wrapper must also implement all the traits you have implemented in the
//! other assignments, you can forward them to the window manager of the
//! current workspace.
//!
//! [`MultiWorkspaceSupport`]: ../../cplwm_api/wm/trait.MultiWorkspaceSupport.html
//!
//! # Status
//!
//! COMPLETED: PARTIAL
//!
//! COMMENTS:
//! This manager only delegates everything on the the currently selected Wordspace. So certain
//! invariants do not hold at the moment.
//!

// Add imports here
use cplwm_api::types::*;
use cplwm_api::wm::*;
use wm_common::error::MultiWorkspaceError;
use d_minimising_windows::MinimiseWM;

/// public type
pub type WMName = MultiWorkspaces<MinimiseWM>;

/// MultiWorkspaces
#[derive(RustcDecodable, RustcEncodable, Debug, Clone)]
pub struct MultiWorkspaces<WM: WindowManager> {
    /// all the workspaces
    pub workspaces: Vec<WM>,
    /// index of the current workspace in the workspaces vector
    pub current_workspace: WorkspaceIndex,
    /// the current screen size
    pub screen: Screen,
}

impl<WM: WindowManager> MultiWorkspaces<WM> {
    fn get_current_workspace(&self) -> Result<&WM, MultiWorkspaceError>{
        self.get_workspace(self.get_current_workspace_index())
    }

    fn get_current_workspace_mut(&mut self) -> Result<&mut WM, MultiWorkspaceError> {
        let index = self.get_current_workspace_index();
        self.get_workspace_mut(index)
    }
}

impl<WM: WindowManager> WindowManager for MultiWorkspaces<WM> {
    type Error = MultiWorkspaceError;

    fn new(screen: Screen) -> Self{
        MultiWorkspaces {
            workspaces: vec![WM::new(screen)],
            current_workspace: 0,
            screen: screen,
        }
    }

    fn get_window_layout(&self) -> WindowLayout {
        self.get_current_workspace()
            .and_then(|wm| Ok(wm.get_window_layout()))
            .unwrap_or(WindowLayout::new())
    }

    fn get_windows(&self) -> Vec<Window> {
        self.get_current_workspace()
            .and_then(|wm| Ok(wm.get_windows()))
            .unwrap_or(Vec::new())
    }

    fn add_window(&mut self, window_with_info: WindowWithInfo) -> Result<(), Self::Error>{
        self.get_current_workspace_mut()
            .and_then(|wm| wm.add_window(window_with_info)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }

    fn remove_window(&mut self, window: Window) -> Result<(), Self::Error> {
        self.get_current_workspace_mut()
            .and_then(|wm| wm.remove_window(window)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }

    fn focus_window(&mut self, window: Option<Window>) -> Result<(), Self::Error>{
        self.get_current_workspace_mut()
            .and_then(|wm| wm.focus_window(window)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }

    fn cycle_focus(&mut self, dir: PrevOrNext){
        match self.get_current_workspace_mut() {
            Err(_) => {},
            Ok(wm) => wm.cycle_focus(dir),
        }
    }

    fn get_window_info(&self, window: Window) -> Result<WindowWithInfo, Self::Error>{
        self.get_current_workspace()
            .and_then(|wm| wm.get_window_info(window)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }

    fn get_screen(&self) -> Screen{
        self.screen
    }

    fn resize_screen(&mut self, screen: Screen){
        self.screen = screen;
        for index in 0..self.workspaces.len() {
            self.workspaces.get_mut(index)
                .and_then(|w| {
                    w.resize_screen(screen);
                    Some(())
                });
        }
    }
}

impl<WM: WindowManager> MultiWorkspaceSupport<WM> for MultiWorkspaces<WM> {
    fn get_current_workspace_index(&self) -> WorkspaceIndex {
        self.current_workspace
    }

    fn get_workspace(&self, index: WorkspaceIndex) -> Result<&WM, Self::Error>{
        self.workspaces.get(index).ok_or(MultiWorkspaceError::WorkspaceIndexOutOfBound(index))
    }

    fn get_workspace_mut(&mut self, index: WorkspaceIndex) -> Result<&mut WM, Self::Error>{
        self.workspaces.get_mut(index).ok_or(MultiWorkspaceError::WorkspaceIndexOutOfBound(index))
    }

    fn switch_workspace(&mut self, index: WorkspaceIndex) -> Result<(), Self::Error>{
        if index < self.workspaces.len(){
            self.current_workspace = index;
            Ok(())
        } else if index == self.workspaces.len(){
            self.workspaces.push(WM::new(self.screen));
            self.current_workspace = index;
            Ok(())
        } else {
            Err(MultiWorkspaceError::WorkspaceIndexOutOfBound(index))
        }
    }
}

impl<WM: FloatSupport> FloatSupport for MultiWorkspaces<WM> {
    fn get_floating_windows(&self) -> Vec<Window> {
        self.get_current_workspace()
            .and_then(|wm| Ok(wm.get_floating_windows()))
            .unwrap_or(Vec::new())
    }

    fn toggle_floating(&mut self, window: Window) -> Result<(), Self::Error> {
        self.get_current_workspace_mut()
            .and_then(|wm| wm.toggle_floating(window)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }

    fn set_window_geometry(&mut self, window: Window, new_geometry: Geometry) -> Result<(), Self::Error>{
        self.get_current_workspace_mut()
            .and_then(|wm| wm.set_window_geometry(window, new_geometry)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }
}

impl<WM: MinimiseSupport> MinimiseSupport for MultiWorkspaces<WM> {
    fn get_minimised_windows(&self) -> Vec<Window> {
        self.get_current_workspace()
            .and_then(|wm| Ok(wm.get_minimised_windows()))
            .unwrap_or(Vec::new())
    }

    fn toggle_minimised(&mut self, window: Window) -> Result<(), Self::Error>{
        self.get_current_workspace_mut()
            .and_then(|wm| wm.toggle_minimised(window)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }
}

impl<WM: TilingSupport> TilingSupport for MultiWorkspaces<WM> {
    fn get_master_window(&self) -> Option<Window>{
        match self.get_current_workspace() {
            Err(_) => None,
            Ok(wm) => wm.get_master_window(),
        }
    }

    fn swap_with_master(&mut self, window: Window) -> Result<(), Self::Error>{
        self.get_current_workspace_mut()
            .and_then(|wm| wm.swap_with_master(window)
                .map_err(|_| MultiWorkspaceError::WrappedError))
    }

    fn swap_windows(&mut self, dir: PrevOrNext){
        match self.get_current_workspace_mut() {
            Err(_) => {},
            Ok(wm) => wm.swap_windows(dir),
        }
    }
}


#[cfg(test)]
mod tests {
    use wm_common::tests::window_manager;
    use wm_common::tests::tiling_support;
    use wm_common::tests::float_support;
    use wm_common::tests::float_and_tile_support;
    use wm_common::tests::minimise_support;
    use super::MultiWorkspaces;
    use d_minimising_windows::MinimiseWM;
    use b_tiling_wm::VerticalLayout;

    #[test]
    fn test_empty_tiling_wm(){
        window_manager::test_empty_wm::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_adding_and_removing_some_windows(){
        window_manager::test_adding_and_removing_windows::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_focus_and_unfocus_window() {
        window_manager::test_focus_and_unfocus_window::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_cycle_focus_none_and_one_window() {
        window_manager::test_cycle_focus_none_and_one_window::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_cycle_focus_multiple_windows() {
        window_manager::test_cycle_focus_multiple_windows::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_get_window_info(){
        window_manager::test_get_window_info::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_resize_screen(){
        window_manager::test_resize_screen::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_get_master_window(){
        tiling_support::test_master_tile::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_swap_with_master_window(){
        tiling_support::test_swap_with_master::<MultiWorkspaces<MinimiseWM>>();
    }


    #[test]
    fn test_swap_windows(){
        tiling_support::test_swap_windows::<MultiWorkspaces<MinimiseWM>, VerticalLayout>(VerticalLayout{});
    }

    #[test]
    fn test_tiling_layout(){
        tiling_support::test_get_window_info::<MultiWorkspaces<MinimiseWM>, VerticalLayout>(VerticalLayout{});
    }

    #[test]
    fn test_get_floating_windows(){
        float_support::test_get_floating_windows::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_toggle_floating(){
        float_support::test_toggle_floating::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_set_window_geometry(){
        float_support::test_set_window_geometry::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_window_layout_order(){
        float_support::test_window_layout_order::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_focus_floating_window_order(){
        float_support::test_focus_floating_window_order::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_swapping_master_with_floating_window_no_tiles(){
        float_and_tile_support::test_swapping_master_with_floating_window_no_tiles::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_swapping_master_with_floating_window(){
        float_and_tile_support::test_swapping_master_with_floating_window::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_swap_windows_on_floating(){
        float_and_tile_support::test_swap_windows_on_floating::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_swap_windows_with_float_focused(){
        float_and_tile_support::test_swap_windows_with_float_focused::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_toggle_floating_focus(){
        float_and_tile_support::test_toggle_floating_focus::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_minimise() {
        minimise_support::test_minimise::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_minimise_state_after_focus() {
        minimise_support::test_minimise_state_after_focus::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_minimise_of_floating_window() {
        minimise_support::test_minimise_of_floating_window::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_minimise_of_tiled_window() {
        minimise_support::test_minimise_of_tiled_window::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_minimise_order() {
        minimise_support::test_minimise_order::<MultiWorkspaces<MinimiseWM>>();
    }

    #[test]
    fn test_minimise_state_after_cycle_focus() {
        minimise_support::test_minimise_state_after_cycle_focus::<MultiWorkspaces<MinimiseWM>>();
    }


}
