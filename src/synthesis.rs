use vvcore::*;

/// VOICEVOX COREで音声を合成する。
pub fn synthesis(text: &str) -> Result<Vec<u8>, ResultCode> {
    let dir = std::ffi::CString::new("voicevox_core/open_jtalk_dic_utf_8-1.11").unwrap();

    let vvc = VoicevoxCore::new_from_options(AccelerationMode::Auto, 0, false, dir.as_c_str())?;
    vvc.load_model(1).unwrap();

    let wav = vvc.tts_simple(text, 1)?;

    Ok(wav.as_slice().to_vec())
}
