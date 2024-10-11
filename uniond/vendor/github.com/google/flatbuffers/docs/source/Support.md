Platform / Language / Feature support    {#flatbuffers_support}
=====================================

FlatBuffers is actively being worked on, which means that certain platform /
language / feature combinations may not be available yet.

This page tries to track those issues, to make informed decisions easier.
In general:

  * Languages: language support beyond the ones created by the original
    FlatBuffer authors typically depends on community contributions.
  * Features: C++ was the first language supported, since our original
    target was high performance game development. It thus has the richest
    feature set, and is likely most robust. Other languages are catching up
    however.
  * Platforms: All language implementations are typically portable to most
    platforms, unless where noted otherwise.

NOTE: this table is a start, it needs to be extended.

Feature                        | C++    | Java  | C#       | Go    | Python | JS    | TS  | C      | PHP | Dart    | Lobster | Rust   | Swift
------------------------------ | ------ | ----- | -------- | ----- | ------ | ----- | --- | ------ | --- | ------- | ------- | ------ | ------
Codegen for all basic features | Yes    | Yes   | Yes      | Yes   | Yes    | Yes   | Yes | Yes    | WiP | Yes     | Yes     | Yes    | Yes
JSON parsing                   | Yes    | No    | No       | No    | No     | No    | No  | Yes    | No  | No      | Yes     | No     | No
Simple mutation                | Yes    | Yes   | Yes      | Yes   | No     | No    | No  | No     | No  | No      | No      | No     | Yes
Reflection                     | Yes    | No    | No       | No    | No     | No    | No  | Basic  | No  | No      | No      | No     | No
Buffer verifier                | Yes    | No    | No       | No    | No     | No    | No  | Yes    | No  | No      | No      | No     | No
Native Object API              | Yes    | No    | Yes      | Yes   | Yes    | Yes   | Yes | No     | No  | Yes     | No      | No     | No
Optional Scalars               | Yes    | Yes   | Yes      | No    | No     | Yes   | Yes | Yes    | No  | No      | Yes     | Yes    | Yes
Flexbuffers                    | Yes    | Yes   | ?        | ?     | ?      | ?     | ?   | ?      | ?   | ?       | ?       | Yes    | ?
Testing: basic                 | Yes    | Yes   | Yes      | Yes   | Yes    | Yes   | Yes | Yes    | ?   | Yes     | Yes     | Yes    | Yes
Testing: fuzz                  | Yes    | No    | No       | Yes   | Yes    | No    | No  | No     | ?   | No      | No      | Yes    | No
Performance:                   | Superb | Great | Great    | Great | Ok     | ?     | ?   | Superb | ?   | ?       | Great   | Superb | Great
Platform: Windows              | VS2010 | Yes   | Yes      | ?     | ?      | ?     | Yes | VS2010 | ?   | Yes     | Yes     | Yes    | No
Platform: Linux                | GCC282 | Yes   | ?        | Yes   | Yes    | ?     | Yes | Yes    | ?   | Yes     | Yes     | Yes    | Yes
Platform: OS X                 | Xcode4 | ?     | ?        | ?     | Yes    | ?     | Yes | Yes    | ?   | Yes     | Yes     | Yes    | Yes
Platform: Android              | NDK10d | Yes   | ?        | ?     | ?      | ?     | ?   | ?      | ?   | Flutter | Yes     | ?      | No
Platform: iOS                  | ?      | ?     | ?        | ?     | ?      | ?     | ?   | ?      | ?   | Flutter | Yes     | ?      | Yes
Engine: Unity                  | ?      | ?     | Yes      | ?     | ?      | ?     | ?   | ?      | ?   | ?       | No      | ?      | No
Primary authors (github)       | aard   | aard  | ev/js/df | rw    | rw     | ew/ev | kr  | mik    | ch  | df      | aard    | rw/cn  | mi/mz

Above | Github username
----- | -----------------------------
aard  | aardappel (previously: gwvo)
ch    | chobie
cn    | caspern
df    | dnfield
ev    | evolutional
ew    | evanw
js    | jonsimantov
kr    | krojew
mi    | mustiikhalil
mik   | mikkelfj
mz    | mzaks
rw    | rw

<br>
