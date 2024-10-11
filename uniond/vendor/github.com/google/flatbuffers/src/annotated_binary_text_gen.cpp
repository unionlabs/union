#include "annotated_binary_text_gen.h"

#include <algorithm>
#include <cstdint>
#include <fstream>
#include <ostream>
#include <sstream>
#include <string>

#include "binary_annotator.h"
#include "flatbuffers/base.h"
#include "flatbuffers/util.h"

namespace flatbuffers {
namespace {

struct OutputConfig {
  size_t largest_type_string = 10;

  size_t largest_value_string = 20;

  size_t max_bytes_per_line = 8;

  size_t offset_max_char = 4;

  char delimiter = '|';

  bool include_vector_contents = true;
};

static std::string ToString(const BinarySectionType type) {
  switch (type) {
    case BinarySectionType::Header: return "header";
    case BinarySectionType::Table: return "table";
    case BinarySectionType::RootTable: return "root_table";
    case BinarySectionType::VTable: return "vtable";
    case BinarySectionType::Struct: return "struct";
    case BinarySectionType::String: return "string";
    case BinarySectionType::Vector: return "vector";
    case BinarySectionType::Vector64: return "vector64";
    case BinarySectionType::Unknown: return "unknown";
    case BinarySectionType::Union: return "union";
    case BinarySectionType::Padding: return "padding";
    default: return "todo";
  }
}

static bool IsOffset(const BinaryRegionType type) {
  return type == BinaryRegionType::UOffset ||
         type == BinaryRegionType::SOffset ||
         type == BinaryRegionType::UOffset64;
}

template<typename T> std::string ToString(T value) {
  if (std::is_floating_point<T>::value) {
    std::stringstream ss;
    ss << value;
    return ss.str();
  } else {
    return std::to_string(value);
  }
}

template<typename T>
std::string ToValueString(const BinaryRegion &region, const uint8_t *binary) {
  std::string s;
  s += "0x";
  const T val = ReadScalar<T>(binary + region.offset);
  const uint64_t start_index = region.offset + region.length - 1;
  for (uint64_t i = 0; i < region.length; ++i) {
    s += ToHex(binary[start_index - i]);
  }
  s += " (";
  s += ToString(val);
  s += ")";
  return s;
}

template<>
std::string ToValueString<std::string>(const BinaryRegion &region,
                                       const uint8_t *binary) {
  return std::string(reinterpret_cast<const char *>(binary + region.offset),
                     static_cast<size_t>(region.array_length));
}

static std::string ToValueString(const BinaryRegion &region,
                                 const uint8_t *binary,
                                 const OutputConfig &output_config) {
  std::string s;

  if (region.array_length) {
    if (region.type == BinaryRegionType::Uint8 ||
        region.type == BinaryRegionType::Unknown) {
      // Interpret each value as a ASCII to aid debugging
      for (uint64_t i = 0; i < region.array_length; ++i) {
        const uint8_t c = *(binary + region.offset + i);
        s += isprint(c) ? static_cast<char>(c & 0x7F) : '.';
      }
      return s;
    } else if (region.type == BinaryRegionType::Char) {
      // string value
      return ToValueString<std::string>(region, binary);
    }
  }

  switch (region.type) {
    case BinaryRegionType::Uint32:
      return ToValueString<uint32_t>(region, binary);
    case BinaryRegionType::Int32: return ToValueString<int32_t>(region, binary);
    case BinaryRegionType::Uint16:
      return ToValueString<uint16_t>(region, binary);
    case BinaryRegionType::Int16: return ToValueString<int16_t>(region, binary);
    case BinaryRegionType::Bool: return ToValueString<bool>(region, binary);
    case BinaryRegionType::Uint8: return ToValueString<uint8_t>(region, binary);
    case BinaryRegionType::Char: return ToValueString<char>(region, binary);
    case BinaryRegionType::Byte:
    case BinaryRegionType::Int8: return ToValueString<int8_t>(region, binary);
    case BinaryRegionType::Int64: return ToValueString<int64_t>(region, binary);
    case BinaryRegionType::Uint64:
      return ToValueString<uint64_t>(region, binary);
    case BinaryRegionType::Double: return ToValueString<double>(region, binary);
    case BinaryRegionType::Float: return ToValueString<float>(region, binary);
    case BinaryRegionType::UType: return ToValueString<uint8_t>(region, binary);

    // Handle Offsets separately, incase they add additional details.
    case BinaryRegionType::UOffset64:
      s += ToValueString<uint64_t>(region, binary);
      break;
    case BinaryRegionType::UOffset:
      s += ToValueString<uint32_t>(region, binary);
      break;
    case BinaryRegionType::SOffset:
      s += ToValueString<int32_t>(region, binary);
      break;
    case BinaryRegionType::VOffset:
      s += ToValueString<uint16_t>(region, binary);
      break;

    default: break;
  }
  // If this is an offset type, include the calculated offset location in the
  // value.
  // TODO(dbaileychess): It might be nicer to put this in the comment field.
  if (IsOffset(region.type)) {
    s += " Loc: 0x";
    s += ToHex(region.points_to_offset, output_config.offset_max_char);
  }
  return s;
}

struct DocContinuation {
  // The start column where the value text first starts
  size_t value_start_column = 0;

  // The remaining part of the doc to print.
  std::string value;
};

static std::string GenerateTypeString(const BinaryRegion &region) {
  return ToString(region.type) +
         ((region.array_length)
              ? "[" + std::to_string(region.array_length) + "]"
              : "");
}

static std::string GenerateComment(const BinaryRegionComment &comment,
                                   const BinarySection &) {
  std::string s;
  switch (comment.type) {
    case BinaryRegionCommentType::Unknown: s = "unknown"; break;
    case BinaryRegionCommentType::SizePrefix: s = "size prefix"; break;
    case BinaryRegionCommentType::RootTableOffset:
      s = "offset to root table `" + comment.name + "`";
      break;
    // TODO(dbaileychess): make this lowercase to follow the convention.
    case BinaryRegionCommentType::FileIdentifier: s = "File Identifier"; break;
    case BinaryRegionCommentType::Padding: s = "padding"; break;
    case BinaryRegionCommentType::VTableSize: s = "size of this vtable"; break;
    case BinaryRegionCommentType::VTableRefferingTableLength:
      s = "size of referring table";
      break;
    case BinaryRegionCommentType::VTableFieldOffset:
      s = "offset to field `" + comment.name;
      break;
    case BinaryRegionCommentType::VTableUnknownFieldOffset:
      s = "offset to unknown field (id: " + std::to_string(comment.index) + ")";
      break;

    case BinaryRegionCommentType::TableVTableOffset:
      s = "offset to vtable";
      break;
    case BinaryRegionCommentType::TableField:
      s = "table field `" + comment.name;
      break;
    case BinaryRegionCommentType::TableUnknownField: s = "unknown field"; break;
    case BinaryRegionCommentType::TableOffsetField:
      s = "offset to field `" + comment.name + "`";
      break;
    case BinaryRegionCommentType::StructField:
      s = "struct field `" + comment.name + "`";
      break;
    case BinaryRegionCommentType::ArrayField:
      s = "array field `" + comment.name + "`[" +
          std::to_string(comment.index) + "]";
      break;
    case BinaryRegionCommentType::StringLength: s = "length of string"; break;
    case BinaryRegionCommentType::StringValue: s = "string literal"; break;
    case BinaryRegionCommentType::StringTerminator:
      s = "string terminator";
      break;
    case BinaryRegionCommentType::VectorLength:
      s = "length of vector (# items)";
      break;
    case BinaryRegionCommentType::VectorValue:
      s = "value[" + std::to_string(comment.index) + "]";
      break;
    case BinaryRegionCommentType::VectorTableValue:
      s = "offset to table[" + std::to_string(comment.index) + "]";
      break;
    case BinaryRegionCommentType::VectorStringValue:
      s = "offset to string[" + std::to_string(comment.index) + "]";
      break;
    case BinaryRegionCommentType::VectorUnionValue:
      s = "offset to union[" + std::to_string(comment.index) + "]";
      break;

    default: break;
  }
  if (!comment.default_value.empty()) { s += " " + comment.default_value; }

  switch (comment.status) {
    case BinaryRegionStatus::OK: break;  // no-op
    case BinaryRegionStatus::WARN: s = "WARN: " + s; break;
    case BinaryRegionStatus::WARN_NO_REFERENCES:
      s = "WARN: nothing refers to this section.";
      break;
    case BinaryRegionStatus::WARN_CORRUPTED_PADDING:
      s = "WARN: could be corrupted padding region.";
      break;
    case BinaryRegionStatus::WARN_PADDING_LENGTH:
      s = "WARN: padding is longer than expected.";
      break;
    case BinaryRegionStatus::ERROR: s = "ERROR: " + s; break;
    case BinaryRegionStatus::ERROR_OFFSET_OUT_OF_BINARY:
      s = "ERROR: " + s + ". Invalid offset, points outside the binary.";
      break;
    case BinaryRegionStatus::ERROR_INCOMPLETE_BINARY:
      s = "ERROR: " + s + ". Incomplete binary, expected to read " +
          comment.status_message + " bytes.";
      break;
    case BinaryRegionStatus::ERROR_LENGTH_TOO_LONG:
      s = "ERROR: " + s + ". Longer than the binary.";
      break;
    case BinaryRegionStatus::ERROR_LENGTH_TOO_SHORT:
      s = "ERROR: " + s + ". Shorter than the minimum length: ";
      break;
    case BinaryRegionStatus::ERROR_REQUIRED_FIELD_NOT_PRESENT:
      s = "ERROR: " + s + ". Required field is not present.";
      break;
    case BinaryRegionStatus::ERROR_INVALID_UNION_TYPE:
      s = "ERROR: " + s + ". Invalid union type value.";
      break;
    case BinaryRegionStatus::ERROR_CYCLE_DETECTED:
      s = "ERROR: " + s + ". Invalid offset, cycle detected.";
      break;
  }

  return s;
}

static void GenerateDocumentation(std::ostream &os, const BinaryRegion &region,
                                  const BinarySection &section,
                                  const uint8_t *binary,
                                  DocContinuation &continuation,
                                  const OutputConfig &output_config) {
  // Check if there is a doc continuation that should be prioritized.
  if (continuation.value_start_column) {
    os << std::string(continuation.value_start_column - 2, ' ');
    os << output_config.delimiter << " ";

    os << continuation.value.substr(0, output_config.max_bytes_per_line);
    continuation.value = continuation.value.substr(
        std::min(output_config.max_bytes_per_line, continuation.value.size()));
    return;
  }

  size_t size_of = 0;
  {
    std::stringstream ss;
    ss << std::setw(static_cast<int>(output_config.largest_type_string))
       << std::left;
    ss << GenerateTypeString(region);
    os << ss.str();
    size_of = ss.str().size();
  }
  os << " " << output_config.delimiter << " ";
  if (region.array_length) {
    // Record where the value is first being outputted.
    continuation.value_start_column = 3 + size_of;

    // Get the full-length value, which we will chunk below.
    const std::string value = ToValueString(region, binary, output_config);

    std::stringstream ss;
    ss << std::setw(static_cast<int>(output_config.largest_value_string))
       << std::left;
    ss << value.substr(0, output_config.max_bytes_per_line);
    os << ss.str();

    continuation.value =
        value.substr(std::min(output_config.max_bytes_per_line, value.size()));
  } else {
    std::stringstream ss;
    ss << std::setw(static_cast<int>(output_config.largest_value_string))
       << std::left;
    ss << ToValueString(region, binary, output_config);
    os << ss.str();
  }

  os << " " << output_config.delimiter << " ";
  os << GenerateComment(region.comment, section);
}

static void GenerateRegion(std::ostream &os, const BinaryRegion &region,
                           const BinarySection &section, const uint8_t *binary,
                           const OutputConfig &output_config) {
  bool doc_generated = false;
  DocContinuation doc_continuation;
  for (uint64_t i = 0; i < region.length; ++i) {
    if ((i % output_config.max_bytes_per_line) == 0) {
      // Start a new line of output
      os << std::endl;
      os << "  +0x" << ToHex(region.offset + i, output_config.offset_max_char);
      os << " " << output_config.delimiter;
    }

    // Add each byte
    os << " " << ToHex(binary[region.offset + i]);

    // Check for end of line or end of region conditions.
    if (((i + 1) % output_config.max_bytes_per_line == 0) ||
        i + 1 == region.length) {
      if (i + 1 == region.length) {
        // We are out of bytes but haven't the kMaxBytesPerLine, so we need to
        // zero those out to align everything globally.
        for (uint64_t j = i + 1; (j % output_config.max_bytes_per_line) != 0;
             ++j) {
          os << "   ";
        }
      }
      os << " " << output_config.delimiter;
      // This is the end of the first line or its the last byte of the region,
      // generate the end-of-line documentation.
      if (!doc_generated) {
        os << " ";
        GenerateDocumentation(os, region, section, binary, doc_continuation,
                              output_config);

        // If we have a value in the doc continuation, that means the doc is
        // being printed on multiple lines.
        doc_generated = doc_continuation.value.empty();
      }
    }
  }
}

static void GenerateSection(std::ostream &os, const BinarySection &section,
                            const uint8_t *binary,
                            const OutputConfig &output_config) {
  os << std::endl;
  os << ToString(section.type);
  if (!section.name.empty()) { os << " (" + section.name + ")"; }
  os << ":";

  // As a space saving measure, skip generating every vector element, just put
  // the first and last elements in the output. Skip the whole thing if there
  // are only three or fewer elements, as it doesn't save space.
  if ((section.type == BinarySectionType::Vector ||
       section.type == BinarySectionType::Vector64) &&
      !output_config.include_vector_contents && section.regions.size() > 4) {
    // Generate the length region which should be first.
    GenerateRegion(os, section.regions[0], section, binary, output_config);

    // Generate the first element.
    GenerateRegion(os, section.regions[1], section, binary, output_config);

    // Indicate that we omitted elements.
    os << std::endl
       << "  <" << section.regions.size() - 3 << " regions omitted>";

    // Generate the last element.
    GenerateRegion(os, section.regions.back(), section, binary, output_config);
    os << std::endl;
    return;
  }

  for (const BinaryRegion &region : section.regions) {
    GenerateRegion(os, region, section, binary, output_config);
  }
  os << std::endl;
}
}  // namespace

bool AnnotatedBinaryTextGenerator::Generate(
    const std::string &filename, const std::string &schema_filename) {
  OutputConfig output_config;
  output_config.max_bytes_per_line = options_.max_bytes_per_line;
  output_config.include_vector_contents = options_.include_vector_contents;

  // Given the length of the binary, we can calculate the maximum number of
  // characters to display in the offset hex: (i.e. 2 would lead to 0XFF being
  // the max output).
  output_config.offset_max_char =
      binary_length_ > 0xFFFFFF
          ? 8
          : (binary_length_ > 0xFFFF ? 6 : (binary_length_ > 0xFF ? 4 : 2));

  // Find the largest type string of all the regions in this file, so we can
  // align the output nicely.
  output_config.largest_type_string = 0;
  for (const auto &section : annotations_) {
    for (const auto &region : section.second.regions) {
      std::string s = GenerateTypeString(region);
      if (s.size() > output_config.largest_type_string) {
        output_config.largest_type_string = s.size();
      }

      // Don't consider array regions, as they will be split to multiple lines.
      if (!region.array_length) {
        s = ToValueString(region, binary_, output_config);
        if (s.size() > output_config.largest_value_string) {
          output_config.largest_value_string = s.size();
        }
      }
    }
  }

  // Modify the output filename.
  std::string output_filename = StripExtension(filename);
  output_filename += options_.output_postfix;
  output_filename +=
      "." + (options_.output_extension.empty() ? GetExtension(filename)
                                               : options_.output_extension);

  std::ofstream ofs(output_filename.c_str());

  ofs << "// Annotated Flatbuffer Binary" << std::endl;
  ofs << "//" << std::endl;
  ofs << "// Schema file: " << schema_filename << std::endl;
  ofs << "// Binary file: " << filename << std::endl;

  // Generate each of the binary sections
  for (const auto &section : annotations_) {
    GenerateSection(ofs, section.second, binary_, output_config);
  }

  ofs.close();
  return true;
}

}  // namespace flatbuffers
