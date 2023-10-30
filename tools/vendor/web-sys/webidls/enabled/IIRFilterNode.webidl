/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is https://www.w3.org/TR/webaudio
 *
 * Copyright © 2016 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

dictionary IIRFilterOptions : AudioNodeOptions {
    required sequence<double> feedforward;
    required sequence<double> feedback;
};

[Pref="dom.webaudio.enabled",
Constructor(BaseAudioContext context, IIRFilterOptions options)]
interface IIRFilterNode : AudioNode {
    undefined getFrequencyResponse(Float32Array frequencyHz, Float32Array magResponse, Float32Array phaseResponse);
};
