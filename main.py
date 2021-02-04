import argparse
import numpy as np
import cv2

FRAMERATE = 30
AVERAGE_AMOUNT = 30
TOL = 10

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
        print(frame.shape)

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
                    lambda x: x[0] >= TOL and x[0] < frame.shape[1] - TOL and x[1] >= TOL and x[1] < frame.shape[0] - TOL,
                    hull.squeeze(1)
                )
                furthest = max(valid_points, key=lambda x: (cx - x[0]) ** 2 + (cy - x[1]) ** 2, default=None)
                if furthest is not None:
                    print(furthest)
                    cv2.circle(frame, (furthest[0], furthest[1]), 4, (255, 255, 0), -2)

        cv2.imshow("Live Feed", frame)
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
        contour = max(contours, key=lambda x: cv2.contourArea(x))
        #cv2.drawContours(frame, [contours], 0, (0, 150, 255), 2)
    else:
        contour = None

    return contour

if __name__ == "__main__":
    main()