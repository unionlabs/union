/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * http://w3c.github.io/webrtc-pc/ (with https://github.com/w3c/webrtc-pc/pull/178)
 */

[NoInterfaceObject]
interface RTCIdentityProviderRegistrar {
  undefined register(RTCIdentityProvider idp);

  /* Whether an IdP was passed to register() to chrome code. */
  [ChromeOnly]
  readonly attribute boolean hasIdp;
  /* The following two chrome-only functions forward to the corresponding
   * function on the registered IdP.  This is necessary because the
   * JS-implemented WebIDL can't see these functions on `idp` above, chrome JS
   * gets an Xray onto the content code that suppresses functions, see
   * https://developer.mozilla.org/en-US/docs/Xray_vision#Xrays_for_JavaScript_objects
   */
  /* Forward to idp.generateAssertion() */
  [ChromeOnly, Throws]
  Promise<RTCIdentityAssertionResult>
  generateAssertion(DOMString contents, DOMString origin,
                    optional RTCIdentityProviderOptions options);
  /* Forward to idp.validateAssertion() */
  [ChromeOnly, Throws]
  Promise<RTCIdentityValidationResult>
  validateAssertion(DOMString assertion, DOMString origin);
};

dictionary RTCIdentityProvider {
  required GenerateAssertionCallback generateAssertion;
  required ValidateAssertionCallback validateAssertion;
};

callback GenerateAssertionCallback =
  Promise<RTCIdentityAssertionResult>
    (DOMString contents, DOMString origin,
     RTCIdentityProviderOptions options);
callback ValidateAssertionCallback =
  Promise<RTCIdentityValidationResult> (DOMString assertion, DOMString origin);

dictionary RTCIdentityAssertionResult {
  required RTCIdentityProviderDetails idp;
  required DOMString assertion;
};

dictionary RTCIdentityProviderDetails {
  required DOMString domain;
  DOMString protocol = "default";
};

dictionary RTCIdentityValidationResult {
  required DOMString identity;
  required DOMString contents;
};

dictionary RTCIdentityProviderOptions {
  DOMString protocol = "default";
  DOMString usernameHint;
  DOMString peerIdentity;
};

