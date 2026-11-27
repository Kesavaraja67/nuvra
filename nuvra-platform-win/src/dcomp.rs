// nuvra-platform-win/src/dcomp.rs
// DirectComposition target connection to Win32 HWND (ADR-0002, D-STK-03).
// Every unsafe block carries a // SAFETY: comment.

use thiserror::Error;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::DirectComposition::{
    DCompositionCreateDevice, IDCompositionDevice, IDCompositionTarget, IDCompositionVisual,
};
use windows::Win32::Graphics::Dxgi::IDXGISwapChain;

#[derive(Debug, Error)]
pub enum DCompError {
    #[error("DirectComposition device creation failed: {0}")]
    CreateDevice(windows::core::Error),
    #[error("CreateTargetForHwnd failed: {0}")]
    CreateTarget(windows::core::Error),
    #[error("CreateVisual failed: {0}")]
    CreateVisual(windows::core::Error),
    #[error("SetContent failed: {0}")]
    SetContent(windows::core::Error),
    #[error("SetRoot failed: {0}")]
    SetRoot(windows::core::Error),
    #[error("Commit failed: {0}")]
    Commit(windows::core::Error),
}

/// DirectComposition composition host binding a DXGI swapchain to an HWND.
pub struct CompositionHost {
    device: IDCompositionDevice,
    _target: IDCompositionTarget,
    _visual: IDCompositionVisual,
}

impl CompositionHost {
    /// Create a DirectComposition target for `hwnd` and connect `swapchain` as the root visual content.
    pub fn create_for_hwnd(hwnd: HWND, swapchain: &IDXGISwapChain) -> Result<Self, DCompError> {
        // SAFETY: DirectComposition API calls creating device, target, visual, and committing graph.
        unsafe {
            let device: IDCompositionDevice =
                DCompositionCreateDevice(None).map_err(DCompError::CreateDevice)?;

            let target = device
                .CreateTargetForHwnd(hwnd, true)
                .map_err(DCompError::CreateTarget)?;

            let visual = device.CreateVisual().map_err(DCompError::CreateVisual)?;

            visual
                .SetContent(swapchain)
                .map_err(DCompError::SetContent)?;

            target.SetRoot(&visual).map_err(DCompError::SetRoot)?;

            device.Commit().map_err(DCompError::Commit)?;

            Ok(Self {
                device,
                _target: target,
                _visual: visual,
            })
        }
    }

    /// Commit visual updates to the DirectComposition compositor.
    pub fn commit(&self) -> Result<(), DCompError> {
        // SAFETY: DComposition Commit call to push visual tree updates.
        unsafe { self.device.Commit().map_err(DCompError::Commit) }
    }
}
