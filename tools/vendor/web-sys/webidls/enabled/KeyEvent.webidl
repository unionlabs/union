/* -*- Mode: IDL; tab-width: 2; indent-tabs-mode: nil; c-basic-offset: 2 -*- */
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this file,
 * You can obtain one at http://mozilla.org/MPL/2.0/.
 */

// http://www.w3.org/TR/1999/WD-DOM-Level-2-19990923/events.html#Events-KeyEvent
interface KeyEvent
{
  const unsigned long DOM_VK_CANCEL         = 0x03;
  const unsigned long DOM_VK_HELP           = 0x06;
  const unsigned long DOM_VK_BACK_SPACE     = 0x08;
  const unsigned long DOM_VK_TAB            = 0x09;
  const unsigned long DOM_VK_CLEAR          = 0x0C;
  const unsigned long DOM_VK_RETURN         = 0x0D;
  // DOM_VK_ENTER has been never used for representing native key events.
  // Therefore, it's removed for preventing developers being confused.
  // const unsigned long DOM_VK_ENTER          = 0x0E;
  const unsigned long DOM_VK_SHIFT          = 0x10;
  const unsigned long DOM_VK_CONTROL        = 0x11;
  const unsigned long DOM_VK_ALT            = 0x12;
  const unsigned long DOM_VK_PAUSE          = 0x13;
  const unsigned long DOM_VK_CAPS_LOCK      = 0x14;
  const unsigned long DOM_VK_KANA           = 0x15;
  const unsigned long DOM_VK_HANGUL         = 0x15;
  const unsigned long DOM_VK_EISU           = 0x16; // Japanese Mac keyboard only
  const unsigned long DOM_VK_JUNJA          = 0x17;
  const unsigned long DOM_VK_FINAL          = 0x18;
  const unsigned long DOM_VK_HANJA          = 0x19;
  const unsigned long DOM_VK_KANJI          = 0x19;
  const unsigned long DOM_VK_ESCAPE         = 0x1B;
  const unsigned long DOM_VK_CONVERT        = 0x1C;
  const unsigned long DOM_VK_NONCONVERT     = 0x1D;
  const unsigned long DOM_VK_ACCEPT         = 0x1E;
  const unsigned long DOM_VK_MODECHANGE     = 0x1F;
  const unsigned long DOM_VK_SPACE          = 0x20;
  const unsigned long DOM_VK_PAGE_UP        = 0x21;
  const unsigned long DOM_VK_PAGE_DOWN      = 0x22;
  const unsigned long DOM_VK_END            = 0x23;
  const unsigned long DOM_VK_HOME           = 0x24;
  const unsigned long DOM_VK_LEFT           = 0x25;
  const unsigned long DOM_VK_UP             = 0x26;
  const unsigned long DOM_VK_RIGHT          = 0x27;
  const unsigned long DOM_VK_DOWN           = 0x28;
  const unsigned long DOM_VK_SELECT         = 0x29;
  const unsigned long DOM_VK_PRINT          = 0x2A;
  const unsigned long DOM_VK_EXECUTE        = 0x2B;
  const unsigned long DOM_VK_PRINTSCREEN    = 0x2C;
  const unsigned long DOM_VK_INSERT         = 0x2D;
  const unsigned long DOM_VK_DELETE         = 0x2E;

  // DOM_VK_0 - DOM_VK_9 match their ascii values
  const unsigned long DOM_VK_0              = 0x30;
  const unsigned long DOM_VK_1              = 0x31;
  const unsigned long DOM_VK_2              = 0x32;
  const unsigned long DOM_VK_3              = 0x33;
  const unsigned long DOM_VK_4              = 0x34;
  const unsigned long DOM_VK_5              = 0x35;
  const unsigned long DOM_VK_6              = 0x36;
  const unsigned long DOM_VK_7              = 0x37;
  const unsigned long DOM_VK_8              = 0x38;
  const unsigned long DOM_VK_9              = 0x39;

  const unsigned long DOM_VK_COLON          = 0x3A;
  const unsigned long DOM_VK_SEMICOLON      = 0x3B;
  const unsigned long DOM_VK_LESS_THAN      = 0x3C;
  const unsigned long DOM_VK_EQUALS         = 0x3D;
  const unsigned long DOM_VK_GREATER_THAN   = 0x3E;
  const unsigned long DOM_VK_QUESTION_MARK  = 0x3F;
  const unsigned long DOM_VK_AT             = 0x40;

  // DOM_VK_A - DOM_VK_Z match their ascii values
  const unsigned long DOM_VK_A              = 0x41;
  const unsigned long DOM_VK_B              = 0x42;
  const unsigned long DOM_VK_C              = 0x43;
  const unsigned long DOM_VK_D              = 0x44;
  const unsigned long DOM_VK_E              = 0x45;
  const unsigned long DOM_VK_F              = 0x46;
  const unsigned long DOM_VK_G              = 0x47;
  const unsigned long DOM_VK_H              = 0x48;
  const unsigned long DOM_VK_I              = 0x49;
  const unsigned long DOM_VK_J              = 0x4A;
  const unsigned long DOM_VK_K              = 0x4B;
  const unsigned long DOM_VK_L              = 0x4C;
  const unsigned long DOM_VK_M              = 0x4D;
  const unsigned long DOM_VK_N              = 0x4E;
  const unsigned long DOM_VK_O              = 0x4F;
  const unsigned long DOM_VK_P              = 0x50;
  const unsigned long DOM_VK_Q              = 0x51;
  const unsigned long DOM_VK_R              = 0x52;
  const unsigned long DOM_VK_S              = 0x53;
  const unsigned long DOM_VK_T              = 0x54;
  const unsigned long DOM_VK_U              = 0x55;
  const unsigned long DOM_VK_V              = 0x56;
  const unsigned long DOM_VK_W              = 0x57;
  const unsigned long DOM_VK_X              = 0x58;
  const unsigned long DOM_VK_Y              = 0x59;
  const unsigned long DOM_VK_Z              = 0x5A;

  const unsigned long DOM_VK_WIN            = 0x5B;
  const unsigned long DOM_VK_CONTEXT_MENU   = 0x5D;
  const unsigned long DOM_VK_SLEEP          = 0x5F;

  // Numpad keys
  const unsigned long DOM_VK_NUMPAD0        = 0x60;
  const unsigned long DOM_VK_NUMPAD1        = 0x61;
  const unsigned long DOM_VK_NUMPAD2        = 0x62;
  const unsigned long DOM_VK_NUMPAD3        = 0x63;
  const unsigned long DOM_VK_NUMPAD4        = 0x64;
  const unsigned long DOM_VK_NUMPAD5        = 0x65;
  const unsigned long DOM_VK_NUMPAD6        = 0x66;
  const unsigned long DOM_VK_NUMPAD7        = 0x67;
  const unsigned long DOM_VK_NUMPAD8        = 0x68;
  const unsigned long DOM_VK_NUMPAD9        = 0x69;
  const unsigned long DOM_VK_MULTIPLY       = 0x6A;
  const unsigned long DOM_VK_ADD            = 0x6B;
  const unsigned long DOM_VK_SEPARATOR      = 0x6C;
  const unsigned long DOM_VK_SUBTRACT       = 0x6D;
  const unsigned long DOM_VK_DECIMAL        = 0x6E;
  const unsigned long DOM_VK_DIVIDE         = 0x6F;

  const unsigned long DOM_VK_F1             = 0x70;
  const unsigned long DOM_VK_F2             = 0x71;
  const unsigned long DOM_VK_F3             = 0x72;
  const unsigned long DOM_VK_F4             = 0x73;
  const unsigned long DOM_VK_F5             = 0x74;
  const unsigned long DOM_VK_F6             = 0x75;
  const unsigned long DOM_VK_F7             = 0x76;
  const unsigned long DOM_VK_F8             = 0x77;
  const unsigned long DOM_VK_F9             = 0x78;
  const unsigned long DOM_VK_F10            = 0x79;
  const unsigned long DOM_VK_F11            = 0x7A;
  const unsigned long DOM_VK_F12            = 0x7B;
  const unsigned long DOM_VK_F13            = 0x7C;
  const unsigned long DOM_VK_F14            = 0x7D;
  const unsigned long DOM_VK_F15            = 0x7E;
  const unsigned long DOM_VK_F16            = 0x7F;
  const unsigned long DOM_VK_F17            = 0x80;
  const unsigned long DOM_VK_F18            = 0x81;
  const unsigned long DOM_VK_F19            = 0x82;
  const unsigned long DOM_VK_F20            = 0x83;
  const unsigned long DOM_VK_F21            = 0x84;
  const unsigned long DOM_VK_F22            = 0x85;
  const unsigned long DOM_VK_F23            = 0x86;
  const unsigned long DOM_VK_F24            = 0x87;

  const unsigned long DOM_VK_NUM_LOCK       = 0x90;
  const unsigned long DOM_VK_SCROLL_LOCK    = 0x91;

  // OEM specific virtual keyCode of Windows should pass through DOM keyCode
  // for compatibility with the other web browsers on Windows.
  const unsigned long DOM_VK_WIN_OEM_FJ_JISHO   = 0x92;
  const unsigned long DOM_VK_WIN_OEM_FJ_MASSHOU = 0x93;
  const unsigned long DOM_VK_WIN_OEM_FJ_TOUROKU = 0x94;
  const unsigned long DOM_VK_WIN_OEM_FJ_LOYA    = 0x95;
  const unsigned long DOM_VK_WIN_OEM_FJ_ROYA    = 0x96;

  const unsigned long DOM_VK_CIRCUMFLEX     = 0xA0;
  const unsigned long DOM_VK_EXCLAMATION    = 0xA1;
  const unsigned long DOM_VK_DOUBLE_QUOTE   = 0xA2;
  const unsigned long DOM_VK_HASH           = 0xA3;
  const unsigned long DOM_VK_DOLLAR         = 0xA4;
  const unsigned long DOM_VK_PERCENT        = 0xA5;
  const unsigned long DOM_VK_AMPERSAND      = 0xA6;
  const unsigned long DOM_VK_UNDERSCORE     = 0xA7;
  const unsigned long DOM_VK_OPEN_PAREN     = 0xA8;
  const unsigned long DOM_VK_CLOSE_PAREN    = 0xA9;
  const unsigned long DOM_VK_ASTERISK       = 0xAA;
  const unsigned long DOM_VK_PLUS           = 0xAB;
  const unsigned long DOM_VK_PIPE           = 0xAC;
  const unsigned long DOM_VK_HYPHEN_MINUS   = 0xAD;

  const unsigned long DOM_VK_OPEN_CURLY_BRACKET  = 0xAE;
  const unsigned long DOM_VK_CLOSE_CURLY_BRACKET = 0xAF;

  const unsigned long DOM_VK_TILDE          = 0xB0;

  const unsigned long DOM_VK_VOLUME_MUTE    = 0xB5;
  const unsigned long DOM_VK_VOLUME_DOWN    = 0xB6;
  const unsigned long DOM_VK_VOLUME_UP      = 0xB7;

  const unsigned long DOM_VK_COMMA          = 0xBC;
  const unsigned long DOM_VK_PERIOD         = 0xBE;
  const unsigned long DOM_VK_SLASH          = 0xBF;
  const unsigned long DOM_VK_BACK_QUOTE     = 0xC0;
  const unsigned long DOM_VK_OPEN_BRACKET   = 0xDB; // square bracket
  const unsigned long DOM_VK_BACK_SLASH     = 0xDC;
  const unsigned long DOM_VK_CLOSE_BRACKET  = 0xDD; // square bracket
  const unsigned long DOM_VK_QUOTE          = 0xDE; // Apostrophe

  const unsigned long DOM_VK_META           = 0xE0;
  const unsigned long DOM_VK_ALTGR          = 0xE1;

  // OEM specific virtual keyCode of Windows should pass through DOM keyCode
  // for compatibility with the other web browsers on Windows.
  const unsigned long DOM_VK_WIN_ICO_HELP    = 0xE3;
  const unsigned long DOM_VK_WIN_ICO_00      = 0xE4;

  // IME processed key.
  const unsigned long DOM_VK_PROCESSKEY      = 0xE5;

  // OEM specific virtual keyCode of Windows should pass through DOM keyCode
  // for compatibility with the other web browsers on Windows.
  const unsigned long DOM_VK_WIN_ICO_CLEAR   = 0xE6;
  const unsigned long DOM_VK_WIN_OEM_RESET   = 0xE9;
  const unsigned long DOM_VK_WIN_OEM_JUMP    = 0xEA;
  const unsigned long DOM_VK_WIN_OEM_PA1     = 0xEB;
  const unsigned long DOM_VK_WIN_OEM_PA2     = 0xEC;
  const unsigned long DOM_VK_WIN_OEM_PA3     = 0xED;
  const unsigned long DOM_VK_WIN_OEM_WSCTRL  = 0xEE;
  const unsigned long DOM_VK_WIN_OEM_CUSEL   = 0xEF;
  const unsigned long DOM_VK_WIN_OEM_ATTN    = 0xF0;
  const unsigned long DOM_VK_WIN_OEM_FINISH  = 0xF1;
  const unsigned long DOM_VK_WIN_OEM_COPY    = 0xF2;
  const unsigned long DOM_VK_WIN_OEM_AUTO    = 0xF3;
  const unsigned long DOM_VK_WIN_OEM_ENLW    = 0xF4;
  const unsigned long DOM_VK_WIN_OEM_BACKTAB = 0xF5;

  // Following keys are not used on most keyboards.  However, for compatibility
  // with other browsers on Windows, we should define them.
  const unsigned long DOM_VK_ATTN           = 0xF6;
  const unsigned long DOM_VK_CRSEL          = 0xF7;
  const unsigned long DOM_VK_EXSEL          = 0xF8;
  const unsigned long DOM_VK_EREOF          = 0xF9;
  const unsigned long DOM_VK_PLAY           = 0xFA;
  const unsigned long DOM_VK_ZOOM           = 0xFB;
  const unsigned long DOM_VK_PA1            = 0xFD;

  // OEM specific virtual keyCode of Windows should pass through DOM keyCode
  // for compatibility with the other web browsers on Windows.
  const unsigned long DOM_VK_WIN_OEM_CLEAR  = 0xFE;

  undefined initKeyEvent(DOMString type,
                    optional boolean canBubble = false,
                    optional boolean cancelable = false,
                    optional Window? view = null,
                    optional boolean ctrlKey = false,
                    optional boolean altKey = false,
                    optional boolean shiftKey = false,
                    optional boolean metaKey = false,
                    optional unsigned long keyCode = 0,
                    optional unsigned long charCode = 0);
};
