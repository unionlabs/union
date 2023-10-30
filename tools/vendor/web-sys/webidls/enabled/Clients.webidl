/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://slightlyoff.github.io/ServiceWorker/spec/service_worker/index.html
 *
 */

[Exposed=ServiceWorker]
interface Clients {
  // The objects returned will be new instances every time
  [NewObject]
  Promise<any> get(DOMString id);
  [NewObject]
  Promise<sequence<Client>> matchAll(optional ClientQueryOptions options);
  [NewObject]
  Promise<WindowClient?> openWindow(USVString url);
  [NewObject]
  Promise<undefined> claim();
};

dictionary ClientQueryOptions {
  boolean includeUncontrolled = false;
  ClientType type = "window";
};

enum ClientType {
  "window",
  "worker",
  "sharedworker",
  // https://github.com/w3c/ServiceWorker/issues/1036
  "serviceworker",
  "all"
};

