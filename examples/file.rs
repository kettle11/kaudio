use kaudio::*;

fn main() {
    let mut reader = hound::WavReader::open("examples/harp.wav").unwrap();

    let samples = reader.samples::<i16>().map(|x| x.unwrap()).collect();

    let spec = reader.spec();
    println!("SPEC: {:?}", spec);

    let sound = Sound {
        data: samples,
        channels: 2,
    };

    let source = FileSource { offset: 0, sound };

    begin_audio_thread(source);
    loop {}
}

struct FileSource {
    offset: usize,
    sound: Sound,
}

impl AudioSource for FileSource {
    fn provide_samples(&mut self, samples: &mut [i16]) {
        let channels = 2;
        for i in (0..samples.len()).step_by(channels) {
            for j in 0..2 {
                samples[i + j] = self.sound.data[self.offset + j];
            }
            self.offset += channels;

            if self.offset >= self.sound.data.len() {
                self.offset = 0;
            }
        }
    }
}
