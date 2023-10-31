#!/bin/sh

# vim: indentexpr= nosmartindent autoindent
# vim: tabstop=2 shiftwidth=2 softtabstop=2

# This regex was manually written, derived from the rules in UAX #29.
# Particularly, from Table 1c, which lays out a regex for grapheme clusters.

CR="\p{gcb=CR}"
LF="\p{gcb=LF}"
Control="\p{gcb=Control}"
Prepend="\p{gcb=Prepend}"
L="\p{gcb=L}"
V="\p{gcb=V}"
LV="\p{gcb=LV}"
LVT="\p{gcb=LVT}"
T="\p{gcb=T}"
RI="\p{gcb=RI}"
Extend="\p{gcb=Extend}"
ZWJ="\p{gcb=ZWJ}"
SpacingMark="\p{gcb=SpacingMark}"

Any="\p{any}"
ExtendPict="\p{Extended_Pictographic}"

echo "(?x)
$CR $LF
|
$Control
|
$Prepend*
(
  (
    ($L* ($V+ | $LV $V* | $LVT) $T*)
    |
    $L+
    |
    $T+
  )
  |
  $RI $RI
  |
  $ExtendPict ($Extend* $ZWJ $ExtendPict)*
  |
  [^$Control $CR $LF]
)
[$Extend $ZWJ $SpacingMark]*
|
$Any
"
