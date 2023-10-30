pub type DtdEntity = *mut ::core::ffi::c_void;
pub type DtdNotation = *mut ::core::ffi::c_void;
pub type IXmlCharacterData = *mut ::core::ffi::c_void;
pub type IXmlNode = *mut ::core::ffi::c_void;
pub type IXmlNodeSelector = *mut ::core::ffi::c_void;
pub type IXmlNodeSerializer = *mut ::core::ffi::c_void;
pub type IXmlText = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Data_Xml_Dom\"`*"]
#[repr(transparent)]
pub struct NodeType(pub i32);
impl NodeType {
    pub const Invalid: Self = Self(0i32);
    pub const ElementNode: Self = Self(1i32);
    pub const AttributeNode: Self = Self(2i32);
    pub const TextNode: Self = Self(3i32);
    pub const DataSectionNode: Self = Self(4i32);
    pub const EntityReferenceNode: Self = Self(5i32);
    pub const EntityNode: Self = Self(6i32);
    pub const ProcessingInstructionNode: Self = Self(7i32);
    pub const CommentNode: Self = Self(8i32);
    pub const DocumentNode: Self = Self(9i32);
    pub const DocumentTypeNode: Self = Self(10i32);
    pub const DocumentFragmentNode: Self = Self(11i32);
    pub const NotationNode: Self = Self(12i32);
}
impl ::core::marker::Copy for NodeType {}
impl ::core::clone::Clone for NodeType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type XmlAttribute = *mut ::core::ffi::c_void;
pub type XmlCDataSection = *mut ::core::ffi::c_void;
pub type XmlComment = *mut ::core::ffi::c_void;
pub type XmlDocument = *mut ::core::ffi::c_void;
pub type XmlDocumentFragment = *mut ::core::ffi::c_void;
pub type XmlDocumentType = *mut ::core::ffi::c_void;
pub type XmlDomImplementation = *mut ::core::ffi::c_void;
pub type XmlElement = *mut ::core::ffi::c_void;
pub type XmlEntityReference = *mut ::core::ffi::c_void;
pub type XmlLoadSettings = *mut ::core::ffi::c_void;
pub type XmlNamedNodeMap = *mut ::core::ffi::c_void;
pub type XmlNodeList = *mut ::core::ffi::c_void;
pub type XmlProcessingInstruction = *mut ::core::ffi::c_void;
pub type XmlText = *mut ::core::ffi::c_void;
