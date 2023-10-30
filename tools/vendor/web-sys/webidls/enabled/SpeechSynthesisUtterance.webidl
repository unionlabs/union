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

[Pref="media.webspeech.synth.enabled",
 Constructor,
 Constructor(DOMString text)]
interface SpeechSynthesisUtterance : EventTarget {
  attribute DOMString text;
  attribute DOMString lang;
  attribute SpeechSynthesisVoice? voice;
  attribute float volume;
  attribute float rate;
  attribute float pitch;

  attribute EventHandler onstart;
  attribute EventHandler onend;
  attribute EventHandler onerror;
  attribute EventHandler onpause;
  attribute EventHandler onresume;
  attribute EventHandler onmark;
  attribute EventHandler onboundary;

  [ChromeOnly]
  readonly attribute DOMString chosenVoiceURI;
};
