#if defined(__x86_64) || defined(__x86_64__)
# if defined(__ELF__)
#  if defined(__BLST_PORTABLE__)
#   include "elf/sha256-portable-x86_64.s"
#   define blst_sha256_block_data_order blst_sha256_block_ssse3
#  endif
#  include "elf/sha256-x86_64.s"
#  if defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "elf/ctx_inverse_mod_384-x86_64.s"
#  endif
#  if !defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "elf/ctq_inverse_mod_384-x86_64.s"
#  endif
#  include "elf/add_mod_384-x86_64.s"
#  include "elf/add_mod_384x384-x86_64.s"
#  if defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "elf/mulx_mont_384-x86_64.s"
#   include "elf/mulx_mont_256-x86_64.s"
#  endif
#  if !defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "elf/mulq_mont_384-x86_64.s"
#   include "elf/mulq_mont_256-x86_64.s"
#  endif
#  include "elf/add_mod_256-x86_64.s"
#  include "elf/ct_inverse_mod_256-x86_64.s"
#  include "elf/div3w-x86_64.s"
#  include "elf/ct_is_square_mod_384-x86_64.s"
# elif defined(_WIN64) || defined(__CYGWIN__)
#  if defined(__BLST_PORTABLE__)
#   include "coff/sha256-portable-x86_64.s"
#   define blst_sha256_block_data_order                     blst_sha256_block_ssse3
#   define LSEH_begin_blst_sha256_block_data_order          LSEH_begin_blst_sha256_block_ssse3
#   define LSEH_body_blst_sha256_block_data_order           LSEH_body_blst_sha256_block_ssse3
#   define LSEH_info_blst_sha256_block_data_order_prologue  LSEH_info_blst_sha256_block_ssse3_prologue
#   define LSEH_body_blst_sha256_block_data_order           LSEH_body_blst_sha256_block_ssse3
#   define LSEH_epilogue_blst_sha256_block_data_order       LSEH_epilogue_blst_sha256_block_ssse3
#   define LSEH_info_blst_sha256_block_data_order_body      LSEH_info_blst_sha256_block_ssse3_body
#   define LSEH_epilogue_blst_sha256_block_data_order       LSEH_epilogue_blst_sha256_block_ssse3
#   define LSEH_end_blst_sha256_block_data_order            LSEH_end_blst_sha256_block_ssse3
#   define LSEH_info_blst_sha256_block_data_order_epilogue  LSEH_info_blst_sha256_block_ssse3_epilogue
#  endif
#  include "coff/sha256-x86_64.s"
#  if defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "coff/ctx_inverse_mod_384-x86_64.s"
#  endif
#  if !defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "coff/ctq_inverse_mod_384-x86_64.s"
#  endif
#  include "coff/add_mod_384-x86_64.s"
#  include "coff/add_mod_384x384-x86_64.s"
#  if defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "coff/mulx_mont_384-x86_64.s"
#   include "coff/mulx_mont_256-x86_64.s"
#  endif
#  if !defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "coff/mulq_mont_384-x86_64.s"
#   include "coff/mulq_mont_256-x86_64.s"
#  endif
#  include "coff/add_mod_256-x86_64.s"
#  include "coff/ct_inverse_mod_256-x86_64.s"
#  include "coff/div3w-x86_64.s"
#  include "coff/ct_is_square_mod_384-x86_64.s"
# elif defined(__APPLE__)
#  include "mach-o/sha256-x86_64.s"
#  if defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "mach-o/ctx_inverse_mod_384-x86_64.s"
#  endif
#  if !defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "mach-o/ctq_inverse_mod_384-x86_64.s"
#  endif
#  include "mach-o/add_mod_384-x86_64.s"
#  include "mach-o/add_mod_384x384-x86_64.s"
#  if defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "mach-o/mulx_mont_384-x86_64.s"
#   include "mach-o/mulx_mont_256-x86_64.s"
#  endif
#  if !defined(__ADX__) || defined(__BLST_PORTABLE__)
#   include "mach-o/mulq_mont_384-x86_64.s"
#   include "mach-o/mulq_mont_256-x86_64.s"
#  endif
#  include "mach-o/add_mod_256-x86_64.s"
#  include "mach-o/ct_inverse_mod_256-x86_64.s"
#  include "mach-o/div3w-x86_64.s"
#  include "mach-o/ct_is_square_mod_384-x86_64.s"
# endif
#elif defined(__aarch64__)
# if defined(__CHERI_PURE_CAPABILITY__)
#  include "cheri/sha256-armv8.S"
#  include "cheri/ct_inverse_mod_384-armv8.S"
#  include "cheri/add_mod_384-armv8.S"
#  define __add_mod_384     __add_mont_384
#  define __sub_mod_384     __sub_mont_384
#  include "cheri/mul_mont_384-armv8.S"
#  include "cheri/mul_mont_256-armv8.S"
#  include "cheri/add_mod_256-armv8.S"
#  include "cheri/ct_inverse_mod_256-armv8.S"
#  include "cheri/div3w-armv8.S"
#  include "cheri/ct_is_square_mod_384-armv8.S"
# elif defined(__ELF__)
#  include "elf/sha256-armv8.S"
#  include "elf/ct_inverse_mod_384-armv8.S"
#  include "elf/add_mod_384-armv8.S"
#  define __add_mod_384     __add_mont_384
#  define __sub_mod_384     __sub_mont_384
#  include "elf/mul_mont_384-armv8.S"
#  include "elf/mul_mont_256-armv8.S"
#  include "elf/add_mod_256-armv8.S"
#  include "elf/ct_inverse_mod_256-armv8.S"
#  include "elf/div3w-armv8.S"
#  include "elf/ct_is_square_mod_384-armv8.S"
# elif defined(_WIN64)
#  include "coff/sha256-armv8.S"
#  include "coff/ct_inverse_mod_384-armv8.S"
#  include "coff/add_mod_384-armv8.S"
#  define __add_mod_384     __add_mont_384
#  define __sub_mod_384     __sub_mont_384
#  include "coff/mul_mont_384-armv8.S"
#  include "coff/mul_mont_256-armv8.S"
#  include "coff/add_mod_256-armv8.S"
#  include "coff/ct_inverse_mod_256-armv8.S"
#  include "coff/div3w-armv8.S"
#  include "coff/ct_is_square_mod_384-armv8.S"
# elif defined(__APPLE__)
#  include "mach-o/sha256-armv8.S"
#  include "mach-o/ct_inverse_mod_384-armv8.S"
#  include "mach-o/add_mod_384-armv8.S"
#  define __add_mod_384     __add_mont_384
#  define __sub_mod_384     __sub_mont_384
#  include "mach-o/mul_mont_384-armv8.S"
#  include "mach-o/mul_mont_256-armv8.S"
#  include "mach-o/add_mod_256-armv8.S"
#  include "mach-o/ct_inverse_mod_256-armv8.S"
#  include "mach-o/div3w-armv8.S"
#  include "mach-o/ct_is_square_mod_384-armv8.S"
# endif
#elif defined(__BLST_NO_ASM__) || \
      (defined(__SIZEOF_POINTER__) && __SIZEOF_POINTER__==4)
/* inaccurate way to detect a 32-bit processor, but it's close enough */
#else
# error "unsupported platform"
#endif
