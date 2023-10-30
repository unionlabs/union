/* -*- Mode: IDL; tab-width: 1; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://fetch.spec.whatwg.org/#request-class
 */

typedef (Request or USVString) RequestInfo;
typedef unsigned long nsContentPolicyType;

[Constructor(RequestInfo input, optional RequestInit init),
 Exposed=(Window,Worker)]
interface Request {
  readonly attribute ByteString method;
  readonly attribute USVString url;
  [SameObject] readonly attribute Headers headers;

  readonly attribute RequestDestination destination;
  readonly attribute USVString referrer;
  readonly attribute ReferrerPolicy referrerPolicy;
  readonly attribute RequestMode mode;
  readonly attribute RequestCredentials credentials;
  readonly attribute RequestCache cache;
  readonly attribute RequestRedirect redirect;
  readonly attribute DOMString integrity;


  [BinaryName="getOrCreateSignal"]
  readonly attribute AbortSignal signal;

  [Throws,
   NewObject] Request clone();

  // Bug 1124638 - Allow chrome callers to set the context.
  [ChromeOnly]
  undefined overrideContentPolicyType(nsContentPolicyType context);
};
Request includes Body;

dictionary RequestInit {
  ByteString method;
  HeadersInit headers;
  BodyInit? body;
  USVString referrer;
  ReferrerPolicy referrerPolicy;
  RequestMode mode;
  RequestCredentials credentials;
  RequestCache cache;
  RequestRedirect redirect;
  DOMString integrity;

  AbortSignal? signal;

  [Func="mozilla::dom::DOMPrefs::FetchObserverEnabled"]
  ObserverCallback observe;
};

enum RequestDestination {
  "",
  "audio", "audioworklet", "document", "embed", "font", "image", "manifest", "object",
  "paintworklet", "report", "script", "sharedworker", "style",  "track", "video",
  "worker", "xslt"
};

enum RequestMode { "same-origin", "no-cors", "cors", "navigate" };
enum RequestCredentials { "omit", "same-origin", "include" };
enum RequestCache { "default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached" };
enum RequestRedirect { "follow", "error", "manual" };
enum ReferrerPolicy {
  "", "no-referrer", "no-referrer-when-downgrade", "origin",
  "origin-when-cross-origin", "unsafe-url", "same-origin", "strict-origin",
  "strict-origin-when-cross-origin"
};
