/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this file,
* You can obtain one at http://mozilla.org/MPL/2.0/.
*
* The origin of this IDL file is
* https://w3c.github.io/push-api/
*/

// invalid widl
// interface Principal;

enum PushEncryptionKeyName
{
  "p256dh",
  "auth"
};

dictionary PushSubscriptionKeys
{
  ByteString p256dh;
  ByteString auth;
};

dictionary PushSubscriptionJSON
{
  USVString endpoint;
  PushSubscriptionKeys keys;
};

dictionary PushSubscriptionInit
{
  required USVString endpoint;
  required USVString scope;
  ArrayBuffer? p256dhKey;
  ArrayBuffer? authSecret;
  BufferSource? appServerKey;
};

[Exposed=(Window,Worker), Func="mozilla::dom::DOMPrefs::PushEnabled",
 ChromeConstructor(PushSubscriptionInit initDict)]
interface PushSubscription
{
  readonly attribute USVString endpoint;
  readonly attribute PushSubscriptionOptions options;
  [Throws]
  ArrayBuffer? getKey(PushEncryptionKeyName name);
  [Throws, UseCounter]
  Promise<boolean> unsubscribe();

  // Implements the custom serializer specified in Push API, section 9.
  [Throws]
  PushSubscriptionJSON toJSON();
};
