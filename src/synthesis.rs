use std::io::Write;
use std::sync::Arc;
use vvcore::*;
use once_cell::sync::Lazy;
use songbird::input::{
    Input,
    Codec,
    Metadata,
    Container,
    children_to_reader
};

static VOICEVOX_CORE: Lazy<VoicevoxCore> = Lazy::new(|| {
    let dir = std::ffi::CString::new("voicevox_core/open_jtalk_dic_utf_8-1.11").unwrap();

    let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, false, dir.as_c_str()).unwrap();
    vvc.load_model(3).unwrap();

    vvc
});

/// VOICEVOX COREで音声を合成する。
pub fn synthesis(text: &str) -> Result<Vec<u8>, ResultCode> {
    let wav = VOICEVOX_CORE.tts_simple(text, 3)?;

    Ok(wav.as_slice().to_vec())
}

/// ffmpegで音声を処理する
pub fn ffmpeg(data: &[u8]) -> Input {
    let metadata = {
        let rdr = hound::WavReader::new(data).unwrap();
        let spec = rdr.spec();
        let duration = rdr.duration() as f64 / spec.sample_rate as f64;
        Metadata {
            channels: Some(spec.channels as u8),
            duration: Some(std::time::Duration::from_secs_f64(duration)),
            sample_rate: Some(spec.sample_rate),
            ..Default::default()
        }
    };

    let args = [
        "-i",
        "-",
        "-f",
        "s16le",
        "-af",
        "atempo=1.2",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-"
    ];

    let mut command = std::process::Command::new("ffmpeg")
        .args(&args)
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");


    let mut stdin = command.stdin.take().expect("Failed to open stdin");
    let data = Arc::new(data.to_vec());
    std::thread::spawn(move || {
        stdin.write_all(&data).expect("Failed to write to stdin");  
    });

    Input::new(
        true,
        children_to_reader::<f32>(vec![command]),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata)
    )
}
