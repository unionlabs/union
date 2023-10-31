/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtaone at http://mozilla.org/MPL/2.0/. */

/**
 * Options for nsINativeOSFileInternals::Read
 */
dictionary NativeOSFileReadOptions
{
  /**
   * If specified, convert the raw bytes to a String
   * with the specified encoding. Otherwise, return
   * the raw bytes as a TypedArray.
   */
  DOMString? encoding;

  /**
   * If specified, limit the number of bytes to read.
   */
  unsigned long long? bytes;
};

/**
 * Options for nsINativeOSFileInternals::WriteAtomic
 */
dictionary NativeOSFileWriteAtomicOptions
{
  /**
   * If specified, specify the number of bytes to write.
   * NOTE: This takes (and should take) a uint64 here but the actual
   * value is limited to int32. This needs to be fixed, see Bug 1063635.
   */
  unsigned long long? bytes;

  /**
   * If specified, write all data to a temporary file in the
   * |tmpPath|. Else, write to the given path directly.
   */
  DOMString? tmpPath = null;

  /**
   * If specified and true, a failure will occur if the file
   * already exists in the given path.
   */
  boolean noOverwrite = false;

  /**
   * If specified and true, this will sync any buffered data
   * for the file to disk. This might be slower, but safer.
   */
  boolean flush = false;

  /**
   * If specified, this will backup the destination file as
   * specified.
   */
  DOMString? backupTo = null;
};
