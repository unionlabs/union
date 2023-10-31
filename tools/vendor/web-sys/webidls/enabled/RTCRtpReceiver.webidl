/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://lists.w3.org/Archives/Public/public-webrtc/2014May/0067.html
 */

[Pref="media.peerconnection.enabled",
 JSImplementation="@mozilla.org/dom/rtpreceiver;1"]
interface RTCRtpReceiver {
  readonly attribute MediaStreamTrack   track;
  Promise<RTCStatsReport>               getStats();
  [Pref="media.peerconnection.rtpsourcesapi.enabled"]
  sequence<RTCRtpContributingSource>    getContributingSources();
  [Pref="media.peerconnection.rtpsourcesapi.enabled"]
  sequence<RTCRtpSynchronizationSource> getSynchronizationSources();

  [ChromeOnly]
  undefined setStreamIds(sequence<DOMString> streamIds);
  [ChromeOnly]
  undefined setRemoteSendBit(boolean sendBit);
  [ChromeOnly]
  undefined processTrackAdditionsAndRemovals(
      RTCRtpTransceiver transceiver,
      object postProcessing);
};
