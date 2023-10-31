/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://dvcs.w3.org/hg/dap/raw-file/default/media-stream-capture/MediaRecorder.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

enum RecordingState { "inactive", "recording", "paused" };

[Constructor(MediaStream stream, optional MediaRecorderOptions options),
 Constructor(AudioNode node, optional unsigned long output = 0,
             optional MediaRecorderOptions options)]
interface MediaRecorder : EventTarget {

  readonly attribute MediaStream stream;

  readonly attribute RecordingState state;

  readonly attribute DOMString mimeType;

  attribute EventHandler ondataavailable;

  attribute EventHandler onerror;

  attribute EventHandler onstart;

  attribute EventHandler onstop;

  attribute EventHandler onwarning;

  [Throws]
  undefined start(optional long timeSlice);

  [Throws]
  undefined stop();

  [Throws]
  undefined pause();

  [Throws]
  undefined resume();

  [Throws]
  undefined requestData();

  static boolean isTypeSupported(DOMString type);
};

dictionary MediaRecorderOptions {
  DOMString mimeType = ""; // Default encoding mimeType.
  unsigned long audioBitsPerSecond;
  unsigned long videoBitsPerSecond;
  unsigned long bitsPerSecond;
};
