use std::collections::HashSet;

extern crate id3;
use id3::{Error, ErrorKind, Tag, TagLike, Version};
use id3::frame::{ExtendedText};

use std::io::BufReader;
use itertools::Itertools;
use rodio::Source;

struct RgTrackTags {
    rg_track_gain:f32,
    rg_track_peak:f32
}

struct RgAlbumTags {
    rg_album_gain:f32,
    rg_album_peak:f32
}

pub fn add_rg_track_tags(path:String) {
    let rg_track_tags = calc_rg_track_tags(&path);

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

pub fn add_rg_album_tags(paths:HashSet<String>) {
    let rg_album_tags = calc_rg_album_tags(&paths);

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

// TODO
fn calc_rg_track_tags(path: &String) -> RgTrackTags {
     let rg_tags = RgTrackTags {
         rg_track_gain: -9.0,
         rg_track_peak: 1.0
     };

    return rg_tags;
}

fn calc_rg_album_tags(paths:&HashSet<String>) -> RgAlbumTags {
    let rg_tags = RgAlbumTags {
        rg_album_gain: -19.0,
        rg_album_peak: 1.0
    };

    return rg_tags;
}

fn calc_replay_gain(path: &str) -> f64 {
    let file = std::fs::File::open(path).unwrap();
    let decoder = rodio::Decoder::new(BufReader::new(file)).unwrap();
    let sample_rate = decoder.sample_rate();
    let sample_chunk: u32 = sample_rate / 20;
    // let total_duration = decoder.total_duration();
    let channels = decoder.channels();
    let samples = decoder.into_iter().collect_vec();
    let samples_per_channel = samples.len() / channels as usize;

    let mut rms_vec = [(); 2].map(|_| Vec::new());
    for (i, channel) in samples.chunks_exact(samples_per_channel).enumerate()
    {
        for (j, sample) in channel.chunks(sample_chunk as usize).enumerate()
        {
            // TODO
            // equal_loudness_filter
            
            // Call RMS calculation
            let rms = calc_rms(sample);
            rms_vec[i].push(rms);

            // take mean of stereo channels
            if i == 1 {
                rms_vec[0][j] += rms_vec[1][j];
                rms_vec[0][j] /= 2 as f64;
                
                // Convert to dB
                let const_log_factor = 1e-10;
                rms_vec[0][j] = 20 as f64 * (rms_vec[0][j] + const_log_factor).log10() as f64;
            }
        }
    }

    // Sort vector of floats
    rms_vec[0].sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rg_index = ((rms_vec[0].len() as f64) * (0.95 as f64)).round() as usize;
    let replay_gain = rms_vec[0][rg_index];

    replay_gain
}

fn calc_rms(sample: &[i16]) -> f64{
    let mut sqr_sum = 0.0;
    for sample_val in sample {
        let val = *sample_val as f64;
        sqr_sum += val * val;
    }
    let rms = (sqr_sum / sample.len() as f64).sqrt();
    rms
}