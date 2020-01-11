mod windows;

struct SineSource {
    offset: f32,
}

impl windows::AudioSource for SineSource {
    fn provide_samples(&mut self, samples: &mut [i16]) {
        let channels = 2;
        for i in (0..samples.len()).step_by(channels) {
            let v = f32::sin(self.offset);

            for j in 0..channels {
                samples[i + j] = (std::i16::MAX as f32 * v * 0.3) as i16;
            }

            self.offset += 0.03;
        }
    }
}

fn main() {
    let sine_source = SineSource { offset: 0.0 };
    windows::init_backend(sine_source);
    loop {}
}
