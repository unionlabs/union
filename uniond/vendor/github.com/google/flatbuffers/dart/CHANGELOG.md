# Changelog

## 23.5.26

- omit type annotationes for local variables (#7067, #7069, #7070)
- remove BSD 3-clause license (#7073)
- correctly parse lists of enums (#7157)
- align naming conventions for generated code (#7187)
- add `putBool` to fix errors when serializing structs with booleans (#7359)
- fix handling of +/-inf defaults in codegen (#7588)
- fix import issues in generated code (#7621)
- Fix incorrect storage of floats as ints in some cases (#7703)
- add final modifiers to the library implementation (#7943)

## 2.0.5

- switch to null safety (#6696)
- add Object APIs (pack/unpack) (#6682, #6723, #6846)
- add custom builder buffer allocator support (#6711)
- add `Builder.size()` - finished buffer size (#6403)
- make `writeString()` argument non-nullable (#6737)
- make tables fixed size (expect the number of fields when creating) (#6735)
- make table deduplication optional (param `deduplicateTables`) (#6734)
- change `Builder.reset()` to reuse an existing buffer (#6661)
- change table building to assert() instead of exceptions (#6754)
- optimize `writeString()` for ASCII (param `asciiOptimization`) (#6736)
- change `StringReader` to make ASCII optimization optional (param `asciiOptimization`) (#6758)
- change `[byte]` and `[ubyte]` representation to `dart:typed_data` `Int8List` and `Uint8List` (#6839)
- rename `lowFinish()` to `buffer` getter (#6712)
- fix `Builder._writeString()` - always write trailing zero byte (#6390)
- fix `Builder.reset()` - clear vTables (#6386)
- make sure added padding is zeroed, same as in C++ (#6716)
- many performance improvements (#6755)

## 1.9.2

- Ensure `_writeString` adds enough padding to null terminate strings.

## 1.9.1

- Changed constant identifiers to be compatible with Dart 2.x
- No longer supports Dart 1.x

## 1.9.0

- Initial release, supports Dart 1.x and many dev versions of Dart 2.x
