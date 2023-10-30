/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[Pref="dom.gamepad.non_standard_events.enabled",
 Constructor(DOMString type, optional GamepadButtonEventInit eventInitDict)]
interface GamepadButtonEvent : GamepadEvent
{
  readonly attribute unsigned long button;
};

dictionary GamepadButtonEventInit : GamepadEventInit
{
  unsigned long button = 0;
};
