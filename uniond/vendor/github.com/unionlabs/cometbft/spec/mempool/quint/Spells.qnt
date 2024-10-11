// -*- mode: Bluespec; -*-
module Spells{
    pure def require(cond: bool): bool = cond

    pure def setEqualsOrOneEmpty(s: Set[x], t: Set[x]): bool = {
      (s.size()==0 or t.size()==0) or s == t
    }

    pure def setSubsetOfAtMost(s: Set[x], max: int): Set[x] = {
      if (max>=size(s)) s else oneOf(s.powerset().filter(u => size(u)==max))
    }

    pure def mapValues(m: x -> y): Set[y] = {
      m.keys().fold(Set(), (s, i) => s.union(Set(m.get(i))))
    }

    pure def setChooseSome(s: Set[x]): x = {
      head(s.fold(List(), (t,x) => t.append(x)))
    }

    pure def nonEmptyPowerset(s: Set[x]): Set[Set[x]] = {
      s.powerset().exclude(Set(Set()))
    }
    
}
