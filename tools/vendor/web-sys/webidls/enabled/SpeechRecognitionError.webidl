/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

enum SpeechRecognitionErrorCode {
  "no-speech",
  "aborted",
  "audio-capture",
  "network",
  "not-allowed",
  "service-not-allowed",
  "bad-grammar",
  "language-not-supported"
};

[Pref="media.webspeech.recognition.enable",
 Func="SpeechRecognition::IsAuthorized",
 Constructor(DOMString type, optional SpeechRecognitionErrorInit eventInitDict)]
interface SpeechRecognitionError : Event
{
  readonly attribute SpeechRecognitionErrorCode error;
  readonly attribute DOMString? message;
};

dictionary SpeechRecognitionErrorInit : EventInit
{
  SpeechRecognitionErrorCode error = "no-speech";
  DOMString message = "";
};
