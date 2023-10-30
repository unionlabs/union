/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * This is in a separate file so it can be shared with unittests.
 */

/* Must be in the same order as comparable fsmdef_states_t in fsmdef_states.h */
enum PCImplSignalingState {
  "SignalingInvalid",
  "SignalingStable",
  "SignalingHaveLocalOffer",
  "SignalingHaveRemoteOffer",
  "SignalingHaveLocalPranswer",
  "SignalingHaveRemotePranswer",
  "SignalingClosed",
};

enum PCImplIceConnectionState {
    "new",
    "checking",
    "connected",
    "completed",
    "failed",
    "disconnected",
    "closed"
};

// Deliberately identical to the values specified in webrtc
enum PCImplIceGatheringState {
  "new",
  "gathering",
  "complete"
};

