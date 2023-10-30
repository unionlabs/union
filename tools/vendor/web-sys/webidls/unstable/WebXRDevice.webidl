/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/*
 * WebXR Device API
 * W3C Working Draft, 31 March 2022
 * The origin of this IDL file is:
 * https://www.w3.org/TR/2022/CR-webxr-20220331/
 */

partial interface Navigator {
  [SecureContext, SameObject] readonly attribute XRSystem xr;
};

[SecureContext, Exposed=Window] interface XRSystem : EventTarget {
  // Methods
  Promise<boolean> isSessionSupported(XRSessionMode mode);
  [NewObject] Promise<XRSession> requestSession(XRSessionMode mode, optional XRSessionInit options = {});

  // Events
  attribute EventHandler ondevicechange;
};

enum XRSessionMode {
  "inline",
  "immersive-vr",
  "immersive-ar"
};

dictionary XRSessionInit {
  sequence<any> requiredFeatures;
  sequence<any> optionalFeatures;
};

enum XRVisibilityState {
  "visible",
  "visible-blurred",
  "hidden",
};

[SecureContext, Exposed=Window] interface XRSession : EventTarget {
  // Attributes
  readonly attribute XRVisibilityState visibilityState;
  readonly attribute float? frameRate;
  readonly attribute Float32Array? supportedFrameRates;
  [SameObject] readonly attribute XRRenderState renderState;
  [SameObject] readonly attribute XRInputSourceArray inputSources;

  // Methods
  undefined updateRenderState(optional XRRenderStateInit state = {});
  Promise<undefined> updateTargetFrameRate(float rate);
  [NewObject] Promise<XRReferenceSpace> requestReferenceSpace(XRReferenceSpaceType type);

  unsigned long requestAnimationFrame(XRFrameRequestCallback callback);
  undefined cancelAnimationFrame(unsigned long handle);

  Promise<undefined> end();

  // Events
  attribute EventHandler onend;
  attribute EventHandler oninputsourceschange;
  attribute EventHandler onselect;
  attribute EventHandler onselectstart;
  attribute EventHandler onselectend;
  attribute EventHandler onsqueeze;
  attribute EventHandler onsqueezestart;
  attribute EventHandler onsqueezeend;
  attribute EventHandler onvisibilitychange;
  attribute EventHandler onframeratechange;
};

dictionary XRRenderStateInit {
  double depthNear;
  double depthFar;
  double inlineVerticalFieldOfView;
  XRWebGLLayer? baseLayer;
  sequence<XRLayer>? layers;
};

[SecureContext, Exposed=Window] interface XRRenderState {
  readonly attribute double depthNear;
  readonly attribute double depthFar;
  readonly attribute double? inlineVerticalFieldOfView;
  readonly attribute XRWebGLLayer? baseLayer;
};

callback XRFrameRequestCallback = undefined (DOMHighResTimeStamp time, XRFrame frame);

[SecureContext, Exposed=Window] interface XRFrame {
  [SameObject] readonly attribute XRSession session;
  readonly attribute DOMHighResTimeStamp predictedDisplayTime;

  XRViewerPose? getViewerPose(XRReferenceSpace referenceSpace);
  XRPose? getPose(XRSpace space, XRSpace baseSpace);
};

[SecureContext, Exposed=Window] interface XRSpace : EventTarget {

};

enum XRReferenceSpaceType {
  "viewer",
  "local",
  "local-floor",
  "bounded-floor",
  "unbounded"
};

[SecureContext, Exposed=Window]
interface XRReferenceSpace : XRSpace {
  [NewObject] XRReferenceSpace getOffsetReferenceSpace(XRRigidTransform originOffset);

  attribute EventHandler onreset;
};

[SecureContext, Exposed=Window]
interface XRBoundedReferenceSpace : XRReferenceSpace {
  readonly attribute FrozenArray<DOMPointReadOnly> boundsGeometry;
};

enum XREye {
  "none",
  "left",
  "right"
};

[SecureContext, Exposed=Window] interface XRView {
  readonly attribute XREye eye;
  readonly attribute Float32Array projectionMatrix;
  [SameObject] readonly attribute XRRigidTransform transform;
  readonly attribute double? recommendedViewportScale;

  undefined requestViewportScale(double? scale);
};

[SecureContext, Exposed=Window] interface XRViewport {
  readonly attribute long x;
  readonly attribute long y;
  readonly attribute long width;
  readonly attribute long height;
};

[SecureContext, Exposed=Window]
interface XRRigidTransform {
  constructor(optional DOMPointInit position = {}, optional DOMPointInit orientation = {});
  [SameObject] readonly attribute DOMPointReadOnly position;
  [SameObject] readonly attribute DOMPointReadOnly orientation;
  readonly attribute Float32Array matrix;
  [SameObject] readonly attribute XRRigidTransform inverse;
};

[SecureContext, Exposed=Window] interface XRPose {
  [SameObject] readonly attribute XRRigidTransform transform;
  [SameObject] readonly attribute DOMPointReadOnly? linearVelocity;
  [SameObject] readonly attribute DOMPointReadOnly? angularVelocity;

  readonly attribute boolean emulatedPosition;
};

[SecureContext, Exposed=Window] interface XRViewerPose : XRPose {
  [SameObject] readonly attribute FrozenArray<XRView> views;
};

enum XRHandedness {
  "none",
  "left",
  "right"
};

enum XRTargetRayMode {
  "gaze",
  "tracked-pointer",
  "screen"
};

[SecureContext, Exposed=Window]
interface XRInputSource {
  readonly attribute XRHandedness handedness;
  readonly attribute XRTargetRayMode targetRayMode;
  [SameObject] readonly attribute XRSpace targetRaySpace;
  [SameObject] readonly attribute XRSpace? gripSpace;
  [SameObject] readonly attribute FrozenArray<DOMString> profiles;
};

[SecureContext, Exposed=Window]
interface XRInputSourceArray {
  iterable<XRInputSource>;
  readonly attribute unsigned long length;
  getter XRInputSource(unsigned long index);
};

[SecureContext, Exposed=Window]
interface XRLayer : EventTarget {};


typedef (WebGLRenderingContext or
         WebGL2RenderingContext) XRWebGLRenderingContext;

dictionary XRWebGLLayerInit {
  boolean antialias = true;
  boolean depth = true;
  boolean stencil = false;
  boolean alpha = true;
  boolean ignoreDepthValues = false;
  double framebufferScaleFactor = 1.0;
};

[SecureContext, Exposed=Window]
interface XRWebGLLayer: XRLayer {
  constructor(XRSession session,
             XRWebGLRenderingContext context,
             optional XRWebGLLayerInit layerInit = {});
  // Attributes
  readonly attribute boolean antialias;
  readonly attribute boolean ignoreDepthValues;
  attribute float? fixedFoveation;

  [SameObject] readonly attribute WebGLFramebuffer? framebuffer;
  readonly attribute unsigned long framebufferWidth;
  readonly attribute unsigned long framebufferHeight;

  // Methods
  XRViewport? getViewport(XRView view);

  // Static Methods
  static double getNativeFramebufferScaleFactor(XRSession session);
};

partial dictionary WebGLContextAttributes {
    boolean xrCompatible = false;
};

partial interface mixin WebGLRenderingContextBase {
    [NewObject] Promise<undefined> makeXRCompatible();
};

[SecureContext, Exposed=Window]
interface XRSessionEvent : Event {
  constructor(DOMString type, XRSessionEventInit eventInitDict);
  [SameObject] readonly attribute XRSession session;
};

dictionary XRSessionEventInit : EventInit {
  required XRSession session;
};

[SecureContext, Exposed=Window]
interface XRInputSourceEvent : Event {
  constructor(DOMString type, XRInputSourceEventInit eventInitDict);
  [SameObject] readonly attribute XRFrame frame;
  [SameObject] readonly attribute XRInputSource inputSource;
};

dictionary XRInputSourceEventInit : EventInit {
  required XRFrame frame;
  required XRInputSource inputSource;
};

[SecureContext, Exposed=Window]
interface XRInputSourcesChangeEvent : Event {
  constructor(DOMString type, XRInputSourcesChangeEventInit eventInitDict);
  [SameObject] readonly attribute XRSession session;
  [SameObject] readonly attribute FrozenArray<XRInputSource> added;
  [SameObject] readonly attribute FrozenArray<XRInputSource> removed;
};

dictionary XRInputSourcesChangeEventInit : EventInit {
  required XRSession session;
  required FrozenArray<XRInputSource> added;
  required FrozenArray<XRInputSource> removed;

};

[SecureContext, Exposed=Window]
interface XRReferenceSpaceEvent : Event {
  constructor(DOMString type, XRReferenceSpaceEventInit eventInitDict);
  [SameObject] readonly attribute XRReferenceSpace referenceSpace;
  [SameObject] readonly attribute XRRigidTransform? transform;
};

dictionary XRReferenceSpaceEventInit : EventInit {
  required XRReferenceSpace referenceSpace;
  XRRigidTransform? transform = null;
};

dictionary XRSessionSupportedPermissionDescriptor: PermissionDescriptor {
  XRSessionMode mode;
};

dictionary XRPermissionDescriptor: PermissionDescriptor {
  XRSessionMode mode;
  sequence<any> requiredFeatures;
  sequence<any> optionalFeatures;
};

[Exposed=Window]
interface XRPermissionStatus: PermissionStatus {
  attribute FrozenArray<any> granted;
};