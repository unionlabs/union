import assert from 'assert'
import * as flatbuffers from 'flatbuffers';
import { Foo } from './required-strings/foo.js';


var builder = new flatbuffers.Builder();

function main() {
  testMissingFirstRequiredString();
  builder.clear();
  testMissingSecondRequiredString();
}

function testMissingFirstRequiredString() {
	const undefined_string = builder.createString(undefined);
	const defined_string = builder.createString('cat');

	assert.throws(() => Foo.createFoo(
		builder, undefined_string, defined_string
	));
}

function testMissingSecondRequiredString() {
	const defined_string = builder.createString('cat');
	const undefined_string = builder.createString(undefined);

	assert.throws(() => Foo.createFoo(
		builder, defined_string, undefined_string
	));
}

main();
