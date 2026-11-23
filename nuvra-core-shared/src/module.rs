// nuvra-core-shared/src/module.rs
// Module trait — D-ARCH-02, ADR-0005.
// R-13: No unsafe.
#![forbid(unsafe_code)]

use crate::event::OsEvent;
use thiserror::Error;

/// Capabilities a module may declare. Used for permission display and runtime checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    ClipboardRead,
    MicrophoneAccess,
    NotificationAccess,
    FileSystemRead,
    AudioLoopback,
    NetworkAccess,
}

/// Plain-data snapshot produced by a module for the renderer.
/// Must be cheaply cloneable — called from the render path.
/// D-ARCH-02: Modules never draw; the renderer draws ModuleState.
#[derive(Debug, Clone, Default)]
pub enum ModuleState {
    #[default]
    Idle,
    Media(crate::event::MediaSnapshot),
    Mic(crate::event::MicSnapshot),
    Notification(crate::event::NotificationSnapshot),
    Clipboard(crate::event::ClipboardSnapshot),
    Shelf(crate::event::ShelfSnapshot),
}

/// Event emitted by a module back to the host event bus.
#[derive(Debug, Clone)]
pub struct ModuleEvent {
    pub module_id: &'static str,
    pub state: ModuleState,
}

#[derive(Debug, Error)]
pub enum ModuleError {
    #[error("module init failed: {0}")]
    Init(String),
    #[error("module event handler failed: {0}")]
    Event(String),
}

/// Core module trait — D-ARCH-02, ADR-0005.
/// Every module in `nuvra-mod-*` implements this trait.
pub trait Module: Send + 'static {
    /// Stable identifier, used for routing events and logging.
    fn id(&self) -> &'static str;

    /// Capabilities this module requires (shown to the user before first use).
    fn capabilities(&self) -> &[Capability];

    /// Initialize the module. Called once after spawn, before any events.
    /// `tx`: sender to push `ModuleEvent` back to the host event bus.
    fn init(&mut self, tx: flume::Sender<ModuleEvent>) -> Result<(), ModuleError>;

    /// Handle an OS event. Must not block the caller thread for I/O.
    fn on_event(&mut self, event: OsEvent) -> Result<(), ModuleError>;

    /// Return the current state snapshot. Called from the render path via
    /// arc-swap — must never block, allocate, or acquire locks.
    fn state(&self) -> ModuleState;

    /// Clean shutdown. Called once before the thread exits.
    fn shutdown(&mut self);
}
