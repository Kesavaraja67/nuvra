// Nuvra — main entry point (v1 Core)
// ADR-0001 (Rust), ADR-0002 (renderer), ADR-0003 (threading/event bus)
// R-13: unsafe is ONLY in nuvra-platform-win; this crate is safe.
#![forbid(unsafe_code)]

use nuvra_core_shared::module::ModuleState;
use nuvra_platform_win::dcomp::CompositionHost;
use nuvra_platform_win::dxgi::create_device_and_swapchain;
use nuvra_platform_win::hotkey::HotKeyRegistration;
use nuvra_platform_win::monitor::{get_primary_monitor_bounds, is_foreground_fullscreen};
use nuvra_platform_win::renderer::{NotchRenderer, PillRenderParams};
use nuvra_platform_win::tray::TrayIcon;
use nuvra_platform_win::window::{NotchWindow, WindowEvent};
use nuvra_render::engine::{NotchEngine, Theme};
use nuvra_render::tokens;
use std::time::Instant;
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, VK_M, VK_N};

fn main() {
    tracing::info!("Nuvra v1 Core starting...");

    // Diagnostic spike mode preserved for headless CI/VM verification (R-09)
    if std::env::var("NUVRA_RUN_SPIKE")
        .map(|v| v == "1")
        .unwrap_or(false)
    {
        if let Err(err) = nuvra_render::spike::run_spike() {
            eprintln!("Verification spike failed: {err}");
            std::process::exit(1);
        }
        println!("Verification spike passed.");
        return;
    }

    let initial_width = tokens::window::EXPANDED_WIDTH as u32;
    let initial_height = tokens::window::EXPANDED_MAX_HEIGHT as u32;

    // 1. Create topmost layered click-through Notch window
    let window = match NotchWindow::new(initial_width, initial_height) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Failed to initialize Notch window: {e}");
            std::process::exit(1);
        }
    };

    // 2. Position window centered horizontally at top of primary monitor
    let monitor = get_primary_monitor_bounds();
    let center_x = monitor.left + (monitor.width() - initial_width as i32) / 2;
    window.set_geometry(center_x, monitor.top, initial_width, initial_height);

    // 3. Initialize D3D11 device and swap chain (falls back to WARP if needed)
    let device_bundle =
        match create_device_and_swapchain(window.hwnd(), initial_width, initial_height) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to initialize D3D11 / DXGI device: {e}");
                std::process::exit(1);
            }
        };

    // 4. Connect swap chain to DirectComposition visual tree
    let dcomp = match CompositionHost::create_for_hwnd(window.hwnd(), &device_bundle.swapchain) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("DirectComposition setup failed: {e}");
            std::process::exit(1);
        }
    };

    // 5. Initialize Direct2D hardware-accelerated renderer (D-STK-03)
    let mut renderer = match NotchRenderer::new(&device_bundle.swapchain) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize NotchRenderer: {e}");
            std::process::exit(1);
        }
    };

    // 6. Register global hotkeys (D-WIN-09)
    let _hk_mute = HotKeyRegistration::register(window.hwnd(), 1, MOD_CONTROL | MOD_ALT, VK_M).ok();
    let _hk_expand =
        HotKeyRegistration::register(window.hwnd(), 2, MOD_CONTROL | MOD_ALT, VK_N).ok();

    // 7. Initialize System Tray Icon
    let _tray = TrayIcon::new(window.hwnd()).ok();

    // 8. Initialize Notch Engine
    let mut engine = NotchEngine::new(Theme::Obsidian);
    let mut last_frame_time = Instant::now();
    let event_rx = window.event_receiver().clone();
    let active_module_state = ModuleState::Idle;

    println!("Nuvra v1 Notch engine active.");
    println!("  -> Docked at top-center of primary monitor.");
    println!("  -> Move cursor to top edge (or press Ctrl+Alt+N) to expand.");
    println!("  -> Press Ctrl+Alt+M to toggle global microphone mute.");
    println!("  -> Right-click Nuvra icon in system tray to quit.");

    // One-shot test exit mode for headless test runs
    let test_run = std::env::var("NUVRA_TEST_ONESHOT")
        .map(|v| v.trim() == "1")
        .unwrap_or(false);

    // 9. Win32 message pump and spring simulation loop
    loop {
        if !window.pump_messages() {
            break;
        }

        // Process window interaction events
        while let Ok(evt) = event_rx.try_recv() {
            match evt {
                WindowEvent::MouseEnter => {
                    engine.set_hovered(true);
                }
                WindowEvent::MouseLeave => {
                    engine.set_hovered(false);
                }
                WindowEvent::Click { .. } => {
                    engine.toggle_expand();
                }
                WindowEvent::HotKey { id } => {
                    if id == 2 {
                        engine.toggle_expand();
                    } else if id == 1 {
                        let _ = nuvra_platform_win::audio::toggle_microphone_mute();
                    }
                }
                WindowEvent::DpiChanged { new_dpi: _ } => {
                    let m = get_primary_monitor_bounds();
                    let x = m.left + (m.width() - initial_width as i32) / 2;
                    window.set_geometry(x, m.top, initial_width, initial_height);
                }
                WindowEvent::DisplayChanged => {
                    let m = get_primary_monitor_bounds();
                    let x = m.left + (m.width() - initial_width as i32) / 2;
                    window.set_geometry(x, m.top, initial_width, initial_height);
                }
            }
        }

        // Fullscreen auto-hide check (D-WIN-07)
        let is_fullscreen = is_foreground_fullscreen();
        if is_fullscreen {
            // Auto-collapse when user is in fullscreen game/video
            engine.set_hovered(false);
        }

        let now = Instant::now();
        let dt = now.duration_since(last_frame_time).as_secs_f32().min(0.05);
        last_frame_time = now;

        let frame = engine.update(&active_module_state, dt);

        // Update interactive hit bounds matching current spring pill width/height
        let hit_x = ((initial_width as f32 - frame.width) / 2.0) as i32;
        window.update_hit_bounds(hit_x, 0, frame.width as u32, frame.height as u32);

        // Render current pill frame with Direct2D and present to DXGI swapchain
        let params = PillRenderParams {
            x: hit_x as f32,
            y: 0.0,
            width: frame.width,
            height: frame.height,
            radius: frame.radius,
            surface_color: frame.colors.surface,
            edge_color: frame.colors.edge,
        };
        let _ = renderer.render_and_present(&device_bundle.swapchain, &params);

        let _ = dcomp.commit();

        if test_run {
            println!(
                "Nuvra oneshot test run complete: frame width={}, height={}",
                frame.width, frame.height
            );
            break;
        }

        // Cap loop rate to display refresh (~120 FPS / 8ms) when animating, sleep when settled
        if frame.is_settled {
            std::thread::sleep(std::time::Duration::from_millis(16));
        } else {
            std::thread::sleep(std::time::Duration::from_millis(4));
        }
    }
}
