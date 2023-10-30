/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/*
 * WebXR Device API
 * W3C Working Draft, 31 March 2022
 * The origin of this IDL file is:
 * https://www.w3.org/TR/2022/WD-webxr-gamepads-module-1-20220426/
 */

partial interface XRInputSource {
  [SameObject] readonly attribute Gamepad? gamepad;
};
