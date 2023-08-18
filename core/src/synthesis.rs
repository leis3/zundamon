use hound;
use vvcore::*;
use once_cell::sync::Lazy;
use std::io::Cursor;
use songbird::input::{Reader, Metadata, Input, Container, Codec};

static VOICEVOX_CORE: Lazy<VoicevoxCore> = Lazy::new(|| {
    let dir = std::ffi::CString::new("voicevox_core/open_jtalk_dic_utf_8-1.11").unwrap();

    let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, false, dir.as_c_str()).unwrap();
    vvc.load_model(3).unwrap();

    vvc
});

/// VOICEVOX COREで音声を合成する。
pub fn synthesis(text: &str, speaker_id: u32) -> Result<Vec<u8>, ResultCode> {
    if !VOICEVOX_CORE.is_model_loaded(speaker_id) {
        VOICEVOX_CORE.load_model(speaker_id).unwrap();
    }
    
    let wav = VOICEVOX_CORE.tts_simple(text, speaker_id)?;

    Ok(wav.as_slice().to_vec())
}


pub fn convert_wav_to_input(wav_data: &[u8]) -> Input {
    let reader = hound::WavReader::new(wav_data).unwrap();

    let spec = hound::WavSpec {
        channels: reader.spec().channels,
        sample_rate: 48000,  // 48kHz
        bits_per_sample: reader.spec().bits_per_sample,
        sample_format: reader.spec().sample_format,
    };

    let duration = reader.duration() as f64 / spec.sample_rate as f64;

    let mut output_data = Vec::new();  // バッファを作成
    {
        let mut writer = hound::WavWriter::new(Cursor::new(&mut output_data), spec).unwrap();

        for sample in reader.into_samples::<i16>() {
            let sample: i16 = sample.unwrap();
            writer.write_sample(sample).unwrap();
            writer.write_sample(sample).unwrap();
        }
    }

    Input::new(
        true,
        Reader::from(output_data),
        Codec::FloatPcm,
        Container::Raw,
        Some({
            Metadata {
                channels: Some(spec.channels as u8),
                duration: Some(std::time::Duration::from_secs_f64(duration)),
                sample_rate: Some(spec.sample_rate),
                ..Default::default()
            }
        })
    )
}

// /// ffmpegで音声を処理する
// pub fn ffmpeg(data: &[u8]) -> Input {
//     let metadata = {
//         let rdr = hound::WavReader::new(data).unwrap();
//         let spec = rdr.spec();
//         let duration = rdr.duration() as f64 / spec.sample_rate as f64;
//         Metadata {
//             channels: Some(spec.channels as u8),
//             duration: Some(std::time::Duration::from_secs_f64(duration)),
//             sample_rate: Some(spec.sample_rate),
//             ..Default::default()
//         }
//     };

//     let args = [
//         "-i",
//         "-",
//         "-f",
//         "s16le",
//         "-af",
//         "atempo=1.2",
//         "-ac",
//         "2",
//         "-ar",
//         "48000",
//         "-acodec",
//         "pcm_f32le",
//         "-"
//     ];

//     let mut command = std::process::Command::new("ffmpeg")
//         .args(args)
//         .stderr(std::process::Stdio::null())
//         .stdin(std::process::Stdio::piped())
//         .stdout(std::process::Stdio::piped())
//         .spawn()
//         .expect("Failed to spawn process");


//     let mut stdin = command.stdin.take().expect("Failed to open stdin");
//     let data = Arc::new(data.to_vec());
//     std::thread::spawn(move || {
//         // `/skip`によってBroken pipeになる可能性があるので結果を無視する
//         let _ = stdin.write_all(&data);
//     });

//     Input::new(
//         true,
//         children_to_reader::<f32>(vec![command]),
//         Codec::FloatPcm,
//         Container::Raw,
//         Some(metadata)
//     )
// }
