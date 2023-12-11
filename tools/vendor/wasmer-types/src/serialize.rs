use crate::entity::PrimaryMap;
use crate::{
    compilation::target::CpuFeature, CompileModuleInfo, CompiledFunctionFrameInfo, CustomSection,
    DeserializeError, Dwarf, Features, FunctionBody, FunctionIndex, LocalFunctionIndex,
    MemoryIndex, MemoryStyle, ModuleInfo, OwnedDataInitializer, Relocation, SectionIndex,
    SerializeError, SignatureIndex, TableIndex, TableStyle,
};
use enumset::EnumSet;
use rkyv::check_archived_value;
use rkyv::{
    archived_value, de::deserializers::SharedDeserializeMap, ser::serializers::AllocSerializer,
    ser::Serializer as RkyvSerializer, Archive, CheckBytes, Deserialize as RkyvDeserialize,
    Serialize as RkyvSerialize,
};
use std::convert::TryInto;
use std::mem;

/// The compilation related data for a serialized modules
#[derive(Archive, Default, RkyvDeserialize, RkyvSerialize)]
#[allow(missing_docs)]
#[archive_attr(derive(CheckBytes))]
pub struct SerializableCompilation {
    pub function_bodies: PrimaryMap<LocalFunctionIndex, FunctionBody>,
    pub function_relocations: PrimaryMap<LocalFunctionIndex, Vec<Relocation>>,
    pub function_frame_info: PrimaryMap<LocalFunctionIndex, CompiledFunctionFrameInfo>,
    pub function_call_trampolines: PrimaryMap<SignatureIndex, FunctionBody>,
    pub dynamic_function_trampolines: PrimaryMap<FunctionIndex, FunctionBody>,
    pub custom_sections: PrimaryMap<SectionIndex, CustomSection>,
    pub custom_section_relocations: PrimaryMap<SectionIndex, Vec<Relocation>>,
    // The section indices corresponding to the Dwarf debug info
    pub debug: Option<Dwarf>,
    // Custom section containing libcall trampolines.
    pub libcall_trampolines: SectionIndex,
    // Length of each libcall trampoline.
    pub libcall_trampoline_len: u32,
}

impl SerializableCompilation {
    /// Serialize a Compilation into bytes
    /// The bytes will have the following format:
    /// RKYV serialization (any length) + POS (8 bytes)
    pub fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        let mut serializer = AllocSerializer::<4096>::default();
        let pos = serializer
            .serialize_value(self)
            .map_err(to_serialize_error)? as u64;
        let mut serialized_data = serializer.into_serializer().into_inner();
        serialized_data.extend_from_slice(&pos.to_le_bytes());
        Ok(serialized_data.to_vec())
    }
}

/// Serializable struct that is able to serialize from and to a `ArtifactInfo`.
#[derive(Archive, RkyvDeserialize, RkyvSerialize)]
#[allow(missing_docs)]
#[archive_attr(derive(CheckBytes))]
pub struct SerializableModule {
    /// The main serializable compilation object
    pub compilation: SerializableCompilation,
    /// Compilation informations
    pub compile_info: CompileModuleInfo,
    /// Datas initializers
    pub data_initializers: Box<[OwnedDataInitializer]>,
    /// CPU Feature flags for this compilation
    pub cpu_features: u64,
}

fn to_serialize_error(err: impl std::error::Error) -> SerializeError {
    SerializeError::Generic(format!("{}", err))
}

impl SerializableModule {
    /// Serialize a Module into bytes
    /// The bytes will have the following format:
    /// RKYV serialization (any length) + POS (8 bytes)
    pub fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        let mut serializer = AllocSerializer::<4096>::default();
        let pos = serializer
            .serialize_value(self)
            .map_err(to_serialize_error)? as u64;
        let mut serialized_data = serializer.into_serializer().into_inner();
        serialized_data.extend_from_slice(&pos.to_le_bytes());
        Ok(serialized_data.to_vec())
    }

    /// Deserialize a Module from a slice.
    /// The slice must have the following format:
    /// RKYV serialization (any length) + POS (8 bytes)
    ///
    /// # Safety
    ///
    /// This method is unsafe since it deserializes data directly
    /// from memory.
    /// Right now we are not doing any extra work for validation, but
    /// `rkyv` has an option to do bytecheck on the serialized data before
    /// serializing (via `rkyv::check_archived_value`).
    pub unsafe fn deserialize_unchecked(metadata_slice: &[u8]) -> Result<Self, DeserializeError> {
        let archived = Self::archive_from_slice(metadata_slice)?;
        Self::deserialize_from_archive(archived)
    }

    /// Deserialize a Module from a slice.
    /// The slice must have the following format:
    /// RKYV serialization (any length) + POS (8 bytes)
    ///
    /// Unlike [`Self::deserialize`], this function will validate the data.
    ///
    /// # Safety
    /// Unsafe because it loads executable code into memory.
    /// The loaded bytes must be trusted.
    pub unsafe fn deserialize(metadata_slice: &[u8]) -> Result<Self, DeserializeError> {
        let archived = Self::archive_from_slice_checked(metadata_slice)?;
        Self::deserialize_from_archive(archived)
    }

    /// # Safety
    ///
    /// This method is unsafe.
    /// Please check `SerializableModule::deserialize` for more details.
    pub unsafe fn archive_from_slice(
        metadata_slice: &[u8],
    ) -> Result<&ArchivedSerializableModule, DeserializeError> {
        if metadata_slice.len() < 8 {
            return Err(DeserializeError::Incompatible(
                "invalid serialized data".into(),
            ));
        }
        let mut pos: [u8; 8] = Default::default();
        pos.copy_from_slice(&metadata_slice[metadata_slice.len() - 8..metadata_slice.len()]);
        let pos: u64 = u64::from_le_bytes(pos);
        Ok(archived_value::<Self>(
            &metadata_slice[..metadata_slice.len() - 8],
            pos as usize,
        ))
    }

    /// Deserialize an archived module.
    ///
    /// In contrast to [`Self::deserialize`], this method performs validation
    /// and is not unsafe.
    pub fn archive_from_slice_checked(
        metadata_slice: &[u8],
    ) -> Result<&ArchivedSerializableModule, DeserializeError> {
        if metadata_slice.len() < 8 {
            return Err(DeserializeError::Incompatible(
                "invalid serialized data".into(),
            ));
        }
        let mut pos: [u8; 8] = Default::default();
        pos.copy_from_slice(&metadata_slice[metadata_slice.len() - 8..metadata_slice.len()]);
        let pos: u64 = u64::from_le_bytes(pos);
        check_archived_value::<Self>(&metadata_slice[..metadata_slice.len() - 8], pos as usize)
            .map_err(|err| DeserializeError::CorruptedBinary(err.to_string()))
    }

    /// Deserialize a compilation module from an archive
    pub fn deserialize_from_archive(
        archived: &ArchivedSerializableModule,
    ) -> Result<Self, DeserializeError> {
        let mut deserializer = SharedDeserializeMap::new();
        RkyvDeserialize::deserialize(archived, &mut deserializer)
            .map_err(|e| DeserializeError::CorruptedBinary(format!("{:?}", e)))
    }

    /// Create a `ModuleInfo` for instantiation
    pub fn create_module_info(&self) -> ModuleInfo {
        self.compile_info.module.as_ref().clone()
    }

    /// Returns the `ModuleInfo` for instantiation
    pub fn module_info(&self) -> &ModuleInfo {
        &self.compile_info.module
    }

    /// Returns the features for this Artifact
    pub fn features(&self) -> &Features {
        &self.compile_info.features
    }

    /// Returns the CPU features for this Artifact
    pub fn cpu_features(&self) -> EnumSet<CpuFeature> {
        EnumSet::from_u64(self.cpu_features)
    }

    /// Returns data initializers to pass to `VMInstance::initialize`
    pub fn data_initializers(&self) -> &[OwnedDataInitializer] {
        &self.data_initializers
    }

    /// Returns the memory styles associated with this `Artifact`.
    pub fn memory_styles(&self) -> &PrimaryMap<MemoryIndex, MemoryStyle> {
        &self.compile_info.memory_styles
    }

    /// Returns the table plans associated with this `Artifact`.
    pub fn table_styles(&self) -> &PrimaryMap<TableIndex, TableStyle> {
        &self.compile_info.table_styles
    }
}

/// Metadata header which holds an ABI version and the length of the remaining
/// metadata.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct MetadataHeader {
    magic: [u8; 8],
    version: u32,
    len: u32,
}

impl MetadataHeader {
    /// Current ABI version. Increment this any time breaking changes are made
    /// to the format of the serialized data.
    pub const CURRENT_VERSION: u32 = 5;

    /// Magic number to identify wasmer metadata.
    const MAGIC: [u8; 8] = *b"WASMER\0\0";

    /// Length of the metadata header.
    pub const LEN: usize = 16;

    /// Alignment of the metadata.
    pub const ALIGN: usize = 16;

    /// Creates a new header for metadata of the given length.
    pub fn new(len: usize) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::CURRENT_VERSION,
            len: len.try_into().expect("metadata exceeds maximum length"),
        }
    }

    /// Convert the header into its bytes representation.
    pub fn into_bytes(self) -> [u8; 16] {
        unsafe { mem::transmute(self) }
    }

    /// Parses the header and returns the length of the metadata following it.
    pub fn parse(bytes: &[u8]) -> Result<usize, DeserializeError> {
        if bytes.as_ptr() as usize % 8 != 0 {
            return Err(DeserializeError::CorruptedBinary(
                "misaligned metadata".to_string(),
            ));
        }
        let bytes: [u8; 16] = bytes
            .get(..16)
            .ok_or_else(|| {
                DeserializeError::CorruptedBinary("invalid metadata header".to_string())
            })?
            .try_into()
            .unwrap();
        let header: Self = unsafe { mem::transmute(bytes) };
        if header.magic != Self::MAGIC {
            return Err(DeserializeError::Incompatible(
                "The provided bytes were not serialized by Wasmer".to_string(),
            ));
        }
        if header.version != Self::CURRENT_VERSION {
            return Err(DeserializeError::Incompatible(
                "The provided bytes were serialized by an incompatible version of Wasmer"
                    .to_string(),
            ));
        }
        Ok(header.len as usize)
    }
}
