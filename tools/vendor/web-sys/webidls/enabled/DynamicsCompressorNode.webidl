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

dictionary DynamicsCompressorOptions : AudioNodeOptions {
             float attack = 0.003;
             float knee = 30;
             float ratio = 12;
             float release = 0.25;
             float threshold = -24;
};

[Pref="dom.webaudio.enabled",
 Constructor(BaseAudioContext context, optional DynamicsCompressorOptions options)]
interface DynamicsCompressorNode : AudioNode {

    readonly attribute AudioParam threshold; // in Decibels
    readonly attribute AudioParam knee; // in Decibels
    readonly attribute AudioParam ratio; // unit-less
    readonly attribute float reduction; // in Decibels
    readonly attribute AudioParam attack; // in Seconds
    readonly attribute AudioParam release; // in Seconds

};
