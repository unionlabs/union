/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

/**
  * Dictionary used to display CSP info.
  */

dictionary CSP {
  boolean report-only = false;

  sequence<DOMString> default-src;
  sequence<DOMString> script-src;
  sequence<DOMString> object-src;
  sequence<DOMString> style-src;
  sequence<DOMString> img-src;
  sequence<DOMString> media-src;
  sequence<DOMString> frame-src;
  sequence<DOMString> font-src;
  sequence<DOMString> connect-src;
  sequence<DOMString> report-uri;
  sequence<DOMString> frame-ancestors;
  // sequence<DOMString> reflected-xss; // not supported in Firefox
  sequence<DOMString> base-uri;
  sequence<DOMString> form-action;
  sequence<DOMString> referrer;
  sequence<DOMString> manifest-src;
  sequence<DOMString> upgrade-insecure-requests;
  sequence<DOMString> child-src;
  sequence<DOMString> block-all-mixed-content;
  sequence<DOMString> require-sri-for;
  sequence<DOMString> sandbox;
  sequence<DOMString> worker-src;
};

dictionary CSPPolicies {
  sequence<CSP> csp-policies;
};
