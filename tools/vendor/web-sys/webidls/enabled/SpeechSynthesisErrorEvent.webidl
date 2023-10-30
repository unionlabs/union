/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dvcs.w3.org/hg/speech-api/raw-file/tip/speechapi.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

enum SpeechSynthesisErrorCode {
  "canceled",
  "interrupted",
  "audio-busy",
  "audio-hardware",
  "network",
  "synthesis-unavailable",
  "synthesis-failed",
  "language-unavailable",
  "voice-unavailable",
  "text-too-long",
  "invalid-argument",
};

[Constructor(DOMString type, SpeechSynthesisErrorEventInit eventInitDict),
 Pref="media.webspeech.synth.enabled"]
interface SpeechSynthesisErrorEvent : SpeechSynthesisEvent {
  readonly attribute SpeechSynthesisErrorCode error;
};

dictionary SpeechSynthesisErrorEventInit : SpeechSynthesisEventInit
{
  required SpeechSynthesisErrorCode error;
};
