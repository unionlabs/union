/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://webaudio.github.io/web-audio-api/#idl-def-AudioScheduledSourceNode
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[RustDeprecated="doesn't exist in Safari, use parent class methods instead"]
interface AudioScheduledSourceNode : AudioNode {
};

AudioScheduledSourceNode includes rustAudioScheduledSourceNode;

interface mixin rustAudioScheduledSourceNode {
                    attribute EventHandler onended;
    [Throws]
    undefined start (optional double when = 0);

    [Throws]
    undefined stop (optional double when = 0);
};
