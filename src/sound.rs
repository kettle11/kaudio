pub struct Sound {
    /// Sounds are stored internally as an i16 at 44100 HZ
    pub data: Vec<f32>,
    /// Channels are interleaved if greater than 1
    pub channels: u8,
}

impl Sound {
    pub fn new(data: Vec<f32>, channels: u8) -> Self {
        Self { data, channels }
    }
}

/// Resample interleaved audio.
pub fn resample(data: &Vec<f32>, channels: usize, old_rate: f32, new_rate: f32) -> Vec<f32> {
    let step = new_rate / old_rate;

    let mut samples = Vec::with_capacity(new_rate as usize * channels);
    for i in 0..data.len() / channels {
        let position = i as f32 * channels as f32 * step;

        for j in 0..channels {
            let p = position as usize + j;
            let start = data[p];
            let value = (data[p + channels] - start) * position.fract() + start;
            samples.push(value);
        }
    }
    samples
}
