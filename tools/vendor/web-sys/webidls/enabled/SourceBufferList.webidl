/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * http://dvcs.w3.org/hg/html-media/raw-file/default/media-source/media-source.html
 *
 * Copyright © 2012 W3C® (MIT, ERCIM, Keio), All Rights Reserved. W3C
 * liability, trademark and document use rules apply.
 */

[Func="mozilla::dom::MediaSource::Enabled"]
interface SourceBufferList : EventTarget {
  readonly attribute unsigned long length;
  attribute EventHandler onaddsourcebuffer;
  attribute EventHandler onremovesourcebuffer;
  getter SourceBuffer (unsigned long index);
};
