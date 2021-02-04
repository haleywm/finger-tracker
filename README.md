# Finger Tracker

The purpose of this project is to track the location of a finger using webcam input and opencv-2.


## Setup

Please ensure that you have python 3 installed, with the opencv `cv2` library available. I reccommend installing a precompiled package using `pip install -U opencv-python` unless you already have it for other uses.

Other dependancies can be install with `pip install -r requirements.txt`.

## Usage

Run with `python3 main.py width height pipelocation`, i.e. `python3 main.py 1280 760 out.pipe`. The pipe location argument takes relative or absolute arguments.

There are additional arguments that can be listed with `python3 main.py --help`.

```
positional arguments:
  width                 Output position width
  height                Output position height
  pipe                  Location to create a pipe for writing data

optional arguments:
  -h, --help            show this help message and exit
  -c CAMERAID, --cameraid CAMERAID
                        Camera ID to use
  -t TOLERANCE, --tolerance TOLERANCE
                        Border around edges of camera to not search for pixles
  -v, --verbose         If additional details should be printed to console
```
