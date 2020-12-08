use crate::*;

/// Spatial sound are always mono
pub struct SpatialSound {
    /// Sounds are stored internally as an i16 at 44100 HZ
    pub data: Vec<f32>,
}

impl SpatialSound {
    pub fn new(sound: Sound, peak_microwatts: f32) -> Self {
        // Decibel to power level expressed in microwatts.
        // Sounds are stored internally as microwatts.
        // Microwatts offer a reasonable level of precision over the range of typical
        // sounds in a human audio soundscape.
        // let power = 0.000001 * f32::powf(10.0, decibels / 10.);

        // Make the data mono
        let mut data: Vec<f32> = sound
            .data
            .chunks_exact(sound.channels as usize)
            .map(|d| d[0] /*d.iter().fold(0.0, |a, b| a + b)*/)
            .collect();

        let max = data.iter().fold(0.0 as f32, |a, b| a.max(b.abs()));

        // Normalize values to max
        for i in data.iter_mut() {
            *i = (*i / max) * peak_microwatts;
        }

        Self { data }
    }
}
pub struct SpatialAudioManager {
    current_id: usize,
    to_audio_thread: rtrb::Producer<Message>,
    free_handles: Vec<SoundHandle>,
}

impl SpatialAudioManager {
    pub fn new() -> Self {
        // 500 messages can be sent at a time.
        let (producer, consumer) = rtrb::RingBuffer::new(500).split();

        // A loud alarm clock measured in microwatts.
        let max_default = 20.;
        let audio_thread = AudioThread {
            sounds: Vec::new(),
            playing_sounds: Vec::new(),
            incoming_messages: consumer,
            frame_buffer: Vec::new(),
            max_value: max_default,
            max_default,
        };
        begin_audio_thread(audio_thread);
        Self {
            current_id: 0,
            to_audio_thread: producer,
            free_handles: Vec::new(),
        }
    }

    pub fn add_sound(&mut self, sound: SpatialSound) -> SoundHandle {
        let handle = if let Some(handle) = self.free_handles.pop() {
            self.to_audio_thread
                .push(Message::ReplaceSound(sound, handle))
                .unwrap();
            handle
        } else {
            self.to_audio_thread.push(Message::NewSound(sound)).unwrap();
            self.current_id += 1;
            SoundHandle(self.current_id - 1)
        };
        handle
    }

    pub fn delete_sound(&mut self, handle: SoundHandle) {
        // This means that next time a sound is added
        // the old sound will be dropped on the audio thread.
        // Instead the audio thread could pass back the sound object
        // over a channel to prevent the potential audio thread block.
        self.free_handles.push(handle);
    }

    pub fn play_sound(&mut self, sound_handle: SoundHandle, distance: f32, radius: f32) {
        self.to_audio_thread
            .push(Message::PlaySound(sound_handle, distance, radius))
            .unwrap();
    }

    pub fn play_loop(&mut self, sound_handle: SoundHandle, distance: f32, radius: f32) {
        self.to_audio_thread
            .push(Message::LoopSound(sound_handle, distance, radius))
            .unwrap();
    }
}

enum Message {
    NewSound(SpatialSound),
    ReplaceSound(SpatialSound, SoundHandle),
    PlaySound(SoundHandle, f32, f32),
    LoopSound(SoundHandle, f32, f32),
}

#[derive(Copy, Clone)]
pub struct SoundHandle(usize);

pub struct PlayingSound {
    handle: SoundHandle,
    offset: usize,
    repeat: bool,
    last_value: f32,
    distance: f32,
    radius: f32,
}
struct AudioThread {
    sounds: Vec<SpatialSound>,
    playing_sounds: Vec<PlayingSound>,
    incoming_messages: rtrb::Consumer<Message>,
    frame_buffer: Vec<f32>,
    max_value: f32,
    max_default: f32,
}

impl AudioThread {
    pub fn play_sound(&mut self, handle: SoundHandle, repeat: bool, distance: f32, radius: f32) {
        self.playing_sounds.push(PlayingSound {
            handle,
            offset: 0,
            repeat,
            last_value: self.sounds[handle.0].data[0],
            distance,
            radius,
        });
    }
}
impl AudioThread {
    fn handle_messages(&mut self) {
        while let Ok(message) = self.incoming_messages.pop() {
            match message {
                Message::NewSound(sound) => self.sounds.push(sound),
                Message::ReplaceSound(sound, handle) => self.sounds[handle.0] = sound,
                Message::PlaySound(handle, distance, radius) => {
                    self.play_sound(handle, false, distance, radius)
                }
                Message::LoopSound(handle, distance, radius) => {
                    self.play_sound(handle, true, distance, radius)
                }
            }
        }
    }
}

impl AudioSource for AudioThread {
    fn initialize(&mut self, frame_size: usize) {
        self.frame_buffer.resize(frame_size, 0.);
    }
    fn provide_samples(&mut self, output_samples: &mut [i16]) {
        // Should this be moved to a post update step?
        self.handle_messages();

        // The following looks like a bunch of code but all it is doing is playing sounds,
        // and potentially looping them.
        // Sounds to be removed are flagged as such and removed later.
        // The removal algorithm works by ensuring items to be removed are
        // always at the end of the array.
        let output_channels = 2;
        let output_samples_length = output_samples.len();
        let playing_sound_count = self.playing_sounds.len();
        let mut to_delete = 0;

        // Zero the output audio
        self.frame_buffer.clear();
        self.frame_buffer.resize(output_samples_length, 0.);

        // Keep track of the loudest sample to normalize everything else around it.
        let mut max_sample: f32 = 0.0;

        for i in 0..playing_sound_count {
            let playing_sound = &mut self.playing_sounds[i];
            let sound = &self.sounds[playing_sound.handle.0];
            let mut length_total = output_samples_length;
            let sound_length = sound.data.len();

            let mut will_remove = false;

            let mut attenuation = playing_sound.radius / playing_sound.distance;
            // Repeatedly read from sound buffer until output buffer is full
            // or sound is complete.

            // Low pass filter experimentation;
            // let mut last_value = [i16_to_f32(sound.data[0]), i16_to_f32(sound.data[1])];
            let a0 = (0.1 / playing_sound.distance).max(0.00001);

            // Things get too quiet due to the low pass filter.
            //  attenuation += ((0.2) / a0).max(0.2);

            while length_total > 0 {
                let read_length = (sound_length - playing_sound.offset).min(length_total);
                for i in (0..read_length).step_by(output_channels) {
                    // The .min(input_channels-1) here means that the last channel
                    // of the sound will be copied to extra output channels.
                    // So a mono-channel will be copied to both output channels.
                    let sample = sound.data[playing_sound.offset] * attenuation;

                    // Low pass filter
                    let sample = a0 * sample + (1.0 - a0) * playing_sound.last_value;
                    playing_sound.last_value = sample;

                    // This could have the weird, but uncommon, effect of two waves cancelling each-other out
                    // but the max value would go very loud and everything would be toned down.
                    let frame_buffer_value = self.frame_buffer[i] + sample;
                    max_sample = max_sample.max(frame_buffer_value.abs());

                    for j in 0..output_channels {
                        self.frame_buffer[i + j] = frame_buffer_value;
                    }
                    playing_sound.offset += 1;
                }

                if playing_sound.offset >= sound_length {
                    playing_sound.offset = 0;

                    // If this sound doesn't loop, break here and note that this should be removed.
                    if !playing_sound.repeat {
                        will_remove = true;
                        break;
                    }
                }

                length_total -= read_length;
            }

            if will_remove {
                to_delete += 1;
            } else {
                self.playing_sounds.swap(i - to_delete, i);
            }
        }

        // If sounds go loud, slowly return the sounds we can hear to normal.
        //
        self.max_value -= 1.;

        // Ensure that sounds are normalized into range.
        // The max represents the loudest sound within a typical environment.
        // Sounds that are quieter will sound like a fraction of that loudest sound.
        // The value chosen here is about a loud alarm clock.
        self.max_value = self.max_value.max(self.max_default).max(max_sample);

        //  println!("MAX SAMPLE: {:?}", max_sample);
        //  println!("CURRENT MAX: {:?}", self.max_value);

        // max_sample = max_sample.max(1.0);

        // Copy the samples to the output and normalize the output at the same time.
        output_samples
            .iter_mut()
            .zip(self.frame_buffer.iter())
            .for_each(|(o, i)| {
                let normalized = *i / self.max_value;
                *o = (normalized * i16::MAX as f32) as i16
            });

        if to_delete > 0 {
            self.playing_sounds
                .truncate(playing_sound_count - to_delete);
        }
    }
}
