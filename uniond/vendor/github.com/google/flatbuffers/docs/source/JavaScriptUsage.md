Use in JavaScript    {#flatbuffers_guide_use_javascript}
=================

## Before you get started

Before diving into the FlatBuffers usage in JavaScript, it should be noted that
the [Tutorial](@ref flatbuffers_guide_tutorial) page has a complete guide to
general FlatBuffers usage in all of the supported languages
(including JavaScript). This page is specifically designed to cover the nuances
of FlatBuffers usage in JavaScript.

You should also have read the [Building](@ref flatbuffers_guide_building)
documentation to build `flatc` and should be familiar with
[Using the schema compiler](@ref flatbuffers_guide_using_schema_compiler) and
[Writing a schema](@ref flatbuffers_guide_writing_schema).

## FlatBuffers JavaScript library code location

The generated code for the FlatBuffers JavaScript library can be found at 
https://www.npmjs.com/package/flatbuffers. To use it from sources:

1. Run `npm run compile` from the main folder to generate JS files from TS.
1. In your project, install it as a normal dependency, using the flatbuffers
folder as the source.

## Using the FlatBuffers JavaScript library

*Note: See [Tutorial](@ref flatbuffers_guide_tutorial) for a more in-depth
example of how to use FlatBuffers.*

Due to the complexity related with large amounts of JS flavors and module types,
native JS support has been replaced in 2.0 by transpilation from TypeScript.

Please look at [TypeScript usage](@ref flatbuffers_guide_use_typescript) and
transpile your sources to desired JS flavor. The minimal steps to get up and
running with JS are:

1. Generate TS files from `*.fbs` by using the `--ts` option.
1. Transpile resulting TS files to desired JS flavor using `tsc` (see 
   https://www.typescriptlang.org/download for installation instructions).

~~~{.js}
  // Note: These require functions are an example - use your desired module flavor.
  var fs = require('fs');

  var flatbuffers = require('../flatbuffers').flatbuffers;
  var MyGame = require('./monster_generated').MyGame;

  var data = new Uint8Array(fs.readFileSync('monster.dat'));
  var buf = new flatbuffers.ByteBuffer(data);

  var monster = MyGame.Example.Monster.getRootAsMonster(buf);

  //--------------------------------------------------------------------------//

  // Note: This code is an example of browser-based HTML/JavaScript. See above
  //       for the code using JavaScript module loaders (e.g. Node.js).
  <script src="../js/flatbuffers.js"></script>
  <script src="monster_generated.js"></script>
  <script>
    function readFile() {
      var reader = new FileReader(); // This example uses the HTML5 FileReader.
      var file = document.getElementById(
          'file_input').files[0]; // "monster.dat" from the HTML <input> field.

      reader.onload = function() { // Executes after the file is read.
        var data = new Uint8Array(reader.result);

        var buf = new flatbuffers.ByteBuffer(data);

        var monster = MyGame.Example.Monster.getRootAsMonster(buf);
      }

      reader.readAsArrayBuffer(file);
    }
  </script>

  // Open the HTML file in a browser and select "monster.dat" from with the
  // <input> field.
  <input type="file" id="file_input" onchange="readFile();">
~~~

Now you can access values like this:

~~~{.js}
  var hp = monster.hp();
  var pos = monster.pos();
~~~

## Text parsing FlatBuffers in JavaScript

There currently is no support for parsing text (Schema's and JSON) directly
from JavaScript.
