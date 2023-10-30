/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#media-elements
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

interface HTMLMediaElement : HTMLElement {

  // error state
  readonly attribute MediaError? error;

  // network state
  [CEReactions, SetterNeedsSubjectPrincipal=NonSystem, SetterThrows]
           attribute DOMString src;
  readonly attribute DOMString currentSrc;

  attribute MediaStream? srcObject;

  [CEReactions, SetterThrows]
           attribute DOMString? crossOrigin;
  const unsigned short NETWORK_EMPTY = 0;
  const unsigned short NETWORK_IDLE = 1;
  const unsigned short NETWORK_LOADING = 2;
  const unsigned short NETWORK_NO_SOURCE = 3;
  readonly attribute unsigned short networkState;
  [CEReactions, SetterThrows]
           attribute DOMString preload;
  [NewObject]
  readonly attribute TimeRanges buffered;
  undefined load();
  DOMString canPlayType(DOMString type);

  // ready state
  const unsigned short HAVE_NOTHING = 0;
  const unsigned short HAVE_METADATA = 1;
  const unsigned short HAVE_CURRENT_DATA = 2;
  const unsigned short HAVE_FUTURE_DATA = 3;
  const unsigned short HAVE_ENOUGH_DATA = 4;
  readonly attribute unsigned short readyState;
  readonly attribute boolean seeking;

  // playback state
  [SetterThrows]
           attribute double currentTime;
  [Throws]
  undefined fastSeek(double time);
  readonly attribute unrestricted double duration;
  [ChromeOnly]
  readonly attribute boolean isEncrypted;
  // TODO: Bug 847376 - readonly attribute any startDate;
  readonly attribute boolean paused;
  [SetterThrows]
           attribute double defaultPlaybackRate;
  [SetterThrows]
           attribute double playbackRate;
  [NewObject]
  readonly attribute TimeRanges played;
  [NewObject]
  readonly attribute TimeRanges seekable;
  readonly attribute boolean ended;
  [CEReactions, SetterThrows]
           attribute boolean autoplay;
  [CEReactions, SetterThrows]
           attribute boolean loop;
  [Throws]
  Promise<undefined> play();
  [Throws]
  undefined pause();

  // TODO: Bug 847377 - mediaGroup and MediaController
  // media controller
  //         attribute DOMString mediaGroup;
  //         attribute MediaController? controller;

  // controls
  [CEReactions, SetterThrows]
           attribute boolean controls;
  [SetterThrows]
           attribute double volume;
           attribute boolean muted;
  [CEReactions, SetterThrows]
           attribute boolean defaultMuted;

  // TODO: Bug 847379
  // tracks
  [Pref="media.track.enabled"]
  readonly attribute AudioTrackList audioTracks;
  [Pref="media.track.enabled"]
  readonly attribute VideoTrackList videoTracks;
  readonly attribute TextTrackList? textTracks;
  TextTrack addTextTrack(TextTrackKind kind,
                         optional DOMString label = "",
                         optional DOMString language = "");
};

// Encrypted Media Extensions
partial interface HTMLMediaElement {
  readonly attribute MediaKeys? mediaKeys;

  // undefined, not any: https://www.w3.org/Bugs/Public/show_bug.cgi?id=26457
  [NewObject]
  Promise<undefined> setMediaKeys(MediaKeys? mediaKeys);

  attribute EventHandler onencrypted;

  attribute EventHandler onwaitingforkey;
};

/*
 * HTMLMediaElement::seekToNextFrame() is a Mozilla experimental feature.
 *
 * The SeekToNextFrame() method provides a way to access a video element's video
 * frames one by one without going through the realtime playback. So, it lets
 * authors use "frame" as unit to access the video element's underlying data,
 * instead of "time".
 *
 * The SeekToNextFrame() is a kind of seek operation, so normally, once it is
 * invoked, a "seeking" event is dispatched. However, if the media source has no
 * video data or is not seekable, the operation is ignored without filing the
 * "seeking" event.
 *
 * Once the SeekToNextFrame() is done, a "seeked" event should always be filed
 * and a "ended" event might also be filed depends on where the media element's
 * position before seeking was. There are two cases:
 * Assume the media source has n+1 video frames where n is a non-negative
 * integers and the frame sequence is indexed from zero.
 * (1) If the currentTime is at anywhere smaller than the n-th frame's beginning
 *     time, say the currentTime is now pointing to a position which is smaller
 *     than the x-th frame's beginning time and larger or equal to the (x-1)-th
 *     frame's beginning time, where x belongs to [1, n], then the
 *     SeekToNextFrame() operation seeks the media to the x-th frame, sets the
 *     media's currentTime to the x-th frame's beginning time and dispatches a
 *     "seeked" event.
 * (2) Otherwise, if the currentTime is larger or equal to the n-th frame's
 *     beginning time, then the SeekToNextFrame() operation sets the media's
 *     currentTime to the duration of the media source and dispatches a "seeked"
 *     event and an "ended" event.
 */
partial interface HTMLMediaElement {
  [Throws, Pref="media.seekToNextFrame.enabled"]
  Promise<undefined> seekToNextFrame();
};

/*
 * This is an API for simulating visibility changes to help debug and write
 * tests about suspend-video-decoding.
 *
 * - SetVisible() is for simulating visibility changes.
 * - HasSuspendTaint() is for querying that the element's decoder cannot suspend
 *   video decoding because it has been tainted by an operation, such as
 *   drawImage().
 */
partial interface HTMLMediaElement {
  [Pref="media.test.video-suspend"]
  undefined setVisible(boolean aVisible);

  [Pref="media.test.video-suspend"]
  boolean hasSuspendTaint();
};
