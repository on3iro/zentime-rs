//! Sound playback related functions
use log::{error, info};
use rodio::decoder::DecoderError;
use rodio::StreamError;
use std::io::Cursor;
use std::thread;
use thiserror::Error;

// Code copied from: https://github.com/yuizho/pomors/blob/master/src/sound.rs

/// Error type that describes error that could happen before/during audio playback
#[derive(Debug, Error)]
pub enum AudioPlaybackError {
    /// Denotes that the given [SoundFile] could not be decoded
    #[error("Could not decode audio data")]
    DecodeError(#[from] DecoderError),

    /// Denotes that no output device could be found
    #[error("Failed to find output device")]
    DeviceNotFound(#[from] StreamError),

    /// The sink on the device to playback the sound could not be created
    #[error("Could not play back sound file because sink could not be created")]
    SinkNotCreated,
}

/// Play the sound file from sound_file path or the default sound file
pub fn play(sound_file: Option<String>, volume: f32) -> Result<(), AudioPlaybackError> {
    let custom_sound = match sound_file {
        Some(path) => match std::fs::read(path) {
            Ok(bytes) => Some(SoundFile::Custom(bytes)),
            Err(error) => {
                error!("Could not read custom sound file: {}", error);
                None
            }
        },
        None => None,
    };

    let sound_file = custom_sound.unwrap_or_else(|| {
        info!("No custom sound file provided, falling back to default sound");
        SoundFile::Default
    });

    let audio = rodio::Decoder::new(Cursor::new(sound_file.get_bytes()))?;

    thread::spawn(move || -> Result<(), AudioPlaybackError> {
        let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
        let sink =
            rodio::Sink::try_new(&stream_handle).map_err(|_| AudioPlaybackError::SinkNotCreated)?;
        sink.append(audio);
        sink.set_volume(volume);
        sink.sleep_until_end();
        Ok(())
    })
    .join()
    .unwrap()?;

    Ok(())
}

trait FileData {
    fn get_bytes(&self) -> Vec<u8>;
}

enum SoundFile {
    Default,
    Custom(Vec<u8>),
}

impl FileData for SoundFile {
    fn get_bytes(&self) -> Vec<u8> {
        match self {
            SoundFile::Default => include_bytes!("bell.wav").to_vec(),
            SoundFile::Custom(bytes) => bytes.to_owned(),
        }
    }
}
