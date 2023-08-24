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

pub fn initialize() {
    let _ = &*VOICEVOX_CORE;
}

/// VOICEVOX COREで音声を合成する。
pub fn synthesis(text: &str, speaker_id: u32) -> Result<Vec<u8>, ResultCode> {
    if !VOICEVOX_CORE.is_model_loaded(speaker_id) {
        VOICEVOX_CORE.load_model(speaker_id).unwrap();
    }

    let mut query: serde_json::Value = serde_json::from_str(
        VOICEVOX_CORE.audio_query(text, speaker_id, VoicevoxCore::make_default_audio_query_options())?.as_str()
    ).unwrap();
    if let Some(value) = query.get_mut("output_stereo") {
        *value = true.into();
    }
    if let Some(value) = query.get_mut("output_sampling_rate") {
        *value = 48000.into();
    }
    if let Some(value) = query.get_mut("speed_scale") {
        *value = 1.2.into();
    }
    
    let query = serde_json::to_string(&query).unwrap();
    let wav = VOICEVOX_CORE.synthesis(&query, speaker_id, VoicevoxCore::make_default_synthesis_options())?;

    Ok(wav.as_slice().to_vec())
}

/// ffmpegで音声を処理する
pub fn ffmpeg(data: &[u8]) -> Input {
    let metadata = {
        let rdr = hound::WavReader::new(data).unwrap();
        let duration = rdr.duration() as f64 / rdr.spec().sample_rate as f64;
        Metadata {
            channels: Some(2),
            duration: Some(std::time::Duration::from_secs_f64(duration)),
            sample_rate: Some(48000),
            ..Default::default()
        }
    };

    let mut command = std::process::Command::new("ffmpeg")
        .args(["-i", "-", "-f", "s16le", "-"])
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn process");


    let mut stdin = command.stdin.take().expect("Failed to open stdin");
    let data = Arc::new(data.to_vec());
    std::thread::spawn(move || {
        // `/skip`によってBroken pipeになる可能性があるので結果を無視する
        let _ = stdin.write_all(&data);
    });

    Input::new(
        true,
        children_to_reader::<u8>(vec![command]),
        Codec::Pcm,
        Container::Raw,
        Some(metadata)
    )
}
