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

dictionary AudioBufferOptions {
             unsigned long numberOfChannels = 1;
    required unsigned long length;
    required float         sampleRate;
};

[Pref="dom.webaudio.enabled",
 Constructor(AudioBufferOptions options)]
interface AudioBuffer {

    readonly attribute float sampleRate;
    readonly attribute unsigned long length;

    // in seconds 
    readonly attribute double duration;

    readonly attribute unsigned long numberOfChannels;

    [Throws]
    Float32Array getChannelData(unsigned long channel);

    [Throws]
    undefined copyFromChannel(Float32Array destination, long channelNumber, optional unsigned long startInChannel = 0);
    [Throws]
    undefined copyToChannel(Float32Array source, long channelNumber, optional unsigned long startInChannel = 0);
};
