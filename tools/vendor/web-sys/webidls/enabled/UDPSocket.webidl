/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.w3.org/2012/sysapps/tcp-udp-sockets/#interface-udpsocket
 * http://www.w3.org/2012/sysapps/tcp-udp-sockets/#dictionary-udpoptions
 */

dictionary UDPOptions {
    DOMString      localAddress;
    unsigned short localPort;
    DOMString      remoteAddress;
    unsigned short remotePort;
    boolean        addressReuse = true;
    boolean        loopback = false;
};

[Constructor (optional UDPOptions options),
 Pref="dom.udpsocket.enabled",
 ChromeOnly]
interface UDPSocket : EventTarget {
    readonly    attribute DOMString?       localAddress;
    readonly    attribute unsigned short?  localPort;
    readonly    attribute DOMString?       remoteAddress;
    readonly    attribute unsigned short?  remotePort;
    readonly    attribute boolean          addressReuse;
    readonly    attribute boolean          loopback;
    readonly    attribute SocketReadyState readyState;
    readonly    attribute Promise<undefined>    opened;
    readonly    attribute Promise<undefined>    closed;
//    readonly    attribute ReadableStream   input; //Bug 1056444: Stream API is not ready
//    readonly    attribute WriteableStream  output; //Bug 1056444: Stream API is not ready
                attribute EventHandler     onmessage; //Bug 1056444: use event interface before Stream API is ready
    Promise<undefined> close ();
    [Throws] undefined    joinMulticastGroup (DOMString multicastGroupAddress);
    [Throws] undefined    leaveMulticastGroup (DOMString multicastGroupAddress);
    [Throws] boolean send ((DOMString or Blob or ArrayBuffer or ArrayBufferView) data, optional DOMString? remoteAddress, optional unsigned short? remotePort); //Bug 1056444: use send method before Stream API is ready
};
