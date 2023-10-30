/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://html.spec.whatwg.org/multipage/workers.html
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and Opera
 * Software ASA.
 * You are granted a license to use, reproduce and create derivative works of
 * this document.
 */

[Constructor(USVString scriptURL, optional WorkerOptions options),
 Exposed=(Window,DedicatedWorker,SharedWorker,System)]
interface Worker : EventTarget {
  undefined terminate();

  [Throws]
  undefined postMessage(any message, optional sequence<object> transfer = []);

  attribute EventHandler onmessage;
  attribute EventHandler onmessageerror;
};

Worker includes AbstractWorker;

dictionary WorkerOptions {
  WorkerType type = "classic";
  RequestCredentials credentials = "omit"; // credentials is only used if type is "module"
  DOMString name = "";
};

enum WorkerType { "classic", "module" };

[Constructor(USVString scriptURL),
 Func="mozilla::dom::ChromeWorker::WorkerAvailable",
 Exposed=(Window,DedicatedWorker,SharedWorker,System)]
interface ChromeWorker : Worker {
};
