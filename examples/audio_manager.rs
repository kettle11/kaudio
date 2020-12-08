use kaudio::*;

fn main() {
    let sound = SpatialSound::new(load_wav("examples/ambience.wav").unwrap(), 60.);
    let harp = SpatialSound::new(load_wav("examples/music_box.wav").unwrap(), 30.);
    let generator = SpatialSound::new(load_wav("examples/generator.wav").unwrap(), 120.);

    let mut audio_manager = SpatialAudioManager::new();
    let sound_handle = audio_manager.add_sound(sound);
    let harp = audio_manager.add_sound(harp);
    let generator = audio_manager.add_sound(generator);

    // audio_manager.play_sound(sound_handle, 10.0, 20.0);
    // audio_manager.play_loop(harp, 0.2, 0.1);
    audio_manager.play_loop(generator, 10.0, 0.5);

    loop {}
}
