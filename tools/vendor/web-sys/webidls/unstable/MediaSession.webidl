[Exposed=Window]
partial interface Navigator {
  [SameObject] readonly attribute MediaSession mediaSession;
};

enum MediaSessionPlaybackState {
  "none",
  "paused",
  "playing"
};

enum MediaSessionAction {
  "play",
  "pause",
  "seekbackward",
  "seekforward",
  "previoustrack",
  "nexttrack",
  "skipad",
  "stop",
  "seekto",
  "togglemicrophone",
  "togglecamera",
  "hangup"
};

callback MediaSessionActionHandler = undefined(MediaSessionActionDetails details);

[Exposed=Window]
interface MediaSession {
  attribute MediaMetadata? metadata;

  attribute MediaSessionPlaybackState playbackState;

  undefined setActionHandler(MediaSessionAction action, MediaSessionActionHandler? handler);

  undefined setPositionState(optional MediaPositionState state = {});

  undefined setMicrophoneActive(boolean active);

  undefined setCameraActive(boolean active);
};

[Exposed=Window]
interface MediaMetadata {
  constructor(optional MediaMetadataInit init = {});
  attribute DOMString title;
  attribute DOMString artist;
  attribute DOMString album;
  attribute FrozenArray<MediaImage> artwork;
};

dictionary MediaMetadataInit {
  DOMString title = "";
  DOMString artist = "";
  DOMString album = "";
  sequence<MediaImage> artwork = [];
};

dictionary MediaImage {
  required USVString src;
  DOMString sizes = "";
  DOMString type = "";
};

dictionary MediaPositionState {
  double duration;
  double playbackRate;
  double position;
};

dictionary MediaSessionActionDetails {
  required MediaSessionAction action;
  double? seekOffset;
  double? seekTime;
  boolean? fastSeek;
};
