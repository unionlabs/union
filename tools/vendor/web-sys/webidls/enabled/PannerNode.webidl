/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

enum PanningModelType {
  "equalpower",
  "HRTF"
};

enum DistanceModelType {
  "linear",
  "inverse",
  "exponential"
};

dictionary PannerOptions : AudioNodeOptions {
             PanningModelType  panningModel = "equalpower";
             DistanceModelType distanceModel = "inverse";
             float             positionX = 0;
             float             positionY = 0;
             float             positionZ = 0;
             float             orientationX = 1;
             float             orientationY = 0;
             float             orientationZ = 0;
             double            refDistance = 1;
             double            maxDistance = 10000;
             double            rolloffFactor = 1;
             double            coneInnerAngle = 360;
             double            coneOuterAngle = 360;
             double            coneOuterGain = 0;
};

[Pref="dom.webaudio.enabled",
 Constructor(BaseAudioContext context, optional PannerOptions options)]
interface PannerNode : AudioNode {

    // Default for stereo is equalpower
    attribute PanningModelType panningModel;

    // Uses a 3D cartesian coordinate system
    undefined setPosition(double x, double y, double z);
    undefined setOrientation(double x, double y, double z);
    [Deprecated="PannerNodeDoppler"]
    undefined setVelocity(double x, double y, double z);

    // Cartesian coordinate for position
    readonly attribute AudioParam positionX;
    readonly attribute AudioParam positionY;
    readonly attribute AudioParam positionZ;

    // Cartesian coordinate for orientation
    readonly attribute AudioParam orientationX;
    readonly attribute AudioParam orientationY;
    readonly attribute AudioParam orientationZ;

    // Distance model and attributes
    attribute DistanceModelType distanceModel;
    attribute double refDistance;
    attribute double maxDistance;
    attribute double rolloffFactor;

    // Directional sound cone
    attribute double coneInnerAngle;
    attribute double coneOuterAngle;
    attribute double coneOuterGain;

};
