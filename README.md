# Finger Tracker

The purpose of this project is to track the location of a finger using webcam input and opencv-2, and then run a flock to track that.


## Setup

### Python Finger Tracker

All files are inside `tracker/`.

Please ensure that you have python 3 installed, with the opencv `cv2` library available. I reccommend installing a precompiled package using `pip install -U opencv-python` unless you already have it for other uses.

Other dependancies can be install with `pip install -r requirements.txt`.

### Rust Flock

All files are inside `flock/`.

Ensure that you have rust installed, then compile with `cargo build --release`. The executable can then be run with `cargo run --release`, or from `target/release/flock`.

The executable assumes that python can be run from the command `python3`. It takes a mandatory argument of the path to the trackers main.py file. There is an additional optional argument for the number of boids to spawn, however this defaults to 150.

```
USAGE:
    flock [OPTIONS] <path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --count <count>    Number of boids to spawn [default: 150]

ARGS:
    <path>    Path to the python file
```

## Usage

Run with `python3 main.py`.

There are additional arguments that can be listed with `python3 main.py --help`.

```
usage: main.py [-h] [-c CAMERAID] [-t TOLERANCE]

optional arguments:
  -h, --help            show this help message and exit
  -c CAMERAID, --cameraid CAMERAID
                        Camera ID to use
  -t TOLERANCE, --tolerance TOLERANCE
                        Border around edges of camera to not search for pixles
```
