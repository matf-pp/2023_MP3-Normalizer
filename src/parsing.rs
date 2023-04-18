use std::fs::metadata;
use std::process::Command;

extern crate system_shutdown;
use system_shutdown::*;

extern crate walkdir;
use walkdir::WalkDir;

pub(crate) struct Task {
    pub loudness:f32,
    pub files:Vec<String>,
    pub dest:String,
    pub num_th:i32,
    pub actions:i32
}

impl Task {
pub fn finish(&self) {
        if self.actions & 1 == 1 {
            println!("Opening");
            Command::new("open")
                .arg(&self.dest) // <- Specify the directory you'd like to open.
                .spawn()
                .unwrap();
        }
        for i in 1..4 {
            let functions: Vec<fn() -> ShutdownResult> = vec![shutdown, hibernate, sleep];
            let message: Vec<&str> = vec!["shut down", "hibernate", "sleep"];
            if self.actions & (1 << i) == (1 << i) {
                println!("{}", i);
                match functions[i - 1]() {
                    Ok(_) => println!("Bye!"),
                    Err(error) => eprintln!("Failed to {}: {}", message[i -1], error)
                }
            }
        }
    }
}

pub(crate) fn parse_args(args:Vec<String>) -> Task {
    println!("Usao u parsiranje argumenata");
    let mut loudness:f32 = 89.0;
    let mut files:Vec<String> = Vec::new();
    let mut dest:String = ".".to_string();
    let mut multi_thread:i32 = 1;
    let mut actions:i32 = 0;

    let mut curr:i32 = 0;
    for arg in args {
        if arg == "-i" {
            curr = 1;
        }
        else if arg == "-o" {
            curr = 2;
        }
        else if arg == "-m" {
            curr = 3
        }
        else if arg == "-show" {
            actions |= 1;
        }
        else if arg == "-sd"  {
            actions |= 1 << 1;
        }
        else if arg == "-hi"  {
            actions |= 1 << 2;
        }
        else if arg == "-sl"  {
            actions |= 1 << 3;
        }
        else {
            if curr == 1 {
                let file = metadata(&arg).unwrap();
                if file.is_file() {
                    files.push(arg);
                }
                else {
                    for file in WalkDir::new(arg).into_iter().filter_map(|file| file.ok()) {
                        if file.metadata().unwrap().is_file() {
                            if let Some(extension) = file.path().extension() {
                                if extension == "mp3" || extension == "wav" {
                                    files.push(file.path().display().to_string());
                                }
                            }
                        }
                    }
                }
            }
            else if curr == 2 {
                dest = arg;
            }
            else if curr == 3 {
                multi_thread = arg.parse::<i32>().unwrap();
            }
        }
    }



    let task:Task = Task {
        loudness,
        files,
        dest,
        num_th: multi_thread,
        actions
    };

    return task;
}
