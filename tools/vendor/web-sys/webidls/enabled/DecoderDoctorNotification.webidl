/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

enum DecoderDoctorNotificationType {
  "cannot-play",
  "platform-decoder-not-found",
  "can-play-but-some-missing-decoders",
  "cannot-initialize-pulseaudio",
  "unsupported-libavcodec",
  "decode-error",
  "decode-warning",
};

dictionary DecoderDoctorNotification {
  required DecoderDoctorNotificationType type;
  // True when the issue has been solved.
  required boolean isSolved;
  // Key from dom.properties, used for telemetry and prefs.
  required DOMString decoderDoctorReportId;
  // If provided, formats (or key systems) at issue.
  DOMString formats;
  // If provided, technical details about the decode-error/warning.
  DOMString decodeIssue;
  // If provided, URL of the document where the issue happened.
  DOMString docURL;
  // If provided, URL of the media resource that caused a decode-error/warning.
  DOMString resourceURL;
};
