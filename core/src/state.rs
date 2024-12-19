pub struct UranusState {
    pub left_db: f32,
    pub right_db: f32,
    pub wifi_on: bool,
    pub cell_on: bool,
    pub is_streaming: bool,
    pub is_recording: bool,
    pub is_audio_present: bool,
    pub audio_last_seen: u64,
}

impl UranusState {
    pub fn new() -> UranusState {
        UranusState {
            left_db: -60.0,
            right_db: -60.0,
            wifi_on: true,
            cell_on: false,
            is_streaming: false,
            is_recording: false,
            is_audio_present: false,
            audio_last_seen: 0,
        }
    }
}
