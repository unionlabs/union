# ADR 050: SIGN_MODE_TEXTUAL: Annex 2 XXX

## Changelog

* Oct 3, 2022: Initial Draft

## Status

DRAFT

## Abstract

This annex provides normative guidance on how devices should render a
`SIGN_MODE_TEXTUAL` document.

## Context

`SIGN_MODE_TEXTUAL` allows a legible version of a transaction to be signed
on a hardware security device, such as a Ledger. Early versions of the
design rendered transactions directly to lines of ASCII text, but this
proved awkward from its in-band signaling, and for the need to display
Unicode text within the transaction.

## Decision

`SIGN_MODE_TEXTUAL` renders to an abstract representation, leaving it
up to device-specific software how to present this representation given the
capabilities, limitations, and conventions of the device.

We offer the following normative guidance:

1. The presentation should be as legible as possible to the user, given
the capabilities of the device. If legibility could be sacrificed for other
properties, we would recommend just using some other signing mode.
Legibility should focus on the common case - it is okay for unusual cases
to be less legible.

2. The presentation should be invertible if possible without substantial
sacrifice of legibility.  Any change to the rendered data should result
in a visible change to the presentation. This extends the integrity of the
signing to user-visible presentation.

3. The presentation should follow normal conventions of the device,
without sacrificing legibility or invertibility.

As an illustration of these principles, here is an example algorithm
for presentation on a device which can display a single 80-character
line of printable ASCII characters:

* The presentation is broken into lines, and each line is presented in
sequence, with user controls for going forward or backward a line.

* Expert mode screens are only presented if the device is in expert mode.

* Each line of the screen starts with a number of `>` characters equal
to the screen's indentation level, followed by a `+` character if this
isn't the first line of the screen, followed by a space if either a
`>` or a `+` has been emitted,
or if this header is followed by a `>`, `+`, or space.

* If the line ends with whitespace or an `@` character, an additional `@`
character is appended to the line.

* The following ASCII control characters or backslash (`\`) are converted
to a backslash followed by a letter code, in the manner of string literals
in many languages:

    * a: U+0007 alert or bell
    * b: U+0008 backspace
    * f: U+000C form feed
    * n: U+000A line feed
    * r: U+000D carriage return
    * t: U+0009 horizontal tab
    * v: U+000B vertical tab
    * `\`: U+005C backslash

* All other ASCII control characters, plus non-ASCII Unicode code points,
are shown as either:

    * `\u` followed by 4 uppercase hex characters for code points
    in the basic multilingual plane (BMP).

    * `\U` followed by 8 uppercase hex characters for other code points.

* The screen will be broken into multiple lines to fit the 80-character
limit, considering the above transformations in a way that attempts to
minimize the number of lines generated. Expanded control or Unicode characters
are never split across lines.

Example output:

```
An introductory line.
key1: 123456
key2: a string that ends in whitespace   @
key3: a string that ends in  a single ampersand - @@
 >tricky key4<: note the leading space in the presentation
introducing an aggregate
> key5: false
> key6: a very long line of text, please co\u00F6perate and break into
>+  multiple lines.
> Can we do further nesting?
>> You bet we can!
```

The inverse mapping gives us the only input which could have
generated this output (JSON notation for string data):

```
Indent  Text
------  ----
0       "An introductory line."
0       "key1: 123456"
0       "key2: a string that ends in whitespace   "
0       "key3: a string that ends in  a single ampersand - @"
0       ">tricky key4<: note the leading space in the presentation"
0       "introducing an aggregate"
1       "key5: false"
1       "key6: a very long line of text, please cooperate and break into multiple lines."
1       "Can we do further nesting?"
2       "You bet we can!"
```
