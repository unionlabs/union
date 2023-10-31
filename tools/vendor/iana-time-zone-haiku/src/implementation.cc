#include "iana-time-zone-haiku/src/interface.h"
#include "iana-time-zone-haiku/src/lib.rs.h"

#ifdef __HAIKU__

#include <cstring>

#include <Errors.h>
#include <LocaleRoster.h>
#include <String.h>
#include <TimeZone.h>

namespace iana_time_zone_haiku {
size_t get_tz(rust::Slice<uint8_t> buf) {
    try {
        static_assert(sizeof(char) == sizeof(uint8_t), "Illegal char size");

        if (buf.empty()) {
            return 0;
        }

        // `BLocaleRoster::Default()` returns a reference to a statically allocated object.
        // https://github.com/haiku/haiku/blob/8f16317/src/kits/locale/LocaleRoster.cpp#L143-L147
        BLocaleRoster *locale_roster(BLocaleRoster::Default());
        if (!locale_roster) {
            return 0;
        }

        BTimeZone tz(NULL, NULL);
        if (locale_roster->GetDefaultTimeZone(&tz) != B_OK) {
            return 0;
        }

        BString bname(tz.ID());
        int32_t ilength(bname.Length());
        if (ilength <= 0) {
            return 0;
        }

        size_t length(ilength);
        if (length > buf.size()) {
            return 0;
        }

        // BString::String() returns a borrowed string.
        // https://www.haiku-os.org/docs/api/classBString.html#ae4fe78b06c8e3310093b80305e14ba87
        const char *sname(bname.String());
        if (!sname) {
            return 0;
        }

        std::memcpy(buf.data(), sname, length);
        return length;
    } catch (...) {
        return 0;
    }
}
}  // namespace iana_time_zone_haiku

#else

namespace iana_time_zone_haiku {
size_t get_tz(rust::Slice<uint8_t>) { return 0; }
}  // namespace iana_time_zone_haiku

#endif
