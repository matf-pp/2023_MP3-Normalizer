mod parsing;
mod normalize;

use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::normalize::{add_rg_track_tags, remove_rg_tags};


fn main() {
    let args: Vec<String> = env::args().collect();
    let task: parsing::Task = parsing::parse_args(args);
    let norm:bool = (task.actions & 1 << 4) == 0;

    let files = Arc::new(Mutex::new(task.files.to_vec()));
    let mut threads = Vec::new();
    for _i in 0..task.num_th {
        threads.push(thread::spawn({
            let clone = Arc::clone(&files);

            move || loop {
                let mut v = clone.lock().unwrap();
                let len = v.len();
                if len == 0 {
                    break;
                } else {
                    let file = v.remove(len - 1);
                    if norm {
                        add_rg_track_tags(file);
                    }
                    else {
                        remove_rg_tags(file);
                    }
                }
            }
        }));
    }
    for t in threads {
        t.join().unwrap();
    }

    task.finish();
}
