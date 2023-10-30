/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * This is an internal IDL file
 */

// for gUM request start (getUserMedia:request) notification,
// rawID and mediaSource won't be set.
// for gUM request stop (recording-device-stopped) notification due to page reload,
// only windowID will be set.
// for gUM request stop (recording-device-stopped) notification due to track stop,
// only windowID, rawID and mediaSource will be set

[NoInterfaceObject]
interface GetUserMediaRequest {
  readonly attribute unsigned long long windowID;
  readonly attribute unsigned long long innerWindowID;
  readonly attribute DOMString callID;
  readonly attribute DOMString rawID;
  readonly attribute DOMString mediaSource;
  MediaStreamConstraints getConstraints();
  readonly attribute boolean isSecure;
  readonly attribute boolean isHandlingUserInput;
};
