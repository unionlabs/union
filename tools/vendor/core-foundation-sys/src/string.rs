// Copyright 2013-2015 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::os::raw::{c_char, c_ushort, c_void};

use base::{Boolean, CFOptionFlags, CFIndex, CFAllocatorRef, CFRange, CFTypeID};

pub type UniChar = c_ushort;

// CFString.h

pub type CFStringCompareFlags = CFOptionFlags;
//static kCFCompareCaseInsensitive: CFStringCompareFlags = 1;
//static kCFCompareBackwards: CFStringCompareFlags = 4;
//static kCFCompareAnchored: CFStringCompareFlags = 8;
//static kCFCompareNonliteral: CFStringCompareFlags = 16;
//static kCFCompareLocalized: CFStringCompareFlags = 32;
//static kCFCompareNumerically: CFStringCompareFlags = 64;
//static kCFCompareDiacriticInsensitive: CFStringCompareFlags = 128;
//static kCFCompareWidthInsensitive: CFStringCompareFlags = 256;
//static kCFCompareForcedOrdering: CFStringCompareFlags = 512;

pub type CFStringEncoding = u32;

// macOS built-in encodings.

//static kCFStringEncodingMacRoman: CFStringEncoding = 0;
//static kCFStringEncodingWindowsLatin1: CFStringEncoding = 0x0500;
//static kCFStringEncodingISOLatin1: CFStringEncoding = 0x0201;
//static kCFStringEncodingNextStepLatin: CFStringEncoding = 0x0B01;
//static kCFStringEncodingASCII: CFStringEncoding = 0x0600;
//static kCFStringEncodingUnicode: CFStringEncoding = 0x0100;
pub static kCFStringEncodingUTF8: CFStringEncoding = 0x08000100;
//static kCFStringEncodingNonLossyASCII: CFStringEncoding = 0x0BFF;

//static kCFStringEncodingUTF16: CFStringEncoding = 0x0100;
//static kCFStringEncodingUTF16BE: CFStringEncoding = 0x10000100;
//static kCFStringEncodingUTF16LE: CFStringEncoding = 0x14000100;
//static kCFStringEncodingUTF32: CFStringEncoding = 0x0c000100;
//static kCFStringEncodingUTF32BE: CFStringEncoding = 0x18000100;
//static kCFStringEncodingUTF32LE: CFStringEncoding = 0x1c000100;


// CFStringEncodingExt.h

pub type CFStringEncodings = CFIndex;

// External encodings, except those defined above.
// Defined above: kCFStringEncodingMacRoman = 0
//static kCFStringEncodingMacJapanese: CFStringEncoding = 1;
//static kCFStringEncodingMacChineseTrad: CFStringEncoding = 2;
//static kCFStringEncodingMacKorean: CFStringEncoding = 3;
//static kCFStringEncodingMacArabic: CFStringEncoding = 4;
//static kCFStringEncodingMacHebrew: CFStringEncoding = 5;
//static kCFStringEncodingMacGreek: CFStringEncoding = 6;
//static kCFStringEncodingMacCyrillic: CFStringEncoding = 7;
//static kCFStringEncodingMacDevanagari: CFStringEncoding = 9;
//static kCFStringEncodingMacGurmukhi: CFStringEncoding = 10;
//static kCFStringEncodingMacGujarati: CFStringEncoding = 11;
//static kCFStringEncodingMacOriya: CFStringEncoding = 12;
//static kCFStringEncodingMacBengali: CFStringEncoding = 13;
//static kCFStringEncodingMacTamil: CFStringEncoding = 14;
//static kCFStringEncodingMacTelugu: CFStringEncoding = 15;
//static kCFStringEncodingMacKannada: CFStringEncoding = 16;
//static kCFStringEncodingMacMalayalam: CFStringEncoding = 17;
//static kCFStringEncodingMacSinhalese: CFStringEncoding = 18;
//static kCFStringEncodingMacBurmese: CFStringEncoding = 19;
//static kCFStringEncodingMacKhmer: CFStringEncoding = 20;
//static kCFStringEncodingMacThai: CFStringEncoding = 21;
//static kCFStringEncodingMacLaotian: CFStringEncoding = 22;
//static kCFStringEncodingMacGeorgian: CFStringEncoding = 23;
//static kCFStringEncodingMacArmenian: CFStringEncoding = 24;
//static kCFStringEncodingMacChineseSimp: CFStringEncoding = 25;
//static kCFStringEncodingMacTibetan: CFStringEncoding = 26;
//static kCFStringEncodingMacMongolian: CFStringEncoding = 27;
//static kCFStringEncodingMacEthiopic: CFStringEncoding = 28;
//static kCFStringEncodingMacCentralEurRoman: CFStringEncoding = 29;
//static kCFStringEncodingMacVietnamese: CFStringEncoding = 30;
//static kCFStringEncodingMacExtArabic: CFStringEncoding = 31;
//static kCFStringEncodingMacSymbol: CFStringEncoding = 33;
//static kCFStringEncodingMacDingbats: CFStringEncoding = 34;
//static kCFStringEncodingMacTurkish: CFStringEncoding = 35;
//static kCFStringEncodingMacCroatian: CFStringEncoding = 36;
//static kCFStringEncodingMacIcelandic: CFStringEncoding = 37;
//static kCFStringEncodingMacRomanian: CFStringEncoding = 38;
//static kCFStringEncodingMacCeltic: CFStringEncoding = 39;
//static kCFStringEncodingMacGaelic: CFStringEncoding = 40;
//static kCFStringEncodingMacFarsi: CFStringEncoding = 0x8C;
//static kCFStringEncodingMacUkrainian: CFStringEncoding = 0x98;
//static kCFStringEncodingMacInuit: CFStringEncoding = 0xEC;
//static kCFStringEncodingMacVT100: CFStringEncoding = 0xFC;
//static kCFStringEncodingMacHFS: CFStringEncoding = 0xFF;
// Defined above: kCFStringEncodingISOLatin1 = 0x0201
//static kCFStringEncodingISOLatin2: CFStringEncoding = 0x0202;
//static kCFStringEncodingISOLatin3: CFStringEncoding = 0x0203;
//static kCFStringEncodingISOLatin4: CFStringEncoding = 0x0204;
//static kCFStringEncodingISOLatinCyrillic: CFStringEncoding = 0x0205;
//static kCFStringEncodingISOLatinArabic: CFStringEncoding = 0x0206;
//static kCFStringEncodingISOLatinGreek: CFStringEncoding = 0x0207;
//static kCFStringEncodingISOLatinHebrew: CFStringEncoding = 0x0208;
//static kCFStringEncodingISOLatin5: CFStringEncoding = 0x0209;
//static kCFStringEncodingISOLatin6: CFStringEncoding = 0x020A;
//static kCFStringEncodingISOLatinThai: CFStringEncoding = 0x020B;
//static kCFStringEncodingISOLatin7: CFStringEncoding = 0x020D;
//static kCFStringEncodingISOLatin8: CFStringEncoding = 0x020E;
//static kCFStringEncodingISOLatin9: CFStringEncoding = 0x020F;
//static kCFStringEncodingISOLatin10: CFStringEncoding = 0x0210;
//static kCFStringEncodingDOSLatinUS: CFStringEncoding = 0x0400;
//static kCFStringEncodingDOSGreek: CFStringEncoding = 0x0405;
//static kCFStringEncodingDOSBalticRim: CFStringEncoding = 0x0406;
//static kCFStringEncodingDOSLatin1: CFStringEncoding = 0x0410;
//static kCFStringEncodingDOSGreek1: CFStringEncoding = 0x0411;
//static kCFStringEncodingDOSLatin2: CFStringEncoding = 0x0412;
//static kCFStringEncodingDOSCyrillic: CFStringEncoding = 0x0413;
//static kCFStringEncodingDOSTurkish: CFStringEncoding = 0x0414;
//static kCFStringEncodingDOSPortuguese: CFStringEncoding = 0x0415;
//static kCFStringEncodingDOSIcelandic: CFStringEncoding = 0x0416;
//static kCFStringEncodingDOSHebrew: CFStringEncoding = 0x0417;
//static kCFStringEncodingDOSCanadianFrench: CFStringEncoding = 0x0418;
//static kCFStringEncodingDOSArabic: CFStringEncoding = 0x0419;
//static kCFStringEncodingDOSNordic: CFStringEncoding = 0x041A;
//static kCFStringEncodingDOSRussian: CFStringEncoding = 0x041B;
//static kCFStringEncodingDOSGreek2: CFStringEncoding = 0x041C;
//static kCFStringEncodingDOSThai: CFStringEncoding = 0x041D;
//static kCFStringEncodingDOSJapanese: CFStringEncoding = 0x0420;
//static kCFStringEncodingDOSChineseSimplif: CFStringEncoding = 0x0421;
//static kCFStringEncodingDOSKorean: CFStringEncoding = 0x0422;
//static kCFStringEncodingDOSChineseTrad: CFStringEncoding = 0x0423;
// Defined above: kCFStringEncodingWindowsLatin1 = 0x0500
//static kCFStringEncodingWindowsLatin2: CFStringEncoding = 0x0501;
//static kCFStringEncodingWindowsCyrillic: CFStringEncoding = 0x0502;
//static kCFStringEncodingWindowsGreek: CFStringEncoding = 0x0503;
//static kCFStringEncodingWindowsLatin5: CFStringEncoding = 0x0504;
//static kCFStringEncodingWindowsHebrew: CFStringEncoding = 0x0505;
//static kCFStringEncodingWindowsArabic: CFStringEncoding = 0x0506;
//static kCFStringEncodingWindowsBalticRim: CFStringEncoding = 0x0507;
//static kCFStringEncodingWindowsVietnamese: CFStringEncoding = 0x0508;
//static kCFStringEncodingWindowsKoreanJohab: CFStringEncoding = 0x0510;
// Defined above: kCFStringEncodingASCII = 0x0600
//static kCFStringEncodingANSEL: CFStringEncoding = 0x0601;
//static kCFStringEncodingJIS_X0201_76: CFStringEncoding = 0x0620;
//static kCFStringEncodingJIS_X0208_83: CFStringEncoding = 0x0621;
//static kCFStringEncodingJIS_X0208_90: CFStringEncoding = 0x0622;
//static kCFStringEncodingJIS_X0212_90: CFStringEncoding = 0x0623;
//static kCFStringEncodingJIS_C6226_78: CFStringEncoding = 0x0624;
//static kCFStringEncodingShiftJIS_X0213: CFStringEncoding = 0x0628;
//static kCFStringEncodingShiftJIS_X0213_MenKuTen: CFStringEncoding = 0x0629;
//static kCFStringEncodingGB_2312_80: CFStringEncoding = 0x0630;
//static kCFStringEncodingGBK_95: CFStringEncoding = 0x0631;
//static kCFStringEncodingGB_18030_2000: CFStringEncoding = 0x0632;
//static kCFStringEncodingKSC_5601_87: CFStringEncoding = 0x0640;
//static kCFStringEncodingKSC_5601_92_Johab: CFStringEncoding = 0x0641;
//static kCFStringEncodingCNS_11643_92_P1: CFStringEncoding = 0x0651;
//static kCFStringEncodingCNS_11643_92_P2: CFStringEncoding = 0x0652;
//static kCFStringEncodingCNS_11643_92_P3: CFStringEncoding = 0x0653;
//static kCFStringEncodingISO_2022_JP: CFStringEncoding = 0x0820;
//static kCFStringEncodingISO_2022_JP_2: CFStringEncoding = 0x0821;
//static kCFStringEncodingISO_2022_JP_1: CFStringEncoding = 0x0822;
//static kCFStringEncodingISO_2022_JP_3: CFStringEncoding = 0x0823;
//static kCFStringEncodingISO_2022_CN: CFStringEncoding = 0x0830;
//static kCFStringEncodingISO_2022_CN_EXT: CFStringEncoding = 0x0831;
//static kCFStringEncodingISO_2022_KR: CFStringEncoding = 0x0840;
//static kCFStringEncodingEUC_JP: CFStringEncoding = 0x0920;
//static kCFStringEncodingEUC_CN: CFStringEncoding = 0x0930;
//static kCFStringEncodingEUC_TW: CFStringEncoding = 0x0931;
//static kCFStringEncodingEUC_KR: CFStringEncoding = 0x0940;
//static kCFStringEncodingShiftJIS: CFStringEncoding = 0x0A01;
//static kCFStringEncodingKOI8_R: CFStringEncoding = 0x0A02;
//static kCFStringEncodingBig5: CFStringEncoding = 0x0A03;
//static kCFStringEncodingMacRomanLatin1: CFStringEncoding = 0x0A04;
//static kCFStringEncodingHZ_GB_2312: CFStringEncoding = 0x0A05;
//static kCFStringEncodingBig5_HKSCS_1999: CFStringEncoding = 0x0A06;
//static kCFStringEncodingVISCII: CFStringEncoding = 0x0A07;
//static kCFStringEncodingKOI8_U: CFStringEncoding = 0x0A08;
//static kCFStringEncodingBig5_E: CFStringEncoding = 0x0A09;
// Defined above: kCFStringEncodingNextStepLatin = 0x0B01
//static kCFStringEncodingNextStepJapanese: CFStringEncoding = 0x0B02;
//static kCFStringEncodingEBCDIC_US: CFStringEncoding = 0x0C01;
//static kCFStringEncodingEBCDIC_CP037: CFStringEncoding = 0x0C02;
//static kCFStringEncodingUTF7: CFStringEncoding = 0x04000100;
//static kCFStringEncodingUTF7_IMAP: CFStringEncoding = 0x0A10;
//static kCFStringEncodingShiftJIS_X0213_00: CFStringEncoding = 0x0628; /* Deprecated */

#[repr(C)]
pub struct __CFString(c_void);

pub type CFStringRef = *const __CFString;

extern {
    /*
     * CFString.h
     */

    // N.B. organized according to "Functions by task" in docs

    /* Creating a CFString */
    //fn CFSTR
    //fn CFStringCreateArrayBySeparatingStrings
    //fn CFStringCreateByCombiningStrings
    //fn CFStringCreateCopy
    //fn CFStringCreateFromExternalRepresentation
    pub fn CFStringCreateWithBytes(alloc: CFAllocatorRef,
                                   bytes: *const u8,
                                   numBytes: CFIndex,
                                   encoding: CFStringEncoding,
                                   isExternalRepresentation: Boolean)
                                   -> CFStringRef;
    pub fn CFStringCreateWithBytesNoCopy(alloc: CFAllocatorRef,
                                         bytes: *const u8,
                                         numBytes: CFIndex,
                                         encoding: CFStringEncoding,
                                         isExternalRepresentation: Boolean,
                                         contentsDeallocator: CFAllocatorRef)
                                         -> CFStringRef;
    //fn CFStringCreateWithCharacters
    pub fn CFStringCreateWithCharactersNoCopy(alloc: CFAllocatorRef,
                                              chars: *const UniChar,
                                              numChars: CFIndex,
                                              contentsDeallocator: CFAllocatorRef)
                                              -> CFStringRef;
    pub fn CFStringCreateWithCString(alloc: CFAllocatorRef,
                                     cStr: *const c_char,
                                     encoding: CFStringEncoding)
                                     -> CFStringRef;
    //fn CFStringCreateWithCStringNoCopy
    //fn CFStringCreateWithFormat
    //fn CFStringCreateWithFormatAndArguments
    //fn CFStringCreateWithPascalString
    //fn CFStringCreateWithPascalStringNoCopy
    //fn CFStringCreateWithSubstring

    /* Searching Strings */
    //fn CFStringCreateArrayWithFindResults
    //fn CFStringFind
    //fn CFStringFindCharacterFromSet
    //fn CFStringFindWithOptions
    //fn CFStringFindWithOptionsAndLocale
    //fn CFStringGetLineBounds

    /* Comparing Strings */
    //fn CFStringCompare
    //fn CFStringCompareWithOptions
    //fn CFStringCompareWithOptionsAndLocale
    //fn CFStringHasPrefix
    //fn CFStringHasSuffix

    /* Accessing Characters */
    //fn CFStringCreateExternalRepresentation
    pub fn CFStringGetBytes(theString: CFStringRef,
                            range: CFRange,
                            encoding: CFStringEncoding,
                            lossByte: u8,
                            isExternalRepresentation: Boolean,
                            buffer: *mut u8,
                            maxBufLen: CFIndex,
                            usedBufLen: *mut CFIndex)
                            -> CFIndex;
    //fn CFStringGetCharacterAtIndex
    //fn CFStringGetCharacters
    //fn CFStringGetCharactersPtr
    //fn CFStringGetCharacterFromInlineBuffer
    pub fn CFStringGetCString(theString: CFStringRef,
                              buffer: *mut c_char,
                              bufferSize: CFIndex,
                              encoding: CFStringEncoding)
                              -> Boolean;
    pub fn CFStringGetCStringPtr(theString: CFStringRef,
                                 encoding: CFStringEncoding)
                                 -> *const c_char;
    pub fn CFStringGetLength(theString: CFStringRef) -> CFIndex;
    //fn CFStringGetPascalString
    //fn CFStringGetPascalStringPtr
    //fn CFStringGetRangeOfComposedCharactersAtIndex
    //fn CFStringInitInlineBuffer

    /* Working With Hyphenation */
    //fn CFStringGetHyphenationLocationBeforeIndex
    //fn CFStringIsHyphenationAvailableForLocale

    /* Working With Encodings */
    //fn CFStringConvertEncodingToIANACharSetName
    //fn CFStringConvertEncodingToNSStringEncoding
    //fn CFStringConvertEncodingToWindowsCodepage
    //fn CFStringConvertIANACharSetNameToEncoding
    //fn CFStringConvertNSStringEncodingToEncoding
    //fn CFStringConvertWindowsCodepageToEncoding
    //fn CFStringGetFastestEncoding
    //fn CFStringGetListOfAvailableEncodings
    //fn CFStringGetMaximumSizeForEncoding
    //fn CFStringGetMostCompatibleMacStringEncoding
    //fn CFStringGetNameOfEncoding
    //fn CFStringGetSmallestEncoding
    //fn CFStringGetSystemEncoding
    //fn CFStringIsEncodingAvailable

    /* Getting Numeric Values */
    //fn CFStringGetDoubleValue
    //fn CFStringGetIntValue

    /* Getting String Properties */
    //fn CFShowStr
    pub fn CFStringGetTypeID() -> CFTypeID;

    /* String File System Representations */
    //fn CFStringCreateWithFileSystemRepresentation
    //fn CFStringGetFileSystemRepresentation
    //fn CFStringGetMaximumSizeOfFileSystemRepresentation

    /* Getting Paragraph Bounds */
    //fn CFStringGetParagraphBounds

    /* Managing Surrogates */
    //fn CFStringGetLongCharacterForSurrogatePair
    //fn CFStringGetSurrogatePairForLongCharacter
    //fn CFStringIsSurrogateHighCharacter
    //fn CFStringIsSurrogateLowCharacter
}
