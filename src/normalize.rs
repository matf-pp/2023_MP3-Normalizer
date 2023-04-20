use std::fs::copy;

extern crate id3;
use id3::{Content, Error, ErrorKind, Frame, frame, Tag, TagLike, Version};
use id3::frame::{ExtendedText, Unknown};


pub fn normalize(file:String) -> Result<(), Box<dyn std::error::Error>>  {
    // TODO: calculate replay gain for song
    let replaygain_track_gain:f32 = -40.00;
    let replaygain_track_peak:f32 = 1.073917;

    // maybe add album replay_gain
    // let replaygain_album_gain:f32 = -40.00;
    // let replaygain_album_peak:f32 = 1.073917;


    let mut tag = match Tag::read_from_path(&file) {
        Ok(tag) => tag,
        Err(Error{kind: ErrorKind::NoTag, ..}) => Tag::new(),
        Err(err) => return Err(Box::new(err)),
    };

    tag.add_frame(frame::ExtendedText{
        description: "REPLAYGAIN_TRACK_GAIN".to_string(),
        value: replaygain_track_gain.to_string() + " dB",
    });
    tag.add_frame(frame::ExtendedText{
        description: "REPLAYGAIN_TRACK_PEAK".to_string(),
        value: replaygain_track_peak.to_string(),
    });
    // tag.add_frame(frame::ExtendedText{
    //     description: "REPLAYGAIN_ALBUM_GAIN".to_string(),
    //     value: replaygain_album_gain.to_string() + " dB",
    // });
    // tag.add_frame(frame::ExtendedText{
    //     description: "REPLAYGAIN_ALBUM_PEAK".to_string(),
    //     value: replaygain_album_peak.to_string(),
    // });

    tag.write_to_path(&file, Version::Id3v24)?;

    Ok(())
}
