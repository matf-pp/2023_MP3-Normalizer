# MP3 Normalizer based on ReplayGain

[![Codacy Badge](https://app.codacy.com/project/badge/Grade/f97aa752bf4746e18cc4c8387b17fb39)](https://app.codacy.com/gh/matf-pp/2023_MP3-Normalizer/dashboard?utm_source=gh&utm_medium=referral&utm_content=&utm_campaign=Badge_grade)

## About application

The idea was to create a console app that will normalize track or album of tracks. Written in Rust programming language, our application uses multithreaded support, to give users fast normalization.
<br>
Project was based on replay gain, modifying peak and gain difference tags in MP3 file.
+   `Gain difference` was calculated by measuring effective power of the waveform (RMS) substracted by some loudness constant (our default is 89.0 dB)
+   `Peak` was calculated by measuring the amplitude of the track/album, divided by the maximum aplitude possible a track can have

## Run

Download the latest `mp3_normalizer.zip` from releases, and put `mp3_normalizer` file into `/usr/local/bin`.  

Usage examples:

```text
mp3_normalizer -i <INPUT_TRACKS_OR_FOLDER_PATHS>
```
```text
mp3_normalizer -i <INPUT_TRACKS_OR_FOLDER_PATHS> -o <OUTPUT_FOLDER_PATH> -nt 4 -a
```
```text
mp3_normalizer -i music_folder1 music_folder2 input.mp3 -o music_output_folder -nt 4 -a
```

## Features
+   Normalize on preffered loudness
+   Based on ReplayGain standard for normalization
+   Multithreaded support for increased speed
+   Additional command line argument support to further help user specify their use case

### Command line featuers
+   `-i` Specify input folder
+   `-o` Specify output folder
+   `-nt` Number of threads
+   `-l` Set loudness
+   `-st` Set track replay gain
+   `-sa` Set album replay gain
+   `-r` Remove tags
+   `-a` Album replay gain
+   `-ad` Directory replay gain
+   `-show` Open output directory
+   `-sd` Shut down PC after finishing
+   `-hi` Hibernate
+   `-sl` Sleep after finishing

## Used Techonologies

Whole application is written and developed in Rust

### Rust dependencies and their versions
+   walkdir = "2"
+   system_shutdown = "4.0.1"
+   id3 = "1.7.0"
+   minimp3 = "0.5.1"
+   rayon = "1.7.0"
