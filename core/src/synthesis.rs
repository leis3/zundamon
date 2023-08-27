use std::io::Cursor;
use vvcore::*;
use once_cell::sync::Lazy;
use byteorder::{LittleEndian, WriteBytesExt};
use songbird::input::{
    Input,
    Codec,
    Reader,
    Container
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

/// wavデータから`Input`を生成する
pub fn to_input(data: &[u8]) -> Input {
    let (_, data) = wav::read(&mut Cursor::new(data)).unwrap();
    // `synthesis()`で得られたデータはpcm_s16leである一方で
    // `Reader::from`ではVec<u8>を要求しているのでリトルエンディアンで変換する
    let data = {
        let mut buf = Vec::new();
        for &i in data.as_sixteen().unwrap() {
            buf.write_i16::<LittleEndian>(i).unwrap();
        }
        buf
    };
    Input::new(
        true,
        Reader::from(data.clone()),
        Codec::Pcm,
        Container::Raw,
        None
    )
}
