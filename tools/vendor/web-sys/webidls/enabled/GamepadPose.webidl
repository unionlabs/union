/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/gamepad/extensions.html#gamepadpose-interface
 */

[Pref="dom.gamepad.extensions.enabled"]
interface GamepadPose
{
  readonly attribute boolean hasOrientation;
  readonly attribute boolean hasPosition;

  /**
   * position, linearVelocity, and linearAcceleration are 3-component vectors.
   * position is relative to a sitting space. Transforming this point with
   * VRStageParameters.sittingToStandingTransform converts this to standing space.
   */
  [Constant, Throws] readonly attribute Float32Array? position;
  [Constant, Throws] readonly attribute Float32Array? linearVelocity;
  [Constant, Throws] readonly attribute Float32Array? linearAcceleration;

  /* orientation is a 4-entry array representing the components of a quaternion. */
  [Constant, Throws] readonly attribute Float32Array? orientation;
  /* angularVelocity and angularAcceleration are the components of 3-dimensional vectors. */
  [Constant, Throws] readonly attribute Float32Array? angularVelocity;
  [Constant, Throws] readonly attribute Float32Array? angularAcceleration;
};
