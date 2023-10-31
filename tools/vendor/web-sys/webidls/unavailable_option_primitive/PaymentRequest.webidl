/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this WebIDL file is
 *   https://www.w3.org/TR/payment-request/#paymentrequest-interface
 */

dictionary PaymentMethodData {
  required DOMString           supportedMethods;
           object              data;
};

dictionary PaymentCurrencyAmount {
  required DOMString currency;
  required DOMString value;
           DOMString currencySystem = "urn:iso:std:iso:4217";
};

enum PaymentItemType {
  "tax"
};

dictionary PaymentItem {
  required DOMString             label;
  required PaymentCurrencyAmount amount;
           boolean               pending = false;
           PaymentItemType       type;
};

dictionary PaymentShippingOption {
  required DOMString             id;
  required DOMString             label;
  required PaymentCurrencyAmount amount;
           boolean               selected = false;
};

dictionary PaymentDetailsModifier {
  required DOMString             supportedMethods;
           PaymentItem           total;
           sequence<PaymentItem> additionalDisplayItems;
           object                data;
};

dictionary PaymentDetailsBase {
  sequence<PaymentItem>            displayItems;
  sequence<PaymentShippingOption>  shippingOptions;
  sequence<PaymentDetailsModifier> modifiers;
};

dictionary PaymentDetailsInit : PaymentDetailsBase {
           DOMString   id;
  required PaymentItem total;
};

dictionary AddressErrors {
  DOMString addressLine;
  DOMString city;
  DOMString country;
  DOMString dependentLocality;
  DOMString languageCode;
  DOMString organization;
  DOMString phone;
  DOMString postalCode;
  DOMString recipient;
  DOMString region;
  DOMString regionCode;
  DOMString sortingCode;
};

dictionary PaymentDetailsUpdate : PaymentDetailsBase {
  DOMString     error;
  AddressErrors shippingAddressErrors;
  PaymentItem   total;
};

enum PaymentShippingType {
  "shipping",
  "delivery",
  "pickup"
};

dictionary PaymentOptions {
  boolean             requestPayerName = false;
  boolean             requestPayerEmail = false;
  boolean             requestPayerPhone = false;
  boolean             requestShipping = false;
  PaymentShippingType shippingType = "shipping";
};

[Constructor(sequence<PaymentMethodData> methodData, PaymentDetailsInit details,
             optional PaymentOptions options),
 SecureContext,
 Func="mozilla::dom::PaymentRequest::PrefEnabled"]
interface PaymentRequest : EventTarget {
  [NewObject]
  Promise<PaymentResponse> show(optional Promise<PaymentDetailsUpdate> detailsPromise);
  [NewObject]
  Promise<undefined>            abort();
  [NewObject]
  Promise<boolean>         canMakePayment();

  readonly attribute DOMString            id;
  readonly attribute PaymentAddress?      shippingAddress;
  readonly attribute DOMString?           shippingOption;
  readonly attribute PaymentShippingType? shippingType;

           attribute EventHandler         onshippingaddresschange;
           attribute EventHandler         onshippingoptionchange;
           attribute EventHandler         onpaymentmethodchange;
};
