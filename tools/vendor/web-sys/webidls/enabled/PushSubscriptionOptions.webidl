/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this file,
* You can obtain one at http://mozilla.org/MPL/2.0/.
*
* The origin of this IDL file is
* https://w3c.github.io/push-api/
*/

[Exposed=(Window,Worker), Func="mozilla::dom::DOMPrefs::PushEnabled"]
interface PushSubscriptionOptions
{
  [SameObject, Throws]
  readonly attribute ArrayBuffer? applicationServerKey;
};
