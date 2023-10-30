/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this WebIDL file is
 *   https://www.w3.org/TR/payment-request/#paymentrequest-interface
 */
enum BasicCardType {
  "credit",
  "debit",
  "prepaid"
};

dictionary BasicCardRequest {
  sequence<DOMString> supportedNetworks;
  sequence<BasicCardType> supportedTypes;
};

dictionary BasicCardResponse {
           DOMString cardholderName;
  required DOMString cardNumber;
           DOMString expiryMonth;
           DOMString expiryYear;
           DOMString cardSecurityCode;
           PaymentAddress? billingAddress;
};
