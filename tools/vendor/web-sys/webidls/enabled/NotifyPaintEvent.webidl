/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/**
 * The NotifyPaintEvent interface is used for the MozDOMAfterPaint
 * event, which fires at a window when painting has happened in
 * that window.
 */
[ChromeOnly]
interface NotifyPaintEvent : Event
{
  /**
   * Get a list of rectangles which are affected. The rectangles are
   * in CSS pixels relative to the viewport origin.
   */
  [ChromeOnly, NeedsCallerType]
  readonly attribute DOMRectList clientRects;

  /**
   * Get the bounding box of the rectangles which are affected. The rectangle
   * is in CSS pixels relative to the viewport origin.
   */
  [ChromeOnly, NeedsCallerType]
  readonly attribute DOMRect boundingClientRect;

  [ChromeOnly, NeedsCallerType]
  readonly attribute PaintRequestList paintRequests;

  [ChromeOnly, NeedsCallerType]
  readonly attribute unsigned long long transactionId;

  [ChromeOnly, NeedsCallerType]
  readonly attribute DOMHighResTimeStamp paintTimeStamp;
};
