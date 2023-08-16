use std::io::Write;
use std::sync::Arc;
use vvcore::*;
use once_cell::sync::Lazy;
use tracing::error;
use songbird::input::{
    Input,
    Codec,
    Reader,
    Metadata,
    Container
};

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

/// ffmpegで音声を処理する
pub fn ffmpeg(data: &[u8]) -> anyhow::Result<Input> {
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
        .args(args)
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            error!("Failed to spawn process: {e:?}");
            e
        })?;


    let Some(mut stdin) = command.stdin.take() else {
        error!("Failed to open stdin");
        anyhow::bail!("Failed to open stdin");
    };
    let data = Arc::new(data.to_vec());
    std::thread::spawn(move || {
        // `/skip`によってBroken pipeになる可能性があるので結果を無視する
        let _ = stdin.write_all(&data);
    }).join().unwrap();

    let output = command.wait_with_output()
        .map_err(|e| {
            error!("Failed to wait on child: {e:?}");
            e
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        error!("The exit code of ffmpeg was not SUCCESS; stderr: {stderr:?}");
        anyhow::bail!("The exit code of ffmpeg was not SUCCESS");
    }

    let data = output.stdout;

    Ok(Input::new(
        true,
        Reader::from_memory(data),
        Codec::FloatPcm,
        Container::Raw,
        Some(metadata)
    ))
}
