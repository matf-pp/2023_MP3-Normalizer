mod replay_gain_calc;

extern crate id3;
use id3::{Error, ErrorKind, Tag, TagLike, Version};
use id3::frame::{ExtendedText};


#[derive(Clone, Copy)]
pub struct RgTags {
    pub rg_gain:f64,
    pub rg_peak:f64
}

pub fn add_rg_track_tags(path:String, loudness:f64) {
    let rg_track_tags = calc_rg_track_tags(&path, loudness);

    write_rg_tags(&path, rg_track_tags, false);
}

pub fn add_rg_album_tags(paths:Vec<String>, loudness:f64) {
    let rg_album_tags = calc_rg_album_tags(&paths, loudness);

    for path in paths.iter() {
        write_rg_tags(path, rg_album_tags, true);
    }
}

pub fn remove_rg_tags(path:String) {
    let mut tag = match get_tag_from_path(&path) {
        Some(tag) => tag,
        None => return,
    };

    tag.remove_extended_text(Some("REPLAYGAIN_TRACK_GAIN"), None);
    tag.remove_extended_text(Some("REPLAYGAIN_TRACK_PEAK"), None);
    tag.remove_extended_text(Some("REPLAYGAIN_ALBUM_GAIN"), None);
    tag.remove_extended_text(Some("REPLAYGAIN_ALBUM_PEAK"), None);
    // Legacy rg_tags
    tag.remove_extended_text(Some("replaygain_reference_loudness"), None);
    tag.remove_extended_text(Some("replaygain_track_gain"), None);
    tag.remove_extended_text(Some("replaygain_track_peak"), None);
    tag.remove_extended_text(Some("replaygain_album_gain"), None);
    tag.remove_extended_text(Some("replaygain_album_peak"), None);
    tag.remove_extended_text(Some("MP3GAIN_MINMAX"), None);
    tag.remove_extended_text(Some("MP3GAIN_ALBUM_MINMAX"), None);

    tag.write_to_path(&path, Version::Id3v24).expect("Failed writing tag");
}

pub fn write_rg_tags(path:&String, tags:RgTags, ind:bool) {
    let mut tag = match get_tag_from_path(&path) {
        Some(tag) => tag,
        None => return,
    };

    if !ind {
        tag.add_frame(ExtendedText{
            description: "REPLAYGAIN_TRACK_GAIN".to_string(),
            value: tags.rg_gain.to_string() + " dB",
        });
        tag.add_frame(ExtendedText{
            description: "REPLAYGAIN_TRACK_PEAK".to_string(),
            value: tags.rg_peak.to_string(),
        });
    }
    else {
        tag.add_frame(ExtendedText{
            description: "REPLAYGAIN_ALBUM_GAIN".to_string(),
            value: tags.rg_gain.to_string() + " dB",
        });
        tag.add_frame(ExtendedText{
            description: "REPLAYGAIN_ALBUM_PEAK".to_string(),
            value: tags.rg_peak.to_string(),
        });
    }

    tag.write_to_path(&path, Version::Id3v24).expect("Failed writing tag");
}

pub fn get_album_from_path(path:&String) -> Option<String> {
    return match Tag::read_from_path(path) {
        Ok(tag) => tag.album().map(str::to_string),
        Err(_err) => None
    };
}

fn get_tag_from_path(path:&String) -> Option<Tag> {
    return match Tag::read_from_path(path) {
        Ok(tag) => Option::from(tag),
        Err(Error { kind: ErrorKind::NoTag, .. }) => Option::from(Tag::new()),
        Err(_err) => None,
    };
}

fn calc_rg_track_tags(path: &String, loudness: f64) -> RgTags {
    let mut paths: Vec<String> = Vec::new(); paths.push(path.to_string());
    let (rg_track_gain_desired, rg_track_peak_desired) = replay_gain_calc::calc_replay_gain(&paths)[0];

    let rg_tags = RgTags {
        rg_gain: rg_track_gain_desired - loudness,
        rg_peak: rg_track_peak_desired / loudness
    };

    return rg_tags;
}

fn calc_rg_album_tags(paths:&Vec<String>, loudness: f64) -> RgTags {
    let rg_desired = replay_gain_calc::calc_replay_gain(&paths);

    for (i, path) in paths.iter().enumerate() {
        let rg_track_tags = RgTags {
            rg_gain: rg_desired[i].0 - loudness,
            rg_peak: rg_desired[i].1 / loudness
        };
        write_rg_tags(path, rg_track_tags, false);
    }

    let rg_album_desired = rg_desired[paths.len()];

    let rg_tags = RgTags {
        rg_gain: rg_album_desired.0 - loudness,
        rg_peak: rg_album_desired.1 / loudness
    };

    return rg_tags;
}
