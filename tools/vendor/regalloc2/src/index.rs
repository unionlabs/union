#[macro_export]
macro_rules! define_index {
    ($ix:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[cfg_attr(
            feature = "enable-serde",
            derive(::serde::Serialize, ::serde::Deserialize)
        )]
        pub struct $ix(pub u32);
        impl $ix {
            #[inline(always)]
            pub fn new(i: usize) -> Self {
                Self(i as u32)
            }
            #[inline(always)]
            pub fn index(self) -> usize {
                debug_assert!(self.is_valid());
                self.0 as usize
            }
            #[inline(always)]
            pub fn invalid() -> Self {
                Self(u32::MAX)
            }
            #[inline(always)]
            pub fn is_invalid(self) -> bool {
                self == Self::invalid()
            }
            #[inline(always)]
            pub fn is_valid(self) -> bool {
                self != Self::invalid()
            }
            #[inline(always)]
            pub fn next(self) -> $ix {
                debug_assert!(self.is_valid());
                Self(self.0 + 1)
            }
            #[inline(always)]
            pub fn prev(self) -> $ix {
                debug_assert!(self.is_valid());
                Self(self.0 - 1)
            }

            #[inline(always)]
            pub fn raw_u32(self) -> u32 {
                self.0
            }
        }

        impl crate::index::ContainerIndex for $ix {}
    };
}

pub trait ContainerIndex: Clone + Copy + std::fmt::Debug + PartialEq + Eq {}

pub trait ContainerComparator {
    type Ix: ContainerIndex;
    fn compare(&self, a: Self::Ix, b: Self::Ix) -> std::cmp::Ordering;
}

define_index!(Inst);
define_index!(Block);

#[derive(Clone, Copy, Debug)]
pub struct InstRange(Inst, Inst, bool);

impl InstRange {
    #[inline(always)]
    pub fn forward(from: Inst, to: Inst) -> Self {
        debug_assert!(from.index() <= to.index());
        InstRange(from, to, true)
    }

    #[inline(always)]
    pub fn backward(from: Inst, to: Inst) -> Self {
        debug_assert!(from.index() >= to.index());
        InstRange(to, from, false)
    }

    #[inline(always)]
    pub fn first(self) -> Inst {
        debug_assert!(self.len() > 0);
        if self.is_forward() {
            self.0
        } else {
            self.1.prev()
        }
    }

    #[inline(always)]
    pub fn last(self) -> Inst {
        debug_assert!(self.len() > 0);
        if self.is_forward() {
            self.1.prev()
        } else {
            self.0
        }
    }

    #[inline(always)]
    pub fn rest(self) -> InstRange {
        debug_assert!(self.len() > 0);
        if self.is_forward() {
            InstRange::forward(self.0.next(), self.1)
        } else {
            InstRange::backward(self.1.prev(), self.0)
        }
    }

    #[inline(always)]
    pub fn len(self) -> usize {
        self.1.index() - self.0.index()
    }

    #[inline(always)]
    pub fn is_forward(self) -> bool {
        self.2
    }

    #[inline(always)]
    pub fn rev(self) -> Self {
        Self(self.0, self.1, !self.2)
    }

    #[inline(always)]
    pub fn iter(self) -> InstRangeIter {
        InstRangeIter(self)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InstRangeIter(InstRange);

impl Iterator for InstRangeIter {
    type Item = Inst;
    #[inline(always)]
    fn next(&mut self) -> Option<Inst> {
        if self.0.len() == 0 {
            None
        } else {
            let ret = self.0.first();
            self.0 = self.0.rest();
            Some(ret)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_inst_range() {
        let range = InstRange::forward(Inst::new(0), Inst::new(0));
        debug_assert_eq!(range.len(), 0);

        let range = InstRange::forward(Inst::new(0), Inst::new(5));
        debug_assert_eq!(range.first().index(), 0);
        debug_assert_eq!(range.last().index(), 4);
        debug_assert_eq!(range.len(), 5);
        debug_assert_eq!(
            range.iter().collect::<Vec<_>>(),
            vec![
                Inst::new(0),
                Inst::new(1),
                Inst::new(2),
                Inst::new(3),
                Inst::new(4)
            ]
        );
        let range = range.rev();
        debug_assert_eq!(range.first().index(), 4);
        debug_assert_eq!(range.last().index(), 0);
        debug_assert_eq!(range.len(), 5);
        debug_assert_eq!(
            range.iter().collect::<Vec<_>>(),
            vec![
                Inst::new(4),
                Inst::new(3),
                Inst::new(2),
                Inst::new(1),
                Inst::new(0)
            ]
        );
    }
}
