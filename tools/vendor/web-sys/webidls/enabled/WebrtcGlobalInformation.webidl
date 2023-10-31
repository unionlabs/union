/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

dictionary WebrtcGlobalStatisticsReport {
  sequence<RTCStatsReportInternal> reports;
};

callback WebrtcGlobalStatisticsCallback = undefined (WebrtcGlobalStatisticsReport reports);
callback WebrtcGlobalLoggingCallback = undefined (sequence<DOMString> logMessages);

[ChromeOnly]
interface WebrtcGlobalInformation {

  [Throws]
  static undefined getAllStats(WebrtcGlobalStatisticsCallback callback,
                          optional DOMString pcIdFilter);

  static undefined clearAllStats();

  [Throws]
  static undefined getLogging(DOMString pattern,
                         WebrtcGlobalLoggingCallback callback);

  static undefined clearLogging();

  // NSPR WebRTC Trace debug level (0 - 65535)
  //
  // Notes:
  // - Setting a non-zero debug level turns on gathering of log for file output.
  // - Subsequently setting a zero debug level writes that log to disk.

  static attribute long debugLevel;

  // WebRTC AEC debugging enable
  static attribute boolean aecDebug;

  static readonly attribute DOMString aecDebugLogDir;
};
