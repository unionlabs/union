#!/bin/sh

# vim: indentexpr= nosmartindent autoindent
# vim: tabstop=2 shiftwidth=2 softtabstop=2

# See the comments in regex/sentence.sh for the general approach to how this
# regex was written.
#
# Writing the regex for this was *hard*. It took me two days of hacking to get
# this far, and that was after I had finished the sentence regex, so my brain
# was fully cached on this. Unlike the sentence regex, the rules in the regex
# below don't correspond as nicely to the rules in UAX #29. In particular, the
# UAX #29 rules have a ton of overlap with each other, which requires crazy
# stuff in the regex. I'm not even sure the regex below is 100% correct or even
# minimal, however, I did compare this with the ICU word segmenter on a few
# different corpora, and it produces identical results. (In addition to of
# course passing the UCD tests.)
#
# In general, I consider this approach to be a failure. Firstly, this is
# clearly a write-only regex. Secondly, building the minimized DFA for this is
# incredibly slow. Thirdly, the DFA is itself very large (~240KB). Fourthly,
# reversing this regex (for reverse word iteration) results in a >19MB DFA.
# Yes. That's MB. Wat. And it took 5 minutes to build.
#
# I think we might consider changing our approach to this problem. The normal
# path I've seen, I think, is to decode codepoints one at a time, and then
# thread them through a state machine in the code itself. We could take this
# approach, or possibly combine it with a DFA that tells us which Word_Break
# value a codepoint has. I'd prefer the latter approach, but it requires adding
# RegexSet support to regex-automata. Something that should definitely be done,
# but is a fair amount of work.
#
# Gah.

CR="\p{wb=CR}"
LF="\p{wb=LF}"
Newline="\p{wb=Newline}"
ZWJ="\p{wb=ZWJ}"
RI="\p{wb=Regional_Indicator}"
Katakana="\p{wb=Katakana}"
HebrewLet="\p{wb=HebrewLetter}"
ALetter="\p{wb=ALetter}"
SingleQuote="\p{wb=SingleQuote}"
DoubleQuote="\p{wb=DoubleQuote}"
MidNumLet="\p{wb=MidNumLet}"
MidLetter="\p{wb=MidLetter}"
MidNum="\p{wb=MidNum}"
Numeric="\p{wb=Numeric}"
ExtendNumLet="\p{wb=ExtendNumLet}"
WSegSpace="\p{wb=WSegSpace}"

Any="\p{any}"
Ex="[\p{wb=Extend} \p{wb=Format} $ZWJ]"
ExtendPict="\p{Extended_Pictographic}"
AHLetter="[$ALetter $HebrewLet]"
MidNumLetQ="[$MidNumLet $SingleQuote]"

AHLetterRepeat="$AHLetter $Ex* ([$MidLetter $MidNumLetQ] $Ex* $AHLetter $Ex*)*"
NumericRepeat="$Numeric $Ex* ([$MidNum $MidNumLetQ] $Ex* $Numeric $Ex*)*"

echo "(?x)
$CR $LF
|
[$Newline $CR $LF]
|
$WSegSpace $WSegSpace+
|
(
  ([^$Newline $CR $LF]? $Ex* $ZWJ $ExtendPict $Ex*)+
  |
  ($ExtendNumLet $Ex*)* $AHLetter $Ex*
    (
      (
        ($NumericRepeat | $ExtendNumLet $Ex*)*
        |
        [$MidLetter $MidNumLetQ] $Ex*
      )
      $AHLetter $Ex*
    )+
    ($NumericRepeat | $ExtendNumLet $Ex*)*
  |
  ($ExtendNumLet $Ex*)* $AHLetter $Ex* ($NumericRepeat | $ExtendNumLet $Ex*)+
  |
  ($ExtendNumLet $Ex*)* $Numeric $Ex*
    (
      (
        ($AHLetterRepeat | $ExtendNumLet $Ex*)*
        |
        [$MidNum $MidNumLetQ] $Ex*
      )
      $Numeric $Ex*
    )+
    ($AHLetterRepeat | $ExtendNumLet $Ex*)*
  |
  ($ExtendNumLet $Ex*)* $Numeric $Ex* ($AHLetterRepeat | $ExtendNumLet $Ex*)+
  |
  $Katakana $Ex*
    (($Katakana | $ExtendNumLet) $Ex*)+
  |
  $ExtendNumLet $Ex*
    (($ExtendNumLet | $AHLetter | $Numeric | $Katakana) $Ex*)+
)+
|
$HebrewLet $Ex* $SingleQuote $Ex*
|
($HebrewLet $Ex* $DoubleQuote $Ex*)+ $HebrewLet $Ex*
|
$RI $Ex* $RI $Ex*
|
$Any $Ex*
"
