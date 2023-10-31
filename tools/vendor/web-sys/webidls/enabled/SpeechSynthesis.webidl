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

[Pref="media.webspeech.synth.enabled"]
interface SpeechSynthesis : EventTarget{
  readonly attribute boolean pending;
  readonly attribute boolean speaking;
  readonly attribute boolean paused;

  undefined speak(SpeechSynthesisUtterance utterance);
  undefined cancel();
  undefined pause();
  undefined resume();
  sequence<SpeechSynthesisVoice> getVoices();

  attribute EventHandler onvoiceschanged;

  [ChromeOnly]
  // Force an utterance to end. Circumvents bad speech service implementations.
  undefined forceEnd();
};
