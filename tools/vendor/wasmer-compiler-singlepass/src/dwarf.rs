use gimli::write::{Address, EndianVec, Result, Writer};
use gimli::{RunTimeEndian, SectionId};
use wasmer_types::entity::EntityRef;
use wasmer_types::{
    CustomSection, CustomSectionProtection, Endianness, LocalFunctionIndex, Relocation,
    RelocationKind, RelocationTarget, SectionBody,
};

#[derive(Clone, Debug)]
pub struct WriterRelocate {
    pub relocs: Vec<Relocation>,
    writer: EndianVec<RunTimeEndian>,
}

impl WriterRelocate {
    pub const FUNCTION_SYMBOL: usize = 0;
    pub fn new(endianness: Option<Endianness>) -> Self {
        let endianness = match endianness {
            Some(Endianness::Little) => RunTimeEndian::Little,
            Some(Endianness::Big) => RunTimeEndian::Big,
            // We autodetect it, based on the host
            None => RunTimeEndian::default(),
        };
        WriterRelocate {
            relocs: Vec::new(),
            writer: EndianVec::new(endianness),
        }
    }

    pub fn into_section(mut self) -> CustomSection {
        // GCC expects a terminating "empty" length, so write a 0 length at the end of the table.
        self.writer.write_u32(0).unwrap();
        let data = self.writer.into_vec();
        CustomSection {
            protection: CustomSectionProtection::Read,
            bytes: SectionBody::new_with_vec(data),
            relocations: self.relocs,
        }
    }
}

impl Writer for WriterRelocate {
    type Endian = RunTimeEndian;

    fn endian(&self) -> Self::Endian {
        self.writer.endian()
    }

    fn len(&self) -> usize {
        self.writer.len()
    }

    fn write(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write(bytes)
    }

    fn write_at(&mut self, offset: usize, bytes: &[u8]) -> Result<()> {
        self.writer.write_at(offset, bytes)
    }

    fn write_address(&mut self, address: Address, size: u8) -> Result<()> {
        match address {
            Address::Constant(val) => self.write_udata(val, size),
            Address::Symbol { symbol, addend } => {
                // Is a function relocation
                if symbol == Self::FUNCTION_SYMBOL {
                    // We use the addend to detect the function index
                    let function_index = LocalFunctionIndex::new(addend as _);
                    let reloc_target = RelocationTarget::LocalFunc(function_index);
                    let offset = self.len() as u32;
                    let kind = match size {
                        8 => RelocationKind::Abs8,
                        _ => unimplemented!("dwarf relocation size not yet supported: {}", size),
                    };
                    let addend = 0;
                    self.relocs.push(Relocation {
                        kind,
                        reloc_target,
                        offset,
                        addend,
                    });
                    self.write_udata(addend as u64, size)
                } else {
                    unreachable!("Symbol {} in DWARF not recognized", symbol);
                }
            }
        }
    }

    fn write_offset(&mut self, _val: usize, _section: SectionId, _size: u8) -> Result<()> {
        unimplemented!("write_offset not yet implemented");
    }

    fn write_offset_at(
        &mut self,
        _offset: usize,
        _val: usize,
        _section: SectionId,
        _size: u8,
    ) -> Result<()> {
        unimplemented!("write_offset_at not yet implemented");
    }
}
