/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://notifications.spec.whatwg.org/
 *
 * Copyright:
 * To the extent possible under law, the editors have waived all copyright and
 * related or neighboring rights to this work.
 */

[Constructor(DOMString title, optional NotificationOptions options),
 Exposed=(Window,Worker),
 Func="mozilla::dom::Notification::PrefEnabled"]
interface Notification : EventTarget {
  [GetterThrows]
  static readonly attribute NotificationPermission permission;

  [Throws, Func="mozilla::dom::Notification::RequestPermissionEnabledForScope"]
  static Promise<NotificationPermission> requestPermission(optional NotificationPermissionCallback permissionCallback);

  [Throws, Func="mozilla::dom::Notification::IsGetEnabled"]
  static Promise<sequence<Notification>> get(optional GetNotificationOptions filter);

  attribute EventHandler onclick;

  attribute EventHandler onshow;

  attribute EventHandler onerror;

  attribute EventHandler onclose;

  [Pure]
  readonly attribute DOMString title;

  [Pure]
  readonly attribute NotificationDirection dir;

  [Pure]
  readonly attribute DOMString? lang;

  [Pure]
  readonly attribute DOMString? body;

  [Constant]
  readonly attribute DOMString? tag;

  [Pure]
  readonly attribute DOMString? icon;

  [Constant, Func="mozilla::dom::DOMPrefs::NotificationRIEnabled"]
  readonly attribute boolean requireInteraction;

  [Constant]
  readonly attribute any data;

  undefined close();
};

dictionary NotificationOptions {
  NotificationDirection dir = "auto";
  DOMString lang = "";
  DOMString body = "";
  DOMString tag = "";
  DOMString icon = "";
  boolean requireInteraction = false;
  any data = null;
};

dictionary GetNotificationOptions {
  DOMString tag = "";
};

dictionary NotificationBehavior {
  boolean noscreen = false;
  boolean noclear = false;
  boolean showOnlyOnce = false;
  DOMString soundFile = "";
  sequence<unsigned long> vibrationPattern;
};

enum NotificationPermission {
  "default",
  "denied",
  "granted"
};

callback NotificationPermissionCallback = undefined (NotificationPermission permission);

enum NotificationDirection {
  "auto",
  "ltr",
  "rtl"
};
