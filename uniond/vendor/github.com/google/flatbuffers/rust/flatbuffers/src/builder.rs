/*
 * Copyright 2018 Google Inc. All rights reserved.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};
use core::cmp::max;
use core::convert::Infallible;
use core::fmt::{Debug, Display};
use core::iter::{DoubleEndedIterator, ExactSizeIterator};
use core::marker::PhantomData;
use core::ops::{Add, AddAssign, Deref, DerefMut, Index, IndexMut, Sub, SubAssign};
use core::ptr::write_bytes;

use crate::endian_scalar::emplace_scalar;
use crate::primitives::*;
use crate::push::{Push, PushAlignment};
use crate::read_scalar;
use crate::table::Table;
use crate::vector::Vector;
use crate::vtable::{field_index_to_field_offset, VTable};
use crate::vtable_writer::VTableWriter;

/// Trait to implement custom allocation strategies for [`FlatBufferBuilder`].
///
/// An implementation can be used with [`FlatBufferBuilder::new_in`], enabling a custom allocation
/// strategy for the [`FlatBufferBuilder`].
///
/// # Safety
///
/// The implementation of the allocator must match the defined behavior as described by the
/// comments.
pub unsafe trait Allocator: DerefMut<Target = [u8]> {
    /// A type describing allocation failures
    type Error: Display + Debug;
    /// Grows the buffer, with the old contents being moved to the end.
    ///
    /// NOTE: While not unsound, an implementation that doesn't grow the
    /// internal buffer will get stuck in an infinite loop.
    fn grow_downwards(&mut self) -> Result<(), Self::Error>;

    /// Returns the size of the internal buffer in bytes.
    fn len(&self) -> usize;
}

/// Default [`FlatBufferBuilder`] allocator backed by a [`Vec<u8>`].
#[derive(Default)]
pub struct DefaultAllocator(Vec<u8>);

impl DefaultAllocator {
    /// Builds the allocator from an existing buffer.
    pub fn from_vec(buffer: Vec<u8>) -> Self {
        Self(buffer)
    }
}

impl Deref for DefaultAllocator {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DefaultAllocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// SAFETY: The methods are implemented as described by the documentation.
unsafe impl Allocator for DefaultAllocator {
    type Error = Infallible;
    fn grow_downwards(&mut self) -> Result<(), Self::Error> {
        let old_len = self.0.len();
        let new_len = max(1, old_len * 2);

        self.0.resize(new_len, 0);

        if new_len == 1 {
            return Ok(());
        }

        // calculate the midpoint, and safely copy the old end data to the new
        // end position:
        let middle = new_len / 2;
        {
            let (left, right) = &mut self.0[..].split_at_mut(middle);
            right.copy_from_slice(left);
        }
        // finally, zero out the old end data.
        {
            let ptr = self.0[..middle].as_mut_ptr();
            // Safety:
            // ptr is byte aligned and of length middle
            unsafe {
                write_bytes(ptr, 0, middle);
            }
        }
        Ok(())
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FieldLoc {
    off: UOffsetT,
    id: VOffsetT,
}

/// FlatBufferBuilder builds a FlatBuffer through manipulating its internal
/// state. It has an owned `Vec<u8>` that grows as needed (up to the hardcoded
/// limit of 2GiB, which is set by the FlatBuffers format).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlatBufferBuilder<'fbb, A: Allocator = DefaultAllocator> {
    allocator: A,
    head: ReverseIndex,

    field_locs: Vec<FieldLoc>,
    written_vtable_revpos: Vec<UOffsetT>,

    nested: bool,
    finished: bool,

    min_align: usize,
    force_defaults: bool,
    strings_pool: Vec<WIPOffset<&'fbb str>>,

    _phantom: PhantomData<&'fbb ()>,
}

impl<'fbb> FlatBufferBuilder<'fbb, DefaultAllocator> {
    /// Create a FlatBufferBuilder that is ready for writing.
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    #[deprecated(note = "replaced with `with_capacity`", since = "0.8.5")]
    pub fn new_with_capacity(size: usize) -> Self {
        Self::with_capacity(size)
    }
    /// Create a FlatBufferBuilder that is ready for writing, with a
    /// ready-to-use capacity of the provided size.
    ///
    /// The maximum valid value is `FLATBUFFERS_MAX_BUFFER_SIZE`.
    pub fn with_capacity(size: usize) -> Self {
        Self::from_vec(vec![0; size])
    }
    /// Create a FlatBufferBuilder that is ready for writing, reusing
    /// an existing vector.
    pub fn from_vec(buffer: Vec<u8>) -> Self {
        // we need to check the size here because we create the backing buffer
        // directly, bypassing the typical way of using grow_allocator:
        assert!(
            buffer.len() <= FLATBUFFERS_MAX_BUFFER_SIZE,
            "cannot initialize buffer bigger than 2 gigabytes"
        );
        let allocator = DefaultAllocator::from_vec(buffer);
        Self::new_in(allocator)
    }

    /// Destroy the FlatBufferBuilder, returning its internal byte vector
    /// and the index into it that represents the start of valid data.
    pub fn collapse(self) -> (Vec<u8>, usize) {
        let index = self.head.to_forward_index(&self.allocator);
        (self.allocator.0, index)
    }
}

impl<'fbb, A: Allocator> FlatBufferBuilder<'fbb, A> {
    /// Create a [`FlatBufferBuilder`] that is ready for writing with a custom [`Allocator`].
    pub fn new_in(allocator: A) -> Self {
        let head = ReverseIndex::end();
        FlatBufferBuilder {
            allocator,
            head,

            field_locs: Vec::new(),
            written_vtable_revpos: Vec::new(),

            nested: false,
            finished: false,

            min_align: 0,
            force_defaults: false,
            strings_pool: Vec::new(),

            _phantom: PhantomData,
        }
    }

    /// Destroy the [`FlatBufferBuilder`], returning its [`Allocator`] and the index
    /// into it that represents the start of valid data.
    pub fn collapse_in(self) -> (A, usize) {
        let index = self.head.to_forward_index(&self.allocator);
        (self.allocator, index)
    }

    /// Reset the FlatBufferBuilder internal state. Use this method after a
    /// call to a `finish` function in order to re-use a FlatBufferBuilder.
    ///
    /// This function is the only way to reset the `finished` state and start
    /// again.
    ///
    /// If you are using a FlatBufferBuilder repeatedly, make sure to use this
    /// function, because it re-uses the FlatBufferBuilder's existing
    /// heap-allocated `Vec<u8>` internal buffer. This offers significant speed
    /// improvements as compared to creating a new FlatBufferBuilder for every
    /// new object.
    pub fn reset(&mut self) {
        // memset only the part of the buffer that could be dirty:
        self.allocator[self.head.range_to_end()]
            .iter_mut()
            .for_each(|x| *x = 0);

        self.head = ReverseIndex::end();
        self.written_vtable_revpos.clear();

        self.nested = false;
        self.finished = false;

        self.min_align = 0;
        self.strings_pool.clear();
    }

    /// Push a Push'able value onto the front of the in-progress data.
    ///
    /// This function uses traits to provide a unified API for writing
    /// scalars, tables, vectors, and WIPOffsets.
    #[inline]
    pub fn push<P: Push>(&mut self, x: P) -> WIPOffset<P::Output> {
        let sz = P::size();
        self.align(sz, P::alignment());
        self.make_space(sz);
        {
            let (dst, rest) = self.allocator[self.head.range_to_end()].split_at_mut(sz);
            // Safety:
            // Called make_space above
            unsafe { x.push(dst, rest.len()) };
        }
        WIPOffset::new(self.used_space() as UOffsetT)
    }

    /// Push a Push'able value onto the front of the in-progress data, and
    /// store a reference to it in the in-progress vtable. If the value matches
    /// the default, then this is a no-op.
    #[inline]
    pub fn push_slot<X: Push + PartialEq>(&mut self, slotoff: VOffsetT, x: X, default: X) {
        self.assert_nested("push_slot");
        if x != default || self.force_defaults {
            self.push_slot_always(slotoff, x);
        }
    }

    /// Push a Push'able value onto the front of the in-progress data, and
    /// store a reference to it in the in-progress vtable.
    #[inline]
    pub fn push_slot_always<X: Push>(&mut self, slotoff: VOffsetT, x: X) {
        self.assert_nested("push_slot_always");
        let off = self.push(x);
        self.track_field(slotoff, off.value());
    }

    /// Retrieve the number of vtables that have been serialized into the
    /// FlatBuffer. This is primarily used to check vtable deduplication.
    #[inline]
    pub fn num_written_vtables(&self) -> usize {
        self.written_vtable_revpos.len()
    }

    /// Start a Table write.
    ///
    /// Asserts that the builder is not in a nested state.
    ///
    /// Users probably want to use `push_slot` to add values after calling this.
    #[inline]
    pub fn start_table(&mut self) -> WIPOffset<TableUnfinishedWIPOffset> {
        self.assert_not_nested(
            "start_table can not be called when a table or vector is under construction",
        );
        self.nested = true;

        WIPOffset::new(self.used_space() as UOffsetT)
    }

    /// End a Table write.
    ///
    /// Asserts that the builder is in a nested state.
    #[inline]
    pub fn end_table(
        &mut self,
        off: WIPOffset<TableUnfinishedWIPOffset>,
    ) -> WIPOffset<TableFinishedWIPOffset> {
        self.assert_nested("end_table");

        let o = self.write_vtable(off);

        self.nested = false;
        self.field_locs.clear();

        WIPOffset::new(o.value())
    }

    /// Start a Vector write.
    ///
    /// Asserts that the builder is not in a nested state.
    ///
    /// Most users will prefer to call `create_vector`.
    /// Speed optimizing users who choose to create vectors manually using this
    /// function will want to use `push` to add values.
    #[inline]
    pub fn start_vector<T: Push>(&mut self, num_items: usize) {
        self.assert_not_nested(
            "start_vector can not be called when a table or vector is under construction",
        );
        self.nested = true;
        self.align(num_items * T::size(), T::alignment().max_of(SIZE_UOFFSET));
    }

    /// End a Vector write.
    ///
    /// Note that the `num_elems` parameter is the number of written items, not
    /// the byte count.
    ///
    /// Asserts that the builder is in a nested state.
    #[inline]
    pub fn end_vector<T: Push>(&mut self, num_elems: usize) -> WIPOffset<Vector<'fbb, T>> {
        self.assert_nested("end_vector");
        self.nested = false;
        let o = self.push::<UOffsetT>(num_elems as UOffsetT);
        WIPOffset::new(o.value())
    }

    #[inline]
    pub fn create_shared_string<'a: 'b, 'b>(&'a mut self, s: &'b str) -> WIPOffset<&'fbb str> {
        self.assert_not_nested(
            "create_shared_string can not be called when a table or vector is under construction",
        );

        // Saves a ref to allocator since rust doesnt like us refrencing it
        // in the binary_search_by code.
        let buf = &self.allocator;

        let found = self.strings_pool.binary_search_by(|offset| {
            let ptr = offset.value() as usize;
            // Gets The pointer to the size of the string
            let str_memory = &buf[buf.len() - ptr..];
            // Gets the size of the written string from buffer
            let size =
                u32::from_le_bytes([str_memory[0], str_memory[1], str_memory[2], str_memory[3]])
                    as usize;
            // Size of the string size
            let string_size: usize = 4;
            // Fetches actual string bytes from index of string after string size
            // to the size of string plus string size
            let iter = str_memory[string_size..size + string_size].iter();
            // Compares bytes of fetched string and current writable string
            iter.cloned().cmp(s.bytes())
        });

        match found {
            Ok(index) => self.strings_pool[index],
            Err(index) => {
                let address = WIPOffset::new(self.create_byte_string(s.as_bytes()).value());
                self.strings_pool.insert(index, address);
                address
            }
        }
    }

    /// Create a utf8 string.
    ///
    /// The wire format represents this as a zero-terminated byte vector.
    #[inline]
    pub fn create_string<'a: 'b, 'b>(&'a mut self, s: &'b str) -> WIPOffset<&'fbb str> {
        self.assert_not_nested(
            "create_string can not be called when a table or vector is under construction",
        );
        WIPOffset::new(self.create_byte_string(s.as_bytes()).value())
    }

    /// Create a zero-terminated byte vector.
    #[inline]
    pub fn create_byte_string(&mut self, data: &[u8]) -> WIPOffset<&'fbb [u8]> {
        self.assert_not_nested(
            "create_byte_string can not be called when a table or vector is under construction",
        );
        self.align(data.len() + 1, PushAlignment::new(SIZE_UOFFSET));
        self.push(0u8);
        self.push_bytes_unprefixed(data);
        self.push(data.len() as UOffsetT);
        WIPOffset::new(self.used_space() as UOffsetT)
    }

    /// Create a vector of Push-able objects.
    ///
    /// Speed-sensitive users may wish to reduce memory usage by creating the
    /// vector manually: use `start_vector`, `push`, and `end_vector`.
    #[inline]
    pub fn create_vector<'a: 'b, 'b, T: Push + 'b>(
        &'a mut self,
        items: &'b [T],
    ) -> WIPOffset<Vector<'fbb, T::Output>> {
        let elem_size = T::size();
        let slice_size = items.len() * elem_size;
        self.align(slice_size, T::alignment().max_of(SIZE_UOFFSET));
        self.ensure_capacity(slice_size + UOffsetT::size());

        self.head -= slice_size;
        let mut written_len = self.head.distance_to_end();

        let buf = &mut self.allocator[self.head.range_to(self.head + slice_size)];
        for (item, out) in items.iter().zip(buf.chunks_exact_mut(elem_size)) {
            written_len -= elem_size;

            // Safety:
            // Called ensure_capacity and aligned to T above
            unsafe { item.push(out, written_len) };
        }

        WIPOffset::new(self.push::<UOffsetT>(items.len() as UOffsetT).value())
    }

    /// Create a vector of Push-able objects.
    ///
    /// Speed-sensitive users may wish to reduce memory usage by creating the
    /// vector manually: use `start_vector`, `push`, and `end_vector`.
    #[inline]
    pub fn create_vector_from_iter<T: Push>(
        &mut self,
        items: impl ExactSizeIterator<Item = T> + DoubleEndedIterator,
    ) -> WIPOffset<Vector<'fbb, T::Output>> {
        let elem_size = T::size();
        self.align(items.len() * elem_size, T::alignment().max_of(SIZE_UOFFSET));
        let mut actual = 0;
        for item in items.rev() {
            self.push(item);
            actual += 1;
        }
        WIPOffset::new(self.push::<UOffsetT>(actual).value())
    }

    /// Set whether default values are stored.
    ///
    /// In order to save space, fields that are set to their default value
    /// aren't stored in the buffer. Setting `force_defaults` to `true`
    /// disables this optimization.
    ///
    /// By default, `force_defaults` is `false`.
    #[inline]
    pub fn force_defaults(&mut self, force_defaults: bool) {
        self.force_defaults = force_defaults;
    }

    /// Get the byte slice for the data that has been written, regardless of
    /// whether it has been finished.
    #[inline]
    pub fn unfinished_data(&self) -> &[u8] {
        &self.allocator[self.head.range_to_end()]
    }
    /// Get the byte slice for the data that has been written after a call to
    /// one of the `finish` functions.
    /// # Panics
    /// Panics if the buffer is not finished.
    #[inline]
    pub fn finished_data(&self) -> &[u8] {
        self.assert_finished("finished_bytes cannot be called when the buffer is not yet finished");
        &self.allocator[self.head.range_to_end()]
    }
    /// Returns a mutable view of a finished buffer and location of where the flatbuffer starts.
    /// Note that modifying the flatbuffer data may corrupt it.
    /// # Panics
    /// Panics if the flatbuffer is not finished.
    #[inline]
    pub fn mut_finished_buffer(&mut self) -> (&mut [u8], usize) {
        let index = self.head.to_forward_index(&self.allocator);
        (&mut self.allocator[..], index)
    }
    /// Assert that a field is present in the just-finished Table.
    ///
    /// This is somewhat low-level and is mostly used by the generated code.
    #[inline]
    pub fn required(
        &self,
        tab_revloc: WIPOffset<TableFinishedWIPOffset>,
        slot_byte_loc: VOffsetT,
        assert_msg_name: &'static str,
    ) {
        let idx = self.used_space() - tab_revloc.value() as usize;

        // Safety:
        // The value of TableFinishedWIPOffset is the offset from the end of the allocator
        // to an SOffsetT pointing to a valid VTable
        //
        // `self.allocator.len() = self.used_space() + self.head`
        // `self.allocator.len() - tab_revloc = self.used_space() - tab_revloc + self.head`
        // `self.allocator.len() - tab_revloc = idx + self.head`
        let tab = unsafe { Table::new(&self.allocator[self.head.range_to_end()], idx) };
        let o = tab.vtable().get(slot_byte_loc) as usize;
        assert!(o != 0, "missing required field {}", assert_msg_name);
    }

    /// Finalize the FlatBuffer by: aligning it, pushing an optional file
    /// identifier on to it, pushing a size prefix on to it, and marking the
    /// internal state of the FlatBufferBuilder as `finished`. Afterwards,
    /// users can call `finished_data` to get the resulting data.
    #[inline]
    pub fn finish_size_prefixed<T>(&mut self, root: WIPOffset<T>, file_identifier: Option<&str>) {
        self.finish_with_opts(root, file_identifier, true);
    }

    /// Finalize the FlatBuffer by: aligning it, pushing an optional file
    /// identifier on to it, and marking the internal state of the
    /// FlatBufferBuilder as `finished`. Afterwards, users can call
    /// `finished_data` to get the resulting data.
    #[inline]
    pub fn finish<T>(&mut self, root: WIPOffset<T>, file_identifier: Option<&str>) {
        self.finish_with_opts(root, file_identifier, false);
    }

    /// Finalize the FlatBuffer by: aligning it and marking the internal state
    /// of the FlatBufferBuilder as `finished`. Afterwards, users can call
    /// `finished_data` to get the resulting data.
    #[inline]
    pub fn finish_minimal<T>(&mut self, root: WIPOffset<T>) {
        self.finish_with_opts(root, None, false);
    }

    #[inline]
    fn used_space(&self) -> usize {
        self.head.distance_to_end()
    }

    #[inline]
    fn track_field(&mut self, slot_off: VOffsetT, off: UOffsetT) {
        let fl = FieldLoc { id: slot_off, off };
        self.field_locs.push(fl);
    }

    /// Write the VTable, if it is new.
    fn write_vtable(
        &mut self,
        table_tail_revloc: WIPOffset<TableUnfinishedWIPOffset>,
    ) -> WIPOffset<VTableWIPOffset> {
        self.assert_nested("write_vtable");

        // Write the vtable offset, which is the start of any Table.
        // We fill its value later.
        let object_revloc_to_vtable: WIPOffset<VTableWIPOffset> =
            WIPOffset::new(self.push::<UOffsetT>(0xF0F0_F0F0).value());

        // Layout of the data this function will create when a new vtable is
        // needed.
        // --------------------------------------------------------------------
        // vtable starts here
        // | x, x -- vtable len (bytes) [u16]
        // | x, x -- object inline len (bytes) [u16]
        // | x, x -- zero, or num bytes from start of object to field #0   [u16]
        // | ...
        // | x, x -- zero, or num bytes from start of object to field #n-1 [u16]
        // vtable ends here
        // table starts here
        // | x, x, x, x -- offset (negative direction) to the vtable [i32]
        // |               aka "vtableoffset"
        // | -- table inline data begins here, we don't touch it --
        // table ends here -- aka "table_start"
        // --------------------------------------------------------------------
        //
        // Layout of the data this function will create when we re-use an
        // existing vtable.
        //
        // We always serialize this particular vtable, then compare it to the
        // other vtables we know about to see if there is a duplicate. If there
        // is, then we erase the serialized vtable we just made.
        // We serialize it first so that we are able to do byte-by-byte
        // comparisons with already-serialized vtables. This 1) saves
        // bookkeeping space (we only keep revlocs to existing vtables), 2)
        // allows us to convert to little-endian once, then do
        // fast memcmp comparisons, and 3) by ensuring we are comparing real
        // serialized vtables, we can be more assured that we are doing the
        // comparisons correctly.
        //
        // --------------------------------------------------------------------
        // table starts here
        // | x, x, x, x -- offset (negative direction) to an existing vtable [i32]
        // |               aka "vtableoffset"
        // | -- table inline data begins here, we don't touch it --
        // table starts here: aka "table_start"
        // --------------------------------------------------------------------

        // fill the WIP vtable with zeros:
        let vtable_byte_len = get_vtable_byte_len(&self.field_locs);
        self.make_space(vtable_byte_len);

        // compute the length of the table (not vtable!) in bytes:
        let table_object_size = object_revloc_to_vtable.value() - table_tail_revloc.value();
        debug_assert!(table_object_size < 0x10000); // vTable use 16bit offsets.

        // Write the VTable (we may delete it afterwards, if it is a duplicate):
        let vt_start_pos = self.head;
        let vt_end_pos = self.head + vtable_byte_len;
        {
            // write the vtable header:
            let vtfw =
                &mut VTableWriter::init(&mut self.allocator[vt_start_pos.range_to(vt_end_pos)]);
            vtfw.write_vtable_byte_length(vtable_byte_len as VOffsetT);
            vtfw.write_object_inline_size(table_object_size as VOffsetT);

            // serialize every FieldLoc to the vtable:
            for &fl in self.field_locs.iter() {
                let pos: VOffsetT = (object_revloc_to_vtable.value() - fl.off) as VOffsetT;
                vtfw.write_field_offset(fl.id, pos);
            }
        }
        let new_vt_bytes = &self.allocator[vt_start_pos.range_to(vt_end_pos)];
        let found = self
            .written_vtable_revpos
            .binary_search_by(|old_vtable_revpos: &UOffsetT| {
                let old_vtable_pos = self.allocator.len() - *old_vtable_revpos as usize;
                // Safety:
                // Already written vtables are valid by construction
                let old_vtable = unsafe { VTable::init(&self.allocator, old_vtable_pos) };
                new_vt_bytes.cmp(old_vtable.as_bytes())
            });
        let final_vtable_revpos = match found {
            Ok(i) => {
                // The new vtable is a duplicate so clear it.
                VTableWriter::init(&mut self.allocator[vt_start_pos.range_to(vt_end_pos)]).clear();
                self.head += vtable_byte_len;
                self.written_vtable_revpos[i]
            }
            Err(i) => {
                // This is a new vtable. Add it to the cache.
                let new_vt_revpos = self.used_space() as UOffsetT;
                self.written_vtable_revpos.insert(i, new_vt_revpos);
                new_vt_revpos
            }
        };
        // Write signed offset from table to its vtable.
        let table_pos = self.allocator.len() - object_revloc_to_vtable.value() as usize;
        if cfg!(debug_assertions) {
            // Safety:
            // Verified slice length
            let tmp_soffset_to_vt = unsafe {
                read_scalar::<UOffsetT>(&self.allocator[table_pos..table_pos + SIZE_UOFFSET])
            };
            assert_eq!(tmp_soffset_to_vt, 0xF0F0_F0F0);
        }

        let buf = &mut self.allocator[table_pos..table_pos + SIZE_SOFFSET];
        // Safety:
        // Verified length of buf above
        unsafe {
            emplace_scalar::<SOffsetT>(
                buf,
                final_vtable_revpos as SOffsetT - object_revloc_to_vtable.value() as SOffsetT,
            );
        }

        self.field_locs.clear();

        object_revloc_to_vtable
    }

    // Only call this when you know it is safe to double the size of the buffer.
    #[inline]
    fn grow_allocator(&mut self) {
        let starting_active_size = self.used_space();
        self.allocator
            .grow_downwards()
            .expect("Flatbuffer allocation failure");

        let ending_active_size = self.used_space();
        debug_assert_eq!(starting_active_size, ending_active_size);
    }

    // with or without a size prefix changes how we load the data, so finish*
    // functions are split along those lines.
    fn finish_with_opts<T>(
        &mut self,
        root: WIPOffset<T>,
        file_identifier: Option<&str>,
        size_prefixed: bool,
    ) {
        self.assert_not_finished("buffer cannot be finished when it is already finished");
        self.assert_not_nested(
            "buffer cannot be finished when a table or vector is under construction",
        );
        self.written_vtable_revpos.clear();

        let to_align = {
            // for the root offset:
            let a = SIZE_UOFFSET;
            // for the size prefix:
            let b = if size_prefixed { SIZE_UOFFSET } else { 0 };
            // for the file identifier (a string that is not zero-terminated):
            let c = if file_identifier.is_some() {
                FILE_IDENTIFIER_LENGTH
            } else {
                0
            };
            a + b + c
        };

        {
            let ma = PushAlignment::new(self.min_align);
            self.align(to_align, ma);
        }

        if let Some(ident) = file_identifier {
            debug_assert_eq!(ident.len(), FILE_IDENTIFIER_LENGTH);
            self.push_bytes_unprefixed(ident.as_bytes());
        }

        self.push(root);

        if size_prefixed {
            let sz = self.used_space() as UOffsetT;
            self.push::<UOffsetT>(sz);
        }
        self.finished = true;
    }

    #[inline]
    fn align(&mut self, len: usize, alignment: PushAlignment) {
        self.track_min_align(alignment.value());
        let s = self.used_space() as usize;
        self.make_space(padding_bytes(s + len, alignment.value()));
    }

    #[inline]
    fn track_min_align(&mut self, alignment: usize) {
        self.min_align = max(self.min_align, alignment);
    }

    #[inline]
    fn push_bytes_unprefixed(&mut self, x: &[u8]) -> UOffsetT {
        let n = self.make_space(x.len());
        self.allocator[n.range_to(n + x.len())].copy_from_slice(x);

        n.to_forward_index(&self.allocator) as UOffsetT
    }

    #[inline]
    fn make_space(&mut self, want: usize) -> ReverseIndex {
        self.ensure_capacity(want);
        self.head -= want;
        self.head
    }

    #[inline]
    fn ensure_capacity(&mut self, want: usize) -> usize {
        if self.unused_ready_space() >= want {
            return want;
        }
        assert!(
            want <= FLATBUFFERS_MAX_BUFFER_SIZE,
            "cannot grow buffer beyond 2 gigabytes"
        );

        while self.unused_ready_space() < want {
            self.grow_allocator();
        }
        want
    }
    #[inline]
    fn unused_ready_space(&self) -> usize {
        self.allocator.len() - self.head.distance_to_end()
    }
    #[inline]
    fn assert_nested(&self, fn_name: &'static str) {
        // we don't assert that self.field_locs.len() >0 because the vtable
        // could be empty (e.g. for empty tables, or for all-default values).
        debug_assert!(
            self.nested,
            "incorrect FlatBufferBuilder usage: {} must be called while in a nested state",
            fn_name
        );
    }
    #[inline]
    fn assert_not_nested(&self, msg: &'static str) {
        debug_assert!(!self.nested, "{}", msg);
    }
    #[inline]
    fn assert_finished(&self, msg: &'static str) {
        debug_assert!(self.finished, "{}", msg);
    }
    #[inline]
    fn assert_not_finished(&self, msg: &'static str) {
        debug_assert!(!self.finished, "{}", msg);
    }
}

/// Compute the length of the vtable needed to represent the provided FieldLocs.
/// If there are no FieldLocs, then provide the minimum number of bytes
/// required: enough to write the VTable header.
#[inline]
fn get_vtable_byte_len(field_locs: &[FieldLoc]) -> usize {
    let max_voffset = field_locs.iter().map(|fl| fl.id).max();
    match max_voffset {
        None => field_index_to_field_offset(0) as usize,
        Some(mv) => mv as usize + SIZE_VOFFSET,
    }
}

#[inline]
fn padding_bytes(buf_size: usize, scalar_size: usize) -> usize {
    // ((!buf_size) + 1) & (scalar_size - 1)
    (!buf_size).wrapping_add(1) & (scalar_size.wrapping_sub(1))
}

impl<'fbb> Default for FlatBufferBuilder<'fbb> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

/// An index that indexes from the reverse of a slice.
///
/// Note that while the internal representation is an index
/// from the end of a buffer, operations like `Add` and `Sub`
/// behave like a regular index:
///
/// # Examples
///
/// ```ignore
/// let buf = [0, 1, 2, 3, 4, 5];
/// let idx = ReverseIndex::end() - 2;
/// assert_eq!(&buf[idx.range_to_end()], &[4, 5]);
/// assert_eq!(idx.to_forward_index(&buf), 4);
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReverseIndex(usize);

impl ReverseIndex {
    /// Returns an index set to the end.
    ///
    /// Note: Indexing this will result in an out of bounds error.
    pub fn end() -> Self {
        Self(0)
    }

    /// Returns a struct equivalent to the range `self..`
    pub fn range_to_end(self) -> ReverseIndexRange {
        ReverseIndexRange(self, ReverseIndex::end())
    }

    /// Returns a struct equivalent to the range `self..end`
    pub fn range_to(self, end: ReverseIndex) -> ReverseIndexRange {
        ReverseIndexRange(self, end)
    }

    /// Transforms this reverse index into a regular index for the given buffer.
    pub fn to_forward_index<T>(self, buf: &[T]) -> usize {
        buf.len() - self.0
    }

    /// Returns the number of elements until the end of the range.
    pub fn distance_to_end(&self) -> usize {
        self.0
    }
}

impl Sub<usize> for ReverseIndex {
    type Output = Self;

    fn sub(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl SubAssign<usize> for ReverseIndex {
    fn sub_assign(&mut self, rhs: usize) {
        *self = *self - rhs;
    }
}

impl Add<usize> for ReverseIndex {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl AddAssign<usize> for ReverseIndex {
    fn add_assign(&mut self, rhs: usize) {
        *self = *self + rhs;
    }
}
impl<T> Index<ReverseIndex> for [T] {
    type Output = T;

    fn index(&self, index: ReverseIndex) -> &Self::Output {
        let index = index.to_forward_index(self);
        &self[index]
    }
}

impl<T> IndexMut<ReverseIndex> for [T] {
    fn index_mut(&mut self, index: ReverseIndex) -> &mut Self::Output {
        let index = index.to_forward_index(self);
        &mut self[index]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReverseIndexRange(ReverseIndex, ReverseIndex);

impl<T> Index<ReverseIndexRange> for [T] {
    type Output = [T];

    fn index(&self, index: ReverseIndexRange) -> &Self::Output {
        let start = index.0.to_forward_index(self);
        let end = index.1.to_forward_index(self);
        &self[start..end]
    }
}

impl<T> IndexMut<ReverseIndexRange> for [T] {
    fn index_mut(&mut self, index: ReverseIndexRange) -> &mut Self::Output {
        let start = index.0.to_forward_index(self);
        let end = index.1.to_forward_index(self);
        &mut self[start..end]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_index_test() {
        let buf = [0, 1, 2, 3, 4, 5];
        let idx = ReverseIndex::end() - 2;
        assert_eq!(&buf[idx.range_to_end()], &[4, 5]);
        assert_eq!(&buf[idx.range_to(idx + 1)], &[4]);
        assert_eq!(idx.to_forward_index(&buf), 4);
    }
}
