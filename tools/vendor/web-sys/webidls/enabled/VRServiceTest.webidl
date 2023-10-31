/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * This WebIDL is just for WebVR testing.
 */

[Pref="dom.vr.test.enabled",
 HeaderFile="mozilla/dom/VRServiceTest.h"]
interface VRMockDisplay {
  undefined setEyeResolution(unsigned long aRenderWidth, unsigned long aRenderHeight);
  undefined setEyeParameter(VREye eye, double offsetX, double offsetY, double offsetZ,
                       double upDegree, double rightDegree,
                       double downDegree, double leftDegree);
  undefined setPose(Float32Array? position, Float32Array? linearVelocity,
               Float32Array? linearAcceleration, Float32Array? orientation,
               Float32Array? angularVelocity, Float32Array? angularAcceleration);
  undefined setMountState(boolean isMounted);
  undefined update();
};

[Pref="dom.vr.test.enabled",
 HeaderFile="mozilla/dom/VRServiceTest.h"]
interface VRMockController {
  undefined newButtonEvent(unsigned long button, boolean pressed);
  undefined newAxisMoveEvent(unsigned long axis, double value);
  undefined newPoseMove(Float32Array? position, Float32Array? linearVelocity,
                   Float32Array? linearAcceleration, Float32Array? orientation,
                   Float32Array? angularVelocity, Float32Array? angularAcceleration);
};

[Pref="dom.vr.test.enabled",
 HeaderFile="mozilla/dom/VRServiceTest.h"]
interface VRServiceTest {
  [Throws, NewObject]
  Promise<VRMockDisplay> attachVRDisplay(DOMString id);
  [Throws, NewObject]
  Promise<VRMockController> attachVRController(DOMString id);
};
