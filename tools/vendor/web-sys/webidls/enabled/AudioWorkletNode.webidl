/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

dictionary AudioWorkletNodeOptions : AudioNodeOptions {
             unsigned long             numberOfInputs = 1;
             unsigned long             numberOfOutputs = 1;
             sequence<unsigned long>   outputChannelCount;
             record<DOMString, double> parameterData;
             object?                   processorOptions = null;
};

[SecureContext, Pref="dom.audioworklet.enabled",
 Constructor (BaseAudioContext context, DOMString name, optional AudioWorkletNodeOptions options)]
interface AudioWorkletNode : AudioNode {
    [Throws]
    readonly        attribute AudioParamMap              parameters;
    [Throws]
    readonly        attribute MessagePort                port;
                    attribute EventHandler               onprocessorerror;
};
