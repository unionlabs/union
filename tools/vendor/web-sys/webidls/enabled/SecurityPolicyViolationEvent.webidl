/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/. */

enum SecurityPolicyViolationEventDisposition
{
  "enforce", "report"
};

[Constructor(DOMString type, optional SecurityPolicyViolationEventInit eventInitDict),
 Pref="security.csp.enable_violation_events"]
interface SecurityPolicyViolationEvent : Event
{
    readonly attribute DOMString      documentURI;
    readonly attribute DOMString      referrer;
    readonly attribute DOMString      blockedURI;
    readonly attribute DOMString      violatedDirective;
    readonly attribute DOMString      effectiveDirective;
    readonly attribute DOMString      originalPolicy;
    readonly attribute DOMString      sourceFile;
    readonly attribute DOMString      sample;
    readonly attribute SecurityPolicyViolationEventDisposition disposition;
    readonly attribute unsigned short statusCode;
    readonly attribute long           lineNumber;
    readonly attribute long           columnNumber;
};

dictionary SecurityPolicyViolationEventInit : EventInit
{
    DOMString      documentURI = "";
    DOMString      referrer = "";
    DOMString      blockedURI = "";
    DOMString      violatedDirective = "";
    DOMString      effectiveDirective = "";
    DOMString      originalPolicy = "";
    DOMString      sourceFile = "";
    DOMString      sample = "";
    SecurityPolicyViolationEventDisposition disposition = "report";
    unsigned short statusCode = 0;
    long           lineNumber = 0;
    long           columnNumber = 0;
};
