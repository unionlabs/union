/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/screen-orientation
 *
 * Copyright © 2014 W3C® (MIT, ERCIM, Keio, Beihang), All Rights
 * Reserved. W3C liability, trademark and document use rules apply.
 */

enum OrientationType {
  "portrait-primary",
  "portrait-secondary",
  "landscape-primary",
  "landscape-secondary"
};

enum OrientationLockType {
  "any",
  "natural",
  "landscape",
  "portrait",
  "portrait-primary",
  "portrait-secondary",
  "landscape-primary",
  "landscape-secondary"
};

interface ScreenOrientation : EventTarget {
  [Throws]
  Promise<undefined> lock(OrientationLockType orientation);
  [Throws]
  undefined unlock();
  [Throws, NeedsCallerType]
  readonly attribute OrientationType type;
  [Throws, NeedsCallerType]
  readonly attribute unsigned short angle;
  attribute EventHandler onchange;
};
