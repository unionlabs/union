/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

enum DOMRequestReadyState { "pending", "done" };

[Exposed=(Window,Worker,System)]
interface mixin DOMRequestShared {
  readonly attribute DOMRequestReadyState readyState;

  readonly attribute any result;
  readonly attribute DOMException? error;

  attribute EventHandler onsuccess;
  attribute EventHandler onerror;
};

[Exposed=(Window,Worker,System)]
interface DOMRequest : EventTarget {
  // The [TreatNonCallableAsNull] annotation is required since then() should do
  // nothing instead of throwing errors when non-callable arguments are passed.
  // See documentation for Promise.then to see why we return "any".
  [NewObject, Throws]
  any then([TreatNonCallableAsNull] optional AnyCallback? fulfillCallback = null,
           [TreatNonCallableAsNull] optional AnyCallback? rejectCallback = null);

  [ChromeOnly]
  undefined fireDetailedError(DOMException aError);
};

DOMRequest includes DOMRequestShared;
