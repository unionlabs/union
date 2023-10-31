/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/gamepad/extensions.html#gamepadhapticactuator-interface
 */

enum GamepadHapticActuatorType {
  "vibration"
};

[Pref="dom.gamepad.extensions.enabled",
  HeaderFile="mozilla/dom/GamepadHapticActuator.h"]
interface GamepadHapticActuator
{
  readonly attribute GamepadHapticActuatorType type;
  [Throws, NewObject]
  Promise<boolean> pulse(double value, double duration);
};
