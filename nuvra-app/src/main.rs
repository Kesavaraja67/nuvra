// Nuvra — main entry point (v1 Core UI-1 Glassmorphic Host)
// R-13: unsafe is ONLY in nuvra-platform-win; this crate is safe.
#![forbid(unsafe_code)]

mod app_state;

use app_state::{build_live_layout, make_vm, snap_strings, AppData};
use nuvra_core_shared::module::ModuleState;
use nuvra_platform_win::clipboard::{init_com_sta, start_file_drag_drop};
use nuvra_platform_win::dcomp::CompositionHost;
use nuvra_platform_win::draw::drawers::hit_test_shelf_drag;
use nuvra_platform_win::draw::{ModularRenderer, RendererAction};
use nuvra_platform_win::dxgi::create_device_and_swapchain;
use nuvra_platform_win::hotkey::HotKeyRegistration;
use nuvra_platform_win::monitor::{get_primary_monitor_bounds, is_foreground_fullscreen};
use nuvra_platform_win::power::BatteryInfo;
use nuvra_platform_win::tray::{TrayIcon, TRAY_CMD_QUIT, TRAY_CMD_SETTINGS, TRAY_CMD_TOGGLE_NOTCH};
use nuvra_platform_win::window::{NotchWindow, WindowEvent};
use nuvra_render::engine::{NotchEngine, Theme};
use nuvra_render::tokens;
use std::time::Instant;
use windows::Win32::UI::Input::KeyboardAndMouse::{MOD_ALT, MOD_CONTROL, VK_M, VK_N};

fn main() {
    init_com_sta();
    if std::env::var("NUVRA_RUN_SPIKE").map(|v| v.trim() == "1").unwrap_or(false) {
        if let Err(err) = nuvra_render::spike::run_spike() {
            eprintln!("Verification spike failed: {err}");
            std::process::exit(1);
        }
        return;
    }

    let initial_w = tokens::window::FIXED_WINDOW_WIDTH as u32;
    let initial_h = tokens::window::FIXED_WINDOW_HEIGHT as u32;
    let window = NotchWindow::new(initial_w, initial_h).unwrap_or_else(|e| {
        eprintln!("Failed to initialize Notch window: {e}");
        std::process::exit(1);
    });
    let monitor = get_primary_monitor_bounds();
    let init_x = monitor.left + (monitor.width() / 2) - (initial_w as i32) / 2;
    window.set_geometry(init_x, monitor.top, initial_w, initial_h);

    let device_bundle = create_device_and_swapchain(window.hwnd(), initial_w, initial_h).unwrap_or_else(|e| {
        eprintln!("D3D11/DXGI init failed: {e}");
        std::process::exit(1);
    });
    let dcomp = CompositionHost::create_for_hwnd(window.hwnd(), &device_bundle.swapchain).unwrap_or_else(|e| {
        eprintln!("DComp init failed: {e}");
        std::process::exit(1);
    });
    let mut renderer = ModularRenderer::new(&device_bundle.swapchain).unwrap_or_else(|e| {
        eprintln!("ModularRenderer init failed: {e}");
        std::process::exit(1);
    });

    let _hk_mute = HotKeyRegistration::register(window.hwnd(), 1, MOD_CONTROL | MOD_ALT, VK_M).ok();
    let _hk_expand = HotKeyRegistration::register(window.hwnd(), 2, MOD_CONTROL | MOD_ALT, VK_N).ok();
    let tray = TrayIcon::new(window.hwnd()).ok();

    let mut engine = NotchEngine::new(Theme::Obsidian);
    let mut data = AppData::new();
    let mut last_frame = Instant::now();
    let mut hover_enter_time: Option<Instant> = None;
    let mut hover_leave_time: Option<Instant> = None;
    let mut was_cursor_inside = false;
    let mut was_fullscreen = false;
    let mut last_nw: i32 = tokens::window::COLLAPSED_WIDTH as i32;
    let mut last_nh: i32 = tokens::window::COLLAPSED_HEIGHT as i32;
    let mut last_wx: i32 = monitor.left + (monitor.width() / 2) - last_nw / 2;
    let mut last_wy: i32 = monitor.top;
    let mut eq_phase: f32 = 0.0;
    let event_rx = window.event_receiver().clone();
    let module_state = ModuleState::Idle;
    let test_run = std::env::var("NUVRA_TEST_ONESHOT").map(|v| v.trim() == "1").unwrap_or(false);

    loop {
        if !window.pump_messages() {
            break;
        }
        renderer.is_move_mode = window.is_dragging();
        data.poll_os_state();

        while let Ok(evt) = event_rx.try_recv() {
            match evt {
                WindowEvent::MouseEnter | WindowEvent::MouseLeave => {
                    // Handled deterministically below via live GetCursorPos bounds check
                }
                WindowEvent::Click { x, y } => {
                    let snaps = snap_strings(&data);
                    let (_, layout) = build_live_layout(&data, &mut engine, &module_state, 0.0);
                    let vm = make_vm(&data, &snaps, BatteryInfo::query(), window.hover_pos(), eq_phase);

                    if engine.is_expanded() || engine.is_banner() {
                        if let Some(action) = renderer.handle_click(&layout, x as f32, y as f32, &vm) {
                            data.apply_action(action, &mut engine);
                        }
                    } else {
                        engine.set_expanded(true);
                        data.is_pinned = true;
                    }
                }
                WindowEvent::DragStart { x, y } => {
                    if engine.is_expanded() {
                        let (_, layout) = build_live_layout(&data, &mut engine, &module_state, 0.0);
                        if let Some(idx) = hit_test_shelf_drag(&layout, engine.active_drawer(), x as f32, y as f32, data.shelf_store.len()) {
                            if let Some(item) = data.shelf_store.items().get(idx) {
                                let _ = start_file_drag_drop(window.hwnd(), &item.path);
                            }
                        }
                    }
                }
                WindowEvent::DoubleClick { .. } => {
                    if engine.is_vertical_expanded() {
                        engine.set_vertical_expanded(false);
                        engine.set_expanded(false);
                        data.is_pinned = false;
                    } else {
                        engine.set_expanded(true);
                        engine.set_vertical_expanded(true);
                        data.is_pinned = true;
                    }
                }
                WindowEvent::RightClick { .. } => {
                    if engine.is_vertical_expanded() {
                        engine.set_vertical_expanded(false);
                        data.is_pinned = false;
                    } else {
                        engine.set_expanded(true);
                        engine.set_vertical_expanded(true);
                        data.is_pinned = true;
                    }
                }
                WindowEvent::HotKey { id } => {
                    if id == 2 {
                        if engine.is_vertical_expanded() {
                            data.apply_action(RendererAction::CollapseNotch, &mut engine);
                        } else {
                            engine.set_expanded(true);
                            engine.set_vertical_expanded(true);
                            data.is_pinned = true;
                        }
                    } else if id == 1 {
                        data.apply_action(RendererAction::ToggleMute, &mut engine);
                    }
                }
                WindowEvent::FileDropped(path) => {
                    let _ = data.shelf_store.add_path(path);
                    engine.set_expanded(true);
                    engine.set_vertical_expanded(true);
                    engine.set_active_drawer(Some(2));
                    data.is_pinned = true;
                }
                WindowEvent::TrayClick => {
                    if let Some(t) = &tray {
                        match t.show_context_menu() {
                            TRAY_CMD_TOGGLE_NOTCH => {
                                if engine.is_vertical_expanded() {
                                    data.apply_action(RendererAction::CollapseNotch, &mut engine);
                                } else {
                                    engine.set_expanded(true);
                                    engine.set_vertical_expanded(true);
                                    data.is_pinned = true;
                                }
                            }
                            TRAY_CMD_SETTINGS => {
                                engine.set_expanded(true);
                                engine.set_vertical_expanded(true);
                                engine.set_active_drawer(Some(5));
                                data.is_pinned = true;
                            }
                            TRAY_CMD_QUIT => return,
                            _ => {}
                        }
                    }
                }
                WindowEvent::Moved { .. } => {}
                WindowEvent::DpiChanged { .. } | WindowEvent::DisplayChanged => {
                    let m = get_primary_monitor_bounds();
                    window.set_user_center_x(m.left + m.width() / 2);
                    window.set_user_offset_y(0);
                }
            }
        }

        let is_fs = is_foreground_fullscreen();
        if is_fs && !was_fullscreen && !data.is_pinned && !engine.is_vertical_expanded() && engine.active_drawer().is_none() {
            hover_enter_time = None;
            hover_leave_time = None;
            data.apply_action(RendererAction::CollapseNotch, &mut engine);
        }
        was_fullscreen = is_fs;

        // Deterministic Win32 cursor tracking against live animated notch rect (fixes missed TrackMouseEvent)
        let live_hover_pos = window.cursor_screen_pos().and_then(|(cx, cy)| {
            if cx >= last_wx && cx < last_wx + last_nw && cy >= last_wy && cy < last_wy + last_nh {
                Some(((cx - last_wx) as f32, (cy - last_wy) as f32))
            } else {
                None
            }
        });
        let cursor_inside = live_hover_pos.is_some();
        if cursor_inside && !was_cursor_inside {
            hover_leave_time = None;
            if !engine.is_expanded() {
                hover_enter_time = Some(Instant::now());
            }
        } else if !cursor_inside && was_cursor_inside {
            hover_enter_time = None;
            if !data.is_pinned && !engine.is_vertical_expanded() && engine.active_drawer().is_none() {
                hover_leave_time = Some(Instant::now());
            }
        }
        was_cursor_inside = cursor_inside;

        let now_tick = Instant::now();
        if let Some(t) = hover_enter_time {
            if now_tick.duration_since(t).as_millis() as u64 >= tokens::timing::HOVER_INTENT_DEFAULT_MS {
                hover_enter_time = None;
                if cursor_inside && !engine.is_expanded() {
                    engine.set_hovered(true);
                    engine.set_expanded(true);
                }
            }
        }
        if let Some(t) = hover_leave_time {
            if now_tick.duration_since(t).as_millis() as u64 >= tokens::timing::COLLAPSE_DELAY_MS {
                hover_leave_time = None;
                if !cursor_inside && !data.is_pinned && !engine.is_vertical_expanded() && engine.active_drawer().is_none() {
                    engine.set_hovered(false);
                    engine.set_expanded(false);
                }
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last_frame).as_secs_f32().min(0.05);
        last_frame = now;
        if data.media_playing {
            eq_phase += dt;
        }

        let banner_active = data.banner_until.map(|t| Instant::now() < t).unwrap_or(false) && !data.dnd_active;
        engine.set_banner(banner_active);
        data.sync_system_theme(&mut engine);

        let (frame, layout) = build_live_layout(&data, &mut engine, &module_state, dt);
        renderer.set_active_drawer(engine.active_drawer());

        let nw = (frame.width.round() as u32).max(1);
        let nh = (frame.height.round() as u32).max(1);
        let wx = window.user_center_x() - (nw as i32) / 2;
        let wy = monitor.top + window.user_offset_y();
        last_wx = wx;
        last_wy = wy;
        last_nw = nw as i32;
        last_nh = nh as i32;
        window.set_pill_bounds(wx, wy, nw, nh, tokens::radius::BOTTOM);

        let battery = BatteryInfo::query();
        let snaps = snap_strings(&data);
        let vm = make_vm(&data, &snaps, battery, live_hover_pos, eq_phase);
        let _ = renderer.render_frame(&device_bundle.swapchain, &frame.colors, &layout, &vm);
        let _ = dcomp.commit();

        if test_run {
            println!("Nuvra oneshot test run complete: frame width={}, height={}", frame.width, frame.height);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(if frame.is_settled && !data.media_playing { 16 } else { 4 }));
    }
}
