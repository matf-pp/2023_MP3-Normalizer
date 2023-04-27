mod replay_gain_calc;

use std::collections::HashSet;

extern crate id3;
use id3::{Error, ErrorKind, Tag, TagLike, Version};
use id3::frame::{ExtendedText};


struct RgTrackTags {
    rg_track_gain:f64,
    rg_track_peak:f32
}

struct RgAlbumTags {
    rg_album_gain:f64,
    rg_album_peak:f32
}

pub fn add_rg_track_tags(path:String, loudness:f64) {
    let rg_track_tags = calc_rg_track_tags(&path, loudness);

    let mut tag = match get_tag_from_path(&path) {
        Some(tag) => tag,
        None => return,
    };

    tag.add_frame(ExtendedText{
        description: "REPLAYGAIN_TRACK_GAIN".to_string(),
        value: rg_track_tags.rg_track_gain.to_string() + " dB",
    });
    tag.add_frame(ExtendedText{
        description: "REPLAYGAIN_TRACK_PEAK".to_string(),
        value: rg_track_tags.rg_track_peak.to_string(),
    });

    tag.write_to_path(&path, Version::Id3v24).expect("Failed writing tag");
}

pub fn add_rg_album_tags(paths:HashSet<String>, loudness:f64) {
    let rg_album_tags = calc_rg_album_tags(&paths, loudness);

    for path in paths.iter() {
        let mut tag = match get_tag_from_path(&path) {
            Some(tag) => tag,
            None => continue,
        };

        tag.add_frame(ExtendedText{
            description: "REPLAYGAIN_ALBUM_GAIN".to_string(),
            value: rg_album_tags.rg_album_gain.to_string() + " dB",
        });
        tag.add_frame(ExtendedText{
            description: "REPLAYGAIN_ALBUM_PEAK".to_string(),
            value: rg_album_tags.rg_album_peak.to_string(),
        });

        tag.write_to_path(&path, Version::Id3v24).expect("Failed writing tag");
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
    // legacy rg_tags
    tag.remove_extended_text(Some("replaygain_reference_loudness"), None);
    tag.remove_extended_text(Some("replaygain_track_gain"), None);
    tag.remove_extended_text(Some("replaygain_track_peak"), None);
    tag.remove_extended_text(Some("replaygain_album_gain"), None);
    tag.remove_extended_text(Some("replaygain_album_peak"), None);
    tag.remove_extended_text(Some("MP3GAIN_MINMAX"), None);
    tag.remove_extended_text(Some("MP3GAIN_ALBUM_MINMAX"), None);

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

fn calc_rg_track_tags(path: &String, loudness: f64) -> RgTrackTags {
    let mut paths: HashSet<String> = HashSet::new(); paths.insert(path.to_string());
    let rg_track_gain_desired = replay_gain_calc::calc_replay_gain(&paths);

    let rg_tags: RgTrackTags = RgTrackTags {
        rg_track_gain: rg_track_gain_desired - loudness,
        rg_track_peak: 1.0
    };

    return rg_tags;
}

fn calc_rg_album_tags(paths:&HashSet<String>, loudness: f64) -> RgAlbumTags {
    let rg_album_gain_desired: f64 = replay_gain_calc::calc_replay_gain(paths);

    let rg_tags = RgAlbumTags {
        rg_album_gain: rg_album_gain_desired - loudness,
        rg_album_peak: 1.0
    };

    return rg_tags;
}
