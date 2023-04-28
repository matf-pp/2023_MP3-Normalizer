extern crate minimp3;
use minimp3::{Decoder, Frame, Error};

use std::fs::File;
use std::sync::{Arc, Mutex};
use rayon::prelude::*;

pub fn calc_replay_gain(paths: &Vec<String>) -> Vec<f64> {
    let gain_array_tracks1 = Arc::new(Mutex::new(Vec::new()));

    let mut track_gains:Vec<_> = paths.par_iter().map(|path| {
        let clone = Arc::clone(&gain_array_tracks1);
        let mut gain_array_track: Vec<f64> = Vec::new();

        let mut decoder = Decoder::new(File::open(path).unwrap());
        loop {
            match decoder.next_frame() {
                Ok(Frame { data, sample_rate: _sample_rate, channels, .. }) => {
                    let samples = data;
                    let samples_per_channel = samples.len() / channels as usize;
                    let mut rms_vec = Vec::new();
                    // let mut peak: f32 = 0.0;

                    for channel in samples.chunks_exact(samples_per_channel) {
                        // calc_peak(channel, &mut peak);
                        let rms = calc_rms(channel);
                        rms_vec.push(rms);
                    }
                    let len = rms_vec.len() as f64;
                    let mut x = rms_vec.into_iter().sum::<f64>() / len;
                    let const_log_factor = 1e-10;
                    x = 20.0 * (x + const_log_factor).log10() as f64;
                    gain_array_track.push(x);
                    let mut clone1= clone.lock().unwrap();
                    clone1.push(x);
                    drop(clone1);
                },
                Err(Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
        }
        gain_array_track.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let rg_index_track = ((gain_array_track.len() as f64) * 0.95).round() as usize;
        let replay_gain_track = gain_array_track[rg_index_track];
        replay_gain_track
    }).collect();

    // Sort vector of floats
    let mut gain_array_tracks = gain_array_tracks1.lock().unwrap();
    gain_array_tracks.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rg_index_tracks = ((gain_array_tracks.len() as f64) * 0.95).round() as usize;
    let replay_gain_tracks = gain_array_tracks[rg_index_tracks];
    track_gains.push(replay_gain_tracks);

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

/*
fn calc_peak(sample: &[i16], peak: &mut 32) {
    *peak = sample.iter().cloned().map(f32::abs).fold(*peak, f32::max)
}
 */
