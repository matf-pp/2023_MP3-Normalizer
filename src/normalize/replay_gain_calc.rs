extern crate minimp3;
use minimp3::{Decoder, Frame, Error};

use std::fs::File;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;


const CONST_LOG_FACTOR: f64 = 1e-10;

pub fn calc_replay_gain(paths: &Vec<String>) -> Vec<(f64, f64)> {
    let gain_array_tracks = Arc::new(Mutex::new(Vec::new()));

    let mut track_gains:Vec<_> = paths.par_iter().map(|path| {
        let clone_gain = Arc::clone(&gain_array_tracks);

        let mut gain_array_track: Vec<f64> = Vec::new();
        let mut peak_track: f64 = 0.0;

        let mut decoder = Decoder::new(File::open(path).unwrap());
        loop {
            match decoder.next_frame() {
                Ok(Frame { data, sample_rate: _sample_rate, channels, .. }) => {
                    let samples = data;
                    let samples_per_channel = samples.len() / channels as usize;
                    let mut rms_vec = Vec::new();

                    for channel in samples.chunks_exact(samples_per_channel) {
                        calc_peak(channel, &mut peak_track);
                        let rms = calc_rms(channel);
                        rms_vec.push(rms);
                    }
                    let len = rms_vec.len() as f64;

                    let mut rg_db: f64 = rms_vec.into_iter().sum::<f64>() / len;
                    rg_db = 20.0 * (rg_db + CONST_LOG_FACTOR).log10() as f64;

                    gain_array_track.push(rg_db);

                    let mut clone = clone_gain.lock().unwrap();
                    clone.push(rg_db);
                    drop(clone);
                },
                Err(Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
        }
        gain_array_track.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let rg_index_track = ((gain_array_track.len() as f64) * 0.95).round() as usize;
        let replay_gain_track = gain_array_track[rg_index_track];
        peak_track = 20.0 * (peak_track + CONST_LOG_FACTOR).log10() as f64;
        (replay_gain_track, peak_track)
    }).collect();

    // Sort vector of floats
    let mut gain_array_tracks_final: std::sync::MutexGuard<Vec<f64>> = gain_array_tracks.lock().unwrap();
    gain_array_tracks_final.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rg_index_tracks = ((gain_array_tracks_final.len() as f64) * 0.95).round() as usize;
    let replay_gain_tracks = gain_array_tracks_final[rg_index_tracks];
    let peak_tracks = track_gains.iter().map(|pair| pair.1)
        .max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    track_gains.push((replay_gain_tracks, peak_tracks));

    track_gains
}

fn calc_rms(sample: &[i16]) -> f64 {
    let mut sqr_sum = 0.0;
    for sample_val in sample.iter() {
        let val = *sample_val as f64;
        sqr_sum += val * val;
    }
    let rms = (sqr_sum / sample.len() as f64).sqrt();
    rms
}


fn calc_peak(sample: &[i16], peak: &mut f64) {
    for sample_val in sample.iter() {
        if (*sample_val as f64).abs() > *peak {
            *peak = (*sample_val as f64).abs();
        }
    }
}
