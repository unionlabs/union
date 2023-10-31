/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dvcs.w3.org/hg/webevents/raw-file/default/touchevents.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

dictionary TouchInit {
  required long identifier;
  required EventTarget target;
  long clientX = 0;
  long clientY = 0;
  long screenX = 0;
  long screenY = 0;
  long pageX = 0;
  long pageY = 0;
  float radiusX = 0;
  float radiusY = 0;
  float rotationAngle = 0;
  float force = 0;
};

[Constructor(TouchInit touchInitDict), 
 Func="mozilla::dom::Touch::PrefEnabled"]
interface Touch {
  readonly    attribute long         identifier;
  readonly    attribute EventTarget? target;
  [NeedsCallerType]
  readonly    attribute long         screenX;
  [NeedsCallerType]
  readonly    attribute long         screenY;
  readonly    attribute long         clientX;
  readonly    attribute long         clientY;
  readonly    attribute long         pageX;
  readonly    attribute long         pageY;
  [NeedsCallerType]
  readonly    attribute long         radiusX;
  [NeedsCallerType]
  readonly    attribute long         radiusY;
  [NeedsCallerType]
  readonly    attribute float        rotationAngle;
  [NeedsCallerType]
  readonly    attribute float        force;
};
