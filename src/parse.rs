use std::fs::metadata;
use std::process::Command;

extern crate system_shutdown;
use system_shutdown::*;

extern crate walkdir;
use walkdir::WalkDir;


pub(crate) struct Task {
    pub loudness:f32,
    pub paths:Vec<String>,
    pub dest:String,
    pub num_th:i32,
    pub actions:i32
}

impl Task {
pub fn finish(&self) {
        if self.actions & 1 == 1 {
            println!("Opening");
            Command::new("open")
                .arg(&self.dest)
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
    let mut loudness:f32 = 89.0;
    let mut paths:Vec<String> = Vec::new();
    let mut dest:String = ".".to_string();
    let mut num_th:i32 = 1;
    let mut actions:i32 = 0;

    let mut curr:i32 = 0;
    for arg in args {
        let arg1 = arg.as_str();
        match arg1 {
            "-i" => curr = 1,
            "-o" => curr = 2,
            "-nt" => curr = 3,
            "-r" => actions |= 1 << 4,
            "-a" => actions |= 1 << 5,
            "-show" => actions |= 1,
            "-sd" => actions |= 1 << 1,
            "-hi" => actions |= 1 << 2,
            "-sl" => actions |= 1 << 3,
            _ => {
                match curr {
                    1 => {
                        let path = metadata(&arg).unwrap();
                        if path.is_file() {
                            paths.push(arg);
                        }
                        else {
                            for path in WalkDir::new(arg).into_iter().filter_map(|path| path.ok()) {
                                if path.metadata().unwrap().is_file() {
                                    if let Some(extension) = path.path().extension() {
                                        if extension == "mp3" {
                                            paths.push(path.path().display().to_string());
                                        }
                                    }
                                }
                            }
                        }
                    },
                    2 => dest = arg,
                    3 => num_th = arg.parse::<i32>().unwrap(),
                    _ => {}
                }
            }
        }
    }

    let task:Task = Task {
        loudness,
        paths,
        dest,
        num_th,
        actions
    };

    return task;
}
