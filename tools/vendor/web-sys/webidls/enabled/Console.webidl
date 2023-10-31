/* -*- Mode: IDL; tab-width: 8; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* vim: set ts=2 et sw=2 tw=80: */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * For more information on this interface, please see
 * https://console.spec.whatwg.org/#console-namespace
 */

[Exposed=(Window,Worker,WorkerDebugger,Worklet,System),
 ClassString="Console",
 ProtoObjectHack]
namespace console {

  // NOTE: if you touch this namespace, remember to update the ConsoleInstance
  // interface as well!

  // Logging
  [UseCounter]
  undefined assert(optional boolean condition = false, any... data);
  [UseCounter]
  undefined clear();
  [UseCounter]
  undefined count(optional DOMString label = "default");
  [UseCounter]
  undefined countReset(optional DOMString label = "default");
  [UseCounter]
  undefined debug(any... data);
  [UseCounter]
  undefined error(any... data);
  [UseCounter]
  undefined info(any... data);
  [UseCounter]
  undefined log(any... data);
  [UseCounter]
  undefined table(any... data); // FIXME: The spec is still unclear about this.
  [UseCounter]
  undefined trace(any... data);
  [UseCounter]
  undefined warn(any... data);
  [UseCounter]
  undefined dir(any... data); // FIXME: This doesn't follow the spec yet.
  [UseCounter]
  undefined dirxml(any... data);

  // Grouping
  [UseCounter]
  undefined group(any... data);
  [UseCounter]
  undefined groupCollapsed(any... data);
  [UseCounter]
  undefined groupEnd();

  // Timing
  [UseCounter]
  undefined time(optional DOMString label = "default");
  [UseCounter]
  undefined timeLog(optional DOMString label = "default", any... data);
  [UseCounter]
  undefined timeEnd(optional DOMString label = "default");

  // Mozilla only or Webcompat methods

  [UseCounter]
  undefined _exception(any... data);
  [UseCounter]
  undefined timeStamp(optional any data);

  [UseCounter]
  undefined profile(any... data);
  [UseCounter]
  undefined profileEnd(any... data);

  // invalid widl
  // [ChromeOnly]
  // const boolean IS_NATIVE_CONSOLE = true;

  [ChromeOnly, NewObject]
  ConsoleInstance createInstance(optional ConsoleInstanceOptions options);
};

// This is used to propagate console events to the observers.
dictionary ConsoleEvent {
  (unsigned long long or DOMString) ID;
  (unsigned long long or DOMString) innerID;
  DOMString consoleID = "";
  DOMString addonId = "";
  DOMString level = "";
  DOMString filename = "";
  unsigned long lineNumber = 0;
  unsigned long columnNumber = 0;
  DOMString functionName = "";
  double timeStamp = 0;
  sequence<any> arguments;
  sequence<DOMString?> styles;
  boolean private = false;
  // stacktrace is handled via a getter in some cases so we can construct it
  // lazily.  Note that we're not making this whole thing an interface because
  // consumers expect to see own properties on it, which would mean making the
  // props unforgeable, which means lots of JSFunction allocations.  Maybe we
  // should fix those consumers, of course....
  // sequence<ConsoleStackEntry> stacktrace;
  DOMString groupName = "";
  any timer = null;
  any counter = null;
  DOMString prefix = "";
};

// Event for profile operations
dictionary ConsoleProfileEvent {
  DOMString action = "";
  sequence<any> arguments;
};

// This dictionary is used to manage stack trace data.
dictionary ConsoleStackEntry {
  DOMString filename = "";
  unsigned long lineNumber = 0;
  unsigned long columnNumber = 0;
  DOMString functionName = "";
  DOMString? asyncCause;
};

dictionary ConsoleTimerStart {
  DOMString name = "";
};

dictionary ConsoleTimerLogOrEnd {
  DOMString name = "";
  double duration = 0;
};

dictionary ConsoleTimerError {
  DOMString error = "";
  DOMString name = "";
};

dictionary ConsoleCounter {
  DOMString label = "";
  unsigned long count = 0;
};

dictionary ConsoleCounterError {
  DOMString label = "";
  DOMString error = "";
};

[ChromeOnly,
 Exposed=(Window,Worker,WorkerDebugger,Worklet,System)]
// This is basically a copy of the console namespace.
interface ConsoleInstance {
  // Logging
  undefined assert(optional boolean condition = false, any... data);
  undefined clear();
  undefined count(optional DOMString label = "default");
  undefined countReset(optional DOMString label = "default");
  undefined debug(any... data);
  undefined error(any... data);
  undefined info(any... data);
  undefined log(any... data);
  undefined table(any... data); // FIXME: The spec is still unclear about this.
  undefined trace(any... data);
  undefined warn(any... data);
  undefined dir(any... data); // FIXME: This doesn't follow the spec yet.
  undefined dirxml(any... data);

  // Grouping
  undefined group(any... data);
  undefined groupCollapsed(any... data);
  undefined groupEnd();

  // Timing
  undefined time(optional DOMString label = "default");
  undefined timeLog(optional DOMString label = "default", any... data);
  undefined timeEnd(optional DOMString label = "default");

  // Mozilla only or Webcompat methods

  undefined _exception(any... data);
  undefined timeStamp(optional any data);

  undefined profile(any... data);
  undefined profileEnd(any... data);
};

callback ConsoleInstanceDumpCallback = undefined (DOMString message);

enum ConsoleLogLevel {
  "All", "Debug", "Log", "Info", "Clear", "Trace", "TimeLog", "TimeEnd", "Time",
  "Group", "GroupEnd", "Profile", "ProfileEnd", "Dir", "Dirxml", "Warn", "Error",
  "Off"
};

dictionary ConsoleInstanceOptions {
  // An optional function to intercept all strings written to stdout.
  ConsoleInstanceDumpCallback dump;

  // An optional prefix string to be printed before the actual logged message.
  DOMString prefix = "";

  // An ID representing the source of the message. Normally the inner ID of a
  // DOM window.
  DOMString innerID = "";

  // String identified for the console, this will be passed through the console
  // notifications.
  DOMString consoleID = "";

  // Identifier that allows to filter which messages are logged based on their
  // log level.
  ConsoleLogLevel maxLogLevel;

  // String pref name which contains the level to use for maxLogLevel. If the
  // pref doesn't exist, gets removed or it is used in workers, the maxLogLevel
  // will default to the value passed to this constructor (or "all" if it wasn't
  // specified).
  DOMString maxLogLevelPref = "";
};

enum ConsoleLevel { "log", "warning", "error" };

// this interface is just for testing
partial interface ConsoleInstance {
  [ChromeOnly]
  undefined reportForServiceWorkerScope(DOMString scope, DOMString message,
                                   DOMString filename, unsigned long lineNumber,
                                   unsigned long columnNumber,
                                   ConsoleLevel level);
};
