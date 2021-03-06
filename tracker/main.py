import argparse
import numpy as np
import cv2
import sys

FRAMERATE = 30
AVERAGE_AMOUNT = 30
TOL = 10

def main():
    # Will flesh out arguments as I find the need for them
    parser = argparse.ArgumentParser()

    parser.add_argument("-c", "--cameraid", help="Camera ID to use", type=int, default=0)
    parser.add_argument("-t", "--tolerance", help="Border around edges of camera to not search for pixles", type=int, default=10)

    args = parser.parse_args()

    return cameraLoop(args.cameraid, args.tolerance)
    

def cameraLoop(cam_id, tolerance):
    try:
        cap, backSub, cam_res = setupVideoProcessing(cam_id)
    except DisconnectedException as e:
        print(e.message, file=sys.stderr)
        return 1

    while True:
        try:
            result = processNextFrame(cap, backSub, tolerance)
        except DisconnectedException as e:
            print(e.message, file=sys.stderr)
            break
        if result is None:
            # No finger
            output = "None"
        else:
            # Finger Detected
            x = result[0] / cam_res[0]
            y = result[1] / cam_res[1]
            output = f"{x:.4f},{y:.4f}"

        print(output, flush=True)
    
    # Cleanup
    cv2.destroyAllWindows()
    cap.release()

    return 0


def processNextFrame(cap, backSub, tol):
    if not cap.isOpened():
        raise DisconnectedException("Camera is not connected")
    
    returnValue = None

    _, frame = cap.read()

    contour = processFrameThreshhold(frame, backSub)
    
    if contour is not None:
        hull = cv2.convexHull(contour)
        moment = cv2.moments(hull)
        if moment['m00'] != 0:
            # Getting central point
            cx = int(moment['m10'] / moment['m00'])
            cy = int(moment['m01'] / moment['m00'])
            #cv2.circle(frame, (cx, cy), 4, (255, 0, 0), -2)
            # Then getting point further from the center
            valid_points = filter(
                lambda x: x[0] >= tol and x[0] < frame.shape[1] - tol and x[1] >= tol and x[1] < frame.shape[0] - tol,
                hull.squeeze(1)
            )
            furthest = max(valid_points, key=lambda x: (cx - x[0]) ** 2 + (cy - x[1]) ** 2, default=None)
            if furthest is not None:
                returnValue = (furthest[0], furthest[1])
                cv2.circle(frame, returnValue, 4, (255, 255, 0), -2)

    cv2.imshow("Live Feed", frame)
    key = cv2.waitKey(int(1000 / FRAMERATE))
    if key == 27:
        raise DisconnectedException("Program Closed by User Input")

    # returnValue will be None if no finger detected, or (x, y) coordinates of where the finger is pointing
    return returnValue


def processFrameThreshhold(frame, backSub):
    #thresh = backSub.apply(frame, 0.1)
    thresh = backSub.apply(frame)

    contours, _ = cv2.findContours(thresh, cv2.RETR_TREE, cv2.CHAIN_APPROX_SIMPLE)
    if len(contours) > 0:
        contour = max(contours, key=lambda x: cv2.contourArea(x))
        #cv2.drawContours(frame, [contours], 0, (0, 150, 255), 2)
    else:
        contour = None

    return contour

def setupVideoProcessing(cam_id):
    #print("Initializing Video Capture")
    cap = cv2.VideoCapture(cam_id)
    
    backSub = cv2.createBackgroundSubtractorKNN()

    if cap.isOpened():
        _, frame = cap.read()
        cam_res = (frame.shape[1], frame.shape[0])
        #print(f"Video has resolution of {cam_res[0]} x {cam_res[1]}")
        return (cap, backSub, cam_res)
    else:
        raise DisconnectedException("Unable to establish connection")

# An exception for when the camera is diconnected or the script otherwise can't run
class DisconnectedException(Exception):
    def __init__(self, msg):
        self.message = msg

if __name__ == "__main__":
    main()