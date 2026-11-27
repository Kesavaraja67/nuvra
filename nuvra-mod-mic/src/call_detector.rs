// nuvra-mod-mic/src/call_detector.rs
// Meeting and VoIP call activity detection (Section 14.3).
// R-13: No unsafe.
#![forbid(unsafe_code)]

#[derive(Debug, Clone)]
pub struct MeetingAppList {
    pub process_names: Vec<&'static str>,
    pub window_title_keywords: Vec<&'static str>,
}

impl Default for MeetingAppList {
    fn default() -> Self {
        Self {
            process_names: vec![
                "teams.exe",
                "msteams.exe",
                "zoom.exe",
                "webex.exe",
                "ciscowebexstart.exe",
                "discord.exe",
                "slack.exe",
                "skype.exe",
            ],
            window_title_keywords: vec![
                "Google Meet",
                "Meet - ",
                "Zoom Meeting",
                "Microsoft Teams",
                "Webex",
            ],
        }
    }
}

pub struct CallDetector {
    config: MeetingAppList,
}

impl CallDetector {
    pub fn new() -> Self {
        Self {
            config: MeetingAppList::default(),
        }
    }

    /// Check if a process binary name matches a configured meeting/conference app.
    pub fn is_meeting_process(&self, process_name: &str) -> bool {
        let name_lower = process_name.trim().to_lowercase();
        self.config
            .process_names
            .iter()
            .any(|&app| name_lower == app || name_lower.ends_with(app))
    }

    /// Check if a window title matches a web conference session (e.g. Google Meet).
    pub fn is_meeting_window_title(&self, title: &str) -> bool {
        let title_lower = title.trim().to_lowercase();
        self.config
            .window_title_keywords
            .iter()
            .any(|&keyword| title_lower.contains(&keyword.to_lowercase()))
    }
}

impl Default for CallDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_detector_processes() {
        let detector = CallDetector::new();
        assert!(detector.is_meeting_process("Teams.exe"));
        assert!(detector.is_meeting_process("C:\\Program Files\\Zoom\\bin\\zoom.exe"));
        assert!(detector.is_meeting_process("discord.exe"));
        assert!(!detector.is_meeting_process("notepad.exe"));
        assert!(!detector.is_meeting_process("chrome.exe"));
    }

    #[test]
    fn test_call_detector_titles() {
        let detector = CallDetector::new();
        assert!(detector.is_meeting_window_title("Meet - Weekly Standup - Google Chrome"));
        assert!(detector.is_meeting_window_title("Zoom Meeting 40 Minutes"));
        assert!(!detector.is_meeting_window_title("YouTube - Music"));
    }
}
