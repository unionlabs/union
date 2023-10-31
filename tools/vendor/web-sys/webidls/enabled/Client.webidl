/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/ServiceWorker/#client-interface
 *
 */

[Exposed=ServiceWorker]
interface Client {
  readonly attribute USVString url;

  // Remove frameType in bug 1290936
  [BinaryName="GetFrameType"]
  readonly attribute FrameType frameType;

  readonly attribute ClientType type;
  readonly attribute DOMString id;

  // Implement reserved in bug 1264177
  // readonly attribute boolean reserved;

  [Throws]
  undefined postMessage(any message, optional sequence<object> transfer = []);
};

[Exposed=ServiceWorker]
interface WindowClient : Client {
  [BinaryName="GetVisibilityState"]
  readonly attribute VisibilityState visibilityState;
  readonly attribute boolean focused;

  // Implement ancestorOrigins in bug 1264180
  // [SameObject] readonly attribute FrozenArray<USVString> ancestorOrigins;

  [Throws, NewObject]
  Promise<WindowClient> focus();

  [Throws, NewObject]
  Promise<WindowClient> navigate(USVString url);
};

// Remove FrameType in bug 1290936
enum FrameType {
  "auxiliary",
  "top-level",
  "nested",
  "none"
};
