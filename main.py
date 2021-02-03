import argparse
import numpy as np
import cv2

FRAMERATE = 30
AVERAGE_AMOUNT = 30


def main():
    # Will flesh out arguments as I find the need for them
    parser = argparse.ArgumentParser()

    parser.add_argument("-c", "--cameraid", help="Camera ID to use", type=int, default=0)

    args = parser.parse_args()

    print("Establishing capture")

    cap = cv2.VideoCapture(args.cameraid)
    wait_time = int(1000 / FRAMERATE)
    
    backSub = cv2.createBackgroundSubtractorKNN()

    while cap.isOpened():
        _, frame = cap.read()

        img = processFrameThreshhold(frame, backSub)

        cv2.imshow("Live Feed", img)
        key = cv2.waitKey(wait_time)
        if key == 27:
            break
    
    # Cleanup
    cv2.destroyAllWindows()
    cap.release()


def processFrameThreshhold(frame, backSub):
    thresh = backSub.apply(frame, 0.1)

    contours, _ = cv2.findContours(thresh, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)
    if len(contours) > 0:
        contours = max(contours, key=lambda x: cv2.contourArea(x))
        cv2.drawContours(frame, [contours], 0, (0, 150, 255), 2)

    return frame

if __name__ == "__main__":
    main()