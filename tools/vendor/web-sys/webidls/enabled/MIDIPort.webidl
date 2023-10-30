/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://webaudio.github.io/web-midi-api/
 */

enum MIDIPortType {
  "input",
  "output"
};

enum MIDIPortDeviceState {
  "disconnected",
  "connected"
};

enum MIDIPortConnectionState {
  "open",
  "closed",
  "pending"
};

[SecureContext, Pref="dom.webmidi.enabled"]
interface MIDIPort : EventTarget {
  readonly attribute DOMString    id;
  readonly attribute DOMString?   manufacturer;
  readonly attribute DOMString?   name;
  readonly attribute DOMString?   version;
  readonly attribute MIDIPortType type;
  readonly attribute MIDIPortDeviceState state;
  readonly attribute MIDIPortConnectionState connection;
           attribute EventHandler onstatechange;
  Promise<MIDIPort> open();
  Promise<MIDIPort> close();
};

