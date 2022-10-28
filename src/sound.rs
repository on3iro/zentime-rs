use std::io::Cursor;
use std::thread;

// Code copied from: https://github.com/yuizho/pomors/blob/master/src/sound.rs

pub fn play(sound_file: impl FileData) {
    let audio = rodio::Decoder::new(Cursor::new(sound_file.get_bytes()))
        .expect("failed to load audio data");

    thread::spawn(move || {
        let (_stream, stream_handle) =
            rodio::OutputStream::try_default().expect("failed to find output device");
        let sink = rodio::Sink::try_new(&stream_handle).expect("failed to create sink");
        sink.append(audio);
        sink.set_volume(0.3);
        sink.sleep_until_end();
    });
}

pub trait FileData {
    fn get_bytes(&self) -> Vec<u8>;
}

pub enum SoundFile {
    Bell,
}

impl FileData for SoundFile {
    fn get_bytes(&self) -> Vec<u8> {
        match self {
            SoundFile::Bell => include_bytes!("bell.wav").to_vec(),
        }
    }
}
