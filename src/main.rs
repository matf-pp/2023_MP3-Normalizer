mod parse;
mod normalize;

use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::metadata;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::normalize::{add_rg_track_tags, remove_rg_tags, get_album_from_path, add_rg_album_tags, write_rg_tags, RgTags};
use std::time::Instant;
use std::process::exit;


fn main() {
    let now = Instant::now();

    let args: Vec<String> = env::args().collect();
    let task: parse::Task = parse::parse_args(args);

    let loudness = task.loudness;
    assert!(loudness > 0.0);
    let set_tr = (task.actions & 1 << 7) != 0;
    let set_al = (task.actions & 1 << 8) != 0;
    let set = set_tr || set_al;
    let rg_set = task.rg_set;
    assert!(!(set_tr && set_al));
    let remove = (task.actions & 1 << 4) != 0;
    assert!(!(remove && set));
    let norm = !remove && !set;
    let cp = task.dest.clone() != "$$$";
    let album_rg = (task.actions & 1 << 5) != 0;
    let album_rg_dir = (task.actions & 1 << 6) != 0;
    assert!(!(album_rg && album_rg_dir));
    let help = (task.actions & (1 << 9)) != 0;
    if help {
        assert!((task.actions & (1 << 9)) == (1 << 9));
        helper_fn();
        exit(0);
    }

    let albums = Arc::new(Mutex::new(HashMap::new()));

    let new_paths = Arc::new(Mutex::new(Vec::new()));
    if cp {
        let _temp = fs::create_dir(&task.dest);
        assert!(Path::new(&task.dest.as_str()).is_dir(), "Output dir error");

        let paths = Arc::new(Mutex::new(task.paths.to_vec()));

        let mut threads_dir = Vec::new();
        for _i in 0..task.num_th {
            threads_dir.push(thread::spawn({
                let paths_clone = Arc::clone(&paths);
                let new_paths_clone = Arc::clone(&new_paths);
                let dest = task.dest.clone();

                move || loop {
                    let mut v = paths_clone.lock().unwrap();
                    let len = v.len();

                    if len == 0 {
                        drop(v);
                        break;
                    }
                    else {
                        let path = v.remove(len - 1);
                        drop(v);
                        let file = Path::new(path.as_str()).file_name().unwrap();

                        let dest_path = Path::new(dest.clone().as_str()).join(file);
                        //let _file = File::create(&dest_path); This is a problem for some reason
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
                        drop(vv);

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
        true => new_paths,
        false => Arc::new(Mutex::new(task.paths.to_vec()))
    };

    let mut threads = Vec::new();
    for _i in 0..task.num_th {
        threads.push(thread::spawn({
            let paths_clone = Arc::clone(&paths);
            let albums = Arc::clone(&albums);

            move || loop {
                let mut v = paths_clone.lock().unwrap();
                let len = v.len();

                if len == 0 {
                    drop(v);
                    break;
                }
                else {
                    let path = v.remove(len - 1);
                    drop(v);
                    // println!("Obradjujem {}", path);

                    if !norm {
                        if set {
                            let rg_tags = RgTags {
                                rg_gain: rg_set,
                                rg_peak: 1.0
                            };

                            if set_tr {
                                write_rg_tags(&path, rg_tags, false);
                            }
                            else {
                                write_rg_tags(&path, rg_tags, true);
                            }
                        }
                        else {
                            remove_rg_tags(path);
                        }
                    }
                    else {
                        if album_rg || album_rg_dir {
                            let mut albums = albums.lock().unwrap();

                            let mut temp_vec = Vec::new();
                            temp_vec.push(path.clone());

                            if album_rg {
                                let album =  match get_album_from_path(&path) {
                                    Some(album) => album,
                                    None => "__None__".to_string()
                                };

                                albums.entry(album).and_modify(|vec:&mut Vec<String>| {vec.push(path);}).or_insert(temp_vec);
                            }
                            else {
                                albums.entry("".to_string()).and_modify(|vec:&mut Vec<String>| {vec.push(path);}).or_insert(temp_vec);
                            }
                        }
                        else {
                            add_rg_track_tags(path.clone(), loudness);
                        }
                    }
                }
            }
        }));
    }

    for t in threads {
        t.join().unwrap();
    }

    if norm && (album_rg || album_rg_dir) {
        let mut albums_vec = Vec::new();

        let albums = albums.lock().unwrap();
        for (_key, value) in albums.iter() {
            // for x in value.iter() {
            //     println!("{} {}", key, x);
            // }
            albums_vec.push(value.clone());
        }


        let albums = Arc::new(Mutex::new(albums_vec));
        let mut threads_album = Vec::new();
        for _i in 0..task.num_th {
            threads_album.push(thread::spawn({
                let albums_clone = Arc::clone(&albums);

                move || loop {
                    let mut albums = albums_clone.lock().unwrap();
                    let len = albums.len();

                    if len == 0 {
                        drop(albums);
                        break;
                    }
                    else {
                        let album = albums.remove(len - 1);
                        drop(albums);
                        add_rg_album_tags(album, loudness);
                    }
                }
            }));
        }

        for thread_album in threads_album {
            thread_album.join().unwrap();
        }
    }

    task.finish();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
}


fn helper_fn() {
    println!("Help for MP3 Normalizer:");

    println!("-i \t Specify input folder");
    println!("-o \t Specify output folder");
    println!("-nt \t Number of threads");
    println!("-l \t Set loudness");
    println!("-st \t Set track replay gain");
    println!("-sa \t Set album replay gain");
    println!("-r \t run");
    println!("-a \t Normalize album");
    println!("-ad \t idk");
    println!("-show \t Show results");
    println!("-sd \t idk");
    println!("-hi \t idk");
    println!("-sl \t idk");

    println!("");
    println!("Rust already uses some arguments for it's own purposes (ie. -i), so make sure to use -- before parsing some arguments");
}