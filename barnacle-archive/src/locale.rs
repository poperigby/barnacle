// Copyright (C) 2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::ptr;

use libc::{LC_CTYPE_MASK, freelocale, locale_t, newlocale, uselocale};

/// Change from the C to system locale, allowing `libarchive` to handle filenames
/// in UTF-8. We restrict to change LC_CTYPE only, since `libarchive` only needs
/// the charset set. The timing of locale setting for Unix and Windows is
/// different, handle them separately.
///
/// See on `libarchive` website for a more complete description of the issue:
///
///   https://github.com/libarchive/libarchive/issues/587
///   https://github.com/libarchive/libarchive/wiki/Filenames
pub(crate) struct Utf8LocaleGuard {
    save: locale_t,
    utf8_locale: locale_t,
}

impl Utf8LocaleGuard {
    pub(crate) fn new() -> Self {
        let locale = c"C.UTF-8";

        let utf8_locale = unsafe { newlocale(LC_CTYPE_MASK, locale.as_ptr(), ptr::null_mut()) };

        let save = if utf8_locale.is_null() {
            ptr::null_mut()
        } else {
            unsafe { uselocale(utf8_locale) }
        };

        Self { save, utf8_locale }
    }
}

impl Drop for Utf8LocaleGuard {
    fn drop(&mut self) {
        unsafe {
            if !self.utf8_locale.is_null() {
                uselocale(self.save);
                freelocale(self.utf8_locale);
            }
        };
    }
}
