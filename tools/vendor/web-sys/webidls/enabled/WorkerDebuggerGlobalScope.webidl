/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

[Global=(WorkerDebugger), Exposed=WorkerDebugger]
interface WorkerDebuggerGlobalScope : EventTarget {
  [Throws]
  readonly attribute object global;

  [Throws]
  object createSandbox(DOMString name, object prototype);

  [Throws]
  undefined loadSubScript(DOMString url, optional object sandbox);

  undefined enterEventLoop();

  undefined leaveEventLoop();

  undefined postMessage(DOMString message);

  attribute EventHandler onmessage;

  [Throws]
  undefined setImmediate(Function handler);

  undefined reportError(DOMString message);

  [Throws]
  sequence<any> retrieveConsoleEvents();

  [Throws]
  undefined setConsoleEventHandler(AnyCallback? handler);
};

// So you can debug while you debug
partial interface WorkerDebuggerGlobalScope {
  undefined dump(optional DOMString string);
};
