/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * All functions on Directory that accept DOMString arguments for file or
 * directory names only allow relative path to current directory itself. The
 * path should be a descendent path like "path/to/file.txt" and not contain a
 * segment of ".." or ".". So the paths aren't allowed to walk up the directory
 * tree. For example, paths like "../foo", "..", "/foo/bar" or "foo/../bar" are
 * not allowed.
 *
 * http://w3c.github.io/filesystem-api/#idl-def-Directory
 * https://microsoftedge.github.io/directory-upload/proposal.html#directory-interface
 */

// This chromeConstructor is used by the MockFilePicker for testing only.
[ChromeConstructor(DOMString path),
 Exposed=(Window,Worker)]
interface Directory {
  /*
   * The leaf name of the directory.
   */
  [Throws]
  readonly attribute DOMString name;
};

[Exposed=(Window,Worker)]
partial interface Directory {
  // Already defined in the main interface declaration:
  //readonly attribute DOMString name;

  /*
   * The path of the Directory (includes both its basename and leafname).
   * The path begins with the name of the ancestor Directory that was
   * originally exposed to content (say via a directory picker) and traversed
   * to obtain this Directory.  Full filesystem paths are not exposed to
   * unprivilaged content.
   */
  [Throws]
  readonly attribute DOMString path;

  /*
   * Getter for the immediate children of this directory.
   */
  [Throws]
  Promise<sequence<(File or Directory)>> getFilesAndDirectories();

  [Throws]
  Promise<sequence<File>> getFiles(optional boolean recursiveFlag = false);
};
