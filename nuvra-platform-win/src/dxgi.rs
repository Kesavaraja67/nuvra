// nuvra-platform-win/src/dxgi.rs
// D3D11 device + DXGI composition swap chain for the notch renderer (ADR-0002, D-STK-03).
// Every unsafe block carries a // SAFETY: comment.

use windows::core::Interface;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::{
    Direct3D::D3D_DRIVER_TYPE_HARDWARE,
    Direct3D11::{
        D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_SDK_VERSION,
    },
    Dxgi::{
        Common::{DXGI_ALPHA_MODE_PREMULTIPLIED, DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        IDXGIAdapter, IDXGIDevice, IDXGIFactory2, IDXGISwapChain, IDXGISwapChain1,
        DXGI_SCALING_STRETCH, DXGI_SWAP_CHAIN_DESC1, DXGI_SWAP_EFFECT_FLIP_DISCARD,
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

/// Create a hardware D3D11 device + composition swap chain with premultiplied alpha.
/// Falls back to WARP software rasterizer when hardware is unavailable,
/// so the notch renders on every machine including WARP-only CI/VM (D-PERF-01).
pub fn create_device_and_swapchain(
    _hwnd: HWND,
    width: u32,
    height: u32,
) -> Result<DeviceBundle, DxgiError> {
    let force_warp = std::env::var("NUVRA_FORCE_WARP")
        .map(|v| v == "1")
        .unwrap_or(false);

    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;

    let driver_type = if force_warp {
        windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_WARP
    } else {
        D3D_DRIVER_TYPE_HARDWARE
    };

    // SAFETY: D3D11CreateDevice initializes DirectX device and context.
    let result = unsafe {
        D3D11CreateDevice(
            None,
            driver_type,
            None,
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        )
    };

    if result.is_err() && !force_warp {
        // SAFETY: Fallback to WARP software rasterizer if hardware adapter unavailable.
        unsafe {
            D3D11CreateDevice(
                None,
                windows::Win32::Graphics::Direct3D::D3D_DRIVER_TYPE_WARP,
                None,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                None,
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )
            .map_err(|e| DxgiError::CreateDevice(format!("WARP fallback failed: {e:?}")))?;
        }
        tracing::info!("WARP adapter active");
    } else {
        result.map_err(|e| DxgiError::CreateDevice(format!("Device creation failed: {e:?}")))?;
        if force_warp {
            tracing::info!("WARP adapter active");
        }
    }

    let device = device.ok_or(DxgiError::NullDevice)?;
    let context = context.ok_or(DxgiError::NullContext)?;

    // SAFETY: Query DXGI factory from D3D11 device to create composition swap chain.
    let dxgi_device: IDXGIDevice = device
        .cast()
        .map_err(|e| DxgiError::CreateDevice(format!("Cast to IDXGIDevice failed: {e}")))?;

    let adapter: IDXGIAdapter = unsafe {
        dxgi_device
            .GetAdapter()
            .map_err(|e| DxgiError::CreateDevice(format!("GetAdapter failed: {e}")))?
    };

    let factory: IDXGIFactory2 = unsafe {
        adapter
            .GetParent()
            .map_err(|e| DxgiError::CreateDevice(format!("GetParent IDXGIFactory2 failed: {e}")))?
    };

    let desc = DXGI_SWAP_CHAIN_DESC1 {
        Width: width,
        Height: height,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        Stereo: false.into(),
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
        BufferCount: 2,
        Scaling: DXGI_SCALING_STRETCH,
        SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
        AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
        Flags: 0,
    };

    // SAFETY: CreateSwapChainForComposition connects directly to DirectComposition visual tree with per-pixel alpha transparency.
    let swapchain1: IDXGISwapChain1 = unsafe {
        factory
            .CreateSwapChainForComposition(&device, &desc, None)
            .map_err(|e| {
                DxgiError::CreateDevice(format!("CreateSwapChainForComposition failed: {e:?}"))
            })?
    };

    let swapchain: IDXGISwapChain = swapchain1
        .cast()
        .map_err(|e| DxgiError::CreateDevice(format!("Cast to IDXGISwapChain failed: {e}")))?;

    Ok(DeviceBundle {
        device,
        context,
        swapchain,
    })
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
                write!(f, "D3D11CreateDevice returned null device")
            }
            DxgiError::NullContext => {
                write!(f, "D3D11CreateDevice returned null context")
            }
            DxgiError::NullSwapChain => {
                write!(f, "CreateSwapChainForComposition returned null swap chain")
            }
        }
    }
}

impl std::error::Error for DxgiError {}
