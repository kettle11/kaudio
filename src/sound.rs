pub struct Sound {
    /// Sounds are stored internally as an i16 at 44100 HZ
    pub data: Vec<i16>,
    /// Channels are interleaved if greater than 1
    pub channels: u8,
}

impl Sound {
    pub fn new(data: Vec<i16>, channels: u8) -> Self {
        Self { data, channels }
    }
}
