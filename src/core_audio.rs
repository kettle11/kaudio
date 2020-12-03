use std::ffi::c_void;
#[link(name = "AudioToolbox", kind = "framework")]
extern "C" {}

pub type FourCharCode = u32;
pub type OSType = FourCharCode;

pub const kAudioUnitType_Output: u32 = 1635086197;
pub const kAudioUnitSubType_HALOutput: u32 = 1634230636;
pub const kAudioUnitManufacturer_Apple: u32 = 1634758764;
pub const kAudioUnitScope_Input: u32 = 1;
pub const kAudioUnitScope_Output: u32 = 2;
pub const kAudioUnitProperty_StreamFormat: u32 = 8;
pub const kAudioFormatLinearPCM: u32 = 1819304813;
pub const kAudioFormatFlagIsSignedInteger: u32 = 4;
pub const kAudioFormatFlagIsPacked: u32 = 8;
pub const kAudioDevicePropertyBufferFrameSize: u32 = 1718839674;
pub const kAudioUnitScope_Global: u32 = 0;
pub const kAudioOutputUnitProperty_SetInputCallback: u32 = 2005;
pub const kAudioUnitProperty_SetRenderCallback: u32 = 23;

pub const kOutputBus: u32 = 0;
pub const kInputBus: u32 = 1;

pub type OSStatus = i32;
#[repr(C)]
pub struct OpaqueAudioComponent {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct ComponentInstanceRecord {
    _unused: [u8; 0],
}

pub type AudioComponent = *mut OpaqueAudioComponent;
pub type AudioComponentInstance = *mut ComponentInstanceRecord;
pub type AudioUnit = AudioComponentInstance;
pub type AudioUnitPropertyID = u32;
pub type AudioUnitScope = u32;
pub type AudioUnitElement = u32;

#[repr(C)]
pub struct AudioComponentDescription {
    pub componentType: OSType,
    pub componentSubType: OSType,
    pub componentManufacturer: OSType,
    pub componentFlags: u32,
    pub componentFlagsMask: u32,
}

extern "C" {
    pub fn AudioComponentFindNext(
        inComponent: AudioComponent,
        inDesc: *const AudioComponentDescription,
    ) -> AudioComponent;
    pub fn AudioComponentInstanceNew(
        inComponent: AudioComponent,
        outInstance: *mut AudioComponentInstance,
    ) -> OSStatus;
    pub fn AudioUnitInitialize(inUnit: AudioUnit) -> OSStatus;
    pub fn AudioUnitGetProperty(
        inUnit: AudioUnit,
        inID: AudioUnitPropertyID,
        inScope: AudioUnitScope,
        inElement: AudioUnitElement,
        outData: *mut ::std::os::raw::c_void,
        ioDataSize: *mut u32,
    ) -> OSStatus;

    pub fn AudioUnitSetProperty(
        inUnit: AudioUnit,
        inID: AudioUnitPropertyID,
        inScope: AudioUnitScope,
        inElement: AudioUnitElement,
        inData: *const ::std::os::raw::c_void,
        inDataSize: u32,
    ) -> OSStatus;
    pub fn AudioOutputUnitStart(ci: AudioUnit) -> OSStatus;
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AURenderCallbackStruct {
    pub inputProc: AURenderCallback,
    pub inputProcRefCon: *mut ::std::os::raw::c_void,
}

pub type AURenderCallback = unsafe extern "C" fn(
    inRefCon: *mut ::std::os::raw::c_void,
    ioActionFlags: *mut AudioUnitRenderActionFlags,
    inTimeStamp: *const AudioTimeStamp,
    inBusNumber: u32,
    inNumberFrames: u32,
    ioData: *mut AudioBufferList,
) -> OSStatus;

#[repr(C)]
pub struct AudioBufferList {
    pub mNumberBuffers: u32,
    pub mBuffers: [AudioBuffer; 1usize],
}

#[repr(C)]
pub struct AudioBuffer {
    pub mNumberChannels: u32,
    pub mDataByteSize: u32,
    pub mData: *mut ::std::os::raw::c_void,
}

pub struct AudioTimeStamp {
    pub mSampleTime: f64,
    pub mHostTime: u64,
    pub mRateScalar: f64,
    pub mWordClockTime: u64,
    pub mSMPTETime: u32,
    pub mFlags: u32,
    pub mReserved: u32,
}

pub type AudioUnitRenderActionFlags = u32;
pub type AudioSampleType = f32;
pub type AudioUnitSampleType = f32;
pub type AudioFormatID = u32;
pub type AudioFormatFlags = u32;

/// Official documentation:
/// https://developer.apple.com/documentation/coreaudiotypes/audiostreambasicdescription
#[repr(C)]
#[derive(Default, Debug)]
pub struct AudioStreamBasicDescription {
    pub mSampleRate: f64,
    pub mFormatID: AudioFormatID,
    pub mFormatFlags: AudioFormatFlags,
    pub mBytesPerPacket: u32,
    pub mFramesPerPacket: u32,
    pub mBytesPerFrame: u32,
    pub mChannelsPerFrame: u32,
    pub mBitsPerChannel: u32,
    pub mReserved: u32,
}

use crate::AudioSource;

pub fn begin_audio_thread(mut audio_source: impl AudioSource + 'static) {
    // Relevant Apple example documentation here:
    // https://developer.apple.com/library/archive/technotes/tn2091/_index.html

    let audio_component_description = AudioComponentDescription {
        componentType: kAudioUnitType_Output,
        componentSubType: kAudioUnitSubType_HALOutput,
        componentManufacturer: kAudioUnitManufacturer_Apple,
        componentFlags: 0,
        componentFlagsMask: 0,
    };

    unsafe {
        // Can this be used to enumerate available audio devices? Or is that a separate thing?
        let component = AudioComponentFindNext(std::ptr::null_mut(), &audio_component_description);
        if component == std::ptr::null_mut() {
            panic!("Could not find audio device");
        }

        let mut audio_unit: AudioComponentInstance = std::ptr::null_mut();
        let result =
            AudioComponentInstanceNew(component, &mut audio_unit as *mut AudioComponentInstance);

        if result != 0 {
            panic!("ERROR CREATING AUDIO COMPONENT");
        }

        let mut stream_description = AudioStreamBasicDescription {
            ..Default::default()
        };
        let mut size: u32 = std::mem::size_of::<AudioStreamBasicDescription>() as u32;
        let result = AudioUnitGetProperty(
            audio_unit,
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Output,
            kOutputBus,
            &mut stream_description as *mut AudioStreamBasicDescription as *mut c_void,
            &mut size,
        );

        if result != 0 {
            panic!("ERROR CREATING AUDIO COMPONENT");
        }

        // println!("Stream Description: {:?}", stream_description);

        // Now initialize the stream with the formats we want.

        let mut stream_description = AudioStreamBasicDescription {
            mFormatID: kAudioFormatLinearPCM,
            mFormatFlags: kAudioFormatFlagIsSignedInteger | kAudioFormatFlagIsPacked,
            mChannelsPerFrame: 2, // This should be adjustable later
            mSampleRate: 44100.0,
            mFramesPerPacket: 1,
            mBitsPerChannel: 16, // It is fine to use 16 bits for the final output
            mBytesPerFrame: 2 * 2, // 2 channels of 2 bytes each.
            mBytesPerPacket: 2 * 2, // Same as bytes per frame
            mReserved: 0,
        };

        let result = AudioUnitSetProperty(
            audio_unit,
            kAudioUnitProperty_StreamFormat,
            kAudioUnitScope_Input,
            kOutputBus,
            &mut stream_description as *mut AudioStreamBasicDescription as *mut c_void,
            size,
        );

        if result != 0 {
            panic!("ERROR CREATING AUDIO COMPONENT");
        }

        let frame_size = 1024; // This should be adjustable
        let result = AudioUnitSetProperty(
            audio_unit,
            kAudioDevicePropertyBufferFrameSize,
            kAudioUnitScope_Global, // Global means that this setting applies to the entire audio unit.
            kOutputBus,
            &frame_size as *const i32 as *const c_void,
            std::mem::size_of::<u32>() as u32,
        );

        if result != 0 {
            panic!("ERROR CREATING AUDIO COMPONENT");
        }

        audio_source.initialize(frame_size as usize);

        let callback = AURenderCallbackStruct {
            inputProc: callback,
            inputProcRefCon: Box::into_raw(Box::new(CallbackWrapper {
                audio_source: Box::new(audio_source),
            })) as *mut c_void,
        };

        let result = AudioUnitSetProperty(
            audio_unit,
            kAudioUnitProperty_SetRenderCallback,
            kAudioUnitScope_Input,
            kOutputBus,
            &callback as *const AURenderCallbackStruct as *const c_void,
            std::mem::size_of::<AURenderCallbackStruct>() as u32,
        );

        if result != 0 {
            panic!("ERROR CREATING AUDIO COMPONENT");
        }

        let result = AudioUnitInitialize(audio_unit);

        if result != 0 {
            panic!("ERROR CREATING AUDIO COMPONENT");
        }

        let result = AudioOutputUnitStart(audio_unit);

        if result != 0 {
            panic!("ERROR CREATING AUDIO COMPONENT");
        }
    }
}

/* #[repr(C)]
pub struct AudioBufferList {
    pub mNumberBuffers: u32,
    pub mBuffers: [AudioBuffer; 1usize],
}

#[repr(C)]
pub struct AudioBuffer {
    pub mNumberChannels: u32,
    pub mDataByteSize: u32,
    pub mData: *mut ::std::os::raw::c_void,
}
*/
unsafe extern "C" fn callback(
    in_ref_con: *mut ::std::os::raw::c_void,
    _io_action_flags: *mut AudioUnitRenderActionFlags,
    _in_time_stamp: *const AudioTimeStamp,
    _in_bus_number: u32,
    _in_number_frames: u32,
    io_data: *mut AudioBufferList,
) -> i32 {
    let callback_wrapper: *mut CallbackWrapper = in_ref_con as *mut CallbackWrapper;
    let data: &mut [i16] = std::slice::from_raw_parts_mut(
        (*io_data).mBuffers[0].mData as *mut i16,
        ((*io_data).mBuffers[0].mDataByteSize / 2) as usize,
    );

    // The data does not necessarily have to be zeroed if the user callback will zero it,
    // but uninitialized memory can be quite painful on the ears if allowed to slip through.
    for b in data.iter_mut() {
        *b = 0;
    }

    // Call user callback.
    (*callback_wrapper).audio_source.provide_samples(data);

    return 0;
}

struct CallbackWrapper {
    audio_source: Box<dyn AudioSource>,
}
