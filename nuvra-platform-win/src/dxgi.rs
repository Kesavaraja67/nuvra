// nuvra-platform-win/src/dxgi.rs
// D3D11 device + DXGI swap chain bootstrap for the notch renderer (ADR-0002).
// ONLY file in the project permitted unsafe. Every block carries a SAFETY comment.
//
// Phase 0: Structure and type signatures only. Full implementation in v1.
// UNVERIFIED — requires owner to run on Floor hardware with a D3D11 driver.
// Manual test: docs/owner-test-scripts/phase-0-test-script.md § DXGI

use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::{
    Direct3D::D3D_DRIVER_TYPE_HARDWARE,
    Direct3D11::{
        D3D11CreateDeviceAndSwapChain, ID3D11Device, ID3D11DeviceContext,
        D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_SDK_VERSION,
    },
    Dxgi::{
        Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        IDXGISwapChain, DXGI_SWAP_CHAIN_DESC, DXGI_SWAP_EFFECT_FLIP_DISCARD,
        DXGI_USAGE_RENDER_TARGET_OUTPUT,
    },
};

/// Minimal device bundle passed to the renderer (ADR-0002).
/// Dropped together; ordering ensures swap chain is released before device.
pub struct DeviceBundle {
    pub device: ID3D11Device,
    pub context: ID3D11DeviceContext,
    pub swapchain: IDXGISwapChain,
}

/// Create a hardware D3D11 device + flip-discard swap chain for `hwnd`.
/// Falls back to WARP software rasterizer when hardware is unavailable,
/// so the notch renders on every machine including WARP-only CI/VM (D-PERF-01).
///
/// `width` and `height` are in physical pixels (already DPI-scaled by caller).
pub fn create_device_and_swapchain(
    hwnd: HWND,
    width: u32,
    height: u32,
) -> Result<DeviceBundle, DxgiError> {
    let desc = DXGI_SWAP_CHAIN_DESC {
        BufferDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_MODE_DESC {
            Width: width,
            Height: height,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            ..Default::default()
        },
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        OutputWindow: hwnd,
        Windowed: true.into(),
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        Flags: 0,
    };

    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;
    let mut swapchain: Option<IDXGISwapChain> = None;

    let force_warp = std::env::var("NUVRA_FORCE_WARP")
        .map(|v| v == "1")
        .unwrap_or(false);

    if force_warp {
        try_create_device(
            windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_WARP,
            &desc,
            &mut device,
            &mut context,
            &mut swapchain,
        )
        .map_err(|e| DxgiError::CreateDevice(format!("Forced WARP creation failed: {e:?}")))?;
        tracing::info!("WARP adapter active");
    } else {
        // Try hardware first; fall back to WARP on failure.
        let result = try_create_device(
            D3D_DRIVER_TYPE_HARDWARE,
            &desc,
            &mut device,
            &mut context,
            &mut swapchain,
        );

        if result.is_err() {
            try_create_device(
                windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_WARP,
                &desc,
                &mut device,
                &mut context,
                &mut swapchain,
            )
            .map_err(|e| DxgiError::CreateDevice(format!("WARP fallback failed: {e:?}")))?;
            tracing::info!("WARP adapter active");
        }
    }

    Ok(DeviceBundle {
        device: device.ok_or(DxgiError::NullDevice)?,
        context: context.ok_or(DxgiError::NullContext)?,
        swapchain: swapchain.ok_or(DxgiError::NullSwapChain)?,
    })
}

fn try_create_device(
    driver_type: windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE,
    desc: &DXGI_SWAP_CHAIN_DESC,
    device_out: &mut Option<ID3D11Device>,
    context_out: &mut Option<ID3D11DeviceContext>,
    swapchain_out: &mut Option<IDXGISwapChain>,
) -> windows::core::Result<()> {
    // SAFETY: D3D11CreateDeviceAndSwapChain is a well-specified COM factory.
    // All pointers written to `*_out` are valid COM objects valid for the
    // lifetime of the call site. We immediately wrap them in Option<ComPtr>.
    // The swap chain description is fully initialised above.
    unsafe {
        D3D11CreateDeviceAndSwapChain(
            None,
            driver_type,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(desc),
            Some(swapchain_out as *mut _),
            Some(device_out as *mut _),
            None,
            Some(context_out as *mut _),
        )
    }
}

#[derive(Debug)]
pub enum DxgiError {
    CreateDevice(String),
    NullDevice,
    NullContext,
    NullSwapChain,
}

impl std::fmt::Display for DxgiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DxgiError::CreateDevice(msg) => write!(f, "D3D11 device creation failed: {msg}"),
            DxgiError::NullDevice => {
                write!(f, "D3D11CreateDeviceAndSwapChain returned null device")
            }
            DxgiError::NullContext => {
                write!(f, "D3D11CreateDeviceAndSwapChain returned null context")
            }
            DxgiError::NullSwapChain => {
                write!(f, "D3D11CreateDeviceAndSwapChain returned null swap chain")
            }
        }
    }
}

impl std::error::Error for DxgiError {}
