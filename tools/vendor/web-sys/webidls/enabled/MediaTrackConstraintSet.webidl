/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dev.w3.org/2011/webrtc/editor/getusermedia.html
 */

// These dictionaries need to be in a separate file from their use in unions
// in MediaSreamTrack.webidl due to a webidl compiler limitation:
//
// TypeError: Dictionary contains a union that contains a dictionary in the same
// WebIDL file.  This won't compile.  Move the inner dictionary to a different file.

dictionary ConstrainLongRange {
    long min;
    long max;
    long exact;
    long ideal;
};

dictionary ConstrainDoubleRange {
    double min;
    double max;
    double exact;
    double ideal;
};

dictionary ConstrainBooleanParameters {
    boolean exact;
    boolean ideal;
};

dictionary ConstrainDOMStringParameters {
    (DOMString or sequence<DOMString>) exact;
    (DOMString or sequence<DOMString>) ideal;
};
