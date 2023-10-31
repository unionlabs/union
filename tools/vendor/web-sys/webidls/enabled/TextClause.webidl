/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

[ChromeOnly]
interface TextClause
{
  // The start offset of TextClause
  readonly attribute long startOffset;

  // The end offset of TextClause
  readonly attribute long endOffset;

  // If the TextClause is Caret or not
  readonly attribute boolean isCaret;

  // If the TextClause is TargetClause or not
  readonly attribute boolean isTargetClause;
};
