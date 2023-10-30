/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
  Right now, it is not possible to add static functions to a JS implemented
  interface (see bug 863952), so we need to create a simple interface with a
  trivial constructor and no data to hold these functions that really ought to
  be static in RTCPeerConnection.
  TODO(bcampen@mozilla.com) Merge this code into RTCPeerConnection once this
  limitation is gone. (Bug 1017082)
*/

enum RTCLifecycleEvent {
    "initialized",
    "icegatheringstatechange",
    "iceconnectionstatechange"
};

callback PeerConnectionLifecycleCallback = undefined (RTCPeerConnection pc,
                                                 unsigned long long windowId,
                                                 RTCLifecycleEvent eventType);

[ChromeOnly,
 Pref="media.peerconnection.enabled",
 JSImplementation="@mozilla.org/dom/peerconnectionstatic;1",
 Constructor()]
interface RTCPeerConnectionStatic {

  /* One slot per window (the window in which the register call is made),
     automatically unregistered when window goes away.
     Fires when a PC is created, and whenever the ICE connection state or
     gathering state changes. */
  undefined registerPeerConnectionLifecycleCallback(
    PeerConnectionLifecycleCallback cb);
};

