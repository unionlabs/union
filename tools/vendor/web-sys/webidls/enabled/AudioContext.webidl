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

dictionary AudioContextOptions {
             float        sampleRate = 0;
};

[Pref="dom.webaudio.enabled",
 Constructor(optional AudioContextOptions contextOptions)]
interface AudioContext : BaseAudioContext {

    // Bug 1324545: readonly        attribute double outputLatency;
    // Bug 1324545: AudioTimestamp                  getOutputTimestamp ();

    [Throws]
    Promise<undefined> suspend();
    [Throws]
    Promise<undefined> close();

    [NewObject, Throws]
    MediaElementAudioSourceNode createMediaElementSource(HTMLMediaElement mediaElement);

    [NewObject, Throws]
    MediaStreamAudioSourceNode createMediaStreamSource(MediaStream mediaStream);

    // Bug 1324548: MediaStreamTrackAudioSourceNode createMediaStreamTrackSource (AudioMediaStreamTrack mediaStreamTrack);

    [NewObject, Throws]
    MediaStreamAudioDestinationNode createMediaStreamDestination();
};

AudioContext includes rustBaseAudioContext;
