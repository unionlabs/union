/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * This is an internal IDL file
 */

[ChromeOnly,
 JSImplementation="@mozilla.org/dom/createofferrequest;1"]
interface CreateOfferRequest {
  readonly attribute unsigned long long windowID;
  readonly attribute unsigned long long innerWindowID;
  readonly attribute DOMString callID;
  readonly attribute boolean isSecure;
};
