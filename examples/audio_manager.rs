use kaudio::*;

fn main() {
    let mut reader = hound::WavReader::open("examples/artillery.wav").unwrap();

    //  let mut samples = reader.samples::<i32>().map(|x| x.unwrap()).collect();

    let spec = reader.spec();
    let mut samples = match spec.sample_format {
        hound::SampleFormat::Float => reader.samples::<f32>().map(|x| x.unwrap()).collect(),
        hound::SampleFormat::Int => match spec.bits_per_sample {
            8 => reader
                .samples::<i8>()
                .map(|x| (x.unwrap() as f32 / i8::MAX as f32))
                .collect(),
            16 => reader
                .samples::<i16>()
                .map(|x| (x.unwrap() as f32 / i16::MAX as f32))
                .collect(),
            24 => reader
                .samples::<i32>()
                .map(|x| (x.unwrap() as f32 / 8388607.))
                .collect(),
            32 => reader
                .samples::<i32>()
                .map(|x| x.unwrap() as f32 / i32::MAX as f32)
                .collect(),
            _ => unimplemented!(),
        },
    };
    println!("SPEC: {:?}", spec);

    // Resample audio if it doesn't match our desired sample rate.
    if spec.sample_rate != 44100 {
        println!("RESAMPLING");

        samples = resample(
            &samples,
            spec.channels as usize,
            spec.sample_rate as f32,
            44100.,
        );
    }

    let mut audio_manager = AudioManager::new();
    let harp_handle = audio_manager.add_sound(Sound {
        data: samples,
        channels: spec.channels as u8,
    });

    audio_manager.play_loop(harp_handle);
    loop {}
}
