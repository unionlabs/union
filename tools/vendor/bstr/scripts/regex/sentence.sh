#!/bin/sh

# vim: indentexpr= nosmartindent autoindent
# vim: tabstop=2 shiftwidth=2 softtabstop=2

# This is a regex that I reverse engineered from the sentence boundary chain
# rules in UAX #29. Unlike the grapheme regex, which is essentially provided
# for us in UAX #29, no such sentence regex exists.
#
# I looked into how ICU achieves this, since UAX #29 hints that producing
# finite state machines for grapheme/sentence/word/line breaking is possible,
# but only easy to do for graphemes. ICU does this by implementing their own
# DSL for describing the break algorithms in terms of the chaining rules
# directly. You can see an example for sentences in
# icu4c/source/data/brkitr/rules/sent.txt. ICU then builds a finite state
# machine from those rules in a mostly standard way, but implements the
# "chaining" aspect of the rules by connecting overlapping end and start
# states. For example, given SB7:
#
#     (Upper | Lower) ATerm x Upper
#
# Then the naive way to convert this into a regex would be something like
#
#     [\p{sb=Upper}\p{sb=Lower}]\p{sb=ATerm}\p{sb=Upper}
#
# Unfortunately, this is incorrect. Why? Well, consider an example like so:
#
#     U.S.A.
#
# A correct implementation of the sentence breaking algorithm should not insert
# any breaks here, exactly in accordance with repeatedly applying rule SB7 as
# given above. Our regex fails to do this because it will first match `U.S`
# without breaking them---which is correct---but will then start looking for
# its next rule beginning with a full stop (in ATerm) and followed by an
# uppercase letter (A). This will wind up triggering rule SB11 (without
# matching `A`), which inserts a break.
#
# The reason why this happens is because our initial application of rule SB7
# "consumes" the next uppercase letter (S), which we want to reuse as a prefix
# in the next rule application. A natural way to express this would be with
# look-around, although it's not clear that works in every case since you
# ultimately might want to consume that ending uppercase letter. In any case,
# we can't use look-around in our truly regular regexes, so we must fix this.
# The approach we take is to explicitly repeat rules when a suffix of a rule
# is a prefix of another rule. In the case of SB7, the end of the rule, an
# uppercase letter, also happens to match the beginning of the rule. This can
# in turn be repeated indefinitely. Thus, our actual translation to a regex is:
#
#     [\p{sb=Upper}\p{sb=Lower}]\p{sb=ATerm}\p{sb=Upper}(\p{sb=ATerm}\p{sb=Upper}*
#
# It turns out that this is exactly what ICU does, but in their case, they do
# it automatically. In our case, we connect the chaining rules manually. It's
# tedious. With that said, we do no implement Unicode line breaking with this
# approach, which is a far scarier beast. In that case, it would probably be
# worth writing the code to do what ICU does.
#
# In the case of sentence breaks, there aren't *too* many overlaps of this
# nature. We list them out exhaustively to make this clear, because it's
# essentially impossible to easily observe this in the regex. (It took me a
# full day to figure all of this out.) Rules marked with N/A mean that they
# specify a break, and this strategy only really applies to stringing together
# non-breaks.
#
#     SB1   - N/A
#     SB2   - N/A
#     SB3   - None
#     SB4   - N/A
#     SB5   - None
#     SB6   - None
#     SB7   - End overlaps with beginning of SB7
#     SB8   - End overlaps with beginning of SB7
#     SB8a  - End overlaps with beginning of SB6, SB8, SB8a, SB9, SB10, SB11
#     SB9   - None
#     SB10  - None
#     SB11  - None
#     SB998 - N/A
#
# SB8a is in particular quite tricky to get right without look-ahead, since it
# allows ping-ponging between match rules SB8a and SB9-11, where SB9-11
# otherwise indicate that a break has been found. In the regex below, we tackle
# this by only permitting part of SB8a to match inside our core non-breaking
# repetition. In particular, we only allow the parts of SB8a to match that
# permit the non-breaking components to continue. If a part of SB8a matches
# that guarantees a pop out to SB9-11, (like `STerm STerm`), then we let it
# happen. This still isn't correct because an SContinue might be seen which
# would allow moving back into SB998 and thus the non-breaking repetition, so
# we handle that case as well.
#
# Finally, the last complication here is the sprinkling of $Ex* everywhere.
# This essentially corresponds to the implementation of SB5 by following
# UAX #29's recommendation in S6.2. Essentially, we use it avoid ever breaking
# in the middle of a grapheme cluster.

CR="\p{sb=CR}"
LF="\p{sb=LF}"
Sep="\p{sb=Sep}"
Close="\p{sb=Close}"
Sp="\p{sb=Sp}"
STerm="\p{sb=STerm}"
ATerm="\p{sb=ATerm}"
SContinue="\p{sb=SContinue}"
Numeric="\p{sb=Numeric}"
Upper="\p{sb=Upper}"
Lower="\p{sb=Lower}"
OLetter="\p{sb=OLetter}"

Ex="[\p{sb=Extend}\p{sb=Format}]"
ParaSep="[$Sep $CR $LF]"
SATerm="[$STerm $ATerm]"

LetterSepTerm="[$OLetter $Upper $Lower $ParaSep $SATerm]"

echo "(?x)
(
  # SB6
  $ATerm $Ex*
    $Numeric
  |
  # SB7
  [$Upper $Lower] $Ex* $ATerm $Ex*
    $Upper $Ex*
    # overlap with SB7
    ($ATerm $Ex* $Upper $Ex*)*
  |
  # SB8
  $ATerm $Ex* $Close* $Ex* $Sp* $Ex*
    ([^$LetterSepTerm] $Ex*)* $Lower $Ex*
    # overlap with SB7
    ($ATerm $Ex* $Upper $Ex*)*
  |
  # SB8a
  $SATerm $Ex* $Close* $Ex* $Sp* $Ex*
  (
    $SContinue
    |
    $ATerm $Ex*
      # Permit repetition of SB8a
      (($Close $Ex*)* ($Sp $Ex*)* $SATerm)*
      # In order to continue non-breaking matching, we now must observe
      # a match with a rule that keeps us in SB6-8a. Otherwise, we've entered
      # one of SB9-11 and know that a break must follow.
      (
        # overlap with SB6
        $Numeric
        |
        # overlap with SB8
        ($Close $Ex*)* ($Sp $Ex*)*
          ([^$LetterSepTerm] $Ex*)* $Lower $Ex*
          # overlap with SB7
          ($ATerm $Ex* $Upper $Ex*)*
        |
        # overlap with SB8a
        ($Close $Ex*)* ($Sp $Ex*)* $SContinue
      )
    |
    $STerm $Ex*
      # Permit repetition of SB8a
      (($Close $Ex*)* ($Sp $Ex*)* $SATerm)*
      # As with ATerm above, in order to continue non-breaking matching, we
      # must now observe a match with a rule that keeps us out of SB9-11.
      # For STerm, the only such possibility is to see an SContinue. Anything
      # else will result in a break.
      ($Close $Ex*)* ($Sp $Ex*)* $SContinue
  )
  |
  # SB998
  # The logic behind this catch-all is that if we get to this point and
  # see a Sep, CR, LF, STerm or ATerm, then it has to fall into one of
  # SB9, SB10 or SB11. In the cases of SB9-11, we always find a break since
  # SB11 acts as a catch-all to induce a break following a SATerm that isn't
  # handled by rules SB6-SB8a.
  [^$ParaSep $SATerm]
)*
# The following collapses rules SB3, SB4, part of SB8a, SB9, SB10 and SB11.
($SATerm $Ex* ($Close $Ex*)* ($Sp $Ex*)*)* ($CR $LF | $ParaSep)?
"
