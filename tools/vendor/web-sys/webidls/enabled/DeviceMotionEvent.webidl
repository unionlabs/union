/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[NoInterfaceObject]
interface DeviceAcceleration {
  readonly attribute double? x;
  readonly attribute double? y;
  readonly attribute double? z;
};

[NoInterfaceObject]
interface DeviceRotationRate {
  readonly attribute double? alpha;
  readonly attribute double? beta;
  readonly attribute double? gamma;
};

[Pref="device.sensors.motion.enabled", Func="nsGlobalWindowInner::DeviceSensorsEnabled", Constructor(DOMString type, optional DeviceMotionEventInit eventInitDict)]
interface DeviceMotionEvent : Event {
  readonly attribute DeviceAcceleration? acceleration;
  readonly attribute DeviceAcceleration? accelerationIncludingGravity;
  readonly attribute DeviceRotationRate? rotationRate;
  readonly attribute double? interval;
};

dictionary DeviceAccelerationInit {
  double? x = null;
  double? y = null;
  double? z = null;
};

dictionary DeviceRotationRateInit {
  double? alpha = null;
  double? beta = null;
  double? gamma = null;
};

dictionary DeviceMotionEventInit : EventInit {
  DeviceAccelerationInit acceleration;
  DeviceAccelerationInit accelerationIncludingGravity;
  DeviceRotationRateInit rotationRate;
  double? interval = null;
};
