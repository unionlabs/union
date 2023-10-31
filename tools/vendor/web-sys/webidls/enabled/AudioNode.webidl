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

enum ChannelCountMode {
    "max",
    "clamped-max",
    "explicit"
};

enum ChannelInterpretation {
    "speakers",
    "discrete"
};

dictionary AudioNodeOptions {
             unsigned long         channelCount;
             ChannelCountMode      channelCountMode;
             ChannelInterpretation channelInterpretation;
};

[Pref="dom.webaudio.enabled"]
interface AudioNode : EventTarget {

    [Throws]
    AudioNode connect(AudioNode destination, optional unsigned long output = 0, optional unsigned long input = 0);
    [Throws]
    undefined connect(AudioParam destination, optional unsigned long output = 0);
    [Throws]
    undefined disconnect();
    [Throws]
    undefined disconnect(unsigned long output);
    [Throws]
    undefined disconnect(AudioNode destination);
    [Throws]
    undefined disconnect(AudioNode destination, unsigned long output);
    [Throws]
    undefined disconnect(AudioNode destination, unsigned long output, unsigned long input);
    [Throws]
    undefined disconnect(AudioParam destination);
    [Throws]
    undefined disconnect(AudioParam destination, unsigned long output);

    readonly attribute BaseAudioContext context;
    readonly attribute unsigned long numberOfInputs;
    readonly attribute unsigned long numberOfOutputs;

    // Channel up-mixing and down-mixing rules for all inputs.
    [SetterThrows]
    attribute unsigned long channelCount;
    [SetterThrows]
    attribute ChannelCountMode channelCountMode;
    [SetterThrows]
    attribute ChannelInterpretation channelInterpretation;

};
