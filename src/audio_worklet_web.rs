pub use crate::*;
use std::cell::RefCell;

type AudioOutputFormat = f32;

struct ThreadLocalData {
    callback: Box<dyn FnMut(&mut [AudioOutputFormat], StreamInfo) + Send + 'static>,
    // JavaScript will read directly from these.
    audio_scratch_buffers: Vec<Vec<f32>>,
    interleaved_buffer: Vec<f32>,
}

impl ThreadLocalData {
    /// Reserve enough space for an audio frame.
    pub fn resize(&mut self, channels: u32, frame_size: u32) {
        self.audio_scratch_buffers
            .resize_with(channels as usize, || {
                Vec::with_capacity(frame_size as usize)
            });
        for v in self.audio_scratch_buffers.iter_mut() {
            v.resize(frame_size as usize, 0.);
        }

        self.interleaved_buffer
            .resize((frame_size * channels) as usize, 0.);
    }
}

thread_local! {
    static THREAD_AUDIO_CALLBACK: RefCell<Option<ThreadLocalData>> = RefCell::new(None);
}

pub fn begin_audio_thread(
    audio_callback: impl FnMut(&mut [AudioOutputFormat], StreamInfo) + Send + 'static,
) {
    // For now we just set a thread local that the host will later use.
    THREAD_AUDIO_CALLBACK.with(|f| {
        *f.borrow_mut() = Some(ThreadLocalData {
            callback: Box::new(audio_callback),
            audio_scratch_buffers: Vec::new(),
            interleaved_buffer: Vec::new(),
        });
    });
}

// Returns a pointer to the memory location.
#[no_mangle]
pub extern "C" fn kaudio_audio_buffer_location(channel: u32) -> u32 {
    THREAD_AUDIO_CALLBACK.with(|f| {
        let thread_local_data = f.borrow();
        let thread_local_data = thread_local_data.as_ref().unwrap();
        thread_local_data.audio_scratch_buffers[channel as usize].as_ptr() as u32
    })
}

#[no_mangle]
pub extern "C" fn kaudio_run_callback(channels: u32, frame_size: u32, sample_rate: u32) {
    THREAD_AUDIO_CALLBACK.with(|f| {
        let mut thread_local_data = f.borrow_mut();
        let thread_local_data = thread_local_data.as_mut().unwrap();

        thread_local_data.resize(channels, frame_size);

        let stream_info = StreamInfo {
            channels,
            sample_rate,
        };

        (thread_local_data.callback)(&mut thread_local_data.interleaved_buffer, stream_info);

        // There has got to be better code (using iterators or something) for deinterleaving an audio buffer.
        let mut index_in_frame = 0;
        let mut index_in_interleaved_buffer = 0;
        let steps = thread_local_data.interleaved_buffer.len() / (channels * frame_size) as usize;
        let channels = channels as usize;
        for _ in 0..frame_size {
            for i in 0..channels {
                thread_local_data.audio_scratch_buffers[i][index_in_frame] =
                    thread_local_data.interleaved_buffer[index_in_interleaved_buffer];
                index_in_interleaved_buffer += 1;
            }
            index_in_frame += 1;
        }
    });
}
