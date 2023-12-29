use crate::TagSectionReader;
use crate::{AliasSectionReader, InstanceSectionReader};
use crate::{BinaryReader, BinaryReaderError, FunctionBody, Range, Result};
use crate::{DataSectionReader, ElementSectionReader, ExportSectionReader};
use crate::{FunctionSectionReader, ImportSectionReader, TypeSectionReader};
use crate::{GlobalSectionReader, MemorySectionReader, TableSectionReader};
use std::convert::TryInto;
use std::fmt;
use std::iter;

/// An incremental parser of a binary WebAssembly module.
///
/// This type is intended to be used to incrementally parse a WebAssembly module
/// as bytes become available for the module. This can also be used to parse
/// modules that are already entirely resident within memory.
///
/// This primary function for a parser is the [`Parser::parse`] function which
/// will incrementally consume input. You can also use the [`Parser::parse_all`]
/// function to parse a module that is entirely resident in memory.
#[derive(Debug, Clone)]
pub struct Parser {
    state: State,
    offset: u64,
    max_size: u64,
}

#[derive(Debug, Clone)]
enum State {
    ModuleHeader,
    SectionStart,
    FunctionBody { remaining: u32, len: u32 },
    Module { remaining: u32, len: u32 },
}

/// A successful return payload from [`Parser::parse`].
///
/// On success one of two possible values can be returned, either that more data
/// is needed to continue parsing or a chunk of the input was parsed, indicating
/// how much of it was parsed.
#[derive(Debug)]
pub enum Chunk<'a> {
    /// This can be returned at any time and indicates that more data is needed
    /// to proceed with parsing. Zero bytes were consumed from the input to
    /// [`Parser::parse`]. The `usize` value here is a hint as to how many more
    /// bytes are needed to continue parsing.
    NeedMoreData(u64),

    /// A chunk was successfully parsed.
    Parsed {
        /// This many bytes of the `data` input to [`Parser::parse`] were
        /// consumed to produce `payload`.
        consumed: usize,
        /// The value that we actually parsed.
        payload: Payload<'a>,
    },
}

/// Values that can be parsed from a wasm module.
///
/// This enumeration is all possible chunks of pieces that can be parsed by a
/// [`Parser`] from a binary WebAssembly module. Note that for many sections the
/// entire section is parsed all at once, whereas other functions, like the code
/// section, are parsed incrementally. This is a distinction where some
/// sections, like the type section, are required to be fully resident in memory
/// (fully downloaded) before proceeding. Other sections, like the code section,
/// can be processed in a streaming fashion where each function is extracted
/// individually so it can possibly be shipped to another thread while you wait
/// for more functions to get downloaded.
///
/// Note that payloads, when returned, do not indicate that the wasm module is
/// valid. For example when you receive a `Payload::TypeSection` the type
/// section itself has not yet actually been parsed. The reader returned will be
/// able to parse it, but you'll have to actually iterate the reader to do the
/// full parse. Each payload returned is intended to be a *window* into the
/// original `data` passed to [`Parser::parse`] which can be further processed
/// if necessary.
pub enum Payload<'a> {
    /// Indicates the header of a WebAssembly binary.
    ///
    /// This header also indicates the version number that was parsed, which is
    /// currently always 1.
    Version {
        /// The version number found
        num: u32,
        /// The range of bytes that were parsed to consume the header of the
        /// module. Note that this range is relative to the start of the byte
        /// stream.
        range: Range,
    },

    /// A type section was received, and the provided reader can be used to
    /// parse the contents of the type section.
    TypeSection(crate::TypeSectionReader<'a>),
    /// A import section was received, and the provided reader can be used to
    /// parse the contents of the import section.
    ImportSection(crate::ImportSectionReader<'a>),
    /// An alias section was received, and the provided reader can be used to
    /// parse the contents of the alias section.
    AliasSection(crate::AliasSectionReader<'a>),
    /// An instance section was received, and the provided reader can be used to
    /// parse the contents of the instance section.
    InstanceSection(crate::InstanceSectionReader<'a>),
    /// A function section was received, and the provided reader can be used to
    /// parse the contents of the function section.
    FunctionSection(crate::FunctionSectionReader<'a>),
    /// A table section was received, and the provided reader can be used to
    /// parse the contents of the table section.
    TableSection(crate::TableSectionReader<'a>),
    /// A memory section was received, and the provided reader can be used to
    /// parse the contents of the memory section.
    MemorySection(crate::MemorySectionReader<'a>),
    /// An tag section was received, and the provided reader can be used to
    /// parse the contents of the tag section.
    TagSection(crate::TagSectionReader<'a>),
    /// A global section was received, and the provided reader can be used to
    /// parse the contents of the global section.
    GlobalSection(crate::GlobalSectionReader<'a>),
    /// An export section was received, and the provided reader can be used to
    /// parse the contents of the export section.
    ExportSection(crate::ExportSectionReader<'a>),
    /// A start section was received, and the `u32` here is the index of the
    /// start function.
    StartSection {
        /// The start function index
        func: u32,
        /// The range of bytes that specify the `func` field, specified in
        /// offsets relative to the start of the byte stream.
        range: Range,
    },
    /// An element section was received, and the provided reader can be used to
    /// parse the contents of the element section.
    ElementSection(crate::ElementSectionReader<'a>),
    /// A data count section was received, and the `u32` here is the contents of
    /// the data count section.
    DataCountSection {
        /// The number of data segments.
        count: u32,
        /// The range of bytes that specify the `count` field, specified in
        /// offsets relative to the start of the byte stream.
        range: Range,
    },
    /// A data section was received, and the provided reader can be used to
    /// parse the contents of the data section.
    DataSection(crate::DataSectionReader<'a>),
    /// A custom section was found.
    CustomSection {
        /// The name of the custom section.
        name: &'a str,
        /// The offset, relative to the start of the original module, that the
        /// `data` payload for this custom section starts at.
        data_offset: usize,
        /// The actual contents of the custom section.
        data: &'a [u8],
        /// The range of bytes that specify this whole custom section (including
        /// both the name of this custom section and its data) specified in
        /// offsets relative to the start of the byte stream.
        range: Range,
    },

    /// Indicator of the start of the code section.
    ///
    /// This entry is returned whenever the code section starts. The `count`
    /// field indicates how many entries are in this code section. After
    /// receiving this start marker you're guaranteed that the next `count`
    /// items will be either `CodeSectionEntry` or an error will be returned.
    ///
    /// This, unlike other sections, is intended to be used for streaming the
    /// contents of the code section. The code section is not required to be
    /// fully resident in memory when we parse it. Instead a [`Parser`] is
    /// capable of parsing piece-by-piece of a code section.
    CodeSectionStart {
        /// The number of functions in this section.
        count: u32,
        /// The range of bytes that represent this section, specified in
        /// offsets relative to the start of the byte stream.
        range: Range,
        /// The size, in bytes, of the remaining contents of this section.
        ///
        /// This can be used in combination with [`Parser::skip_section`]
        /// where the caller will know how many bytes to skip before feeding
        /// bytes into `Parser` again.
        size: u32,
    },

    /// An entry of the code section, a function, was parsed.
    ///
    /// This entry indicates that a function was successfully received from the
    /// code section, and the payload here is the window into the original input
    /// where the function resides. Note that the function itself has not been
    /// parsed, it's only been outlined. You'll need to process the
    /// `FunctionBody` provided to test whether it parses and/or is valid.
    CodeSectionEntry(crate::FunctionBody<'a>),

    /// Indicator of the start of the module code section.
    ///
    /// This behaves the same as the `CodeSectionStart` payload being returned.
    /// You're guaranteed the next `count` items will be of type
    /// `ModuleSectionEntry`.
    ModuleSectionStart {
        /// The number of inline modules in this section.
        count: u32,
        /// The range of bytes that represent this section, specified in
        /// offsets relative to the start of the byte stream.
        range: Range,
        /// The size, in bytes, of the remaining contents of this section.
        size: u32,
    },

    /// An entry of the module code section, a module, was parsed.
    ///
    /// This variant is special in that it returns a sub-`Parser`. Upon
    /// receiving a `ModuleSectionEntry` it is expected that the returned
    /// `Parser` will be used instead of the parent `Parser` until the parse has
    /// finished. You'll need to feed data into the `Parser` returned until it
    /// returns `Payload::End`. After that you'll switch back to the parent
    /// parser to resume parsing the rest of the module code section.
    ///
    /// Note that binaries will not be parsed correctly if you feed the data for
    /// a nested module into the parent [`Parser`].
    ModuleSectionEntry {
        /// The parser to use to parse the contents of the nested submodule.
        /// This parser should be used until it reports `End`.
        parser: Parser,
        /// The range of bytes, relative to the start of the input stream, of
        /// the bytes containing this submodule.
        range: Range,
    },

    /// An unknown section was found.
    ///
    /// This variant is returned for all unknown sections in a wasm file. This
    /// likely wants to be interpreted as an error by consumers of the parser,
    /// but this can also be used to parse sections unknown to wasmparser at
    /// this time.
    UnknownSection {
        /// The 8-bit identifier for this section.
        id: u8,
        /// The contents of this section.
        contents: &'a [u8],
        /// The range of bytes, relative to the start of the original data
        /// stream, that the contents of this section reside in.
        range: Range,
    },

    /// The end of the WebAssembly module was reached.
    End,
}

impl Parser {
    /// Creates a new module parser.
    ///
    /// Reports errors and ranges relative to `offset` provided, where `offset`
    /// is some logical offset within the input stream that we're parsing.
    pub fn new(offset: u64) -> Parser {
        Parser {
            state: State::ModuleHeader,
            offset,
            max_size: u64::max_value(),
        }
    }

    /// Attempts to parse a chunk of data.
    ///
    /// This method will attempt to parse the next incremental portion of a
    /// WebAssembly binary. Data available for the module is provided as `data`,
    /// and the data can be incomplete if more data has yet to arrive for the
    /// module. The `eof` flag indicates whether `data` represents all possible
    /// data for the module and no more data will ever be received.
    ///
    /// There are two ways parsing can succeed with this method:
    ///
    /// * `Chunk::NeedMoreData` - this indicates that there is not enough bytes
    ///   in `data` to parse a chunk of this module. The caller needs to wait
    ///   for more data to be available in this situation before calling this
    ///   method again. It is guaranteed that this is only returned if `eof` is
    ///   `false`.
    ///
    /// * `Chunk::Parsed` - this indicates that a chunk of the input was
    ///   successfully parsed. The payload is available in this variant of what
    ///   was parsed, and this also indicates how many bytes of `data` was
    ///   consumed. It's expected that the caller will not provide these bytes
    ///   back to the [`Parser`] again.
    ///
    /// Note that all `Chunk` return values are connected, with a lifetime, to
    /// the input buffer. Each parsed chunk borrows the input buffer and is a
    /// view into it for successfully parsed chunks.
    ///
    /// It is expected that you'll call this method until `Payload::End` is
    /// reached, at which point you're guaranteed that the module has completely
    /// parsed. Note that complete parsing, for the top-level wasm module,
    /// implies that `data` is empty and `eof` is `true`.
    ///
    /// # Errors
    ///
    /// Parse errors are returned as an `Err`. Errors can happen when the
    /// structure of the module is unexpected, or if sections are too large for
    /// example. Note that errors are not returned for malformed *contents* of
    /// sections here. Sections are generally not individually parsed and each
    /// returned [`Payload`] needs to be iterated over further to detect all
    /// errors.
    ///
    /// # Examples
    ///
    /// An example of reading a wasm file from a stream (`std::io::Read`) and
    /// incrementally parsing it.
    ///
    /// ```
    /// use std::io::Read;
    /// use anyhow::Result;
    /// use wasmparser::{Parser, Chunk, Payload::*};
    ///
    /// fn parse(mut reader: impl Read) -> Result<()> {
    ///     let mut buf = Vec::new();
    ///     let mut parser = Parser::new(0);
    ///     let mut eof = false;
    ///     let mut stack = Vec::new();
    ///
    ///     loop {
    ///         let (payload, consumed) = match parser.parse(&buf, eof)? {
    ///             Chunk::NeedMoreData(hint) => {
    ///                 assert!(!eof); // otherwise an error would be returned
    ///
    ///                 // Use the hint to preallocate more space, then read
    ///                 // some more data into our buffer.
    ///                 //
    ///                 // Note that the buffer management here is not ideal,
    ///                 // but it's compact enough to fit in an example!
    ///                 let len = buf.len();
    ///                 buf.extend((0..hint).map(|_| 0u8));
    ///                 let n = reader.read(&mut buf[len..])?;
    ///                 buf.truncate(len + n);
    ///                 eof = n == 0;
    ///                 continue;
    ///             }
    ///
    ///             Chunk::Parsed { consumed, payload } => (payload, consumed),
    ///         };
    ///
    ///         match payload {
    ///             // Each of these would be handled individually as necessary
    ///             Version { .. } => { /* ... */ }
    ///             TypeSection(_) => { /* ... */ }
    ///             ImportSection(_) => { /* ... */ }
    ///             AliasSection(_) => { /* ... */ }
    ///             InstanceSection(_) => { /* ... */ }
    ///             FunctionSection(_) => { /* ... */ }
    ///             TableSection(_) => { /* ... */ }
    ///             MemorySection(_) => { /* ... */ }
    ///             TagSection(_) => { /* ... */ }
    ///             GlobalSection(_) => { /* ... */ }
    ///             ExportSection(_) => { /* ... */ }
    ///             StartSection { .. } => { /* ... */ }
    ///             ElementSection(_) => { /* ... */ }
    ///             DataCountSection { .. } => { /* ... */ }
    ///             DataSection(_) => { /* ... */ }
    ///
    ///             // Here we know how many functions we'll be receiving as
    ///             // `CodeSectionEntry`, so we can prepare for that, and
    ///             // afterwards we can parse and handle each function
    ///             // individually.
    ///             CodeSectionStart { .. } => { /* ... */ }
    ///             CodeSectionEntry(body) => {
    ///                 // here we can iterate over `body` to parse the function
    ///                 // and its locals
    ///             }
    ///
    ///             // When parsing nested modules we need to switch which
    ///             // `Parser` we're using.
    ///             ModuleSectionStart { .. } => { /* ... */ }
    ///             ModuleSectionEntry { parser: subparser, .. } => {
    ///                 stack.push(parser);
    ///                 parser = subparser;
    ///             }
    ///
    ///             CustomSection { name, .. } => { /* ... */ }
    ///
    ///             // most likely you'd return an error here
    ///             UnknownSection { id, .. } => { /* ... */ }
    ///
    ///             // Once we've reached the end of a module we either resume
    ///             // at the parent module or we break out of the loop because
    ///             // we're done.
    ///             End => {
    ///                 if let Some(parent_parser) = stack.pop() {
    ///                     parser = parent_parser;
    ///                 } else {
    ///                     break;
    ///                 }
    ///             }
    ///         }
    ///
    ///         // once we're done processing the payload we can forget the
    ///         // original.
    ///         buf.drain(..consumed);
    ///     }
    ///
    ///     Ok(())
    /// }
    ///
    /// # parse(&b"\0asm\x01\0\0\0"[..]).unwrap();
    /// ```
    pub fn parse<'a>(&mut self, data: &'a [u8], eof: bool) -> Result<Chunk<'a>> {
        let (data, eof) = if usize_to_u64(data.len()) > self.max_size {
            (&data[..(self.max_size as usize)], true)
        } else {
            (data, eof)
        };
        // TODO: thread through `offset: u64` to `BinaryReader`, remove
        // the cast here.
        let mut reader = BinaryReader::new_with_offset(data, self.offset as usize);
        match self.parse_reader(&mut reader, eof) {
            Ok(payload) => {
                // Be sure to update our offset with how far we got in the
                // reader
                self.offset += usize_to_u64(reader.position);
                self.max_size -= usize_to_u64(reader.position);
                Ok(Chunk::Parsed {
                    consumed: reader.position,
                    payload,
                })
            }
            Err(e) => {
                // If we're at EOF then there's no way we can recover from any
                // error, so continue to propagate it.
                if eof {
                    return Err(e);
                }

                // If our error doesn't look like it can be resolved with more
                // data being pulled down, then propagate it, otherwise switch
                // the error to "feed me please"
                match e.inner.needed_hint {
                    Some(hint) => Ok(Chunk::NeedMoreData(usize_to_u64(hint))),
                    None => Err(e),
                }
            }
        }
    }

    fn parse_reader<'a>(
        &mut self,
        reader: &mut BinaryReader<'a>,
        eof: bool,
    ) -> Result<Payload<'a>> {
        use Payload::*;

        match self.state {
            State::ModuleHeader => {
                let start = reader.original_position();
                let num = reader.read_file_header()?;
                self.state = State::SectionStart;
                Ok(Version {
                    num,
                    range: Range {
                        start,
                        end: reader.original_position(),
                    },
                })
            }
            State::SectionStart => {
                // If we're at eof and there are no bytes in our buffer, then
                // that means we reached the end of the wasm file since it's
                // just a bunch of sections concatenated after the module
                // header.
                if eof && reader.bytes_remaining() == 0 {
                    return Ok(Payload::End);
                }

                let id = reader.read_var_u7()? as u8;
                let len_pos = reader.position;
                let mut len = reader.read_var_u32()?;

                // Test to make sure that this section actually fits within
                // `Parser::max_size`. This doesn't matter for top-level modules
                // but it is required for nested modules to correctly ensure
                // that all sections live entirely within their section of the
                // file.
                let section_overflow = self
                    .max_size
                    .checked_sub(usize_to_u64(reader.position))
                    .and_then(|s| s.checked_sub(len.into()))
                    .is_none();
                if section_overflow {
                    return Err(BinaryReaderError::new("section too large", len_pos));
                }

                match id {
                    0 => {
                        let start = reader.original_position();
                        let range = Range {
                            start,
                            end: reader.original_position() + len as usize,
                        };
                        let mut content = subreader(reader, len)?;
                        // Note that if this fails we can't read any more bytes,
                        // so clear the "we'd succeed if we got this many more
                        // bytes" because we can't recover from "eof" at this point.
                        let name = content.read_string().map_err(clear_hint)?;
                        Ok(Payload::CustomSection {
                            name,
                            data_offset: content.original_position(),
                            data: content.remaining_buffer(),
                            range,
                        })
                    }
                    1 => section(reader, len, TypeSectionReader::new, TypeSection),
                    2 => section(reader, len, ImportSectionReader::new, ImportSection),
                    3 => section(reader, len, FunctionSectionReader::new, FunctionSection),
                    4 => section(reader, len, TableSectionReader::new, TableSection),
                    5 => section(reader, len, MemorySectionReader::new, MemorySection),
                    6 => section(reader, len, GlobalSectionReader::new, GlobalSection),
                    7 => section(reader, len, ExportSectionReader::new, ExportSection),
                    8 => {
                        let (func, range) = single_u32(reader, len, "start")?;
                        Ok(StartSection { func, range })
                    }
                    9 => section(reader, len, ElementSectionReader::new, ElementSection),
                    10 => {
                        let start = reader.original_position();
                        let count = delimited(reader, &mut len, |r| r.read_var_u32())?;
                        let range = Range {
                            start,
                            end: reader.original_position() + len as usize,
                        };
                        self.state = State::FunctionBody {
                            remaining: count,
                            len,
                        };
                        Ok(CodeSectionStart {
                            count,
                            range,
                            size: len,
                        })
                    }
                    11 => section(reader, len, DataSectionReader::new, DataSection),
                    12 => {
                        let (count, range) = single_u32(reader, len, "data count")?;
                        Ok(DataCountSection { count, range })
                    }
                    13 => section(reader, len, TagSectionReader::new, TagSection),
                    14 => {
                        let start = reader.original_position();
                        let count = delimited(reader, &mut len, |r| r.read_var_u32())?;
                        let range = Range {
                            start,
                            end: reader.original_position() + len as usize,
                        };
                        self.state = State::Module {
                            remaining: count,
                            len,
                        };
                        Ok(ModuleSectionStart {
                            count,
                            range,
                            size: len,
                        })
                    }
                    15 => section(reader, len, InstanceSectionReader::new, InstanceSection),
                    16 => section(reader, len, AliasSectionReader::new, AliasSection),
                    id => {
                        let offset = reader.original_position();
                        let contents = reader.read_bytes(len as usize)?;
                        let range = Range {
                            start: offset,
                            end: offset + len as usize,
                        };
                        Ok(UnknownSection {
                            id,
                            contents,
                            range,
                        })
                    }
                }
            }

            // Once we hit 0 remaining incrementally parsed items, with 0
            // remaining bytes in each section, we're done and can switch back
            // to parsing sections.
            State::FunctionBody {
                remaining: 0,
                len: 0,
            }
            | State::Module {
                remaining: 0,
                len: 0,
            } => {
                self.state = State::SectionStart;
                self.parse_reader(reader, eof)
            }

            // ... otherwise trailing bytes with no remaining entries in these
            // sections indicates an error.
            State::FunctionBody { remaining: 0, len } | State::Module { remaining: 0, len } => {
                debug_assert!(len > 0);
                let offset = reader.original_position();
                Err(BinaryReaderError::new(
                    "trailing bytes at end of section",
                    offset,
                ))
            }

            // Functions are relatively easy to parse when we know there's at
            // least one remaining and at least one byte available to read
            // things.
            //
            // We use the remaining length try to read a u32 size of the
            // function, and using that size we require the entire function be
            // resident in memory. This means that we're reading whole chunks of
            // functions at a time.
            //
            // Limiting via `Parser::max_size` (nested modules) happens above in
            // `fn parse`, and limiting by our section size happens via
            // `delimited`. Actual parsing of the function body is delegated to
            // the caller to iterate over the `FunctionBody` structure.
            State::FunctionBody { remaining, mut len } => {
                let body = delimited(reader, &mut len, |r| {
                    let size = r.read_var_u32()?;
                    let offset = r.original_position();
                    Ok(FunctionBody::new(offset, r.read_bytes(size as usize)?))
                })?;
                self.state = State::FunctionBody {
                    remaining: remaining - 1,
                    len,
                };
                Ok(CodeSectionEntry(body))
            }

            // Modules are trickier than functions. What's going to happen here
            // is that we'll be offloading parsing to a sub-`Parser`. This
            // sub-`Parser` will be delimited to not read past the size of the
            // module that's specified.
            //
            // So the first thing that happens here is we read the size of the
            // module. We use `delimited` to make sure the bytes specifying the
            // size of the module are themselves within the module code section.
            //
            // Once we've read the size of a module, however, there's a few
            // pieces of state that we need to update. We as a parser will not
            // receive the next `size` bytes, so we need to update our internal
            // bookkeeping to account for that:
            //
            // * The `len`, number of bytes remaining in this section, is
            //   decremented by `size`. This can underflow, however, meaning
            //   that the size of the module doesn't fit within the section.
            //
            // * Our `Parser::max_size` field needs to account for the bytes
            //   that we're reading. Note that this is guaranteed to not
            //   underflow, however, because whenever we parse a section header
            //   we guarantee that its contents fit within our `max_size`.
            //
            // To update `len` we do that when updating `self.state`, and to
            // update `max_size` we do that inline. Note that this will get
            // further tweaked after we return with the bytes we read specifying
            // the size of the module itself.
            State::Module { remaining, mut len } => {
                let size = delimited(reader, &mut len, |r| r.read_var_u32())?;
                match len.checked_sub(size) {
                    Some(i) => len = i,
                    None => {
                        return Err(BinaryReaderError::new(
                            "Unexpected EOF",
                            reader.original_position(),
                        ));
                    }
                }
                self.state = State::Module {
                    remaining: remaining - 1,
                    len,
                };
                let range = Range {
                    start: reader.original_position(),
                    end: reader.original_position() + size as usize,
                };
                self.max_size -= u64::from(size);
                self.offset += u64::from(size);
                let mut parser = Parser::new(usize_to_u64(reader.original_position()));
                parser.max_size = size.into();
                Ok(ModuleSectionEntry { parser, range })
            }
        }
    }

    /// Convenience function that can be used to parse a module entirely
    /// resident in memory.
    ///
    /// This function will parse the `data` provided as a WebAssembly module,
    /// assuming that `data` represents the entire WebAssembly module.
    ///
    /// Note that when this function yields `ModuleSectionEntry`
    /// no action needs to be taken with the returned parser. The parser will be
    /// automatically switched to internally and more payloads will continue to
    /// get returned.
    pub fn parse_all(self, mut data: &[u8]) -> impl Iterator<Item = Result<Payload>> {
        let mut stack = Vec::new();
        let mut cur = self;
        let mut done = false;
        iter::from_fn(move || {
            if done {
                return None;
            }
            let payload = match cur.parse(data, true) {
                // Propagate all errors
                Err(e) => {
                    done = true;
                    return Some(Err(e));
                }

                // This isn't possible because `eof` is always true.
                Ok(Chunk::NeedMoreData(_)) => unreachable!(),

                Ok(Chunk::Parsed { payload, consumed }) => {
                    data = &data[consumed..];
                    payload
                }
            };

            match &payload {
                // If a module ends then we either finished the current
                // module or, if there's a parent, we switch back to
                // resuming parsing the parent.
                Payload::End => match stack.pop() {
                    Some(p) => cur = p,
                    None => done = true,
                },

                // When we enter a nested module then we need to update our
                // current parser, saving off the previous state.
                //
                // Afterwards we turn the loop again to recurse in parsing the
                // nested module.
                Payload::ModuleSectionEntry { parser, range: _ } => {
                    stack.push(cur.clone());
                    cur = parser.clone();
                }

                _ => {}
            }

            Some(Ok(payload))
        })
    }

    /// Skip parsing the code or module code section entirely.
    ///
    /// This function can be used to indicate, after receiving
    /// `CodeSectionStart` or `ModuleSectionStart`, that the section
    /// will not be parsed.
    ///
    /// The caller will be responsible for skipping `size` bytes (found in the
    /// `CodeSectionStart` or `ModuleSectionStart` payload). Bytes should
    /// only be fed into `parse` after the `size` bytes have been skipped.
    ///
    /// # Panics
    ///
    /// This function will panic if the parser is not in a state where it's
    /// parsing the code or module code section.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasmparser::{Result, Parser, Chunk, Range, SectionReader, Payload::*};
    ///
    /// fn objdump_headers(mut wasm: &[u8]) -> Result<()> {
    ///     let mut parser = Parser::new(0);
    ///     loop {
    ///         let payload = match parser.parse(wasm, true)? {
    ///             Chunk::Parsed { consumed, payload } => {
    ///                 wasm = &wasm[consumed..];
    ///                 payload
    ///             }
    ///             // this state isn't possible with `eof = true`
    ///             Chunk::NeedMoreData(_) => unreachable!(),
    ///         };
    ///         match payload {
    ///             TypeSection(s) => print_range("type section", &s.range()),
    ///             ImportSection(s) => print_range("import section", &s.range()),
    ///             // .. other sections
    ///
    ///             // Print the range of the code section we see, but don't
    ///             // actually iterate over each individual function.
    ///             CodeSectionStart { range, size, .. } => {
    ///                 print_range("code section", &range);
    ///                 parser.skip_section();
    ///                 wasm = &wasm[size as usize..];
    ///             }
    ///             End => break,
    ///             _ => {}
    ///         }
    ///     }
    ///     Ok(())
    /// }
    ///
    /// fn print_range(section: &str, range: &Range) {
    ///     println!("{:>40}: {:#010x} - {:#010x}", section, range.start, range.end);
    /// }
    /// ```
    pub fn skip_section(&mut self) {
        let skip = match self.state {
            State::FunctionBody { remaining: _, len } | State::Module { remaining: _, len } => len,
            _ => panic!("wrong state to call `skip_section`"),
        };
        self.offset += u64::from(skip);
        self.max_size -= u64::from(skip);
        self.state = State::SectionStart;
    }
}

fn usize_to_u64(a: usize) -> u64 {
    a.try_into().unwrap()
}

/// Parses an entire section resident in memory into a `Payload`.
///
/// Requires that `len` bytes are resident in `reader` and uses `ctor`/`variant`
/// to construct the section to return.
fn section<'a, T>(
    reader: &mut BinaryReader<'a>,
    len: u32,
    ctor: fn(&'a [u8], usize) -> Result<T>,
    variant: fn(T) -> Payload<'a>,
) -> Result<Payload<'a>> {
    let offset = reader.original_position();
    let payload = reader.read_bytes(len as usize)?;
    // clear the hint for "need this many more bytes" here because we already
    // read all the bytes, so it's not possible to read more bytes if this
    // fails.
    let reader = ctor(payload, offset).map_err(clear_hint)?;
    Ok(variant(reader))
}

/// Creates a new `BinaryReader` from the given `reader` which will be reading
/// the first `len` bytes.
///
/// This means that `len` bytes must be resident in memory at the time of this
/// reading.
fn subreader<'a>(reader: &mut BinaryReader<'a>, len: u32) -> Result<BinaryReader<'a>> {
    let offset = reader.original_position();
    let payload = reader.read_bytes(len as usize)?;
    Ok(BinaryReader::new_with_offset(payload, offset))
}

/// Reads a section that is represented by a single uleb-encoded `u32`.
fn single_u32<'a>(reader: &mut BinaryReader<'a>, len: u32, desc: &str) -> Result<(u32, Range)> {
    let range = Range {
        start: reader.original_position(),
        end: reader.original_position() + len as usize,
    };
    let mut content = subreader(reader, len)?;
    // We can't recover from "unexpected eof" here because our entire section is
    // already resident in memory, so clear the hint for how many more bytes are
    // expected.
    let index = content.read_var_u32().map_err(clear_hint)?;
    if !content.eof() {
        return Err(BinaryReaderError::new(
            format!("Unexpected content in the {} section", desc),
            content.original_position(),
        ));
    }
    Ok((index, range))
}

/// Attempts to parse using `f`.
///
/// This will update `*len` with the number of bytes consumed, and it will cause
/// a failure to be returned instead of the number of bytes consumed exceeds
/// what `*len` currently is.
fn delimited<'a, T>(
    reader: &mut BinaryReader<'a>,
    len: &mut u32,
    f: impl FnOnce(&mut BinaryReader<'a>) -> Result<T>,
) -> Result<T> {
    let start = reader.position;
    let ret = f(reader)?;
    *len = match (reader.position - start)
        .try_into()
        .ok()
        .and_then(|i| len.checked_sub(i))
    {
        Some(i) => i,
        None => return Err(BinaryReaderError::new("Unexpected EOF", start)),
    };
    Ok(ret)
}

impl Default for Parser {
    fn default() -> Parser {
        Parser::new(0)
    }
}

impl fmt::Debug for Payload<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Payload::*;
        match self {
            CustomSection {
                name,
                data_offset,
                data: _,
                range,
            } => f
                .debug_struct("CustomSection")
                .field("name", name)
                .field("data_offset", data_offset)
                .field("range", range)
                .field("data", &"...")
                .finish(),
            Version { num, range } => f
                .debug_struct("Version")
                .field("num", num)
                .field("range", range)
                .finish(),
            TypeSection(_) => f.debug_tuple("TypeSection").field(&"...").finish(),
            ImportSection(_) => f.debug_tuple("ImportSection").field(&"...").finish(),
            AliasSection(_) => f.debug_tuple("AliasSection").field(&"...").finish(),
            InstanceSection(_) => f.debug_tuple("InstanceSection").field(&"...").finish(),
            FunctionSection(_) => f.debug_tuple("FunctionSection").field(&"...").finish(),
            TableSection(_) => f.debug_tuple("TableSection").field(&"...").finish(),
            MemorySection(_) => f.debug_tuple("MemorySection").field(&"...").finish(),
            TagSection(_) => f.debug_tuple("TagSection").field(&"...").finish(),
            GlobalSection(_) => f.debug_tuple("GlobalSection").field(&"...").finish(),
            ExportSection(_) => f.debug_tuple("ExportSection").field(&"...").finish(),
            ElementSection(_) => f.debug_tuple("ElementSection").field(&"...").finish(),
            DataSection(_) => f.debug_tuple("DataSection").field(&"...").finish(),
            StartSection { func, range } => f
                .debug_struct("StartSection")
                .field("func", func)
                .field("range", range)
                .finish(),
            DataCountSection { count, range } => f
                .debug_struct("DataCountSection")
                .field("count", count)
                .field("range", range)
                .finish(),
            CodeSectionStart { count, range, size } => f
                .debug_struct("CodeSectionStart")
                .field("count", count)
                .field("range", range)
                .field("size", size)
                .finish(),
            CodeSectionEntry(_) => f.debug_tuple("CodeSectionEntry").field(&"...").finish(),
            ModuleSectionStart { count, range, size } => f
                .debug_struct("ModuleSectionStart")
                .field("count", count)
                .field("range", range)
                .field("size", size)
                .finish(),
            ModuleSectionEntry { parser: _, range } => f
                .debug_struct("ModuleSectionEntry")
                .field("range", range)
                .finish(),
            UnknownSection { id, range, .. } => f
                .debug_struct("UnknownSection")
                .field("id", id)
                .field("range", range)
                .finish(),
            End => f.write_str("End"),
        }
    }
}

fn clear_hint(mut err: BinaryReaderError) -> BinaryReaderError {
    err.inner.needed_hint = None;
    err
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_matches {
        ($a:expr, $b:pat $(,)?) => {
            match $a {
                $b => {}
                a => panic!("`{:?}` doesn't match `{}`", a, stringify!($b)),
            }
        };
    }

    #[test]
    fn header() {
        assert!(Parser::default().parse(&[], true).is_err());
        assert_matches!(
            Parser::default().parse(&[], false),
            Ok(Chunk::NeedMoreData(4)),
        );
        assert_matches!(
            Parser::default().parse(b"\0", false),
            Ok(Chunk::NeedMoreData(3)),
        );
        assert_matches!(
            Parser::default().parse(b"\0asm", false),
            Ok(Chunk::NeedMoreData(4)),
        );
        assert_matches!(
            Parser::default().parse(b"\0asm\x01\0\0\0", false),
            Ok(Chunk::Parsed {
                consumed: 8,
                payload: Payload::Version { num: 1, .. },
            }),
        );
    }

    #[test]
    fn header_iter() {
        for _ in Parser::default().parse_all(&[]) {}
        for _ in Parser::default().parse_all(b"\0") {}
        for _ in Parser::default().parse_all(b"\0asm") {}
        for _ in Parser::default().parse_all(b"\0asm\x01\x01\x01\x01") {}
    }

    fn parser_after_header() -> Parser {
        let mut p = Parser::default();
        assert_matches!(
            p.parse(b"\0asm\x01\0\0\0", false),
            Ok(Chunk::Parsed {
                consumed: 8,
                payload: Payload::Version { num: 1, .. },
            }),
        );
        return p;
    }

    #[test]
    fn start_section() {
        assert_matches!(
            parser_after_header().parse(&[], false),
            Ok(Chunk::NeedMoreData(1)),
        );
        assert!(parser_after_header().parse(&[8], true).is_err());
        assert!(parser_after_header().parse(&[8, 1], true).is_err());
        assert!(parser_after_header().parse(&[8, 2], true).is_err());
        assert_matches!(
            parser_after_header().parse(&[8], false),
            Ok(Chunk::NeedMoreData(1)),
        );
        assert_matches!(
            parser_after_header().parse(&[8, 1], false),
            Ok(Chunk::NeedMoreData(1)),
        );
        assert_matches!(
            parser_after_header().parse(&[8, 2], false),
            Ok(Chunk::NeedMoreData(2)),
        );
        assert_matches!(
            parser_after_header().parse(&[8, 1, 1], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::StartSection { func: 1, .. },
            }),
        );
        assert!(parser_after_header().parse(&[8, 2, 1, 1], false).is_err());
        assert!(parser_after_header().parse(&[8, 0], false).is_err());
    }

    #[test]
    fn end_works() {
        assert_matches!(
            parser_after_header().parse(&[], true),
            Ok(Chunk::Parsed {
                consumed: 0,
                payload: Payload::End,
            }),
        );
    }

    #[test]
    fn type_section() {
        assert!(parser_after_header().parse(&[1], true).is_err());
        assert!(parser_after_header().parse(&[1, 0], false).is_err());
        // assert!(parser_after_header().parse(&[8, 2], true).is_err());
        assert_matches!(
            parser_after_header().parse(&[1], false),
            Ok(Chunk::NeedMoreData(1)),
        );
        assert_matches!(
            parser_after_header().parse(&[1, 1], false),
            Ok(Chunk::NeedMoreData(1)),
        );
        assert_matches!(
            parser_after_header().parse(&[1, 1, 1], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::TypeSection(_),
            }),
        );
        assert_matches!(
            parser_after_header().parse(&[1, 1, 1, 2, 3, 4], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::TypeSection(_),
            }),
        );
    }

    #[test]
    fn custom_section() {
        assert!(parser_after_header().parse(&[0], true).is_err());
        assert!(parser_after_header().parse(&[0, 0], false).is_err());
        assert!(parser_after_header().parse(&[0, 1, 1], false).is_err());
        assert_matches!(
            parser_after_header().parse(&[0, 2, 1], false),
            Ok(Chunk::NeedMoreData(1)),
        );
        assert_matches!(
            parser_after_header().parse(&[0, 1, 0], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::CustomSection {
                    name: "",
                    data_offset: 11,
                    data: b"",
                    range: Range { start: 10, end: 11 },
                },
            }),
        );
        assert_matches!(
            parser_after_header().parse(&[0, 2, 1, b'a'], false),
            Ok(Chunk::Parsed {
                consumed: 4,
                payload: Payload::CustomSection {
                    name: "a",
                    data_offset: 12,
                    data: b"",
                    range: Range { start: 10, end: 12 },
                },
            }),
        );
        assert_matches!(
            parser_after_header().parse(&[0, 2, 0, b'a'], false),
            Ok(Chunk::Parsed {
                consumed: 4,
                payload: Payload::CustomSection {
                    name: "",
                    data_offset: 11,
                    data: b"a",
                    range: Range { start: 10, end: 12 },
                },
            }),
        );
    }

    #[test]
    fn function_section() {
        assert!(parser_after_header().parse(&[10], true).is_err());
        assert!(parser_after_header().parse(&[10, 0], true).is_err());
        assert!(parser_after_header().parse(&[10, 1], true).is_err());
        assert_matches!(
            parser_after_header().parse(&[10], false),
            Ok(Chunk::NeedMoreData(1))
        );
        assert_matches!(
            parser_after_header().parse(&[10, 1], false),
            Ok(Chunk::NeedMoreData(1))
        );
        let mut p = parser_after_header();
        assert_matches!(
            p.parse(&[10, 1, 0], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::CodeSectionStart { count: 0, .. },
            }),
        );
        assert_matches!(
            p.parse(&[], true),
            Ok(Chunk::Parsed {
                consumed: 0,
                payload: Payload::End,
            }),
        );
        let mut p = parser_after_header();
        assert_matches!(
            p.parse(&[10, 2, 1, 0], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::CodeSectionStart { count: 1, .. },
            }),
        );
        assert_matches!(
            p.parse(&[0], false),
            Ok(Chunk::Parsed {
                consumed: 1,
                payload: Payload::CodeSectionEntry(_),
            }),
        );
        assert_matches!(
            p.parse(&[], true),
            Ok(Chunk::Parsed {
                consumed: 0,
                payload: Payload::End,
            }),
        );

        // 1 byte section with 1 function can't read the function body because
        // the section is too small
        let mut p = parser_after_header();
        assert_matches!(
            p.parse(&[10, 1, 1], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::CodeSectionStart { count: 1, .. },
            }),
        );
        assert_eq!(
            p.parse(&[0], false).unwrap_err().message(),
            "Unexpected EOF"
        );

        // section with 2 functions but section is cut off
        let mut p = parser_after_header();
        assert_matches!(
            p.parse(&[10, 2, 2], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::CodeSectionStart { count: 2, .. },
            }),
        );
        assert_matches!(
            p.parse(&[0], false),
            Ok(Chunk::Parsed {
                consumed: 1,
                payload: Payload::CodeSectionEntry(_),
            }),
        );
        assert_matches!(p.parse(&[], false), Ok(Chunk::NeedMoreData(1)));
        assert_eq!(
            p.parse(&[0], false).unwrap_err().message(),
            "Unexpected EOF",
        );

        // trailing data is bad
        let mut p = parser_after_header();
        assert_matches!(
            p.parse(&[10, 3, 1], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::CodeSectionStart { count: 1, .. },
            }),
        );
        assert_matches!(
            p.parse(&[0], false),
            Ok(Chunk::Parsed {
                consumed: 1,
                payload: Payload::CodeSectionEntry(_),
            }),
        );
        assert_eq!(
            p.parse(&[0], false).unwrap_err().message(),
            "trailing bytes at end of section",
        );
    }

    #[test]
    fn module_code_errors() {
        // no bytes to say size of section
        assert!(parser_after_header().parse(&[14], true).is_err());
        // section must start with a u32
        assert!(parser_after_header().parse(&[14, 0], true).is_err());
        // EOF before we finish reading the section
        assert!(parser_after_header().parse(&[14, 1], true).is_err());
    }

    #[test]
    fn module_code_one() {
        let mut p = parser_after_header();
        assert_matches!(p.parse(&[14], false), Ok(Chunk::NeedMoreData(1)));
        assert_matches!(p.parse(&[14, 9], false), Ok(Chunk::NeedMoreData(1)));
        // Module code section, 10 bytes large, one module.
        assert_matches!(
            p.parse(&[14, 10, 1], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::ModuleSectionStart { count: 1, .. },
            })
        );
        // Declare an empty module, which will be 8 bytes large for the header.
        // Switch to the sub-parser on success.
        let mut sub = match p.parse(&[8], false) {
            Ok(Chunk::Parsed {
                consumed: 1,
                payload: Payload::ModuleSectionEntry { parser, .. },
            }) => parser,
            other => panic!("bad parse {:?}", other),
        };

        // Parse the header of the submodule with the sub-parser.
        assert_matches!(sub.parse(&[], false), Ok(Chunk::NeedMoreData(4)));
        assert_matches!(sub.parse(b"\0asm", false), Ok(Chunk::NeedMoreData(4)));
        assert_matches!(
            sub.parse(b"\0asm\x01\0\0\0", false),
            Ok(Chunk::Parsed {
                consumed: 8,
                payload: Payload::Version { num: 1, .. },
            }),
        );

        // The sub-parser should be byte-limited so the next byte shouldn't get
        // consumed, it's intended for the parent parser.
        assert_matches!(
            sub.parse(&[10], false),
            Ok(Chunk::Parsed {
                consumed: 0,
                payload: Payload::End,
            }),
        );

        // The parent parser should now be back to resuming, and we simulate it
        // being done with bytes to ensure that it's safely at the end,
        // completing the module code section.
        assert_matches!(p.parse(&[], false), Ok(Chunk::NeedMoreData(1)));
        assert_matches!(
            p.parse(&[], true),
            Ok(Chunk::Parsed {
                consumed: 0,
                payload: Payload::End,
            }),
        );
    }

    #[test]
    fn nested_section_too_big() {
        let mut p = parser_after_header();
        // Module code section, 12 bytes large, one module. This leaves 11 bytes
        // of payload for the module definition itself.
        assert_matches!(
            p.parse(&[14, 12, 1], false),
            Ok(Chunk::Parsed {
                consumed: 3,
                payload: Payload::ModuleSectionStart { count: 1, .. },
            })
        );
        // Use one byte to say we're a 10 byte module, which fits exactly within
        // our module code section.
        let mut sub = match p.parse(&[10], false) {
            Ok(Chunk::Parsed {
                consumed: 1,
                payload: Payload::ModuleSectionEntry { parser, .. },
            }) => parser,
            other => panic!("bad parse {:?}", other),
        };

        // use 8 bytes to parse the header, leaving 2 remaining bytes in our
        // module.
        assert_matches!(
            sub.parse(b"\0asm\x01\0\0\0", false),
            Ok(Chunk::Parsed {
                consumed: 8,
                payload: Payload::Version { num: 1, .. },
            }),
        );

        // We can't parse a section which declares its bigger than the outer
        // module. This is section 1, one byte big, with one content byte. The
        // content byte, however, lives outside of the parent's module code
        // section.
        assert_eq!(
            sub.parse(&[1, 1, 0], false).unwrap_err().message(),
            "section too large",
        );
    }
}
