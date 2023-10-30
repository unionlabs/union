/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/html5/webvtt/#the-vttcue-interface
 */

enum AutoKeyword { "auto" };

enum LineAlignSetting {
  "start",
  "center",
  "end"
};

enum PositionAlignSetting {
  "line-left",
  "center",
  "line-right",
  "auto"
};

enum AlignSetting {
  "start",
  "center",
  "end",
  "left",
  "right"
};

enum DirectionSetting {
  "",
  "rl",
  "lr"
};

[Constructor(double startTime, double endTime, DOMString text)]
interface VTTCue : TextTrackCue {
  [Pref="media.webvtt.regions.enabled"]
  attribute VTTRegion? region;
  attribute DirectionSetting vertical;
  attribute boolean snapToLines;
  attribute (double or AutoKeyword) line;
  [SetterThrows]
  attribute LineAlignSetting lineAlign;
  [SetterThrows]
  attribute (double or AutoKeyword) position;
  [SetterThrows]
  attribute PositionAlignSetting positionAlign;
  [SetterThrows]
  attribute double size;
  attribute AlignSetting align;
  attribute DOMString text;
  DocumentFragment getCueAsHTML();
};
