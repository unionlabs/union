/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this WebIDL file is
 *   https://www.w3.org/TR/payment-request/#paymentresponse-interface
 */

enum PaymentComplete {
  "success",
  "fail",
  "unknown"
};

[SecureContext,
 Func="mozilla::dom::PaymentRequest::PrefEnabled"]
interface PaymentResponse {
  [Default] object toJSON();

  readonly attribute DOMString       requestId;
  readonly attribute DOMString       methodName;
  readonly attribute object          details;
  readonly attribute PaymentAddress? shippingAddress;
  readonly attribute DOMString?      shippingOption;
  readonly attribute DOMString?      payerName;
  readonly attribute DOMString?      payerEmail;
  readonly attribute DOMString?      payerPhone;

  [NewObject]
  Promise<undefined> complete(optional PaymentComplete result = "unknown");
};
