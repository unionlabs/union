/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

/**
  * This dictionary holds the parameters used to send
  * CSP reports in JSON format.
  */

dictionary CSPReportProperties {
  DOMString document-uri = "";
  DOMString referrer = "";
  DOMString blocked-uri = "";
  DOMString violated-directive = "";
  DOMString original-policy= "";
  DOMString source-file;
  DOMString script-sample;
  long line-number;
  long column-number;
};

dictionary CSPReport {
  CSPReportProperties csp-report;
};
