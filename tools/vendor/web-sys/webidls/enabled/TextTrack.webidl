/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#texttrack
 */

enum TextTrackKind {
  "subtitles",
  "captions",
  "descriptions",
  "chapters",
  "metadata"
};

enum TextTrackMode {
  "disabled",
  "hidden",
  "showing"
};

interface TextTrack : EventTarget {
  readonly attribute TextTrackKind kind;
  readonly attribute DOMString label;
  readonly attribute DOMString language;

  readonly attribute DOMString id;
  readonly attribute DOMString inBandMetadataTrackDispatchType;

           attribute TextTrackMode mode;

  readonly attribute TextTrackCueList? cues;
  readonly attribute TextTrackCueList? activeCues;

  undefined addCue(VTTCue cue);
  [Throws]
  undefined removeCue(VTTCue cue);

           attribute EventHandler oncuechange;
};

partial interface TextTrack {
    readonly attribute SourceBuffer? sourceBuffer;
};
