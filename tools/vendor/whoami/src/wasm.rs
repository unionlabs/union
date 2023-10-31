// WhoAmI
// Copyright © 2017-2021 Jeron Aldaron Lau.
//
// Licensed under any of:
//  - Apache License, Version 2.0 (https://www.apache.org/licenses/LICENSE-2.0)
//  - MIT License (https://mit-license.org/)
//  - Boost Software License, Version 1.0 (https://www.boost.org/LICENSE_1_0.txt)
// At your choosing (See accompanying files LICENSE_APACHE_2_0.txt,
// LICENSE_MIT.txt and LICENSE_BOOST_1_0.txt).

use std::ffi::OsString;

use wasm_bindgen::JsValue;
use web_sys::window;

use crate::{DesktopEnv, Platform};

// Get the user agent
fn user_agent() -> Option<String> {
    window()?.navigator().user_agent().ok()
}

// Get the document domain
fn document_domain() -> Option<String> {
    window()?.document()?.location()?.hostname().ok()
}

struct LangIter {
    array: Vec<JsValue>,
    index: usize,
}

impl Iterator for LangIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(value) = self.array.get(self.index) {
            self.index += 1;
            if let Some(lang) = value.as_string() {
                Some(lang)
            } else {
                self.next()
            }
        } else {
            None
        }
    }
}

#[inline(always)]
pub fn lang() -> impl Iterator<Item = String> {
    let array = if let Some(window) = window() {
        window.navigator().languages().to_vec()
    } else {
        Vec::new()
    };
    let index = 0;

    LangIter { array, index }
}

#[inline(always)]
pub fn username_os() -> OsString {
    username().into()
}

#[inline(always)]
pub fn realname_os() -> OsString {
    realname().into()
}

#[inline(always)]
pub fn devicename_os() -> OsString {
    devicename().into()
}

#[inline(always)]
pub fn hostname_os() -> OsString {
    hostname().into()
}

#[inline(always)]
pub fn distro_os() -> Option<OsString> {
    distro().map(|a| a.into())
}

#[inline(always)]
pub fn username() -> String {
    "anonymous".to_string()
}

#[inline(always)]
pub fn realname() -> String {
    "Anonymous".to_string()
}

pub fn devicename() -> String {
    let orig_string = user_agent().unwrap_or_else(String::new);

    let start = if let Some(s) = orig_string.rfind(' ') {
        s
    } else {
        return "Unknown Browser".to_string();
    };

    let string = orig_string
        .get(start + 1..)
        .unwrap_or("Unknown Browser")
        .replace('/', " ");

    if string == "Safari" {
        if orig_string.contains("Chrome") {
            "Chrome".to_string()
        } else {
            "Safari".to_string()
        }
    } else {
        string
    }
}

#[inline(always)]
pub fn hostname() -> String {
    document_domain()
        .filter(|x| !x.is_empty())
        .unwrap_or_else(|| "localhost".to_string())
}

pub fn distro() -> Option<String> {
    let string = user_agent()?;

    let begin = string.find('(')?;
    let end = string.find(')')?;
    let string = &string[begin + 1..end];

    if string.contains("Win32") || string.contains("Win64") {
        let begin = if let Some(b) = string.find("NT") {
            b
        } else {
            return Some("Windows".to_string());
        };
        let end = if let Some(e) = string.find('.') {
            e
        } else {
            return Some("Windows".to_string());
        };
        let string = &string[begin + 3..end];

        Some(format!("Windows {}", string))
    } else if string.contains("Linux") {
        let string = if string.contains("X11") || string.contains("Wayland") {
            let begin = if let Some(b) = string.find(';') {
                b
            } else {
                return Some("Unknown Linux".to_string());
            };
            &string[begin + 2..]
        } else {
            string
        };

        if string.starts_with("Linux") {
            Some("Unknown Linux".to_string())
        } else {
            let end = if let Some(e) = string.find(';') {
                e
            } else {
                return Some("Unknown Linux".to_string());
            };
            Some(string[..end].to_string())
        }
    } else if let Some(begin) = string.find("Mac OS X") {
        Some(if let Some(end) = string[begin..].find(';') {
            string[begin..begin + end].to_string()
        } else {
            string[begin..].to_string().replace("_", ".")
        })
    } else {
        // TODO:
        // Platform::FreeBsd,
        // Platform::Ios,
        // Platform::Android,
        // Platform::Nintendo,
        // Platform::Xbox,
        // Platform::PlayStation,
        // Platform::Dive,
        // Platform::Fuchsia,
        // Platform::Redox
        Some(string.to_string())
    }
}

pub const fn desktop_env() -> DesktopEnv {
    DesktopEnv::WebBrowser
}

pub fn platform() -> Platform {
    let string = user_agent().unwrap_or_else(String::new);

    let begin = if let Some(b) = string.find('(') {
        b
    } else {
        return Platform::Unknown("Unknown".to_string());
    };
    let end = if let Some(e) = string.find(')') {
        e
    } else {
        return Platform::Unknown("Unknown".to_string());
    };
    let string = &string[begin + 1..end];

    if string.contains("Win32") || string.contains("Win64") {
        Platform::Windows
    } else if string.contains("Linux") {
        Platform::Linux
    } else if string.contains("Mac OS X") {
        Platform::MacOS
    } else {
        // TODO:
        // Platform::FreeBsd,
        // Platform::Ios,
        // Platform::Android,
        // Platform::Nintendo,
        // Platform::Xbox,
        // Platform::PlayStation,
        // Platform::Dive,
        // Platform::Fuchsia,
        // Platform::Redox,
        Platform::Unknown(string.to_string())
    }
}
