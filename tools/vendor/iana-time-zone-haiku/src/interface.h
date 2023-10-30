#pragma once

#include "rust/cxx.h"

#include <cstddef>

namespace iana_time_zone_haiku {
size_t get_tz(rust::Slice<uint8_t> buf);
}
