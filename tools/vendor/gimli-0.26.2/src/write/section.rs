use std::ops::DerefMut;
use std::result;
use std::vec::Vec;

use crate::common::SectionId;
use crate::write::{
    DebugAbbrev, DebugFrame, DebugInfo, DebugInfoReference, DebugLine, DebugLineStr, DebugLoc,
    DebugLocLists, DebugRanges, DebugRngLists, DebugStr, EhFrame, Writer,
};

macro_rules! define_section {
    ($name:ident, $offset:ident, $docs:expr) => {
        #[doc=$docs]
        #[derive(Debug, Default)]
        pub struct $name<W: Writer>(pub W);

        impl<W: Writer> $name<W> {
            /// Return the offset of the next write.
            pub fn offset(&self) -> $offset {
                $offset(self.len())
            }
        }

        impl<W: Writer> From<W> for $name<W> {
            #[inline]
            fn from(w: W) -> Self {
                $name(w)
            }
        }

        impl<W: Writer> Deref for $name<W> {
            type Target = W;

            #[inline]
            fn deref(&self) -> &W {
                &self.0
            }
        }

        impl<W: Writer> DerefMut for $name<W> {
            #[inline]
            fn deref_mut(&mut self) -> &mut W {
                &mut self.0
            }
        }

        impl<W: Writer> Section<W> for $name<W> {
            #[inline]
            fn id(&self) -> SectionId {
                SectionId::$name
            }
        }
    };
}

/// Functionality common to all writable DWARF sections.
pub trait Section<W: Writer>: DerefMut<Target = W> {
    /// Returns the DWARF section kind for this type.
    fn id(&self) -> SectionId;

    /// Returns the ELF section name for this type.
    fn name(&self) -> &'static str {
        self.id().name()
    }
}

/// All of the writable DWARF sections.
#[derive(Debug, Default)]
pub struct Sections<W: Writer> {
    /// The `.debug_abbrev` section.
    pub debug_abbrev: DebugAbbrev<W>,
    /// The `.debug_info` section.
    pub debug_info: DebugInfo<W>,
    /// The `.debug_line` section.
    pub debug_line: DebugLine<W>,
    /// The `.debug_line_str` section.
    pub debug_line_str: DebugLineStr<W>,
    /// The `.debug_ranges` section.
    pub debug_ranges: DebugRanges<W>,
    /// The `.debug_rnglists` section.
    pub debug_rnglists: DebugRngLists<W>,
    /// The `.debug_loc` section.
    pub debug_loc: DebugLoc<W>,
    /// The `.debug_loclists` section.
    pub debug_loclists: DebugLocLists<W>,
    /// The `.debug_str` section.
    pub debug_str: DebugStr<W>,
    /// The `.debug_frame` section.
    pub debug_frame: DebugFrame<W>,
    /// The `.eh_frame` section.
    pub eh_frame: EhFrame<W>,
    /// Unresolved references in the `.debug_info` section.
    pub(crate) debug_info_refs: Vec<DebugInfoReference>,
    /// Unresolved references in the `.debug_loc` section.
    pub(crate) debug_loc_refs: Vec<DebugInfoReference>,
    /// Unresolved references in the `.debug_loclists` section.
    pub(crate) debug_loclists_refs: Vec<DebugInfoReference>,
}

impl<W: Writer + Clone> Sections<W> {
    /// Create a new `Sections` using clones of the given `section`.
    pub fn new(section: W) -> Self {
        Sections {
            debug_abbrev: DebugAbbrev(section.clone()),
            debug_info: DebugInfo(section.clone()),
            debug_line: DebugLine(section.clone()),
            debug_line_str: DebugLineStr(section.clone()),
            debug_ranges: DebugRanges(section.clone()),
            debug_rnglists: DebugRngLists(section.clone()),
            debug_loc: DebugLoc(section.clone()),
            debug_loclists: DebugLocLists(section.clone()),
            debug_str: DebugStr(section.clone()),
            debug_frame: DebugFrame(section.clone()),
            eh_frame: EhFrame(section.clone()),
            debug_info_refs: Vec::new(),
            debug_loc_refs: Vec::new(),
            debug_loclists_refs: Vec::new(),
        }
    }
}

impl<W: Writer> Sections<W> {
    /// For each section, call `f` once with a shared reference.
    pub fn for_each<F, E>(&self, mut f: F) -> result::Result<(), E>
    where
        F: FnMut(SectionId, &W) -> result::Result<(), E>,
    {
        macro_rules! f {
            ($s:expr) => {
                f($s.id(), &$s)
            };
        }
        // Ordered so that earlier sections do not reference later sections.
        f!(self.debug_abbrev)?;
        f!(self.debug_str)?;
        f!(self.debug_line_str)?;
        f!(self.debug_line)?;
        f!(self.debug_ranges)?;
        f!(self.debug_rnglists)?;
        f!(self.debug_loc)?;
        f!(self.debug_loclists)?;
        f!(self.debug_info)?;
        f!(self.debug_frame)?;
        f!(self.eh_frame)?;
        Ok(())
    }

    /// For each section, call `f` once with a mutable reference.
    pub fn for_each_mut<F, E>(&mut self, mut f: F) -> result::Result<(), E>
    where
        F: FnMut(SectionId, &mut W) -> result::Result<(), E>,
    {
        macro_rules! f {
            ($s:expr) => {
                f($s.id(), &mut $s)
            };
        }
        // Ordered so that earlier sections do not reference later sections.
        f!(self.debug_abbrev)?;
        f!(self.debug_str)?;
        f!(self.debug_line_str)?;
        f!(self.debug_line)?;
        f!(self.debug_ranges)?;
        f!(self.debug_rnglists)?;
        f!(self.debug_loc)?;
        f!(self.debug_loclists)?;
        f!(self.debug_info)?;
        f!(self.debug_frame)?;
        f!(self.eh_frame)?;
        Ok(())
    }
}
