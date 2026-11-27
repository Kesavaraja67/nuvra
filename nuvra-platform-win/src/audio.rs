// nuvra-platform-win/src/audio.rs
// Windows CoreAudio endpoint volume and microphone capture session control (Section 14.3).
// Every unsafe block carries a // SAFETY: comment.

use thiserror::Error;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCapture, eConsole, eRender, IMMDevice, IMMDeviceEnumerator, MMDeviceEnumerator,
};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("Failed to create MMDeviceEnumerator: {0}")]
    CreateEnumerator(windows::core::Error),
    #[error("GetDefaultAudioEndpoint failed: {0}")]
    GetDefaultEndpoint(windows::core::Error),
    #[error("Activate IAudioEndpointVolume failed: {0}")]
    ActivateEndpoint(windows::core::Error),
    #[error("GetMute / SetMute failed: {0}")]
    MuteOperation(windows::core::Error),
    #[error("GetVolume / SetVolume failed: {0}")]
    VolumeOperation(windows::core::Error),
}

/// Helper to get the default audio endpoint volume interface.
fn get_endpoint_volume(is_capture: bool) -> Result<IAudioEndpointVolume, AudioError> {
    // SAFETY: MMDeviceEnumerator instantiation and Activate call.
    unsafe {
        let data_flow = if is_capture { eCapture } else { eRender };
        let enumerator: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)
                .map_err(AudioError::CreateEnumerator)?;

        let device: IMMDevice = enumerator
            .GetDefaultAudioEndpoint(data_flow, eConsole)
            .map_err(AudioError::GetDefaultEndpoint)?;

        let endpoint_vol: IAudioEndpointVolume = device
            .Activate(CLSCTX_ALL, None)
            .map_err(AudioError::ActivateEndpoint)?;

        Ok(endpoint_vol)
    }
}

/// Query the default capture device (microphone) mute state.
pub fn get_microphone_mute() -> Result<bool, AudioError> {
    let endpoint = get_endpoint_volume(true)?;
    // SAFETY: IAudioEndpointVolume::GetMute returns Result<BOOL>.
    unsafe {
        let muted = endpoint.GetMute().map_err(AudioError::MuteOperation)?;
        Ok(muted.as_bool())
    }
}

/// Set the default capture device (microphone) mute state globally across all apps.
pub fn set_microphone_mute(muted: bool) -> Result<(), AudioError> {
    let endpoint = get_endpoint_volume(true)?;
    // SAFETY: IAudioEndpointVolume::SetMute with boolean flag.
    unsafe {
        endpoint
            .SetMute(muted, std::ptr::null())
            .map_err(AudioError::MuteOperation)
    }
}

/// Toggle microphone mute state. Returns the new mute state.
pub fn toggle_microphone_mute() -> Result<bool, AudioError> {
    let current = get_microphone_mute().unwrap_or(false);
    let new_state = !current;
    set_microphone_mute(new_state)?;
    Ok(new_state)
}

/// Query the default render device (speakers/headphones) master volume [0.0, 1.0].
pub fn get_master_volume() -> Result<f32, AudioError> {
    let endpoint = get_endpoint_volume(false)?;
    // SAFETY: IAudioEndpointVolume::GetMasterVolumeLevelScalar query.
    unsafe {
        endpoint
            .GetMasterVolumeLevelScalar()
            .map_err(AudioError::VolumeOperation)
    }
}

/// Set the default render device (speakers/headphones) master volume [0.0, 1.0].
pub fn set_master_volume(level: f32) -> Result<(), AudioError> {
    let endpoint = get_endpoint_volume(false)?;
    // SAFETY: IAudioEndpointVolume::SetMasterVolumeLevelScalar with clamped level.
    unsafe {
        endpoint
            .SetMasterVolumeLevelScalar(level.clamp(0.0, 1.0), std::ptr::null())
            .map_err(AudioError::VolumeOperation)
    }
}
