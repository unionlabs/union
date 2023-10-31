/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#idl-def-BaseAudioContext
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

callback DecodeSuccessCallback = undefined (AudioBuffer decodedData);
callback DecodeErrorCallback = undefined (DOMException error);

enum AudioContextState {
    "suspended",
    "running",
    "closed"
};

[RustDeprecated="doesn't exist in Safari, use `AudioContext` instead now"]
interface BaseAudioContext : EventTarget {
};

BaseAudioContext includes rustBaseAudioContext;

interface mixin rustBaseAudioContext {
    readonly        attribute AudioDestinationNode destination;
    readonly        attribute float                sampleRate;
    readonly        attribute double               currentTime;
    readonly        attribute AudioListener        listener;
    readonly        attribute AudioContextState    state;
    [Throws, SameObject, SecureContext, Pref="dom.audioworklet.enabled"]
    readonly        attribute AudioWorklet         audioWorklet;
    // Bug 1324552: readonly        attribute double               baseLatency;

    [Throws]
    Promise<undefined> resume();

                    attribute EventHandler         onstatechange;

    [NewObject, Throws]
    AudioBuffer            createBuffer (unsigned long numberOfChannels,
                                         unsigned long length,
                                         float sampleRate);

    [Throws]
    Promise<AudioBuffer> decodeAudioData(ArrayBuffer audioData,
                                         optional DecodeSuccessCallback successCallback,
                                         optional DecodeErrorCallback errorCallback);

    // AudioNode creation
    [NewObject, Throws]
    AudioBufferSourceNode createBufferSource();

    [NewObject, Throws]
    ConstantSourceNode createConstantSource();

    [NewObject, Throws]
    ScriptProcessorNode createScriptProcessor(optional unsigned long bufferSize = 0,
                                              optional unsigned long numberOfInputChannels = 2,
                                              optional unsigned long numberOfOutputChannels = 2);

    [NewObject, Throws]
    AnalyserNode createAnalyser();

    [NewObject, Throws]
    GainNode createGain();

    [NewObject, Throws]
    DelayNode createDelay(optional double maxDelayTime = 1); // TODO: no = 1

    [NewObject, Throws]
    BiquadFilterNode createBiquadFilter();

    [NewObject, Throws]
    IIRFilterNode createIIRFilter(sequence<double> feedforward, sequence<double> feedback);

    [NewObject, Throws]
    WaveShaperNode createWaveShaper();

    [NewObject, Throws]
    PannerNode createPanner();

    [NewObject, Throws]
    StereoPannerNode createStereoPanner();

    [NewObject, Throws]
    ConvolverNode createConvolver();

    [NewObject, Throws]
    ChannelSplitterNode createChannelSplitter(optional unsigned long numberOfOutputs = 6);

    [NewObject, Throws]
    ChannelMergerNode createChannelMerger(optional unsigned long numberOfInputs = 6);

    [NewObject, Throws]
    DynamicsCompressorNode createDynamicsCompressor();

    [NewObject, Throws]
    OscillatorNode createOscillator();

    [NewObject, Throws]
    PeriodicWave createPeriodicWave(Float32Array real,
                                    Float32Array imag,
                                    optional PeriodicWaveConstraints constraints);
};
