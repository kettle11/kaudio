use kaudio::*;

fn main() {
    let mut reader = hound::WavReader::open("examples/harp.wav").unwrap();

    let samples = reader.samples::<i16>().map(|x| x.unwrap()).collect();

    let spec = reader.spec();
    println!("SPEC: {:?}", spec);

    let mut audio_manager = AudioManager::new();
    let harp_handle = audio_manager.add_sound(Sound {
        data: samples,
        channels: 2,
    });

    audio_manager.play_sound(harp_handle);
    loop {}
}
