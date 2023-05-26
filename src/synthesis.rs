use vvcore::*;
use once_cell::sync::Lazy;

static VOICEVOX_CORE: Lazy<VoicevoxCore> = Lazy::new(|| {
    let dir = std::ffi::CString::new("voicevox_core/open_jtalk_dic_utf_8-1.11").unwrap();

    let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, false, dir.as_c_str()).unwrap();
    vvc.load_model(1).unwrap();

    vvc
});

/// VOICEVOX COREで音声を合成する。
pub fn synthesis(text: &str) -> Result<Vec<u8>, ResultCode> {
    let wav = VOICEVOX_CORE.tts_simple(text, 1)?;

    Ok(wav.as_slice().to_vec())
}
