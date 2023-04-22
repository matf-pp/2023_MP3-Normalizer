mod parse;
mod normalize;

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::fs::{File, metadata};
use std::path;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use crate::normalize::{add_rg_track_tags, remove_rg_tags, get_album_from_path, add_rg_album_tags};


// vrv najgori moguci kod ikada
fn main() {
    let args: Vec<String> = env::args().collect();
    let task: parse::Task = parse::parse_args(args);
    let norm = (task.actions & 1 << 4) == 0;
    let cp = task.dest.clone() != ".";
    let album_rg = (task.actions & 1 << 5) != 0;
    let albums = Arc::new(Mutex::new(HashMap::new()));

    let new_paths = Arc::new(Mutex::new(Vec::new()));
    if cp {
        let _temp = fs::create_dir(&task.dest);
        assert!(Path::new(&task.dest.as_str()).is_dir(), "Output dir error");

        let paths = Arc::new(Mutex::new(task.paths.to_vec()));

        let mut threads_dir = Vec::new();
        for _i in 0..task.num_th {
            threads_dir.push(thread::spawn({
                let clone = Arc::clone(&paths);
                let new_paths_clone = Arc::clone(&new_paths);
                let dest = task.dest.clone();

                move || loop {
                    let mut v = clone.lock().unwrap();
                    let len = v.len();
                    if len == 0 {
                        break;
                    }
                    else {
                        let path = v.remove(len - 1);
                        let file = Path::new(path.as_str()).file_name().unwrap();

                        let dest_path = Path::new(dest.clone().as_str()).join(file);
                        //let _file = File::create(&dest_path); ovo pravi problem iz nekog razloga
                        let dest_path_str = dest_path.to_str().unwrap();

                        match dest_path.metadata() {
                            Ok(metadata) => {
                                if metadata.is_file() {
                                    println!("File {} already exists", dest_path_str);
                                    continue;
                                }
                                else {
                                    println!("Err, skip file {}", path);
                                    continue;
                                }
                            }
                            Err(_) => {}
                        }

                        let mut vv = new_paths_clone.lock().unwrap();
                        vv.push(dest_path_str.to_string());

                        fs::copy(&path, &dest_path_str).unwrap();
                        assert!(metadata(dest_path_str).unwrap().is_file());
                        println!("Copied {} to {}", path, dest_path_str);
                    }
                }
            }));
        }

        for thread_dir in threads_dir {
            thread_dir.join().unwrap();
        }
    }

    let paths = match cp {
        true => {new_paths}
        false => Arc::new(Mutex::new(task.paths.to_vec()))
    };

    let mut threads = Vec::new();
    for _i in 0..task.num_th {
        threads.push(thread::spawn({
            let clone = Arc::clone(&paths);
            let albums = Arc::clone(&albums);

            move || loop {
                let mut v = clone.lock().unwrap();
                let len = v.len();
                if len == 0 {
                    break;
                } else {
                    let path = v.remove(len - 1);
                    if !album_rg {
                        if norm {
                            add_rg_track_tags(path);
                        }
                        else {
                            remove_rg_tags(path);
                        }
                    }
                    else {
                        let album =  match get_album_from_path(&path) {
                            Some(album) => album,
                            None => "__None__".to_string()
                        };

                        let mut albums = albums.lock().unwrap();
                        let mut temp_set = HashSet::new();
                        temp_set.insert(path.clone());

                        albums.entry(album).and_modify(|set:&mut HashSet<String>| {set.insert(path);}).or_insert(temp_set);
                    }
                }
            }
        }));
    }


    for t in threads {
        t.join().unwrap();
    }

    if album_rg {
        let albums = albums.lock().unwrap();
        for (key, value) in albums.iter() {
            for x in value.iter() {
                println!("{} {}", key, x);
            }
            add_rg_album_tags(value.clone());
        }



    }

    task.finish();
}
