/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://w3c.github.io/webrtc-pc/ Editor's Draft 18 January 2018
 */

dictionary RTCRtpContributingSource {
    required DOMHighResTimeStamp timestamp;
    required unsigned long       source;
             double              audioLevel;
};

dictionary RTCRtpSynchronizationSource : RTCRtpContributingSource {
    boolean? voiceActivityFlag;
};

/* Internal enum of types used by RTCRtpSourceEntry */
enum RTCRtpSourceEntryType {
    "contributing",
    "synchronization",
};
/* Internal shared representation of Contributing and Synchronization sources */
dictionary RTCRtpSourceEntry : RTCRtpSynchronizationSource {
    required RTCRtpSourceEntryType sourceType;
};
