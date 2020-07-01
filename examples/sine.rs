use kaudio::*;

fn main() {
    let sine_source = SineSource { time: 0.0 };
    begin_audio_thread(sine_source);
    loop {}
}

struct SineSource {
    time: f64,
}

impl AudioSource for SineSource {
    fn provide_samples(&mut self, samples: &mut [i16]) {
        // Play a middle C
        let step_size = (std::f64::consts::PI * 2.0 * 261.63) / 44100.0;

        let channels = 2;
        for i in (0..samples.len()).step_by(channels) {
            let v = f64::sin(self.time * 1.0);

            for j in 0..2 {
                samples[i + j] = (std::i16::MAX as f64 * v * 0.7) as i16;
            }

            self.time += step_size;

            if self.time > std::f64::consts::PI * 2.0 {
                self.time -= std::f64::consts::PI * 2.0
            }
        }
    }
}
