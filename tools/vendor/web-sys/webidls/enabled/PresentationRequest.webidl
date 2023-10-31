/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/presentation-api/#interface-presentationrequest
 */

[Constructor(DOMString url),
 Constructor(sequence<DOMString> urls),
 Pref="dom.presentation.controller.enabled"]
interface PresentationRequest : EventTarget {
  /*
   * A requesting page use start() to start a new connection, and it will be
   * returned with the promise. UA may show a prompt box with a list of
   * available devices and ask the user to grant permission, choose a device, or
   * cancel the operation.
   *
   * The promise is resolved when the presenting page is successfully loaded and
   * the communication channel is established, i.e., the connection state is
   * "connected".
   *
   * The promise may be rejected duo to one of the following reasons:
   * - "OperationError": Unexpected error occurs.
   * - "NotFoundError":  No available device.
   * - "AbortError":     User dismiss/cancel the device prompt box.
   * - "NetworkError":   Failed to establish the control channel or data channel.
   * - "TimeoutError":   Presenting page takes too long to load.
   * - "SecurityError":  This operation is insecure.
   */
  [Throws]
  Promise<PresentationConnection> start();

  /*
   * A requesting page can use reconnect(presentationId) to reopen a
   * non-terminated presentation connection.
   *
   * The promise is resolved when a new presentation connection is created.
   * The connection state is "connecting".
   *
   * The promise may be rejected duo to one of the following reasons:
   * - "OperationError": Unexpected error occurs.
   * - "NotFoundError":  Can not find a presentation connection with the presentationId.
   * - "SecurityError":  This operation is insecure.
   */
  [Throws]
  Promise<PresentationConnection> reconnect(DOMString presentationId);

 /*
  * UA triggers device discovery mechanism periodically and monitor device
  * availability.
  *
  * The promise may be rejected duo to one of the following reasons:
  * - "NotSupportedError": Unable to continuously monitor the availability.
  * - "SecurityError":  This operation is insecure.
  */
  [Throws]
  Promise<PresentationAvailability> getAvailability();

  /*
   * It is called when a connection associated with a PresentationRequest is created.
   * The event is fired for all connections that are created for the controller.
   */
  attribute EventHandler onconnectionavailable;

  /*
   * A chrome page, or page which has presentation-device-manage permissiongs,
   * uses startWithDevice() to start a new connection with specified device,
   * and it will be returned with the promise. UA may show a prompt box with a
   * list of available devices and ask the user to grant permission, choose a
   * device, or cancel the operation.
   *
   * The promise is resolved when the presenting page is successfully loaded and
   * the communication channel is established, i.e., the connection state is
   * "connected".
   *
   * The promise may be rejected duo to one of the following reasons:
   * - "OperationError": Unexpected error occurs.
   * - "NotFoundError":  No available device.
   * - "NetworkError":   Failed to establish the control channel or data channel.
   * - "TimeoutError":   Presenting page takes too long to load.
   */
  [ChromeOnly, Throws]
  Promise<PresentationConnection> startWithDevice(DOMString deviceId);
};
