use crate::*;

pub struct AudioManager {
    current_id: usize,
    to_audio_thread: std::sync::mpsc::Sender<Message>,
    free_handles: Vec<SoundHandle>,
}

impl AudioManager {
    pub fn new() -> Self {
        let (send, receive) = std::sync::mpsc::channel();
        let audio_thread = AudioThread {
            sounds: Vec::new(),
            playing_sounds: Vec::new(),
            incoming_messages: receive,
        };
        begin_audio_thread(audio_thread);
        Self {
            current_id: 0,
            to_audio_thread: send,
            free_handles: Vec::new(),
        }
    }

    pub fn add_sound(&mut self, sound: Sound) -> SoundHandle {
        let handle = if let Some(handle) = self.free_handles.pop() {
            self.to_audio_thread
                .send(Message::ReplaceSound(sound, handle))
                .unwrap();
            handle
        } else {
            self.to_audio_thread.send(Message::NewSound(sound)).unwrap();
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

    pub fn play_sound(&mut self, sound_handle: SoundHandle) {
        self.to_audio_thread
            .send(Message::PlaySound(sound_handle))
            .unwrap();
    }
}

enum Message {
    NewSound(Sound),
    ReplaceSound(Sound, SoundHandle),
    PlaySound(SoundHandle),
}

#[derive(Copy, Clone)]
pub struct SoundHandle(usize);

pub struct PlayingSound {
    handle: SoundHandle,
    offset: usize,
    repeat: bool,
}
struct AudioThread {
    sounds: Vec<Sound>,
    playing_sounds: Vec<PlayingSound>,
    incoming_messages: std::sync::mpsc::Receiver<Message>,
}

impl AudioThread {
    fn handle_messages(&mut self) {
        while let Ok(message) = self.incoming_messages.try_recv() {
            match message {
                Message::NewSound(sound) => self.sounds.push(sound),
                Message::ReplaceSound(sound, handle) => self.sounds[handle.0] = sound,
                Message::PlaySound(handle) => self.playing_sounds.push(PlayingSound {
                    handle,
                    offset: 0,
                    repeat: false,
                }),
            }
        }
    }
}

impl AudioSource for AudioThread {
    fn provide_samples(&mut self, samples: &mut [i16]) {
        // Should this be moved to a post update step?
        self.handle_messages();

        // The following looks like a bunch of code but all it is doing is playing sounds,
        // and potentially looping them.
        // Sounds to be removed are flagged as such and removed later.
        // The removal algorithm works by ensuring items to be removed are 
        // always at the end of the array.
        let channels = 2;
        let samples_length = samples.len();

        let playing_sound_count = self.playing_sounds.len();
        let mut to_delete = 0;

        for i in 0..playing_sound_count {
            let playing_sound = &mut self.playing_sounds[i];
            let sound = &self.sounds[playing_sound.handle.0];
            let mut length_total = samples_length;
            let sound_length = sound.data.len();

            let mut will_remove = false;
            // Repeatedly read from sound buffer until output buffer is full
            // or sound is complete.
            while length_total > 0 {
                let read_length = (sound_length - playing_sound.offset).min(length_total);
                for i in (0..read_length).step_by(channels) {
                    for j in 0..2 {
                        samples[i + j] += sound.data[playing_sound.offset + j];
                    }
                    playing_sound.offset += channels;
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

        if to_delete > 0 {
            self.playing_sounds
                .truncate(playing_sound_count - to_delete);
        }
    }
}
