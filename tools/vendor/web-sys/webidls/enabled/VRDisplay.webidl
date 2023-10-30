/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

enum VREye {
  "left",
  "right"
};

[Pref="dom.vr.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VRFieldOfView {
  readonly attribute double upDegrees;
  readonly attribute double rightDegrees;
  readonly attribute double downDegrees;
  readonly attribute double leftDegrees;
};

typedef (HTMLCanvasElement or OffscreenCanvas) VRSource;

dictionary VRLayer {
  /**
   * XXX - When WebVR in WebWorkers is implemented, HTMLCanvasElement below
   *       should be replaced with VRSource.
   */
  HTMLCanvasElement? source = null;

  /**
   * The left and right viewports contain 4 values defining the viewport
   * rectangles within the canvas to present to the eye in UV space.
   * [0] left offset of the viewport (0.0 - 1.0)
   * [1] top offset of the viewport (0.0 - 1.0)
   * [2] width of the viewport (0.0 - 1.0)
   * [3] height of the viewport (0.0 - 1.0)
   *
   * When no values are passed, they will be processed as though the left
   * and right sides of the viewport were passed:
   *
   * leftBounds: [0.0, 0.0, 0.5, 1.0]
   * rightBounds: [0.5, 0.0, 0.5, 1.0]
   */
  sequence<float> leftBounds = [];
  sequence<float> rightBounds = [];
};

/**
 * Values describing the capabilities of a VRDisplay.
 * These are expected to be static per-device/per-user.
 */
[Pref="dom.vr.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VRDisplayCapabilities {
  /**
   * hasPosition is true if the VRDisplay is capable of tracking its position.
   */
  readonly attribute boolean hasPosition;

  /**
   * hasOrientation is true if the VRDisplay is capable of tracking its orientation.
   */
  readonly attribute boolean hasOrientation;

  /**
   * Whether the VRDisplay is separate from the device’s
   * primary display. If presenting VR content will obscure
   * other content on the device, this should be false. When
   * false, the application should not attempt to mirror VR content
   * or update non-VR UI because that content will not be visible.
   */
  readonly attribute boolean hasExternalDisplay;

  /**
   * Whether the VRDisplay is capable of presenting content to an HMD or similar device.
   * Can be used to indicate “magic window” devices that are capable of 6DoF tracking but for
   * which requestPresent is not meaningful. If false then calls to requestPresent should
   * always fail, and getEyeParameters should return null.
   */
  readonly attribute boolean canPresent;

  /**
   * Indicates the maximum length of the array that requestPresent() will accept. MUST be 1 if
     canPresent is true, 0 otherwise.
   */
  readonly attribute unsigned long maxLayers;
};

/**
 * Values describing the the stage / play area for devices
 * that support room-scale experiences.
 */
[Pref="dom.vr.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VRStageParameters {
  /**
   * A 16-element array containing the components of a column-major 4x4
   * affine transform matrix. This matrix transforms the sitting-space position
   * returned by get{Immediate}Pose() to a standing-space position.
   */
  [Throws] readonly attribute Float32Array sittingToStandingTransform;

  /**
   * Dimensions of the play-area bounds. The bounds are defined
   * as an axis-aligned rectangle on the floor.
   * The center of the rectangle is at (0,0,0) in standing-space
   * coordinates.
   * These bounds are defined for safety purposes.
   * Content should not require the user to move beyond these
   * bounds; however, it is possible for the user to ignore
   * the bounds resulting in position values outside of
   * this rectangle.
   */
  readonly attribute float sizeX;
  readonly attribute float sizeZ;
};

[Pref="dom.vr.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VRPose
{
  /**
   * position, linearVelocity, and linearAcceleration are 3-component vectors.
   * position is relative to a sitting space. Transforming this point with
   * VRStageParameters.sittingToStandingTransform converts this to standing space.
   */
  [Constant, Throws] readonly attribute Float32Array? position;
  [Constant, Throws] readonly attribute Float32Array? linearVelocity;
  [Constant, Throws] readonly attribute Float32Array? linearAcceleration;

  /* orientation is a 4-entry array representing the components of a quaternion. */
  [Constant, Throws] readonly attribute Float32Array? orientation;
  /* angularVelocity and angularAcceleration are the components of 3-dimensional vectors. */
  [Constant, Throws] readonly attribute Float32Array? angularVelocity;
  [Constant, Throws] readonly attribute Float32Array? angularAcceleration;
};

[Constructor,
 Pref="dom.vr.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VRFrameData {
  readonly attribute DOMHighResTimeStamp timestamp;

  [Throws, Pure] readonly attribute Float32Array leftProjectionMatrix;
  [Throws, Pure] readonly attribute Float32Array leftViewMatrix;

  [Throws, Pure] readonly attribute Float32Array rightProjectionMatrix;
  [Throws, Pure] readonly attribute Float32Array rightViewMatrix;

  [Pure] readonly attribute VRPose pose;
};

[Constructor,
 Pref="dom.vr.test.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VRSubmitFrameResult {
  readonly attribute unsigned long frameNum;
  readonly attribute DOMString? base64Image;
};

[Pref="dom.vr.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VREyeParameters {
  /**
   * offset is a 3-component vector representing an offset to
   * translate the eye. This value may vary from frame
   * to frame if the user adjusts their headset ipd.
   */
  [Constant, Throws] readonly attribute Float32Array offset;

  /* These values may vary as the user adjusts their headset ipd. */
  [Constant] readonly attribute VRFieldOfView fieldOfView;

  /**
   * renderWidth and renderHeight specify the recommended render target
   * size of each eye viewport, in pixels. If multiple eyes are rendered
   * in a single render target, then the render target should be made large
   * enough to fit both viewports.
   */
  [Constant] readonly attribute unsigned long renderWidth;
  [Constant] readonly attribute unsigned long renderHeight;
};

[Pref="dom.vr.enabled",
 HeaderFile="mozilla/dom/VRDisplay.h"]
interface VRDisplay : EventTarget {
  /**
   * presentingGroups is a bitmask indicating which VR session groups
   * have an active VR presentation.
   */
  [ChromeOnly] readonly attribute unsigned long presentingGroups;
  /**
   * Setting groupMask causes submitted frames by VR sessions that
   * aren't included in the bitmasked groups to be ignored.
   * Non-chrome content is not aware of the value of groupMask.
   * VRDisplay.RequestAnimationFrame will still fire for VR sessions
   * that are hidden by groupMask, enabling their performance to be
   * measured by chrome UI that is presented in other groups.
   * This is expected to be used in cases where chrome UI is presenting
   * information during link traversal or presenting options when content
   * performance is too low for comfort.
   * The VR refresh / VSync cycle is driven by the visible content
   * and the non-visible content may have a throttled refresh rate.
   */
  [ChromeOnly] attribute unsigned long groupMask;

  readonly attribute boolean isConnected;
  readonly attribute boolean isPresenting;

  /**
   * Dictionary of capabilities describing the VRDisplay.
   */
  [Constant] readonly attribute VRDisplayCapabilities capabilities;

  /**
   * If this VRDisplay supports room-scale experiences, the optional
   * stage attribute contains details on the room-scale parameters.
   */
  readonly attribute VRStageParameters? stageParameters;

  /* Return the current VREyeParameters for the given eye. */
  VREyeParameters getEyeParameters(VREye whichEye);

  /**
   * An identifier for this distinct VRDisplay. Used as an
   * association point in the Gamepad API.
   */
  [Constant] readonly attribute unsigned long displayId;

  /**
   * A display name, a user-readable name identifying it.
   */
  [Constant] readonly attribute DOMString displayName;

  /**
   * Populates the passed VRFrameData with the information required to render
   * the current frame.
   */
  boolean getFrameData(VRFrameData frameData);

  /**
   * Return a VRPose containing the future predicted pose of the VRDisplay
   * when the current frame will be presented. Subsequent calls to getPose()
   * MUST return a VRPose with the same values until the next call to
   * submitFrame().
   *
   * The VRPose will contain the position, orientation, velocity,
   * and acceleration of each of these properties.
   */
  [NewObject] VRPose getPose();

  [Pref="dom.vr.test.enabled"]
  boolean getSubmitFrameResult(VRSubmitFrameResult result);

  /**
   * Reset the pose for this display, treating its current position and
   * orientation as the "origin/zero" values. VRPose.position,
   * VRPose.orientation, and VRStageParameters.sittingToStandingTransform may be
   * updated when calling resetPose(). This should be called in only
   * sitting-space experiences.
   */
  undefined resetPose();

  /**
   * z-depth defining the near plane of the eye view frustum
   * enables mapping of values in the render target depth
   * attachment to scene coordinates. Initially set to 0.01.
   */
  attribute double depthNear;

  /**
   * z-depth defining the far plane of the eye view frustum
   * enables mapping of values in the render target depth
   * attachment to scene coordinates. Initially set to 10000.0.
   */
  attribute double depthFar;

  /**
   * The callback passed to `requestAnimationFrame` will be called
   * any time a new frame should be rendered. When the VRDisplay is
   * presenting the callback will be called at the native refresh
   * rate of the HMD. When not presenting this function acts
   * identically to how window.requestAnimationFrame acts. Content should
   * make no assumptions of frame rate or vsync behavior as the HMD runs
   * asynchronously from other displays and at differing refresh rates.
   */
  [Throws] long requestAnimationFrame(FrameRequestCallback callback);

  /**
   * Passing the value returned by `requestAnimationFrame` to
   * `cancelAnimationFrame` will unregister the callback.
   */
  [Throws] undefined cancelAnimationFrame(long handle);

  /**
   * Begin presenting to the VRDisplay. Must be called in response to a user gesture.
   * Repeat calls while already presenting will update the VRLayers being displayed.
   */
  [Throws, NeedsCallerType] Promise<undefined> requestPresent(sequence<VRLayer> layers);

  /**
   * Stops presenting to the VRDisplay.
   */
  [Throws] Promise<undefined> exitPresent();

  /**
   * Get the layers currently being presented.
   */
  sequence<VRLayer> getLayers();

  /**
   * The VRLayer provided to the VRDisplay will be captured and presented
   * in the HMD. Calling this function has the same effect on the source
   * canvas as any other operation that uses its source image, and canvases
   * created without preserveDrawingBuffer set to true will be cleared.
   */
  undefined submitFrame();
};
