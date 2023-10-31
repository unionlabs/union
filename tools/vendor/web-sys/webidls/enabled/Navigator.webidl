/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/#the-navigator-object
 * http://www.w3.org/TR/tracking-dnt/
 * http://www.w3.org/TR/geolocation-API/#geolocation_interface
 * http://www.w3.org/TR/battery-status/#navigatorbattery-interface
 * http://www.w3.org/TR/vibration/#vibration-interface
 * http://www.w3.org/2012/sysapps/runtime/#extension-to-the-navigator-interface-1
 * https://dvcs.w3.org/hg/gamepad/raw-file/default/gamepad.html#navigator-interface-extension
 * http://www.w3.org/TR/beacon/#sec-beacon-method
 * https://html.spec.whatwg.org/#navigatorconcurrenthardware
 * http://wicg.github.io/netinfo/#extensions-to-the-navigator-interface
 * https://w3c.github.io/webappsec-credential-management/#framework-credential-management
 * https://w3c.github.io/webdriver/webdriver-spec.html#interface
 * https://wicg.github.io/media-capabilities/#idl-index
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

// http://www.whatwg.org/specs/web-apps/current-work/#the-navigator-object
[HeaderFile="Navigator.h"]
interface Navigator {
  // objects implementing this interface also implement the interfaces given below
};
Navigator includes NavigatorID;
Navigator includes NavigatorLanguage;
Navigator includes NavigatorOnLine;
Navigator includes NavigatorContentUtils;
Navigator includes NavigatorStorageUtils;
Navigator includes NavigatorConcurrentHardware;
Navigator includes NavigatorStorage;
Navigator includes NavigatorAutomationInformation;

[Exposed=(Window,Worker)]
interface mixin NavigatorID {
  // WebKit/Blink/Trident/Presto support this (hardcoded "Mozilla").
  [Constant, Cached, Throws]
  readonly attribute DOMString appCodeName; // constant "Mozilla"
  [Constant, Cached, NeedsCallerType]
  readonly attribute DOMString appName;
  [Constant, Cached, Throws, NeedsCallerType]
  readonly attribute DOMString appVersion;
  [Constant, Cached, Throws, NeedsCallerType]
  readonly attribute DOMString platform;
  [Pure, Cached, Throws, NeedsCallerType]
  readonly attribute DOMString userAgent;
  [Constant, Cached]
  readonly attribute DOMString product; // constant "Gecko"

  // Everyone but WebKit/Blink supports this.  See bug 679971.
  [Exposed=Window]
  boolean taintEnabled(); // constant false
};

[Exposed=(Window,Worker)]
interface mixin NavigatorLanguage {

  // These two attributes are cached because this interface is also implemented
  // by Workernavigator and this way we don't have to go back to the
  // main-thread from the worker thread anytime we need to retrieve them. They
  // are updated when pref intl.accept_languages is changed.

  [Pure, Cached]
  readonly attribute DOMString? language;
  [Pure, Cached, Frozen]
  readonly attribute sequence<DOMString> languages;
};

[Exposed=(Window,Worker)]
interface mixin NavigatorOnLine {
  readonly attribute boolean onLine;
};

interface mixin NavigatorContentUtils {
  // content handler registration
  [Throws, Func="nsGlobalWindowInner::RegisterProtocolHandlerAllowedForContext"]
  undefined registerProtocolHandler(DOMString scheme, DOMString url, DOMString title);
  [Pref="dom.registerContentHandler.enabled", Throws]
  undefined registerContentHandler(DOMString mimeType, DOMString url, DOMString title);
  // NOT IMPLEMENTED
  //DOMString isProtocolHandlerRegistered(DOMString scheme, DOMString url);
  //DOMString isContentHandlerRegistered(DOMString mimeType, DOMString url);
  //undefined unregisterProtocolHandler(DOMString scheme, DOMString url);
  //undefined unregisterContentHandler(DOMString mimeType, DOMString url);
};

[SecureContext, Exposed=(Window,Worker)]
interface mixin NavigatorStorage {
  [Func="mozilla::dom::DOMPrefs::StorageManagerEnabled"]
  readonly attribute StorageManager storage;
};

interface mixin NavigatorStorageUtils {
  // NOT IMPLEMENTED
  //undefined yieldForStorageUpdates();
};

partial interface Navigator {
  [Throws]
  readonly attribute Permissions permissions;
};

// Things that definitely need to be in the spec and and are not for some
// reason.  See https://www.w3.org/Bugs/Public/show_bug.cgi?id=22406
partial interface Navigator {
  [Throws]
  readonly attribute MimeTypeArray mimeTypes;
  [Throws]
  readonly attribute PluginArray plugins;
};

// http://www.w3.org/TR/tracking-dnt/ sort of
partial interface Navigator {
  readonly attribute DOMString doNotTrack;
};

// http://www.w3.org/TR/geolocation-API/#geolocation_interface
interface mixin NavigatorGeolocation {
  [Throws, Pref="geo.enabled"]
  readonly attribute Geolocation geolocation;
};
Navigator includes NavigatorGeolocation;

// http://www.w3.org/TR/battery-status/#navigatorbattery-interface
partial interface Navigator {
  // ChromeOnly to prevent web content from fingerprinting users' batteries.
  [Throws, ChromeOnly, Pref="dom.battery.enabled"]
  Promise<BatteryManager> getBattery();
};

// http://www.w3.org/TR/vibration/#vibration-interface
partial interface Navigator {
    // We don't support sequences in unions yet
    //boolean vibrate ((unsigned long or sequence<unsigned long>) pattern);
    boolean vibrate(unsigned long duration);
    boolean vibrate(sequence<unsigned long> pattern);
};

// http://www.w3.org/TR/pointerevents/#extensions-to-the-navigator-interface
partial interface Navigator {
    [Pref="dom.w3c_pointer_events.enabled"]
    readonly attribute long maxTouchPoints;
};

// https://wicg.github.io/media-capabilities/#idl-index
[Exposed=Window]
partial interface Navigator {
  [SameObject, Func="mozilla::dom::MediaCapabilities::Enabled"]
  readonly attribute MediaCapabilities mediaCapabilities;
};

// NetworkInformation
partial interface Navigator {
  [Throws, Pref="dom.netinfo.enabled"]
  readonly attribute NetworkInformation connection;
};

// https://dvcs.w3.org/hg/gamepad/raw-file/default/gamepad.html#navigator-interface-extension
partial interface Navigator {
  [Throws, Pref="dom.gamepad.enabled"]
  sequence<Gamepad?> getGamepads();
};
partial interface Navigator {
  [Pref="dom.gamepad.test.enabled"]
  GamepadServiceTest requestGamepadServiceTest();
};

partial interface Navigator {
  [Throws, Pref="dom.vr.enabled"]
  Promise<sequence<VRDisplay>> getVRDisplays();
  // TODO: Use FrozenArray once available. (Bug 1236777)
  [Frozen, Cached, Pure, Pref="dom.vr.enabled"]
  readonly attribute sequence<VRDisplay> activeVRDisplays;
  [ChromeOnly, Pref="dom.vr.enabled"]
  readonly attribute boolean isWebVRContentDetected;
  [ChromeOnly, Pref="dom.vr.enabled"]
  readonly attribute boolean isWebVRContentPresenting;
  [ChromeOnly, Pref="dom.vr.enabled"]
  undefined requestVRPresentation(VRDisplay display);
};
partial interface Navigator {
  [Pref="dom.vr.test.enabled"]
  VRServiceTest requestVRServiceTest();
};

// http://webaudio.github.io/web-midi-api/#requestmidiaccess
partial interface Navigator {
  [Throws, Pref="dom.webmidi.enabled"]
  Promise<MIDIAccess> requestMIDIAccess(optional MIDIOptions options);
};

callback NavigatorUserMediaSuccessCallback = undefined (MediaStream stream);
callback NavigatorUserMediaErrorCallback = undefined (MediaStreamError error);

partial interface Navigator {
  [Throws, Func="Navigator::HasUserMediaSupport"]
  readonly attribute MediaDevices mediaDevices;
};

// Service Workers/Navigation Controllers
partial interface Navigator {
  [Func="ServiceWorkerContainer::IsEnabled", SameObject]
  readonly attribute ServiceWorkerContainer serviceWorker;
};

partial interface Navigator {
  [Throws, Pref="beacon.enabled"]
  boolean sendBeacon(DOMString url,
                     optional BodyInit? data = null);
};

partial interface Navigator {
  [Throws, Pref="dom.presentation.enabled", SameObject]
  readonly attribute Presentation? presentation;
};

partial interface Navigator {
  [NewObject]
  Promise<MediaKeySystemAccess>
  requestMediaKeySystemAccess(DOMString keySystem,
                              sequence<MediaKeySystemConfiguration> supportedConfigurations);
};

[Exposed=(Window,Worker)]
interface mixin NavigatorConcurrentHardware {
  readonly attribute unsigned long long hardwareConcurrency;
};

// https://w3c.github.io/webappsec-credential-management/#framework-credential-management
partial interface Navigator {
  [Pref="security.webauth.webauthn", SecureContext, SameObject]
  readonly attribute CredentialsContainer credentials;
};

// https://w3c.github.io/webdriver/webdriver-spec.html#interface
[NoInterfaceObject]
interface NavigatorAutomationInformation {
  [Pref="dom.webdriver.enabled"]
  readonly attribute boolean webdriver;
};
