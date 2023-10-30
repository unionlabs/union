/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

// invalid widl
// interface nsISupports;

[ChromeOnly,
 JSImplementation="@mozilla.org/dom/peerconnectionobserver;1",
 Constructor (RTCPeerConnection domPC)]
interface PeerConnectionObserver
{
  /* JSEP callbacks */
  undefined onCreateOfferSuccess(DOMString offer);
  undefined onCreateOfferError(unsigned long name, DOMString message);
  undefined onCreateAnswerSuccess(DOMString answer);
  undefined onCreateAnswerError(unsigned long name, DOMString message);
  undefined onSetLocalDescriptionSuccess();
  undefined onSetRemoteDescriptionSuccess();
  undefined onSetLocalDescriptionError(unsigned long name, DOMString message);
  undefined onSetRemoteDescriptionError(unsigned long name, DOMString message);
  undefined onAddIceCandidateSuccess();
  undefined onAddIceCandidateError(unsigned long name, DOMString message);
  undefined onIceCandidate(unsigned short level, DOMString mid, DOMString candidate);

  /* Stats callbacks */
  undefined onGetStatsSuccess(optional RTCStatsReportInternal report);
  undefined onGetStatsError(unsigned long name, DOMString message);

  /* Data channel callbacks */
  undefined notifyDataChannel(RTCDataChannel channel);

  /* Notification of one of several types of state changed */
  undefined onStateChange(PCObserverStateType state);

  /* Transceiver management; called when setRemoteDescription causes a
     transceiver to be created on the C++ side */
  undefined onTransceiverNeeded(DOMString kind, TransceiverImpl transceiverImpl);

  /* DTMF callback */
  undefined onDTMFToneChange(MediaStreamTrack track, DOMString tone);

  /* Packet dump callback */
  undefined onPacket(unsigned long level, mozPacketDumpType type, boolean sending,
                ArrayBuffer packet);

  /* Transceiver sync */
  undefined syncTransceivers();
};
