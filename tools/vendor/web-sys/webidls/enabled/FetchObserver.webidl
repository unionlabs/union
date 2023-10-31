/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

callback interface ObserverCallback {
  undefined handleEvent(FetchObserver observer);
};

enum FetchState {
  // Pending states
  "requesting", "responding",
  // Final states
  "aborted", "errored", "complete"
};

[Exposed=(Window,Worker),
 Func="mozilla::dom::DOMPrefs::FetchObserverEnabled"]
interface FetchObserver : EventTarget {
  readonly attribute FetchState state;

  // Events
  attribute EventHandler onstatechange;
  attribute EventHandler onrequestprogress;
  attribute EventHandler onresponseprogress;
};
