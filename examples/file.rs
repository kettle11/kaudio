use kaudio::*;

fn main() {
    let sound = load_wav("examples/harp.wav").unwrap();
    let source = FileSource { offset: 0, sound };

    begin_audio_thread(source);
    loop {}
}

struct FileSource {
    offset: usize,
    sound: Sound,
}

impl AudioSource for FileSource {
    fn initialize(&mut self, frame_size: usize) {}
    fn provide_samples(&mut self, samples: &mut [i16]) {
        let channels = 2;
        for i in (0..samples.len()).step_by(channels) {
            for j in 0..2 {
                samples[i + j] = (self.sound.data[self.offset + j] as f32 / i16::MAX as f32) as i16;
            }
            self.offset += channels;

            if self.offset >= self.sound.data.len() {
                self.offset = 0;
            }
        }
    }
}
