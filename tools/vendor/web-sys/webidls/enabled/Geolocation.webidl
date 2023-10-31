/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/TR/geolocation-API
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

dictionary PositionOptions {
  boolean enableHighAccuracy = false;
  [Clamp] unsigned long timeout = 0x7fffffff;
  [Clamp] unsigned long maximumAge = 0;
};

[NoInterfaceObject]
interface Geolocation {
  [Throws, NeedsCallerType]
  undefined getCurrentPosition(PositionCallback successCallback,
                          optional PositionErrorCallback? errorCallback = null,
                          optional PositionOptions options);

  [Throws, NeedsCallerType]
  long watchPosition(PositionCallback successCallback,
                     optional PositionErrorCallback? errorCallback = null,
                     optional PositionOptions options);

  undefined clearWatch(long watchId);
};

callback PositionCallback = undefined (Position position);

callback PositionErrorCallback = undefined (PositionError positionError);
