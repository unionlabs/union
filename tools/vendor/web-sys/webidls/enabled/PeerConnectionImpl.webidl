/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * PeerConnection.js' interface to the C++ PeerConnectionImpl.
 *
 * Do not confuse with RTCPeerConnection. This interface is purely for
 * communication between the PeerConnection JS DOM binding and the C++
 * implementation in SIPCC.
 *
 * See media/webrtc/signaling/include/PeerConnectionImpl.h
 *
 */

// invalid widl
//interface nsISupports;

/* Must be created first. Observer events will be dispatched on the thread provided */
[ChromeOnly, Constructor]
interface PeerConnectionImpl  {
  /* Must be called first. Observer events dispatched on the thread provided */
  [Throws]
  undefined initialize(PeerConnectionObserver observer, Window window,
                  RTCConfiguration iceServers,
                  nsISupports thread);

  /* JSEP calls */
  [Throws]
  undefined createOffer(optional RTCOfferOptions options);
  [Throws]
  undefined createAnswer();
  [Throws]
  undefined setLocalDescription(long action, DOMString sdp);
  [Throws]
  undefined setRemoteDescription(long action, DOMString sdp);

  /* Stats call, calls either |onGetStatsSuccess| or |onGetStatsError| on our
     observer. (see the |PeerConnectionObserver| interface) */
  [Throws]
  undefined getStats(MediaStreamTrack? selector);

  /* Adds the tracks created by GetUserMedia */
  [Throws]
  TransceiverImpl createTransceiverImpl(DOMString kind,
                                        MediaStreamTrack? track);
  [Throws]
  boolean checkNegotiationNeeded();
  [Throws]
  undefined insertDTMF(TransceiverImpl transceiver, DOMString tones,
                  optional unsigned long duration = 100,
                  optional unsigned long interToneGap = 70);
  [Throws]
  DOMString getDTMFToneBuffer(RTCRtpSender sender);
  [Throws]
  sequence<RTCRtpSourceEntry> getRtpSources(MediaStreamTrack track,
                                            DOMHighResTimeStamp rtpSourceNow);
  DOMHighResTimeStamp getNowInRtpSourceReferenceTime();

  [Throws]
  undefined replaceTrackNoRenegotiation(TransceiverImpl transceiverImpl,
                                   MediaStreamTrack? withTrack);
  [Throws]
  undefined closeStreams();

  [Throws]
  undefined addRIDExtension(MediaStreamTrack recvTrack, unsigned short extensionId);
  [Throws]
  undefined addRIDFilter(MediaStreamTrack recvTrack, DOMString rid);

  // Inserts CSRC data for the RtpSourceObserver for testing
  [Throws]
  undefined insertAudioLevelForContributingSource(MediaStreamTrack recvTrack,
                                             unsigned long source,
                                             DOMHighResTimeStamp timestamp,
                                             boolean hasLevel,
                                             byte level);

  [Throws]
  undefined enablePacketDump(unsigned long level,
                        mozPacketDumpType type,
                        boolean sending);

  [Throws]
  undefined disablePacketDump(unsigned long level,
                         mozPacketDumpType type,
                         boolean sending);

  /* As the ICE candidates roll in this one should be called each time
   * in order to keep the candidate list up-to-date for the next SDP-related
   * call PeerConnectionImpl does not parse ICE candidates, just sticks them
   * into the SDP.
   */
  [Throws]
  undefined addIceCandidate(DOMString candidate, DOMString mid, unsigned short level);

  /* Puts the SIPCC engine back to 'kIdle', shuts down threads, deletes state */
  [Throws]
  undefined close();

  /* Notify DOM window if this plugin crash is ours. */
  boolean pluginCrash(unsigned long long pluginId, DOMString name);

  /* Attributes */
  /* This provides the implementation with the certificate it uses to
   * authenticate itself.  The JS side must set this before calling
   * createOffer/createAnswer or retrieving the value of fingerprint.  This has
   * to be delayed because generating the certificate takes some time. */
  attribute RTCCertificate certificate;
  [Constant]
  readonly attribute DOMString fingerprint;
  readonly attribute DOMString localDescription;
  readonly attribute DOMString currentLocalDescription;
  readonly attribute DOMString pendingLocalDescription;
  readonly attribute DOMString remoteDescription;
  readonly attribute DOMString currentRemoteDescription;
  readonly attribute DOMString pendingRemoteDescription;

  readonly attribute PCImplIceConnectionState iceConnectionState;
  readonly attribute PCImplIceGatheringState iceGatheringState;
  readonly attribute PCImplSignalingState signalingState;
  attribute DOMString id;

  [SetterThrows]
  attribute DOMString peerIdentity;
  readonly attribute boolean privacyRequested;

  /* Data channels */
  [Throws]
  RTCDataChannel createDataChannel(DOMString label, DOMString protocol,
    unsigned short type, boolean ordered,
    unsigned short maxTime, unsigned short maxNum,
    boolean externalNegotiated, unsigned short stream);
};
