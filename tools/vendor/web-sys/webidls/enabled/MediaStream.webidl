/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origins of this IDL file are
 * http://dev.w3.org/2011/webrtc/editor/getusermedia.html
 *
 * Copyright � 2012 W3C� (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

// These dictionaries need to be in a separate file from their
// MediaTrackConstraints* counterparts due to a webidl compiler limitation.

dictionary MediaStreamConstraints {
    (boolean or MediaTrackConstraints) audio = false;
    (boolean or MediaTrackConstraints) video = false;
    boolean picture = false; // Mozilla legacy
    boolean fake;       // For testing purpose. Generates frames of solid
                        // colors if video is enabled, and sound of 1Khz sine
                        // wave if audio is enabled.
    DOMString? peerIdentity = null;
};

dictionary DisplayMediaStreamConstraints {
    (boolean or MediaTrackConstraints) video = true;
    (boolean or MediaTrackConstraints) audio = false;
};

[Exposed=Window,
 Constructor,
 Constructor (MediaStream stream),
 Constructor (sequence<MediaStreamTrack> tracks)]
interface MediaStream : EventTarget {
    readonly    attribute DOMString    id;
    sequence<AudioStreamTrack> getAudioTracks ();
    sequence<VideoStreamTrack> getVideoTracks ();
    sequence<MediaStreamTrack> getTracks ();
    MediaStreamTrack?          getTrackById (DOMString trackId);
    undefined                       addTrack (MediaStreamTrack track);
    undefined                       removeTrack (MediaStreamTrack track);
    MediaStream                clone ();
    readonly    attribute boolean      active;
                attribute EventHandler onaddtrack;
                attribute EventHandler onremovetrack;
    readonly attribute double currentTime;

    [ChromeOnly, Throws]
    static Promise<long> countUnderlyingStreams();

    // Webrtc allows the remote side to name a stream whatever it wants, and we
    // need to surface this to content.
    [ChromeOnly]
    undefined assignId(DOMString id);
};
