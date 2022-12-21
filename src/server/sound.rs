use log::{error, info};
use std::io::Cursor;
use std::thread;

// Code copied from: https://github.com/yuizho/pomors/blob/master/src/sound.rs

/// Play the sound file
pub fn play(sound_file: Option<String>, volume: f32) {
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

    let audio = rodio::Decoder::new(Cursor::new(sound_file.get_bytes()))
        .expect("failed to load audio data");

    thread::spawn(move || {
        let (_stream, stream_handle) =
            rodio::OutputStream::try_default().expect("failed to find output device");
        let sink = rodio::Sink::try_new(&stream_handle).expect("failed to create sink");
        sink.append(audio);
        sink.set_volume(volume);
        sink.sleep_until_end();
    });
}

pub trait FileData {
    fn get_bytes(&self) -> Vec<u8>;
}

pub enum SoundFile {
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
