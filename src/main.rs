mod windows;

struct SineSource {
    time: f32,
}

impl windows::AudioSource for SineSource {
    fn provide_samples(&mut self, samples: &mut [i16]) {
        let channels = 2;
        for i in (0..samples.len()).step_by(channels) {
            let v = f32::sin(self.time * 1.0);

            for j in 0..channels {
                samples[i + j] = (std::i16::MAX as f32 * v * 0.2) as i16;
            }

            self.time += 0.02;

            // println!("v: {:?}", v);
        }
        if self.time > std::f32::consts::PI * 2.0 {
            self.time -= std::f32::consts::PI * 2.0
        }
    }
}

fn main() {
    let sine_source = SineSource { time: 0.0 };
    windows::init_backend(sine_source);
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
