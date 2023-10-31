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

enum BiquadFilterType {
  "lowpass",
  "highpass",
  "bandpass",
  "lowshelf",
  "highshelf",
  "peaking",
  "notch",
  "allpass"
};

dictionary BiquadFilterOptions : AudioNodeOptions {
             BiquadFilterType type = "lowpass";
             float            Q = 1;
             float            detune = 0;
             float            frequency = 350;
             float            gain = 0;
};

[Pref="dom.webaudio.enabled",
 Constructor(BaseAudioContext context, optional BiquadFilterOptions options)]
interface BiquadFilterNode : AudioNode {

    attribute BiquadFilterType type;
    readonly attribute AudioParam frequency; // in Hertz
    readonly attribute AudioParam detune; // in Cents
    readonly attribute AudioParam Q; // Quality factor
    readonly attribute AudioParam gain; // in Decibels

    undefined getFrequencyResponse(Float32Array frequencyHz,
                              Float32Array magResponse,
                              Float32Array phaseResponse);

};
