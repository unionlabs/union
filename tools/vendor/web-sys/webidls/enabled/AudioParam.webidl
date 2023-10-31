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

[Pref="dom.webaudio.enabled"]
interface AudioParam {

                    attribute float value;
    readonly        attribute float defaultValue;
    readonly        attribute float minValue;
    readonly        attribute float maxValue;

    // Parameter automation.
    [Throws]
    AudioParam setValueAtTime(float value, double startTime);
    [Throws]
    AudioParam linearRampToValueAtTime(float value, double endTime);
    [Throws]
    AudioParam exponentialRampToValueAtTime(float value, double endTime);

    // Exponentially approach the target value with a rate having the given time constant.
    [Throws]
    AudioParam setTargetAtTime(float target, double startTime, double timeConstant);

    // Sets an array of arbitrary parameter values starting at time for the given duration.
    // The number of values will be scaled to fit into the desired duration.
    [Throws]
    AudioParam setValueCurveAtTime(Float32Array values, double startTime, double duration);

    // Cancels all scheduled parameter changes with times greater than or equal to startTime.
    [Throws]
    AudioParam cancelScheduledValues(double startTime);

};
