/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * This file declares data structures used to communicate checkerboard reports
 * from C++ code to about:checkerboard (see bug 1238042). These dictionaries
 * are NOT exposed to standard web content.
 */

enum CheckerboardReason {
  "severe",
  "recent"
};

// Individual checkerboard report. Contains fields for the severity of the
// checkerboard event, the timestamp at which it was reported, the detailed
// log of the event, and the reason this report was saved (currently either
// "severe" or "recent").
dictionary CheckerboardReport {
  unsigned long severity;
  DOMTimeStamp timestamp; // milliseconds since epoch
  DOMString log;
  CheckerboardReason reason;
};

// The guard function only allows creation of this interface on the
// about:checkerboard page, and only if it's in the parent process.
[Func="mozilla::dom::CheckerboardReportService::IsEnabled",
 Constructor]
interface CheckerboardReportService {
  /**
   * Gets the available checkerboard reports.
   */
  sequence<CheckerboardReport> getReports();

  /**
   * Gets the state of the apz.record_checkerboarding pref.
   */
  boolean isRecordingEnabled();

  /**
   * Sets the state of the apz.record_checkerboarding pref.
   */
  undefined setRecordingEnabled(boolean aEnabled);

  /**
   * Flush any in-progress checkerboard reports. Since this happens
   * asynchronously, the caller may register an observer with the observer
   * service to be notified when this operation is complete. The observer should
   * listen for the topic "APZ:FlushActiveCheckerboard:Done". Upon receiving
   * this notification, the caller may call getReports() to obtain the flushed
   * reports, along with any other reports that are available.
   */
  undefined flushActiveReports();
};
