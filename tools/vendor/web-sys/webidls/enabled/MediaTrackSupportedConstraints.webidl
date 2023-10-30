/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/2011/webrtc/editor/getusermedia.html
 */

dictionary MediaTrackSupportedConstraints {
    boolean width = true;
    boolean height = true;
    boolean aspectRatio;        // to be supported
    boolean frameRate = true;
    boolean facingMode = true;
    boolean volume;             // to be supported
    boolean sampleRate;         // to be supported
    boolean sampleSize;         // to be supported
    boolean echoCancellation = true;
    boolean noiseSuppression = true;
    boolean autoGainControl = true;
    boolean latency;            // to be supported
    boolean channelCount = true;
    boolean deviceId = true;
    boolean groupId;            // to be supported
};
