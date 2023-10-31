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

dictionary AudioBufferSourceOptions {
             AudioBuffer? buffer;
             float        detune = 0;
             boolean      loop = false;
             double       loopEnd = 0;
             double       loopStart = 0;
             float        playbackRate = 1;
};

[Pref="dom.webaudio.enabled",
 Constructor(BaseAudioContext context, optional AudioBufferSourceOptions options)]
interface AudioBufferSourceNode : AudioScheduledSourceNode {

    attribute AudioBuffer? buffer;

    readonly attribute AudioParam playbackRate;
    readonly attribute AudioParam detune;

    attribute boolean loop;
    attribute double loopStart;
    attribute double loopEnd;

    attribute EventHandler onended;

    [Throws]
    undefined start(optional double when = 0, optional double grainOffset = 0,
               optional double grainDuration);

    [Throws]
    undefined stop (optional double when = 0);
};
