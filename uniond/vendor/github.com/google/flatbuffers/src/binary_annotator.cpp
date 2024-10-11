#include "binary_annotator.h"

#include <algorithm>
#include <cstdint>
#include <iostream>
#include <limits>
#include <string>
#include <vector>

#include "flatbuffers/base.h"
#include "flatbuffers/reflection.h"
#include "flatbuffers/util.h"
#include "flatbuffers/verifier.h"

namespace flatbuffers {
namespace {

static bool BinaryRegionSort(const BinaryRegion &a, const BinaryRegion &b) {
  return a.offset < b.offset;
}

static void SetError(BinaryRegionComment &comment, BinaryRegionStatus status,
                     std::string message = "") {
  comment.status = status;
  comment.status_message = message;
}

static BinaryRegion MakeBinaryRegion(
    const uint64_t offset = 0, const uint64_t length = 0,
    const BinaryRegionType type = BinaryRegionType::Unknown,
    const uint64_t array_length = 0, const uint64_t points_to_offset = 0,
    BinaryRegionComment comment = {}) {
  BinaryRegion region;
  region.offset = offset;
  region.length = length;
  region.type = type;
  region.array_length = array_length;
  region.points_to_offset = points_to_offset;
  region.comment = std::move(comment);
  return region;
}

static BinarySection MakeBinarySection(const std::string &name,
                                       const BinarySectionType type,
                                       std::vector<BinaryRegion> regions) {
  BinarySection section;
  section.name = name;
  section.type = type;
  section.regions = std::move(regions);
  return section;
}

static BinarySection MakeSingleRegionBinarySection(const std::string &name,
                                                   const BinarySectionType type,
                                                   const BinaryRegion &region) {
  std::vector<BinaryRegion> regions;
  regions.push_back(region);
  return MakeBinarySection(name, type, std::move(regions));
}

static bool IsNonZeroRegion(const uint64_t offset, const uint64_t length,
                            const uint8_t *const binary) {
  for (uint64_t i = offset; i < offset + length; ++i) {
    if (binary[i] != 0) { return true; }
  }
  return false;
}

static bool IsPrintableRegion(const uint64_t offset, const uint64_t length,
                              const uint8_t *const binary) {
  for (uint64_t i = offset; i < offset + length; ++i) {
    if (!isprint(binary[i])) { return false; }
  }
  return true;
}

static BinarySection GenerateMissingSection(const uint64_t offset,
                                            const uint64_t length,
                                            const uint8_t *const binary) {
  std::vector<BinaryRegion> regions;

  // Check if the region is all zeros or not, as that can tell us if it is
  // padding or not.
  if (IsNonZeroRegion(offset, length, binary)) {
    // Some of the padding bytes are non-zero, so this might be an unknown
    // section of the binary.
    // TODO(dbaileychess): We could be a bit smarter with different sized
    // alignments. For now, the 8 byte check encompasses all the smaller
    // alignments.
    BinaryRegionComment comment;
    comment.type = BinaryRegionCommentType::Unknown;
    if (length >= 8) {
      SetError(comment, BinaryRegionStatus::WARN_NO_REFERENCES);
    } else {
      SetError(comment, BinaryRegionStatus::WARN_CORRUPTED_PADDING);
    }

    regions.push_back(MakeBinaryRegion(offset, length * sizeof(uint8_t),
                                       BinaryRegionType::Unknown, length, 0,
                                       comment));

    return MakeBinarySection("no known references", BinarySectionType::Unknown,
                             std::move(regions));
  }

  BinaryRegionComment comment;
  comment.type = BinaryRegionCommentType::Padding;
  if (length >= 8) {
    SetError(comment, BinaryRegionStatus::WARN_PADDING_LENGTH);
  }

  // This region is most likely padding.
  regions.push_back(MakeBinaryRegion(offset, length * sizeof(uint8_t),
                                     BinaryRegionType::Uint8, length, 0,
                                     comment));

  return MakeBinarySection("", BinarySectionType::Padding, std::move(regions));
}

}  // namespace

std::map<uint64_t, BinarySection> BinaryAnnotator::Annotate() {
  flatbuffers::Verifier verifier(bfbs_, static_cast<size_t>(bfbs_length_));

  if ((is_size_prefixed_ &&
       !reflection::VerifySizePrefixedSchemaBuffer(verifier)) ||
      !reflection::VerifySchemaBuffer(verifier)) {
    return {};
  }

  // The binary is too short to read as a flatbuffers.
  if (binary_length_ < FLATBUFFERS_MIN_BUFFER_SIZE) { return {}; }

  // Make sure we start with a clean slate.
  vtables_.clear();
  sections_.clear();

  // First parse the header region which always start at offset 0.
  // The returned offset will point to the root_table location.
  const uint64_t root_table_offset = BuildHeader(0);

  if (IsValidOffset(root_table_offset)) {
    // Build the root table, and all else will be referenced from it.
    BuildTable(root_table_offset, BinarySectionType::RootTable,
               schema_->root_table());
  }

  // Now that all the sections are built, make sure the binary sections are
  // contiguous.
  FixMissingRegions();

  // Then scan the area between BinarySections insert padding sections that are
  // implied.
  FixMissingSections();

  return sections_;
}

uint64_t BinaryAnnotator::BuildHeader(const uint64_t header_offset) {
  uint64_t offset = header_offset;
  std::vector<BinaryRegion> regions;

  // If this binary is a size prefixed one, attempt to parse the size.
  if (is_size_prefixed_) {
    BinaryRegionComment prefix_length_comment;
    prefix_length_comment.type = BinaryRegionCommentType::SizePrefix;

    bool has_prefix_value = false;
    const auto prefix_length = ReadScalar<uoffset64_t>(offset);
    if (*prefix_length <= binary_length_) {
      regions.push_back(MakeBinaryRegion(offset, sizeof(uoffset64_t),
                                         BinaryRegionType::Uint64, 0, 0,
                                         prefix_length_comment));
      offset += sizeof(uoffset64_t);
      has_prefix_value = true;
    }

    if (!has_prefix_value) {
      const auto prefix_length = ReadScalar<uoffset_t>(offset);
      if (*prefix_length <= binary_length_) {
        regions.push_back(MakeBinaryRegion(offset, sizeof(uoffset_t),
                                           BinaryRegionType::Uint32, 0, 0,
                                           prefix_length_comment));
        offset += sizeof(uoffset_t);
        has_prefix_value = true;
      }
    }

    if (!has_prefix_value) {
      SetError(prefix_length_comment, BinaryRegionStatus::ERROR);
    }
  }

  const auto root_table_offset = ReadScalar<uint32_t>(offset);

  if (!root_table_offset.has_value()) {
    // This shouldn't occur, since we validate the min size of the buffer
    // before. But for completion sake, we shouldn't read passed the binary end.
    return std::numeric_limits<uint64_t>::max();
  }

  const auto root_table_loc = offset + *root_table_offset;

  BinaryRegionComment root_offset_comment;
  root_offset_comment.type = BinaryRegionCommentType::RootTableOffset;
  root_offset_comment.name = schema_->root_table()->name()->str();

  if (!IsValidOffset(root_table_loc)) {
    SetError(root_offset_comment,
             BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY);
  }

  regions.push_back(MakeBinaryRegion(offset, sizeof(uint32_t),
                                     BinaryRegionType::UOffset, 0,
                                     root_table_loc, root_offset_comment));
  offset += sizeof(uint32_t);

  if (IsValidRead(offset, flatbuffers::kFileIdentifierLength) &&
      IsPrintableRegion(offset, flatbuffers::kFileIdentifierLength, binary_)) {
    BinaryRegionComment comment;
    comment.type = BinaryRegionCommentType::FileIdentifier;
    // Check if the file identifier region has non-zero data, and assume its
    // the file identifier. Otherwise, it will get filled in with padding
    // later.
    regions.push_back(MakeBinaryRegion(
        offset, flatbuffers::kFileIdentifierLength * sizeof(uint8_t),
        BinaryRegionType::Char, flatbuffers::kFileIdentifierLength, 0,
        comment));
  }

  AddSection(header_offset, MakeBinarySection("", BinarySectionType::Header,
                                              std::move(regions)));

  return root_table_loc;
}

BinaryAnnotator::VTable *BinaryAnnotator::GetOrBuildVTable(
    const uint64_t vtable_offset, const reflection::Object *const table,
    const uint64_t offset_of_referring_table) {
  // Get a list of vtables (if any) already defined at this offset.
  std::list<VTable> &vtables = vtables_[vtable_offset];

  // See if this vtable for the table type has been generated before.
  for (VTable &vtable : vtables) {
    if (vtable.referring_table == table) { return &vtable; }
  }

  // If we are trying to make a new vtable and it is already encompassed by
  // another binary section, something is corrupted.
  if (vtables.empty() && ContainsSection(vtable_offset)) { return nullptr; }

  const std::string referring_table_name = table->name()->str();

  BinaryRegionComment vtable_size_comment;
  vtable_size_comment.type = BinaryRegionCommentType::VTableSize;

  const auto vtable_length = ReadScalar<uint16_t>(vtable_offset);
  if (!vtable_length.has_value()) {
    const uint64_t remaining = RemainingBytes(vtable_offset);

    SetError(vtable_size_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
             "2");

    AddSection(vtable_offset,
               MakeSingleRegionBinarySection(
                   referring_table_name, BinarySectionType::VTable,
                   MakeBinaryRegion(vtable_offset, remaining,
                                    BinaryRegionType::Unknown, remaining, 0,
                                    vtable_size_comment)));
    return nullptr;
  }

  // Vtables start with the size of the vtable
  const uint16_t vtable_size = vtable_length.value();

  if (!IsValidOffset(vtable_offset + vtable_size - 1)) {
    SetError(vtable_size_comment, BinaryRegionStatus::ERROR_LENGTH_TOO_LONG);
    // The vtable_size points to off the end of the binary.
    AddSection(vtable_offset,
               MakeSingleRegionBinarySection(
                   referring_table_name, BinarySectionType::VTable,
                   MakeBinaryRegion(vtable_offset, sizeof(uint16_t),
                                    BinaryRegionType::Uint16, 0, 0,
                                    vtable_size_comment)));

    return nullptr;
  } else if (vtable_size < 2 * sizeof(uint16_t)) {
    SetError(vtable_size_comment, BinaryRegionStatus::ERROR_LENGTH_TOO_SHORT,
             "4");
    // The size includes itself and the table size which are both uint16_t.
    AddSection(vtable_offset,
               MakeSingleRegionBinarySection(
                   referring_table_name, BinarySectionType::VTable,
                   MakeBinaryRegion(vtable_offset, sizeof(uint16_t),
                                    BinaryRegionType::Uint16, 0, 0,
                                    vtable_size_comment)));
    return nullptr;
  }

  std::vector<BinaryRegion> regions;

  regions.push_back(MakeBinaryRegion(vtable_offset, sizeof(uint16_t),
                                     BinaryRegionType::Uint16, 0, 0,
                                     vtable_size_comment));
  uint64_t offset = vtable_offset + sizeof(uint16_t);

  BinaryRegionComment ref_table_len_comment;
  ref_table_len_comment.type =
      BinaryRegionCommentType::VTableRefferingTableLength;

  // Ensure we can read the next uint16_t field, which is the size of the
  // referring table.
  const auto table_length = ReadScalar<uint16_t>(offset);

  if (!table_length.has_value()) {
    const uint64_t remaining = RemainingBytes(offset);
    SetError(ref_table_len_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
             "2");

    AddSection(offset, MakeSingleRegionBinarySection(
                           referring_table_name, BinarySectionType::VTable,
                           MakeBinaryRegion(
                               offset, remaining, BinaryRegionType::Unknown,
                               remaining, 0, ref_table_len_comment)));
    return nullptr;
  }

  // Then they have the size of the table they reference.
  const uint16_t table_size = table_length.value();

  if (!IsValidOffset(offset_of_referring_table + table_size - 1)) {
    SetError(ref_table_len_comment, BinaryRegionStatus::ERROR_LENGTH_TOO_LONG);
  } else if (table_size < 4) {
    SetError(ref_table_len_comment, BinaryRegionStatus::ERROR_LENGTH_TOO_SHORT,
             "4");
  }

  regions.push_back(MakeBinaryRegion(offset, sizeof(uint16_t),
                                     BinaryRegionType::Uint16, 0, 0,
                                     ref_table_len_comment));
  offset += sizeof(uint16_t);

  const uint64_t offset_start = offset;

  // A mapping between field (and its id) to the relative offset (uin16_t) from
  // the start of the table.
  std::map<uint16_t, VTable::Entry> fields;

  // Counter for determining if the binary has more vtable entries than the
  // schema provided. This can occur if the binary was created at a newer schema
  // version and is being processed with an older one.
  uint16_t fields_processed = 0;

  // Loop over all the fields.
  ForAllFields(table, /*reverse=*/false, [&](const reflection::Field *field) {
    const uint64_t field_offset = offset_start + field->id() * sizeof(uint16_t);

    if (field_offset >= vtable_offset + vtable_size) {
      // This field_offset is too large for this vtable, so it must come from a
      // newer schema than the binary was create with or the binary writer did
      // not write it. For either case, it is safe to ignore.

      // TODO(dbaileychess): We could show which fields are not set an their
      // default values if we want. We just need a way to make it obvious that
      // it isn't part of the buffer.
      return;
    }

    BinaryRegionComment field_comment;
    field_comment.type = BinaryRegionCommentType::VTableFieldOffset;
    field_comment.name = std::string(field->name()->c_str()) +
                         "` (id: " + std::to_string(field->id()) + ")";

    const auto offset_from_table = ReadScalar<uint16_t>(field_offset);

    if (!offset_from_table.has_value()) {
      const uint64_t remaining = RemainingBytes(field_offset);

      SetError(field_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "2");
      regions.push_back(MakeBinaryRegion(field_offset, remaining,
                                         BinaryRegionType::Unknown, remaining,
                                         0, field_comment));

      return;
    }

    if (!IsValidOffset(offset_of_referring_table + offset_from_table.value() -
                       1)) {
      SetError(field_comment, BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY);
      regions.push_back(MakeBinaryRegion(field_offset, sizeof(uint16_t),
                                         BinaryRegionType::VOffset, 0, 0,
                                         field_comment));
      return;
    }

    VTable::Entry entry;
    entry.field = field;
    entry.offset_from_table = offset_from_table.value();
    fields.insert(std::make_pair(field->id(), entry));

    std::string default_label;
    if (offset_from_table.value() == 0) {
      // Not present, so could be default or be optional.
      if (field->required()) {
        SetError(field_comment,
                 BinaryRegionStatus::ERROR_REQUIRED_FIELD_NOT_PRESENT);
        // If this is a required field, make it known this is an error.
        regions.push_back(MakeBinaryRegion(field_offset, sizeof(uint16_t),
                                           BinaryRegionType::VOffset, 0, 0,
                                           field_comment));
        return;
      } else {
        // Its an optional field, so get the default value and interpret and
        // provided an annotation for it.
        if (IsScalar(field->type()->base_type())) {
          default_label += "<defaults to ";
          default_label += IsFloat(field->type()->base_type())
                               ? std::to_string(field->default_real())
                               : std::to_string(field->default_integer());
          default_label += "> (";
        } else {
          default_label += "<null> (";
        }
        default_label +=
            reflection::EnumNameBaseType(field->type()->base_type());
        default_label += ")";
      }
    }
    field_comment.default_value = default_label;

    regions.push_back(MakeBinaryRegion(field_offset, sizeof(uint16_t),
                                       BinaryRegionType::VOffset, 0, 0,
                                       field_comment));

    fields_processed++;
  });

  // Check if we covered all the expectant fields. If not, we need to add them
  // as unknown fields.
  uint16_t expectant_vtable_fields =
      (vtable_size - sizeof(uint16_t) - sizeof(uint16_t)) / sizeof(uint16_t);

  // Prevent a bad binary from declaring a really large vtable_size, that we can
  // not independently verify.
  expectant_vtable_fields = std::min(
      static_cast<uint16_t>(fields_processed * 3), expectant_vtable_fields);

  for (uint16_t id = fields_processed; id < expectant_vtable_fields; ++id) {
    const uint64_t field_offset = offset_start + id * sizeof(uint16_t);

    const auto offset_from_table = ReadScalar<uint16_t>(field_offset);

    BinaryRegionComment field_comment;
    field_comment.type = BinaryRegionCommentType::VTableUnknownFieldOffset;
    field_comment.index = id;

    if (!offset_from_table.has_value()) {
      const uint64_t remaining = RemainingBytes(field_offset);
      SetError(field_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "2");
      regions.push_back(MakeBinaryRegion(field_offset, remaining,
                                         BinaryRegionType::Unknown, remaining,
                                         0, field_comment));
      continue;
    }

    VTable::Entry entry;
    entry.field = nullptr;  // No field to reference.
    entry.offset_from_table = offset_from_table.value();
    fields.insert(std::make_pair(id, entry));

    regions.push_back(MakeBinaryRegion(field_offset, sizeof(uint16_t),
                                       BinaryRegionType::VOffset, 0, 0,
                                       field_comment));
  }

  // If we have never added this vtable before record the Binary section.
  if (vtables.empty()) {
    sections_[vtable_offset] = MakeBinarySection(
        referring_table_name, BinarySectionType::VTable, std::move(regions));
  } else {
    // Add the current table name to the name of the section.
    sections_[vtable_offset].name += ", " + referring_table_name;
  }

  VTable vtable;
  vtable.referring_table = table;
  vtable.fields = std::move(fields);
  vtable.table_size = table_size;
  vtable.vtable_size = vtable_size;

  // Add this vtable to the collection of vtables at this offset.
  vtables.push_back(std::move(vtable));

  // Return the vtable we just added.
  return &vtables.back();
}

void BinaryAnnotator::BuildTable(const uint64_t table_offset,
                                 const BinarySectionType type,
                                 const reflection::Object *const table) {
  if (ContainsSection(table_offset)) { return; }

  BinaryRegionComment vtable_offset_comment;
  vtable_offset_comment.type = BinaryRegionCommentType::TableVTableOffset;

  const auto vtable_soffset = ReadScalar<int32_t>(table_offset);

  if (!vtable_soffset.has_value()) {
    const uint64_t remaining = RemainingBytes(table_offset);
    SetError(vtable_offset_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
             "4");

    AddSection(
        table_offset,
        MakeSingleRegionBinarySection(
            table->name()->str(), type,
            MakeBinaryRegion(table_offset, remaining, BinaryRegionType::Unknown,
                             remaining, 0, vtable_offset_comment)));

    // If there aren't enough bytes left to read the vtable offset, there is
    // nothing we can do.
    return;
  }

  // Tables start with the vtable
  const uint64_t vtable_offset = table_offset - vtable_soffset.value();

  if (!IsValidOffset(vtable_offset)) {
    SetError(vtable_offset_comment,
             BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY);

    AddSection(table_offset,
               MakeSingleRegionBinarySection(
                   table->name()->str(), type,
                   MakeBinaryRegion(table_offset, sizeof(int32_t),
                                    BinaryRegionType::SOffset, 0, vtable_offset,
                                    vtable_offset_comment)));

    // There isn't much to do with an invalid vtable offset, as we won't be able
    // to intepret the rest of the table fields.
    return;
  }

  std::vector<BinaryRegion> regions;
  regions.push_back(MakeBinaryRegion(table_offset, sizeof(int32_t),
                                     BinaryRegionType::SOffset, 0,
                                     vtable_offset, vtable_offset_comment));

  // Parse the vtable first so we know what the rest of the fields in the table
  // are.
  const VTable *const vtable =
      GetOrBuildVTable(vtable_offset, table, table_offset);

  if (vtable == nullptr) {
    // There is no valid vtable for this table, so we cannot process the rest of
    // the table entries.
    return;
  }

  // This is the size and length of this table.
  const uint16_t table_size = vtable->table_size;
  uint64_t table_end_offset = table_offset + table_size;

  if (!IsValidOffset(table_end_offset - 1)) {
    // We already validated the table size in BuildVTable, but we have to make
    // sure we don't use a bad value here.
    table_end_offset = binary_length_;
  }

  // We need to iterate over the vtable fields by their offset in the binary,
  // not by their IDs. So copy them over to another vector that we can sort on
  // the offset_from_table property.
  std::vector<VTable::Entry> fields;
  for (const auto &vtable_field : vtable->fields) {
    fields.push_back(vtable_field.second);
  }

  std::stable_sort(fields.begin(), fields.end(),
                   [](const VTable::Entry &a, const VTable::Entry &b) {
                     return a.offset_from_table < b.offset_from_table;
                   });

  // Iterate over all the fields by order of their offset.
  for (size_t i = 0; i < fields.size(); ++i) {
    const reflection::Field *field = fields[i].field;
    const uint16_t offset_from_table = fields[i].offset_from_table;

    if (offset_from_table == 0) {
      // Skip non-present fields.
      continue;
    }

    // The field offsets are relative to the start of the table.
    const uint64_t field_offset = table_offset + offset_from_table;

    if (!IsValidOffset(field_offset)) {
      // The field offset is larger than the binary, nothing we can do.
      continue;
    }

    // We have a vtable entry for a non-existant field, that means its a binary
    // generated by a newer schema than we are currently processing.
    if (field == nullptr) {
      // Calculate the length of this unknown field.
      const uint64_t unknown_field_length =
          // Check if there is another unknown field after this one.
          ((i + 1 < fields.size())
               ? table_offset + fields[i + 1].offset_from_table
               // Otherwise use the known end of the table.
               : table_end_offset) -
          field_offset;

      if (unknown_field_length == 0) { continue; }

      std::string hint;

      if (unknown_field_length == 4) {
        const auto relative_offset = ReadScalar<uint32_t>(field_offset);
        if (relative_offset.has_value()) {
          // The field is 4 in length, so it could be an offset? Provide a hint.
          hint += "<possibly an offset? Check Loc: +0x";
          hint += ToHex(field_offset + relative_offset.value());
          hint += ">";
        }
      }

      BinaryRegionComment unknown_field_comment;
      unknown_field_comment.type = BinaryRegionCommentType::TableUnknownField;

      if (!IsValidRead(field_offset, unknown_field_length)) {
        const uint64_t remaining = RemainingBytes(field_offset);

        SetError(unknown_field_comment,
                 BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
                 std::to_string(unknown_field_length));

        regions.push_back(MakeBinaryRegion(field_offset, remaining,
                                           BinaryRegionType::Unknown, remaining,
                                           0, unknown_field_comment));
        continue;
      }

      unknown_field_comment.default_value = hint;

      regions.push_back(MakeBinaryRegion(
          field_offset, unknown_field_length, BinaryRegionType::Unknown,
          unknown_field_length, 0, unknown_field_comment));
      continue;
    }

    if (IsScalar(field->type()->base_type())) {
      // These are the raw values store in the table.
      const uint64_t type_size = GetTypeSize(field->type()->base_type());
      const BinaryRegionType region_type =
          GetRegionType(field->type()->base_type());

      BinaryRegionComment scalar_field_comment;
      scalar_field_comment.type = BinaryRegionCommentType::TableField;
      scalar_field_comment.name =
          std::string(field->name()->c_str()) + "` (" +
          reflection::EnumNameBaseType(field->type()->base_type()) + ")";

      if (!IsValidRead(field_offset, type_size)) {
        const uint64_t remaining = RemainingBytes(field_offset);
        SetError(scalar_field_comment,
                 BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
                 std::to_string(type_size));

        regions.push_back(MakeBinaryRegion(field_offset, remaining,
                                           BinaryRegionType::Unknown, remaining,
                                           0, scalar_field_comment));
        continue;
      }

      if (IsUnionType(field)) {
        // This is a type for a union. Validate the value
        const auto enum_value = ReadScalar<uint8_t>(field_offset);

        // This should always have a value, due to the IsValidRead check above.
        if (!IsValidUnionValue(field, enum_value.value())) {
          SetError(scalar_field_comment,
                   BinaryRegionStatus::ERROR_INVALID_UNION_TYPE);

          regions.push_back(MakeBinaryRegion(field_offset, type_size,
                                             region_type, 0, 0,
                                             scalar_field_comment));
          continue;
        }
      }

      regions.push_back(MakeBinaryRegion(field_offset, type_size, region_type,
                                         0, 0, scalar_field_comment));
      continue;
    }

    // Read the offset
    uint64_t offset = 0;
    uint64_t length = sizeof(uint32_t);
    BinaryRegionType region_type = BinaryRegionType::UOffset;

    if (field->offset64()) {
      length = sizeof(uint64_t);
      region_type = BinaryRegionType::UOffset64;
      offset = ReadScalar<uint64_t>(field_offset).value_or(0);
    } else {
      offset = ReadScalar<uint32_t>(field_offset).value_or(0);
    }
    // const auto offset_from_field = ReadScalar<uint32_t>(field_offset);
    uint64_t offset_of_next_item = 0;
    BinaryRegionComment offset_field_comment;
    offset_field_comment.type = BinaryRegionCommentType::TableOffsetField;
    offset_field_comment.name = field->name()->c_str();
    const std::string offset_prefix =
        "offset to field `" + std::string(field->name()->c_str()) + "`";

    // Validate any field that isn't inline (i.e., non-structs).
    if (!IsInlineField(field)) {
      if (offset == 0) {
        const uint64_t remaining = RemainingBytes(field_offset);

        SetError(offset_field_comment,
                 BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "4");

        regions.push_back(MakeBinaryRegion(field_offset, remaining,
                                           BinaryRegionType::Unknown, remaining,
                                           0, offset_field_comment));
        continue;
      }

      offset_of_next_item = field_offset + offset;

      if (!IsValidOffset(offset_of_next_item)) {
        SetError(offset_field_comment,
                 BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY);
        regions.push_back(MakeBinaryRegion(field_offset, length, region_type, 0,
                                           offset_of_next_item,
                                           offset_field_comment));
        continue;
      }
    }

    switch (field->type()->base_type()) {
      case reflection::BaseType::Obj: {
        const reflection::Object *next_object =
            schema_->objects()->Get(field->type()->index());

        if (next_object->is_struct()) {
          // Structs are stored inline.
          BuildStruct(field_offset, regions, field->name()->c_str(),
                      next_object);
        } else {
          offset_field_comment.default_value = "(table)";

          regions.push_back(MakeBinaryRegion(field_offset, length, region_type,
                                             0, offset_of_next_item,
                                             offset_field_comment));

          BuildTable(offset_of_next_item, BinarySectionType::Table,
                     next_object);
        }
      } break;

      case reflection::BaseType::String: {
        offset_field_comment.default_value = "(string)";
        regions.push_back(MakeBinaryRegion(field_offset, length, region_type, 0,
                                           offset_of_next_item,
                                           offset_field_comment));
        BuildString(offset_of_next_item, table, field);
      } break;

      case reflection::BaseType::Vector: {
        offset_field_comment.default_value = "(vector)";
        regions.push_back(MakeBinaryRegion(field_offset, length, region_type, 0,
                                           offset_of_next_item,
                                           offset_field_comment));
        BuildVector(offset_of_next_item, table, field, table_offset,
                    vtable->fields);
      } break;
      case reflection::BaseType::Vector64: {
        offset_field_comment.default_value = "(vector64)";
        regions.push_back(MakeBinaryRegion(field_offset, length, region_type, 0,
                                           offset_of_next_item,
                                           offset_field_comment));
        BuildVector(offset_of_next_item, table, field, table_offset,
                    vtable->fields);
      } break;

      case reflection::BaseType::Union: {
        const uint64_t union_offset = offset_of_next_item;

        // The union type field is always one less than the union itself.
        const uint16_t union_type_id = field->id() - 1;

        auto vtable_field = vtable->fields.find(union_type_id);
        if (vtable_field == vtable->fields.end()) {
          // TODO(dbaileychess): need to capture this error condition.
          break;
        }
        offset_field_comment.default_value = "(union)";

        const uint64_t type_offset =
            table_offset + vtable_field->second.offset_from_table;

        const auto realized_type = ReadScalar<uint8_t>(type_offset);
        if (!realized_type.has_value()) {
          const uint64_t remaining = RemainingBytes(type_offset);
          SetError(offset_field_comment,
                   BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "1");
          regions.push_back(MakeBinaryRegion(
              type_offset, remaining, BinaryRegionType::Unknown, remaining, 0,
              offset_field_comment));
          continue;
        }

        if (!IsValidUnionValue(field, realized_type.value())) {
          // We already export an error in the union type field, so just skip
          // building the union itself and it will default to an unreference
          // Binary section.
          continue;
        }

        const std::string enum_type =
            BuildUnion(union_offset, realized_type.value(), field);

        offset_field_comment.default_value =
            "(union of type `" + enum_type + "`)";

        regions.push_back(MakeBinaryRegion(field_offset, length, region_type, 0,
                                           union_offset, offset_field_comment));

      } break;

      default: break;
    }
  }

  // Handle the case where there is padding after the last known binary
  // region. Calculate where we left off towards the expected end of the
  // table.
  const uint64_t i = regions.back().offset + regions.back().length + 1;

  if (i < table_end_offset) {
    const uint64_t pad_bytes = table_end_offset - i + 1;

    BinaryRegionComment padding_comment;
    padding_comment.type = BinaryRegionCommentType::Padding;

    regions.push_back(MakeBinaryRegion(i - 1, pad_bytes * sizeof(uint8_t),
                                       BinaryRegionType::Uint8, pad_bytes, 0,
                                       padding_comment));
  }

  AddSection(table_offset,
             MakeBinarySection(table->name()->str(), type, std::move(regions)));
}

uint64_t BinaryAnnotator::BuildStruct(const uint64_t struct_offset,
                                      std::vector<BinaryRegion> &regions,
                                      const std::string referring_field_name,
                                      const reflection::Object *const object) {
  if (!object->is_struct()) { return struct_offset; }
  uint64_t offset = struct_offset;

  // Loop over all the fields in increasing order
  ForAllFields(object, /*reverse=*/false, [&](const reflection::Field *field) {
    if (IsScalar(field->type()->base_type())) {
      // Structure Field value
      const uint64_t type_size = GetTypeSize(field->type()->base_type());
      const BinaryRegionType region_type =
          GetRegionType(field->type()->base_type());

      BinaryRegionComment comment;
      comment.type = BinaryRegionCommentType::StructField;
      comment.name = referring_field_name + "." + field->name()->str();
      comment.default_value = "of '" + object->name()->str() + "' (" +
                              std::string(reflection::EnumNameBaseType(
                                  field->type()->base_type())) +
                              ")";

      if (!IsValidRead(offset, type_size)) {
        const uint64_t remaining = RemainingBytes(offset);
        SetError(comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
                 std::to_string(type_size));
        regions.push_back(MakeBinaryRegion(offset, remaining,
                                           BinaryRegionType::Unknown, remaining,
                                           0, comment));

        // TODO(dbaileychess): Should I bail out here? This sets offset to the
        // end of the binary. So all other reads in the loop should fail.
        offset += remaining;
        return;
      }

      regions.push_back(
          MakeBinaryRegion(offset, type_size, region_type, 0, 0, comment));
      offset += type_size;
    } else if (field->type()->base_type() == reflection::BaseType::Obj) {
      // Structs are stored inline, even when nested.
      offset = BuildStruct(offset, regions,
                           referring_field_name + "." + field->name()->str(),
                           schema_->objects()->Get(field->type()->index()));
    } else if (field->type()->base_type() == reflection::BaseType::Array) {
      const bool is_scalar = IsScalar(field->type()->element());
      const uint64_t type_size = GetTypeSize(field->type()->element());
      const BinaryRegionType region_type =
          GetRegionType(field->type()->element());

      // Arrays are just repeated structures.
      for (uint16_t i = 0; i < field->type()->fixed_length(); ++i) {
        if (is_scalar) {
          BinaryRegionComment array_comment;
          array_comment.type = BinaryRegionCommentType::ArrayField;
          array_comment.name =
              referring_field_name + "." + field->name()->str();
          array_comment.index = i;
          array_comment.default_value =
              "of '" + object->name()->str() + "' (" +
              std::string(
                  reflection::EnumNameBaseType(field->type()->element())) +
              ")";

          if (!IsValidRead(offset, type_size)) {
            const uint64_t remaining = RemainingBytes(offset);

            SetError(array_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
                     std::to_string(type_size));

            regions.push_back(MakeBinaryRegion(offset, remaining,
                                               BinaryRegionType::Unknown,
                                               remaining, 0, array_comment));

            // TODO(dbaileychess): Should I bail out here? This sets offset to
            // the end of the binary. So all other reads in the loop should
            // fail.
            offset += remaining;
            break;
          }

          regions.push_back(MakeBinaryRegion(offset, type_size, region_type, 0,
                                             0, array_comment));

          offset += type_size;
        } else {
          // Array of Structs.
          //
          // TODO(dbaileychess): This works, but the comments on the fields lose
          // some context. Need to figure a way how to plumb the nested arrays
          // comments together that isn't too confusing.
          offset =
              BuildStruct(offset, regions,
                          referring_field_name + "." + field->name()->str(),
                          schema_->objects()->Get(field->type()->index()));
        }
      }
    }

    // Insert any padding after this field.
    const uint16_t padding = field->padding();
    if (padding > 0 && IsValidOffset(offset + padding)) {
      BinaryRegionComment padding_comment;
      padding_comment.type = BinaryRegionCommentType::Padding;

      regions.push_back(MakeBinaryRegion(offset, padding,
                                         BinaryRegionType::Uint8, padding, 0,
                                         padding_comment));
      offset += padding;
    }
  });

  return offset;
}

void BinaryAnnotator::BuildString(const uint64_t string_offset,
                                  const reflection::Object *const table,
                                  const reflection::Field *const field) {
  // Check if we have already generated this string section, and this is a
  // shared string instance.
  if (ContainsSection(string_offset)) { return; }

  std::vector<BinaryRegion> regions;
  const auto string_length = ReadScalar<uint32_t>(string_offset);

  BinaryRegionComment string_length_comment;
  string_length_comment.type = BinaryRegionCommentType::StringLength;

  if (!string_length.has_value()) {
    const uint64_t remaining = RemainingBytes(string_offset);

    SetError(string_length_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
             "4");

    regions.push_back(MakeBinaryRegion(string_offset, remaining,
                                       BinaryRegionType::Unknown, remaining, 0,
                                       string_length_comment));

  } else {
    const uint32_t string_size = string_length.value();
    const uint64_t string_end =
        string_offset + sizeof(uint32_t) + string_size + sizeof(char);

    if (!IsValidOffset(string_end - 1)) {
      SetError(string_length_comment,
               BinaryRegionStatus::ERROR_LENGTH_TOO_LONG);

      regions.push_back(MakeBinaryRegion(string_offset, sizeof(uint32_t),
                                         BinaryRegionType::Uint32, 0, 0,
                                         string_length_comment));
    } else {
      regions.push_back(MakeBinaryRegion(string_offset, sizeof(uint32_t),
                                         BinaryRegionType::Uint32, 0, 0,
                                         string_length_comment));

      BinaryRegionComment string_comment;
      string_comment.type = BinaryRegionCommentType::StringValue;

      regions.push_back(MakeBinaryRegion(string_offset + sizeof(uint32_t),
                                         string_size, BinaryRegionType::Char,
                                         string_size, 0, string_comment));

      BinaryRegionComment string_terminator_comment;
      string_terminator_comment.type =
          BinaryRegionCommentType::StringTerminator;

      regions.push_back(MakeBinaryRegion(
          string_offset + sizeof(uint32_t) + string_size, sizeof(char),
          BinaryRegionType::Char, 0, 0, string_terminator_comment));
    }
  }

  AddSection(string_offset,
             MakeBinarySection(std::string(table->name()->c_str()) + "." +
                                   field->name()->c_str(),
                               BinarySectionType::String, std::move(regions)));
}

void BinaryAnnotator::BuildVector(
    const uint64_t vector_offset, const reflection::Object *const table,
    const reflection::Field *const field, const uint64_t parent_table_offset,
    const std::map<uint16_t, VTable::Entry> vtable_fields) {
  if (ContainsSection(vector_offset)) { return; }

  BinaryRegionComment vector_length_comment;
  vector_length_comment.type = BinaryRegionCommentType::VectorLength;

  const bool is_64_bit_vector =
      field->type()->base_type() == reflection::BaseType::Vector64;

  flatbuffers::Optional<uint64_t> vector_length;
  uint32_t vector_length_size_type = 0;
  BinaryRegionType region_type = BinaryRegionType::Uint32;
  BinarySectionType section_type = BinarySectionType::Vector;

  if (is_64_bit_vector) {
    auto v = ReadScalar<uint64_t>(vector_offset);
    if (v.has_value()) { vector_length = v.value(); }
    vector_length_size_type = sizeof(uint64_t);
    region_type = BinaryRegionType::Uint64;
    section_type = BinarySectionType::Vector64;
  } else {
    auto v = ReadScalar<uint32_t>(vector_offset);
    if (v.has_value()) { vector_length = v.value(); }
    vector_length_size_type = sizeof(uint32_t);
    region_type = BinaryRegionType::Uint32;
    section_type = BinarySectionType::Vector;
  }

  if (!vector_length.has_value()) {
    const uint64_t remaining = RemainingBytes(vector_offset);
    SetError(vector_length_comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
             "4");

    AddSection(
        vector_offset,
        MakeSingleRegionBinarySection(
            std::string(table->name()->c_str()) + "." + field->name()->c_str(),
            BinarySectionType::Vector,
            MakeBinaryRegion(vector_offset, remaining,
                             BinaryRegionType::Unknown, remaining, 0,
                             vector_length_comment)));
    return;
  }

  // Validate there are enough bytes left in the binary to process all the
  // items.
  const uint64_t last_item_offset =
      vector_offset + vector_length_size_type +
      vector_length.value() * GetElementSize(field);

  if (!IsValidOffset(last_item_offset - 1)) {
    SetError(vector_length_comment, BinaryRegionStatus::ERROR_LENGTH_TOO_LONG);
    AddSection(
        vector_offset,
        MakeSingleRegionBinarySection(
            std::string(table->name()->c_str()) + "." + field->name()->c_str(),
            BinarySectionType::Vector,
            MakeBinaryRegion(vector_offset, vector_length_size_type,
                             region_type, 0, 0, vector_length_comment)));

    return;
  }

  std::vector<BinaryRegion> regions;

  regions.push_back(MakeBinaryRegion(vector_offset, vector_length_size_type,
                                     region_type, 0, 0, vector_length_comment));
  // Consume the vector length offset.
  uint64_t offset = vector_offset + vector_length_size_type;

  switch (field->type()->element()) {
    case reflection::BaseType::Obj: {
      const reflection::Object *object =
          schema_->objects()->Get(field->type()->index());

      if (object->is_struct()) {
        // Vector of structs
        for (size_t i = 0; i < vector_length.value(); ++i) {
          // Structs are inline to the vector.
          const uint64_t next_offset =
              BuildStruct(offset, regions, "[" + NumToString(i) + "]", object);
          if (next_offset == offset) { break; }
          offset = next_offset;
        }
      } else {
        // Vector of objects
        for (size_t i = 0; i < vector_length.value(); ++i) {
          BinaryRegionComment vector_object_comment;
          vector_object_comment.type =
              BinaryRegionCommentType::VectorTableValue;
          vector_object_comment.index = i;

          const auto table_relative_offset = ReadScalar<uint32_t>(offset);
          if (!table_relative_offset.has_value()) {
            const uint64_t remaining = RemainingBytes(offset);
            SetError(vector_object_comment,
                     BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "4");

            regions.push_back(
                MakeBinaryRegion(offset, remaining, BinaryRegionType::Unknown,
                                 remaining, 0, vector_object_comment));
            break;
          }

          // The table offset is relative from the offset location itself.
          const uint64_t table_offset = offset + table_relative_offset.value();

          if (!IsValidOffset(table_offset)) {
            SetError(vector_object_comment,
                     BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY);
            regions.push_back(MakeBinaryRegion(
                offset, sizeof(uint32_t), BinaryRegionType::UOffset, 0,
                table_offset, vector_object_comment));

            offset += sizeof(uint32_t);
            continue;
          }

          if (table_offset == parent_table_offset) {
            SetError(vector_object_comment,
                     BinaryRegionStatus::ERROR_CYCLE_DETECTED);
            // A cycle detected where a table vector field is pointing to
            // itself. This should only happen in corrupted files.
            regions.push_back(MakeBinaryRegion(
                offset, sizeof(uint32_t), BinaryRegionType::UOffset, 0,
                table_offset, vector_object_comment));

            offset += sizeof(uint32_t);
            continue;
          }

          regions.push_back(MakeBinaryRegion(
              offset, sizeof(uint32_t), BinaryRegionType::UOffset, 0,
              table_offset, vector_object_comment));

          // Consume the offset to the table.
          offset += sizeof(uint32_t);

          BuildTable(table_offset, BinarySectionType::Table, object);
        }
      }
    } break;
    case reflection::BaseType::String: {
      // Vector of strings
      for (size_t i = 0; i < vector_length.value(); ++i) {
        BinaryRegionComment vector_object_comment;
        vector_object_comment.type = BinaryRegionCommentType::VectorStringValue;
        vector_object_comment.index = i;

        const auto string_relative_offset = ReadScalar<uint32_t>(offset);
        if (!string_relative_offset.has_value()) {
          const uint64_t remaining = RemainingBytes(offset);

          SetError(vector_object_comment,
                   BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "4");

          regions.push_back(
              MakeBinaryRegion(offset, remaining, BinaryRegionType::Unknown,
                               remaining, 0, vector_object_comment));
          break;
        }

        // The string offset is relative from the offset location itself.
        const uint64_t string_offset = offset + string_relative_offset.value();

        if (!IsValidOffset(string_offset)) {
          SetError(vector_object_comment,
                   BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY);
          regions.push_back(MakeBinaryRegion(
              offset, sizeof(uint32_t), BinaryRegionType::UOffset, 0,
              string_offset, vector_object_comment));

          offset += sizeof(uint32_t);
          continue;
        }

        regions.push_back(MakeBinaryRegion(
            offset, sizeof(uint32_t), BinaryRegionType::UOffset, 0,
            string_offset, vector_object_comment));

        BuildString(string_offset, table, field);

        offset += sizeof(uint32_t);
      }
    } break;
    case reflection::BaseType::Union: {
      // Vector of unions
      // Unions have both their realized type (uint8_t for now) that are
      // stored separately. These are stored in the field->index() - 1
      // location.
      const uint16_t union_type_vector_id = field->id() - 1;

      auto vtable_entry = vtable_fields.find(union_type_vector_id);
      if (vtable_entry == vtable_fields.end()) {
        // TODO(dbaileychess): need to capture this error condition.
        break;
      }

      const uint64_t union_type_vector_field_offset =
          parent_table_offset + vtable_entry->second.offset_from_table;

      const auto union_type_vector_field_relative_offset =
          ReadScalar<uint16_t>(union_type_vector_field_offset);

      if (!union_type_vector_field_relative_offset.has_value()) {
        const uint64_t remaining = RemainingBytes(offset);
        BinaryRegionComment vector_union_comment;
        vector_union_comment.type = BinaryRegionCommentType::VectorUnionValue;
        SetError(vector_union_comment,
                 BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "2");

        regions.push_back(MakeBinaryRegion(offset, remaining,
                                           BinaryRegionType::Unknown, remaining,
                                           0, vector_union_comment));

        break;
      }

      // Get the offset to the first type (the + sizeof(uint32_t) is to skip
      // over the vector length which we already know). Validation happens
      // within the loop below.
      const uint64_t union_type_vector_data_offset =
          union_type_vector_field_offset +
          union_type_vector_field_relative_offset.value() + sizeof(uint32_t);

      for (size_t i = 0; i < vector_length.value(); ++i) {
        BinaryRegionComment comment;
        comment.type = BinaryRegionCommentType::VectorUnionValue;
        comment.index = i;

        const auto union_relative_offset = ReadScalar<uint32_t>(offset);
        if (!union_relative_offset.has_value()) {
          const uint64_t remaining = RemainingBytes(offset);

          SetError(comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "4");

          regions.push_back(MakeBinaryRegion(offset, remaining,
                                             BinaryRegionType::Unknown,
                                             remaining, 0, comment));

          break;
        }

        // The union offset is relative from the offset location itself.
        const uint64_t union_offset = offset + union_relative_offset.value();

        if (!IsValidOffset(union_offset)) {
          SetError(comment, BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY);

          regions.push_back(MakeBinaryRegion(offset, sizeof(uint32_t),
                                             BinaryRegionType::UOffset, 0,
                                             union_offset, comment));
          continue;
        }

        const auto realized_type =
            ReadScalar<uint8_t>(union_type_vector_data_offset + i);

        if (!realized_type.has_value()) {
          SetError(comment, BinaryRegionStatus::ERROR_INCOMPLETE_BINARY, "1");
          regions.push_back(MakeBinaryRegion(
              offset, 0, BinaryRegionType::Unknown, 0, 0, comment));
          continue;
        }

        if (!IsValidUnionValue(vtable_entry->second.field->type()->index(),
                               realized_type.value())) {
          // We already export an error in the union type field, so just skip
          // building the union itself and it will default to an unreference
          // Binary section.
          offset += sizeof(uint32_t);
          continue;
        }

        const std::string enum_type =
            BuildUnion(union_offset, realized_type.value(), field);

        comment.default_value = "(`" + enum_type + "`)";
        regions.push_back(MakeBinaryRegion(offset, sizeof(uint32_t),
                                           BinaryRegionType::UOffset, 0,
                                           union_offset, comment));

        offset += sizeof(uint32_t);
      }
    } break;
    default: {
      if (IsScalar(field->type()->element())) {
        const BinaryRegionType binary_region_type =
            GetRegionType(field->type()->element());

        const uint64_t type_size = GetTypeSize(field->type()->element());

        // TODO(dbaileychess): It might be nicer to user the
        // BinaryRegion.array_length field to indicate this.
        for (size_t i = 0; i < vector_length.value(); ++i) {
          BinaryRegionComment vector_scalar_comment;
          vector_scalar_comment.type = BinaryRegionCommentType::VectorValue;
          vector_scalar_comment.index = i;

          if (!IsValidRead(offset, type_size)) {
            const uint64_t remaining = RemainingBytes(offset);

            SetError(vector_scalar_comment,
                     BinaryRegionStatus::ERROR_INCOMPLETE_BINARY,
                     std::to_string(type_size));

            regions.push_back(
                MakeBinaryRegion(offset, remaining, BinaryRegionType::Unknown,
                                 remaining, 0, vector_scalar_comment));
            break;
          }

          if (IsUnionType(field->type()->element())) {
            // This is a type for a union. Validate the value
            const auto enum_value = ReadScalar<uint8_t>(offset);

            // This should always have a value, due to the IsValidRead check
            // above.
            if (!IsValidUnionValue(field->type()->index(),
                                   enum_value.value())) {
              SetError(vector_scalar_comment,
                       BinaryRegionStatus::ERROR_INVALID_UNION_TYPE);
              regions.push_back(MakeBinaryRegion(offset, type_size,
                                                 binary_region_type, 0, 0,
                                                 vector_scalar_comment));
              offset += type_size;
              continue;
            }
          }

          regions.push_back(MakeBinaryRegion(offset, type_size,
                                             binary_region_type, 0, 0,
                                             vector_scalar_comment));
          offset += type_size;
        }
      }
    } break;
  }
  AddSection(vector_offset,
             MakeBinarySection(std::string(table->name()->c_str()) + "." +
                                   field->name()->c_str(),
                               section_type, std::move(regions)));
}

std::string BinaryAnnotator::BuildUnion(const uint64_t union_offset,
                                        const uint8_t realized_type,
                                        const reflection::Field *const field) {
  const reflection::Enum *next_enum =
      schema_->enums()->Get(field->type()->index());

  const reflection::EnumVal *enum_val = next_enum->values()->Get(realized_type);

  if (ContainsSection(union_offset)) { return enum_val->name()->c_str(); }

  const reflection::Type *union_type = enum_val->union_type();

  if (union_type->base_type() == reflection::BaseType::Obj) {
    const reflection::Object *object =
        schema_->objects()->Get(union_type->index());

    if (object->is_struct()) {
      // Union of vectors point to a new Binary section
      std::vector<BinaryRegion> regions;

      BuildStruct(union_offset, regions, field->name()->c_str(), object);

      AddSection(
          union_offset,
          MakeBinarySection(std::string(object->name()->c_str()) + "." +
                                field->name()->c_str(),
                            BinarySectionType::Union, std::move(regions)));
    } else {
      BuildTable(union_offset, BinarySectionType::Table, object);
    }
  }
  // TODO(dbaileychess): handle the other union types.

  return enum_val->name()->c_str();
}

void BinaryAnnotator::FixMissingRegions() {
  std::vector<BinaryRegion> regions_to_insert;
  for (auto &current_section : sections_) {
    BinarySection &section = current_section.second;
    if (section.regions.empty()) {
      // TODO(dbaileychess): is this possible?
      continue;
    }

    uint64_t offset = section.regions[0].offset + section.regions[0].length;
    for (size_t i = 1; i < section.regions.size(); ++i) {
      BinaryRegion &region = section.regions[i];

      const uint64_t next_offset = region.offset;
      if (!IsValidOffset(next_offset)) {
        // TODO(dbaileychess): figure out how we get into this situation.
        continue;
      }

      if (offset < next_offset) {
        const uint64_t padding_bytes = next_offset - offset;

        BinaryRegionComment comment;
        comment.type = BinaryRegionCommentType::Padding;

        if (IsNonZeroRegion(offset, padding_bytes, binary_)) {
          SetError(comment, BinaryRegionStatus::WARN_NO_REFERENCES);
          regions_to_insert.push_back(
              MakeBinaryRegion(offset, padding_bytes, BinaryRegionType::Unknown,
                               padding_bytes, 0, comment));
        } else {
          regions_to_insert.push_back(
              MakeBinaryRegion(offset, padding_bytes, BinaryRegionType::Uint8,
                               padding_bytes, 0, comment));
        }
      }
      offset = next_offset + region.length;
    }

    if (!regions_to_insert.empty()) {
      section.regions.insert(section.regions.end(), regions_to_insert.begin(),
                             regions_to_insert.end());
      std::stable_sort(section.regions.begin(), section.regions.end(),
                       BinaryRegionSort);
      regions_to_insert.clear();
    }
  }
}

void BinaryAnnotator::FixMissingSections() {
  uint64_t offset = 0;

  std::vector<BinarySection> sections_to_insert;

  for (auto &current_section : sections_) {
    BinarySection &section = current_section.second;
    const uint64_t section_start_offset = current_section.first;
    const uint64_t section_end_offset =
        section.regions.back().offset + section.regions.back().length;

    if (offset < section_start_offset) {
      // We are at an offset that is less then the current section.
      const uint64_t pad_bytes = section_start_offset - offset + 1;

      sections_to_insert.push_back(
          GenerateMissingSection(offset - 1, pad_bytes, binary_));
    }
    offset = section_end_offset + 1;
  }

  // Handle the case where there are still bytes left in the binary that are
  // unaccounted for.
  if (offset < binary_length_) {
    const uint64_t pad_bytes = binary_length_ - offset + 1;
    sections_to_insert.push_back(
        GenerateMissingSection(offset - 1, pad_bytes, binary_));
  }

  for (const BinarySection &section_to_insert : sections_to_insert) {
    AddSection(section_to_insert.regions[0].offset, section_to_insert);
  }
}

bool BinaryAnnotator::ContainsSection(const uint64_t offset) {
  auto it = sections_.lower_bound(offset);
  // If the section is found, check that it is exactly equal its offset.
  if (it != sections_.end() && it->first == offset) { return true; }

  // If this was the first section, there are no other previous sections to
  // check.
  if (it == sections_.begin()) { return false; }

  // Go back one section.
  --it;

  // And check that if the offset is covered by the section.
  return offset >= it->first && offset < it->second.regions.back().offset +
                                             it->second.regions.back().length;
}

}  // namespace flatbuffers
