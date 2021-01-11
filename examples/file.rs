use kaudio::*;

fn main() {
    // This example assumes a wav file with a single channel.
    let sound = load_wav("examples/bell.wav").unwrap();

    let mut offset = 0;
    begin_audio_thread(move |samples, stream_info| {
        let channels = stream_info.channels() as usize;

        for i in (0..samples.len()).step_by(channels) {
            for j in 0..channels {
                samples[i + j] = sound.data[offset]
            }
            offset += 1;

            if offset >= sound.data.len() {
                offset = 0;
            }
        }
    });
    loop {}
}
