/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#audioworkletglobalscope
 */

[Global=(Worklet,AudioWorklet),Exposed=AudioWorklet]
interface AudioWorkletGlobalScope : WorkletGlobalScope {
    undefined registerProcessor (DOMString name, VoidFunction processorCtor);
    readonly  attribute   unsigned long long currentFrame;
    readonly  attribute   double currentTime;
    readonly  attribute   float sampleRate;
};
