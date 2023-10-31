/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * The origin of this IDL file is
 * https://wicg.github.io/media-capabilities/
 *
 * Copyright © 2018 the Contributors to the Media Capabilities Specification
 */

dictionary MediaConfiguration {
  VideoConfiguration video;
  AudioConfiguration audio;
};

dictionary MediaDecodingConfiguration : MediaConfiguration {
  required MediaDecodingType type;
};

dictionary MediaEncodingConfiguration : MediaConfiguration {
  required MediaEncodingType type;
};

enum MediaDecodingType {
  "file",
  "media-source",
};

enum MediaEncodingType {
  "record",
  "transmission"
};

// all members are specified as required in the spec.
// We enforce that requirement in the MediaCapabilities code instead
// See https://github.com/heycam/webidl/issues/76
dictionary VideoConfiguration {
  DOMString contentType;
  unsigned long width;
  unsigned long height;
  unsigned long long bitrate;
  DOMString framerate;
};

// contentType member is specified as required in the spec.
// We enforce that requirement in the MediaCapabilities code instead
// See https://github.com/heycam/webidl/issues/76
dictionary AudioConfiguration {
  DOMString contentType;
  DOMString channels;
  unsigned long long bitrate;
  unsigned long samplerate;
};

[Exposed=(Window, Worker), Func="mozilla::dom::MediaCapabilities::Enabled",
 HeaderFile="mozilla/dom/MediaCapabilities.h"]
interface MediaCapabilitiesInfo {
  readonly attribute boolean supported;
  readonly attribute boolean smooth;
  readonly attribute boolean powerEfficient;
};

[Exposed=(Window, Worker), Func="mozilla::dom::MediaCapabilities::Enabled"]
interface MediaCapabilities {
  [NewObject]
  Promise<MediaCapabilitiesInfo> decodingInfo(MediaDecodingConfiguration configuration);
  [NewObject]
  Promise<MediaCapabilitiesInfo> encodingInfo(MediaEncodingConfiguration configuration);
};
