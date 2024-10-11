import assert from 'assert'
import * as flatbuffers from 'flatbuffers'

import typescript_include from './typescript_include_generated.cjs'

const foobar = typescript_include.foobar;

function main() {
  // Validate the enums.
  assert.strictEqual(foobar.Abc.a, 0);
  assert.strictEqual(foobar.class_.arguments_, 0);

  // Validate building a table.
  var fbb = new flatbuffers.Builder(1);
  var name = fbb.createString("Foo Bar");

  foobar.Tab.startTab(fbb);
  foobar.Tab.addAbc(fbb, foobar.Abc.a);
  foobar.Tab.addArg(fbb, foobar.class_.arguments_);
  foobar.Tab.addName(fbb, name);
  var tab = foobar.Tab.endTab(fbb);

  fbb.finish(tab);

  // Call as a sanity check. Would be better to validate actual output here.
  fbb.asUint8Array();

  console.log('FlatBuffers Bazel Import test: completed successfully');
}

main();
