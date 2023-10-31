/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this file,
* You can obtain one at http://mozilla.org/MPL/2.0/.
*
* The origin of this IDL file is
* https://w3c.github.io/push-api/
*/

dictionary PushSubscriptionOptionsInit {
  boolean userVisibleOnly = false;
  (BufferSource or DOMString)? applicationServerKey = null;
};

// The main thread JS implementation. Please see comments in
// dom/push/PushManager.h for the split between PushManagerImpl and PushManager.
[JSImplementation="@mozilla.org/push/PushManager;1",
 ChromeOnly, Constructor(DOMString scope)]
interface PushManagerImpl {
  Promise<PushSubscription>    subscribe(optional PushSubscriptionOptionsInit options);
  Promise<PushSubscription?>   getSubscription();
  Promise<PushPermissionState> permissionState(optional PushSubscriptionOptionsInit options);
};

[Exposed=(Window,Worker), Func="mozilla::dom::DOMPrefs::PushEnabled",
 ChromeConstructor(DOMString scope)]
interface PushManager {
  [Throws, UseCounter]
  Promise<PushSubscription>    subscribe(optional PushSubscriptionOptionsInit options);
  [Throws]
  Promise<PushSubscription?>   getSubscription();
  [Throws]
  Promise<PushPermissionState> permissionState(optional PushSubscriptionOptionsInit options);
};

enum PushPermissionState
{
    "granted",
    "denied",
    "prompt"
};
