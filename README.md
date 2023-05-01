# MP3 Normalizer based on ReplayGain

[![Codacy Badge](https://app.codacy.com/project/badge/Grade/f97aa752bf4746e18cc4c8387b17fb39)](https://app.codacy.com/gh/matf-pp/2023_MP3-Normalizer/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)

## About application

Console application written in Rust, that normalizes track or album. 

## Run

Clone repository, open folder containing repository and run the following:

```
cargo build --release
```
You should see the `release` folder where you can find and run executable `mp3-normalizer`. For example after running your executable you can enter the following:
```
cargo run -i "INPUT_TRACK_OR_FOLDER_PATH" -o "OUTPUT_TRACK_OR_FOLDER_PATH"
```

## Features

+ Normalize on preffered loudness
+ Based on ReplayGain standard for normalization
+ Multithreaded support for increased speed
+ Additional command line argument support to further help user specify their use case

### Command line featuers
+ `-i` Specify input folder
+ `-o` Specify output folder
+ `-nt` Number of threads
+ `-l` Set loudness
+ `-st` Set track replay gain
+ `-sa` Set album replay gain
+ `-r` Remove tags
+ `-a` Album replay gain
+ `-ad` Directory replay gain
+ `-show` Open output directory
+ `-sd` Shut down PC after finishing
+ `-hi` Hibernate
+ `-sl` Sleep after finishing

## Used Techonologies

Whole application is written and developed in Rust

### Rust dependencies and their versions
+ walkdir = "2"
+ system_shutdown = "4.0.1"
+ id3 = "1.7.0"
+ minimp3 = "0.5.1"
+ rayon = "1.7.0"
