extern crate minimp3;
use minimp3::{Decoder, Frame, Error};

use std::collections::HashSet;
use std::fs::File;


pub fn calc_replay_gain(paths: &HashSet<String>) -> f64 {
    let mut gain_array: Vec<f64> = Vec::new();

    for path in paths {
        let mut decoder = Decoder::new(File::open(path).unwrap());
        loop {
            match decoder.next_frame() {
                Ok(Frame { data, sample_rate: _sample_rate, channels, .. }) => {
                    let samples = data;
                    let samples_per_channel = samples.len() / channels as usize;
                    let mut rms_vec = Vec::new();

                    for channel in samples.chunks_exact(samples_per_channel) {
                        let rms = calc_rms(channel);
                        rms_vec.push(rms);
                    }
                    let len = rms_vec.len() as f64;
                    let mut x = rms_vec.into_iter().sum::<f64>() / len;
                    let const_log_factor = 1e-10;
                    x = 20.0 * (x + const_log_factor).log10() as f64;
                    gain_array.push(x);
                },
                Err(Error::Eof) => break,
                Err(e) => panic!("{:?}", e),
            }
        }

    }
    // Sort vector of floats
    gain_array.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let rg_index = ((gain_array.len() as f64) * 0.95).round() as usize;
    let replay_gain = gain_array[rg_index];

    replay_gain
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
