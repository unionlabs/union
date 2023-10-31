/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://www.whatwg.org/specs/web-apps/current-work/
 *
 * © Copyright 2004-2011 Apple Computer, Inc., Mozilla Foundation, and
 * Opera Software ASA. You are granted a license to use, reproduce
 * and create derivative works of this document.
 */

enum CanvasWindingRule { "nonzero", "evenodd" };

dictionary ContextAttributes2D {
  // whether or not we're planning to do a lot of readback operations
  boolean willReadFrequently = false;
  // signal if the canvas contains an alpha channel
  boolean alpha = true;
};

dictionary HitRegionOptions {
  Path2D? path = null;
  DOMString id = "";
  Element? control = null;
};

typedef (HTMLImageElement or
         SVGImageElement) HTMLOrSVGImageElement;

typedef (HTMLOrSVGImageElement or
         HTMLCanvasElement or
         HTMLVideoElement or
         ImageBitmap) CanvasImageSource;

interface CanvasRenderingContext2D {

  // back-reference to the canvas.  Might be null if we're not
  // associated with a canvas.
  readonly attribute HTMLCanvasElement? canvas;

  // Show the caret if appropriate when drawing
  [Func="CanvasUtils::HasDrawWindowPrivilege"]
  const unsigned long DRAWWINDOW_DRAW_CARET   = 0x01;
  // Don't flush pending layout notifications that could otherwise
  // be batched up
  [Func="CanvasUtils::HasDrawWindowPrivilege"]
  const unsigned long DRAWWINDOW_DO_NOT_FLUSH = 0x02;
  // Draw scrollbars and scroll the viewport if they are present
  [Func="CanvasUtils::HasDrawWindowPrivilege"]
  const unsigned long DRAWWINDOW_DRAW_VIEW    = 0x04;
  // Use the widget layer manager if available. This means hardware
  // acceleration may be used, but it might actually be slower or
  // lower quality than normal. It will however more accurately reflect
  // the pixels rendered to the screen.
  [Func="CanvasUtils::HasDrawWindowPrivilege"]
  const unsigned long DRAWWINDOW_USE_WIDGET_LAYERS = 0x08;
  // Don't synchronously decode images - draw what we have
  [Func="CanvasUtils::HasDrawWindowPrivilege"]
  const unsigned long DRAWWINDOW_ASYNC_DECODE_IMAGES = 0x10;

  /**
   * Renders a region of a window into the canvas.  The contents of
   * the window's viewport are rendered, ignoring viewport clipping
   * and scrolling.
   *
   * @param x
   * @param y
   * @param w
   * @param h specify the area of the window to render, in CSS
   * pixels.
   *
   * @param backgroundColor the canvas is filled with this color
   * before we render the window into it. This color may be
   * transparent/translucent. It is given as a CSS color string
   * (e.g., rgb() or rgba()).
   *
   * @param flags Used to better control the drawWindow call.
   * Flags can be ORed together.
   *
   * Of course, the rendering obeys the current scale, transform and
   * globalAlpha values.
   *
   * Hints:
   * -- If 'rgba(0,0,0,0)' is used for the background color, the
   * drawing will be transparent wherever the window is transparent.
   * -- Top-level browsed documents are usually not transparent
   * because the user's background-color preference is applied,
   * but IFRAMEs are transparent if the page doesn't set a background.
   * -- If an opaque color is used for the background color, rendering
   * will be faster because we won't have to compute the window's
   * transparency.
   *
   * This API cannot currently be used by Web content. It is chrome
   * and Web Extensions (with a permission) only.
   */
  [Throws, Func="CanvasUtils::HasDrawWindowPrivilege"]
  undefined drawWindow(Window window, double x, double y, double w, double h,
                  DOMString bgColor, optional unsigned long flags = 0);

  /**
   * This causes a context that is currently using a hardware-accelerated
   * backend to fallback to a software one. All state should be preserved.
   */
  [ChromeOnly]
  undefined demote();
};

CanvasRenderingContext2D includes CanvasState;
CanvasRenderingContext2D includes CanvasTransform;
CanvasRenderingContext2D includes CanvasCompositing;
CanvasRenderingContext2D includes CanvasImageSmoothing;
CanvasRenderingContext2D includes CanvasFillStrokeStyles;
CanvasRenderingContext2D includes CanvasShadowStyles;
CanvasRenderingContext2D includes CanvasFilters;
CanvasRenderingContext2D includes CanvasRect;
CanvasRenderingContext2D includes CanvasDrawPath;
CanvasRenderingContext2D includes CanvasUserInterface;
CanvasRenderingContext2D includes CanvasText;
CanvasRenderingContext2D includes CanvasDrawImage;
CanvasRenderingContext2D includes CanvasImageData;
CanvasRenderingContext2D includes CanvasPathDrawingStyles;
CanvasRenderingContext2D includes CanvasTextDrawingStyles;
CanvasRenderingContext2D includes CanvasPathMethods;
CanvasRenderingContext2D includes CanvasHitRegions;

interface mixin CanvasState {
  // state
  undefined save(); // push state on state stack
  undefined restore(); // pop state stack and restore state
};

interface mixin CanvasTransform {
  // transformations (default transform is the identity matrix)
// NOT IMPLEMENTED           attribute SVGMatrix currentTransform;
  [Throws, LenientFloat]
  undefined scale(double x, double y);
  [Throws, LenientFloat]
  undefined rotate(double angle);
  [Throws, LenientFloat]
  undefined translate(double x, double y);
  [Throws, LenientFloat]
  undefined transform(double a, double b, double c, double d, double e, double f);
  [Throws, LenientFloat]
  undefined setTransform(double a, double b, double c, double d, double e, double f);
  [Throws]
  undefined resetTransform();
  [NewObject, Throws]
  DOMMatrix getTransform();
};

[NoInterfaceObject]
interface mixin CanvasCompositing {
  attribute unrestricted double globalAlpha; // (default 1.0)
  [Throws]
  attribute DOMString globalCompositeOperation; // (default source-over)
};

interface mixin CanvasImageSmoothing {
  // drawing images
  attribute boolean imageSmoothingEnabled;
};

interface mixin CanvasFillStrokeStyles {
  // colors and styles (see also the CanvasPathDrawingStyles interface)
  attribute (DOMString or CanvasGradient or CanvasPattern) strokeStyle; // (default black)
  attribute (DOMString or CanvasGradient or CanvasPattern) fillStyle; // (default black)
  [NewObject]
  CanvasGradient createLinearGradient(double x0, double y0, double x1, double y1);
  [NewObject, Throws]
  CanvasGradient createRadialGradient(double x0, double y0, double r0, double x1, double y1, double r1);
  [NewObject, Throws]
  CanvasPattern? createPattern(CanvasImageSource image, [TreatNullAs=EmptyString] DOMString repetition);
};

interface mixin CanvasShadowStyles {
  [LenientFloat]
  attribute double shadowOffsetX; // (default 0)
  [LenientFloat]
  attribute double shadowOffsetY; // (default 0)
  [LenientFloat]
  attribute double shadowBlur; // (default 0)
  attribute DOMString shadowColor; // (default transparent black)
};

interface mixin CanvasFilters {
  [Pref="canvas.filters.enabled", SetterThrows]
  attribute DOMString filter; // (default empty string = no filter)
};

interface mixin CanvasRect {
  [LenientFloat]
  undefined clearRect(double x, double y, double w, double h);
  [LenientFloat]
  undefined fillRect(double x, double y, double w, double h);
  [LenientFloat]
  undefined strokeRect(double x, double y, double w, double h);
};

interface mixin CanvasDrawPath {
  // path API (see also CanvasPathMethods)
  undefined beginPath();
  undefined fill(optional CanvasWindingRule winding = "nonzero");
  undefined fill(Path2D path, optional CanvasWindingRule winding = "nonzero");
  undefined stroke();
  undefined stroke(Path2D path);
  undefined clip(optional CanvasWindingRule winding = "nonzero");
  undefined clip(Path2D path, optional CanvasWindingRule winding = "nonzero");
// NOT IMPLEMENTED  undefined resetClip();
  [NeedsSubjectPrincipal]
  boolean isPointInPath(unrestricted double x, unrestricted double y, optional CanvasWindingRule winding = "nonzero");
  [NeedsSubjectPrincipal] // Only required because overloads can't have different extended attributes.
  boolean isPointInPath(Path2D path, unrestricted double x, unrestricted double y, optional CanvasWindingRule winding = "nonzero");
  [NeedsSubjectPrincipal]
  boolean isPointInStroke(double x, double y);
  [NeedsSubjectPrincipal] // Only required because overloads can't have different extended attributes.
  boolean isPointInStroke(Path2D path, unrestricted double x, unrestricted double y);
};

interface mixin CanvasUserInterface {
  [Pref="canvas.focusring.enabled", Throws] undefined drawFocusIfNeeded(Element element);
// NOT IMPLEMENTED  undefined drawSystemFocusRing(Path path, HTMLElement element);
  [Pref="canvas.customfocusring.enabled"] boolean drawCustomFocusRing(Element element);
// NOT IMPLEMENTED  boolean drawCustomFocusRing(Path path, HTMLElement element);
// NOT IMPLEMENTED  undefined scrollPathIntoView();
// NOT IMPLEMENTED  undefined scrollPathIntoView(Path path);
};

interface mixin CanvasText {
  // text (see also the CanvasPathDrawingStyles interface)
  [Throws, LenientFloat]
  undefined fillText(DOMString text, double x, double y, optional double maxWidth);
  [Throws, LenientFloat]
  undefined strokeText(DOMString text, double x, double y, optional double maxWidth);
  [NewObject, Throws]
  TextMetrics measureText(DOMString text);
};

interface mixin CanvasDrawImage {
  [Throws, LenientFloat]
  undefined drawImage(CanvasImageSource image, double dx, double dy);
  [Throws, LenientFloat]
  undefined drawImage(CanvasImageSource image, double dx, double dy, double dw, double dh);
  [Throws, LenientFloat]
  undefined drawImage(CanvasImageSource image, double sx, double sy, double sw, double sh, double dx, double dy, double dw, double dh);
};

interface mixin CanvasImageData {
  // pixel manipulation
  [NewObject, Throws]
  ImageData createImageData(double sw, double sh);
  [NewObject, Throws]
  ImageData createImageData(ImageData imagedata);
  [NewObject, Throws, NeedsSubjectPrincipal]
  ImageData getImageData(double sx, double sy, double sw, double sh);
  [Throws]
  undefined putImageData(ImageData imagedata, double dx, double dy);
  [Throws]
  undefined putImageData(ImageData imagedata, double dx, double dy, double dirtyX, double dirtyY, double dirtyWidth, double dirtyHeight);
};

interface mixin CanvasPathDrawingStyles {
  // line caps/joins
  [LenientFloat]
  attribute double lineWidth; // (default 1)
  attribute DOMString lineCap; // "butt", "round", "square" (default "butt")
  [GetterThrows]
  attribute DOMString lineJoin; // "round", "bevel", "miter" (default "miter")
  [LenientFloat]
  attribute double miterLimit; // (default 10)

  // dashed lines
  [LenientFloat, Throws] undefined setLineDash(sequence<double> segments); // default empty
  sequence<double> getLineDash();
  [LenientFloat] attribute double lineDashOffset;
};

interface mixin CanvasTextDrawingStyles {
  // text
  [SetterThrows]
  attribute DOMString font; // (default 10px sans-serif)
  attribute DOMString textAlign; // "start", "end", "left", "right", "center" (default: "start")
  attribute DOMString textBaseline; // "top", "hanging", "middle", "alphabetic", "ideographic", "bottom" (default: "alphabetic")
};

interface mixin CanvasPathMethods {
  // shared path API methods
  undefined closePath();
  [LenientFloat]
  undefined moveTo(double x, double y);
  [LenientFloat]
  undefined lineTo(double x, double y);
  [LenientFloat]
  undefined quadraticCurveTo(double cpx, double cpy, double x, double y);

  [LenientFloat]
  undefined bezierCurveTo(double cp1x, double cp1y, double cp2x, double cp2y, double x, double y);

  [Throws, LenientFloat]
  undefined arcTo(double x1, double y1, double x2, double y2, double radius);
// NOT IMPLEMENTED  [LenientFloat] undefined arcTo(double x1, double y1, double x2, double y2, double radiusX, double radiusY, double rotation);

  [LenientFloat]
  undefined rect(double x, double y, double w, double h);

  [Throws, LenientFloat]
  undefined arc(double x, double y, double radius, double startAngle, double endAngle, optional boolean anticlockwise = false);

  [Throws, LenientFloat]
  undefined ellipse(double x, double y, double radiusX, double radiusY, double rotation, double startAngle, double endAngle, optional boolean anticlockwise = false);
};

interface mixin CanvasHitRegions {
  // hit regions
  [Pref="canvas.hitregions.enabled", Throws] undefined addHitRegion(optional HitRegionOptions options);
  [Pref="canvas.hitregions.enabled"] undefined removeHitRegion(DOMString id);
  [Pref="canvas.hitregions.enabled"] undefined clearHitRegions();
};

interface CanvasGradient {
  // opaque object
  [Throws]
  // addColorStop should take a double
  undefined addColorStop(float offset, DOMString color);
};

interface CanvasPattern {
  // opaque object
  // [Throws, LenientFloat] - could not do this overload because of bug 1020975
  // undefined setTransform(double a, double b, double c, double d, double e, double f);

  // No throw necessary here - SVGMatrix is always good.
  undefined setTransform(SVGMatrix matrix);
};

interface TextMetrics {

  // x-direction
  readonly attribute double width; // advance width

  readonly attribute double actualBoundingBoxLeft;
  readonly attribute double actualBoundingBoxRight;

  readonly attribute double fontBoundingBoxAscent;
  readonly attribute double fontBoundingBoxDescent;

  // y-direction
  readonly attribute double actualBoundingBoxAscent;
  readonly attribute double actualBoundingBoxDescent;

  /*
   * NOT IMPLEMENTED YET
  
  readonly attribute double emHeightAscent;
  readonly attribute double emHeightDescent;
  readonly attribute double hangingBaseline;
  readonly attribute double alphabeticBaseline;
  readonly attribute double ideographicBaseline;
  */

};

[Pref="canvas.path.enabled",
 Constructor,
 Constructor(Path2D other),
 Constructor(DOMString pathString)]
interface Path2D
{
  undefined addPath(Path2D path, optional SVGMatrix transformation);
};
Path2D includes CanvasPathMethods;
