/* -*- Mode: IDL; tab-width: 4; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
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

dictionary OfflineAudioContextOptions {
             unsigned long numberOfChannels = 1;
    required unsigned long length;
    required float         sampleRate;
};

[Constructor (OfflineAudioContextOptions contextOptions),
Constructor(unsigned long numberOfChannels, unsigned long length, float sampleRate),
Pref="dom.webaudio.enabled"]
interface OfflineAudioContext : BaseAudioContext {

    [Throws]
    Promise<AudioBuffer> startRendering();

    // TODO: Promise<undefined>        suspend (double suspendTime);

    readonly        attribute unsigned long length;
                    attribute EventHandler  oncomplete;
};

OfflineAudioContext includes rustBaseAudioContext;
