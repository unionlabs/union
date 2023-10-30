/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

// For Javascript markers, the |stack| of a ProfileTimelineMarker
// holds an object of this type.  It intentionally looks like a
// SavedStack object and is a representation of the frame that is
// about to be constructed at the entry point.
dictionary ProfileTimelineStackFrame {
  long line;
  long column = 0;
  DOMString source;
  DOMString functionDisplayName;
  object? parent = null;
  object? asyncParent = null;
  DOMString asyncCause;
};

dictionary ProfileTimelineLayerRect {
  long x = 0;
  long y = 0;
  long width = 0;
  long height = 0;
};

enum ProfileTimelineMessagePortOperationType {
  "serializeData",
  "deserializeData",
};

enum ProfileTimelineWorkerOperationType {
  "serializeDataOffMainThread",
  "serializeDataOnMainThread",
  "deserializeDataOffMainThread",
  "deserializeDataOnMainThread",
};

dictionary ProfileTimelineMarker {
  DOMString name = "";
  DOMHighResTimeStamp start = 0;
  DOMHighResTimeStamp end = 0;
  object? stack = null;

  unsigned short processType;
  boolean isOffMainThread;

  /* For ConsoleTime, Timestamp and Javascript markers.  */
  DOMString causeName;

  /* For ConsoleTime markers.  */
  object? endStack = null;

  /* For DOMEvent markers.  */
  DOMString type;
  unsigned short eventPhase;

  /* For document::DOMContentLoaded and document::Load markers. Using this
   * instead of the `start` and `end` timestamps is strongly discouraged. */
  unsigned long long unixTime; // in microseconds

  /* For Paint markers.  */
  sequence<ProfileTimelineLayerRect> rectangles;

  /* For Style markers. */
  boolean isAnimationOnly;

  /* For MessagePort markers. */
  ProfileTimelineMessagePortOperationType messagePortOperation;

  /* For Worker markers. */
  ProfileTimelineWorkerOperationType workerOperation;
};
