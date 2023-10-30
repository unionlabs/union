/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

/**
 * TCPSocket exposes a TCP client socket (no server sockets yet)
 * to highly privileged apps. It provides a buffered, non-blocking
 * interface for sending. For receiving, it uses an asynchronous,
 * event handler based interface.
 */

enum TCPSocketBinaryType {
  "arraybuffer",
  "string"
};

dictionary SocketOptions {
  boolean useSecureTransport = false;
  TCPSocketBinaryType binaryType = "string";
};

enum TCPReadyState {
  "connecting",
  "open",
  "closing",
  "closed",
};

[Constructor(DOMString host, unsigned short port, optional SocketOptions options),
 Func="mozilla::dom::TCPSocket::ShouldTCPSocketExist",
 Exposed=(Window,System)]
interface TCPSocket : EventTarget {
  /**
   * Upgrade an insecure connection to use TLS. Throws if the ready state is not OPEN.
   */
  [Throws] undefined upgradeToSecure();

  /**
   * The UTF16 host of this socket object.
   */
  readonly attribute USVString host;

  /**
   * The port of this socket object.
   */
  readonly attribute unsigned short port;

  /**
   * True if this socket object is an SSL socket.
   */
  readonly attribute boolean ssl;

  /**
   * The number of bytes which have previously been buffered by calls to
   * send on this socket.
   */
  readonly attribute unsigned long long bufferedAmount;

  /**
   * Pause reading incoming data and invocations of the ondata handler until
   * resume is called. Can be called multiple times without resuming.
   */
  undefined suspend();

  /**
   * Resume reading incoming data and invoking ondata as usual. There must be
   * an equal number of resume as suspends that took place. Throws if the
   * socket is not suspended.
   */
  [Throws]
  undefined resume();

  /**
   * Close the socket.
   */
  undefined close();

  /**
   * Close the socket immediately without waiting for unsent data.
   */
  [ChromeOnly] undefined closeImmediately();

  /**
   * Write data to the socket.
   *
   * @param data The data to write to the socket.
   *
   * @return Send returns true or false as a hint to the caller that
   *         they may either continue sending more data immediately, or
   *         may want to wait until the other side has read some of the
   *         data which has already been written to the socket before
   *         buffering more. If send returns true, then less than 64k
   *         has been buffered and it's safe to immediately write more.
   *         If send returns false, then more than 64k has been buffered,
   *         and the caller may wish to wait until the ondrain event
   *         handler has been called before buffering more data by more
   *         calls to send.
   *
   * @throws Throws if the ready state is not OPEN.
   */
  [Throws]
  boolean send(ByteString data);

  /**
   * Write data to the socket.
   *
   * @param data The data to write to the socket.
   * @param byteOffset The offset within the data from which to begin writing.
   * @param byteLength The number of bytes to write.
   *                   Defaults to the byte length of the ArrayBuffer if not present,
   *                   and clamped to (length - byteOffset).
   *
   * @return Send returns true or false as a hint to the caller that
   *         they may either continue sending more data immediately, or
   *         may want to wait until the other side has read some of the
   *         data which has already been written to the socket before
   *         buffering more. If send returns true, then less than 64k
   *         has been buffered and it's safe to immediately write more.
   *         If send returns false, then more than 64k has been buffered,
   *         and the caller may wish to wait until the ondrain event
   *         handler has been called before buffering more data by more
   *         calls to send.
   *
   * @throws Throws if the ready state is not OPEN.
   */
  [Throws]
  boolean send(ArrayBuffer data, optional unsigned long byteOffset = 0, optional unsigned long byteLength);

  /**
   * The readyState attribute indicates which state the socket is currently
   * in.
   */
  readonly attribute TCPReadyState readyState;

  /**
   * The binaryType attribute indicates which mode this socket uses for
   * sending and receiving data. If the binaryType: "arraybuffer" option
   * was passed to the open method that created this socket, binaryType
   * will be "arraybuffer". Otherwise, it will be "string".
   */
  readonly attribute TCPSocketBinaryType binaryType;

  /**
   * The "open" event is dispatched when the connection to the server
   * has been established. If the connection is refused, the "error" event
   * will be dispatched, instead.
   */
  attribute EventHandler onopen;

  /**
   * After send has buffered more than 64k of data, it returns false to
   * indicate that the client should pause before sending more data, to
   * avoid accumulating large buffers. This is only advisory, and the client
   * is free to ignore it and buffer as much data as desired, but if reducing
   * the size of buffers is important (especially for a streaming application)
   * the "drain" event will be dispatched once the previously-buffered data has
   * been written to the network, at which point the client can resume calling
   * send again.
   */
  attribute EventHandler ondrain;

  /**
   * The "data" event will be dispatched repeatedly and asynchronously after
   * "open" is dispatched, every time some data was available from the server
   * and was read. The event object will be a TCPSocketEvent; if the "arraybuffer"
   * binaryType was passed to the constructor, the data attribute of the event
   * object will be an ArrayBuffer. If not, it will be a normal JavaScript string,
   * truncated at the first null byte found in the payload and the remainder
   * interpreted as ASCII bytes.
   *
   * At any time, the client may choose to pause reading and receiving "data"
   * events by calling the socket's suspend() method. Further "data" events
   * will be paused until resume() is called.
   */
  attribute EventHandler ondata;

  /**
   * The "error" event will be dispatched when there is an error. The event
   * object will be a TCPSocketErrorEvent.
   *
   * If an "error" event is dispatched before an "open" one, the connection
   * was refused, and the "close" event will not be dispatched. If an "error"
   * event is dispatched after an "open" event, the connection was lost,
   * and a "close" event will be dispatched subsequently.
   */
  attribute EventHandler onerror;

  /**
   * The "close" event is dispatched once the underlying network socket
   * has been closed, either by the server, or by the client calling
   * close.
   *
   * If the "error" event was not dispatched before "close", then one of
   * the sides cleanly closed the connection.
   */
  attribute EventHandler onclose;
};
