/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * Source: https://streams.spec.whatwg.org/#ws-class-definition
 */
 
[Exposed=(Window,Worker,Worklet), Transferable]
interface WritableStream {
  constructor(optional object underlyingSink, optional QueuingStrategy strategy = {});

  readonly attribute boolean locked;

  Promise<undefined> abort(optional any reason);
  Promise<undefined> close();
  WritableStreamDefaultWriter getWriter();
};
