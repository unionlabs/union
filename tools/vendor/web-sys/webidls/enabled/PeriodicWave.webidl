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

dictionary PeriodicWaveConstraints {
  boolean disableNormalization = false;
};

dictionary PeriodicWaveOptions : PeriodicWaveConstraints {
             sequence<float> real;
             sequence<float> imag;
};

[Pref="dom.webaudio.enabled",
 // XXXbz The second arg is not optional in the spec, but that looks
 // like a spec bug to me.  See
 // <https://github.com/WebAudio/web-audio-api/issues/1116>.
 Constructor(BaseAudioContext context, optional PeriodicWaveOptions options)]
interface PeriodicWave {
};
