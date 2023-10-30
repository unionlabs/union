/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://www.w3.org/TR/webrtc/#rtcdtmfsender
 */

[JSImplementation="@mozilla.org/dom/rtcdtmfsender;1"]
interface RTCDTMFSender : EventTarget {
    undefined insertDTMF(DOMString tones,
                    optional unsigned long duration = 100,
                    optional unsigned long interToneGap = 70);
             attribute EventHandler  ontonechange;
    readonly attribute DOMString     toneBuffer;
};
