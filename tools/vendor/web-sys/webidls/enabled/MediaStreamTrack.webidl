/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/2011/webrtc/editor/getusermedia.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

// These two enums are in the spec even though they're not used directly in the
// API due to https://www.w3.org/Bugs/Public/show_bug.cgi?id=19936
// Their binding code is used in the implementation.

enum VideoFacingModeEnum {
    "user",
    "environment",
    "left",
    "right"
};

enum MediaSourceEnum {
    "camera",
    "screen",
    "application",
    "window",
    "browser",
    "microphone",
    "audioCapture",
    "other"
    // If values are added, adjust n_values in Histograms.json (2 places)
};

typedef (long or ConstrainLongRange) ConstrainLong;
typedef (double or ConstrainDoubleRange) ConstrainDouble;
typedef (boolean or ConstrainBooleanParameters) ConstrainBoolean;
typedef (DOMString or sequence<DOMString> or ConstrainDOMStringParameters) ConstrainDOMString;

// Note: When adding new constraints, remember to update the SelectSettings()
// function in MediaManager.cpp to make OverconstrainedError's constraint work!

dictionary MediaTrackConstraintSet {
    ConstrainLong width;
    ConstrainLong height;
    ConstrainDouble frameRate;
    ConstrainDOMString facingMode;
    DOMString mediaSource = "camera";
    long long browserWindow;
    boolean scrollWithPage;
    ConstrainDOMString deviceId;
    ConstrainLong viewportOffsetX;
    ConstrainLong viewportOffsetY;
    ConstrainLong viewportWidth;
    ConstrainLong viewportHeight;
    ConstrainBoolean echoCancellation;
    ConstrainBoolean noiseSuppression;
    ConstrainBoolean autoGainControl;
    ConstrainLong channelCount;
};

dictionary MediaTrackConstraints : MediaTrackConstraintSet {
    sequence<MediaTrackConstraintSet> advanced;
};

enum MediaStreamTrackState {
    "live",
    "ended"
};

[Exposed=Window]
interface MediaStreamTrack : EventTarget {
    readonly    attribute DOMString             kind;
    readonly    attribute DOMString             id;
    [NeedsCallerType]
    readonly    attribute DOMString             label;
                attribute boolean               enabled;
    readonly    attribute boolean               muted;
                attribute EventHandler          onmute;
                attribute EventHandler          onunmute;
    readonly    attribute MediaStreamTrackState readyState;
                attribute EventHandler          onended;
    MediaStreamTrack       clone ();
    undefined                   stop ();
//  MediaTrackCapabilities getCapabilities ();
    MediaTrackConstraints  getConstraints ();
    [NeedsCallerType]
    MediaTrackSettings     getSettings ();

    [Throws, NeedsCallerType]
    Promise<undefined>          applyConstraints (optional MediaTrackConstraints constraints);
//              attribute EventHandler          onoverconstrained;

    [ChromeOnly]
    undefined mutedChanged(boolean muted);
};
