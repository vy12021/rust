// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A Unicode scalar value
//!
//! This module provides the `CharExt` trait, as well as its
//! implementation for the primitive `char` type, in order to allow
//! basic character manipulation.
//!
//! A `char` represents a
//! *[Unicode scalar
//! value](http://www.unicode.org/glossary/#unicode_scalar_value)*, as it can
//! contain any Unicode code point except high-surrogate and low-surrogate code
//! points.
//!
//! As such, only values in the ranges \[0x0,0xD7FF\] and \[0xE000,0x10FFFF\]
//! (inclusive) are allowed. A `char` can always be safely cast to a `u32`;
//! however the converse is not always true due to the above range limits
//! and, as such, should be performed via the `from_u32` function.
//!
//! *[See also the `char` primitive type](../primitive.char.html).*

#![stable(feature = "rust1", since = "1.0.0")]

use core::char::CharExt as C;
use core::option::Option::{self, Some, None};
use core::iter::Iterator;
use tables::{derived_property, property, general_category, conversions};

// stable reexports
pub use core::char::{MAX, from_u32, from_u32_unchecked, from_digit, EscapeUnicode, EscapeDefault};

// unstable reexports
pub use tables::UNICODE_VERSION;

/// An iterator over the lowercase mapping of a given character, returned from
/// the [`to_lowercase` method](../primitive.char.html#method.to_lowercase) on
/// characters.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct ToLowercase(CaseMappingIter);

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for ToLowercase {
    type Item = char;
    fn next(&mut self) -> Option<char> { self.0.next() }
}

/// An iterator over the uppercase mapping of a given character, returned from
/// the [`to_uppercase` method](../primitive.char.html#method.to_uppercase) on
/// characters.
#[stable(feature = "rust1", since = "1.0.0")]
pub struct ToUppercase(CaseMappingIter);

#[stable(feature = "rust1", since = "1.0.0")]
impl Iterator for ToUppercase {
    type Item = char;
    fn next(&mut self) -> Option<char> { self.0.next() }
}


enum CaseMappingIter {
    Three(char, char, char),
    Two(char, char),
    One(char),
    Zero
}

impl CaseMappingIter {
    fn new(chars: [char; 3]) -> CaseMappingIter {
        if chars[2] == '\0' {
            if chars[1] == '\0' {
                CaseMappingIter::One(chars[0])  // Including if chars[0] == '\0'
            } else {
                CaseMappingIter::Two(chars[0], chars[1])
            }
        } else {
            CaseMappingIter::Three(chars[0], chars[1], chars[2])
        }
    }
}

impl Iterator for CaseMappingIter {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        match *self {
            CaseMappingIter::Three(a, b, c) => {
                *self = CaseMappingIter::Two(b, c);
                Some(a)
            }
            CaseMappingIter::Two(b, c) => {
                *self = CaseMappingIter::One(c);
                Some(b)
            }
            CaseMappingIter::One(c) => {
                *self = CaseMappingIter::Zero;
                Some(c)
            }
            CaseMappingIter::Zero => None,
        }
    }
}

#[stable(feature = "rust1", since = "1.0.0")]
#[lang = "char"]
impl char {
    /// Checks if a `char` parses as a numeric digit in the given radix.
    ///
    /// Compared to `is_numeric()`, this function only recognizes the characters
    /// `0-9`, `a-z` and `A-Z`.
    ///
    /// # Return value
    ///
    /// Returns `true` if `c` is a valid digit under `radix`, and `false`
    /// otherwise.
    ///
    /// # Panics
    ///
    /// Panics if given a radix > 36.
    ///
    /// # Examples
    ///
    /// ```
    /// let c = '1';
    ///
    /// assert!(c.is_digit(10));
    ///
    /// assert!('f'.is_digit(16));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_digit(self, radix: u32) -> bool { C::is_digit(self, radix) }

    /// Converts a character to the corresponding digit.
    ///
    /// # Return value
    ///
    /// If `c` is between '0' and '9', the corresponding value between 0 and
    /// 9. If `c` is 'a' or 'A', 10. If `c` is 'b' or 'B', 11, etc. Returns
    /// none if the character does not refer to a digit in the given radix.
    ///
    /// # Panics
    ///
    /// Panics if given a radix outside the range [0..36].
    ///
    /// # Examples
    ///
    /// ```
    /// let c = '1';
    ///
    /// assert_eq!(c.to_digit(10), Some(1));
    ///
    /// assert_eq!('f'.to_digit(16), Some(15));
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_digit(self, radix: u32) -> Option<u32> { C::to_digit(self, radix) }

    /// Returns an iterator that yields the hexadecimal Unicode escape of a
    /// character, as `char`s.
    ///
    /// All characters are escaped with Rust syntax of the form `\\u{NNNN}`
    /// where `NNNN` is the shortest hexadecimal representation of the code
    /// point.
    ///
    /// # Examples
    ///
    /// ```
    /// for c in '❤'.escape_unicode() {
    ///     print!("{}", c);
    /// }
    /// println!("");
    /// ```
    ///
    /// This prints:
    ///
    /// ```text
    /// \u{2764}
    /// ```
    ///
    /// Collecting into a `String`:
    ///
    /// ```
    /// let heart: String = '❤'.escape_unicode().collect();
    ///
    /// assert_eq!(heart, r"\u{2764}");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn escape_unicode(self) -> EscapeUnicode { C::escape_unicode(self) }

    /// Returns an iterator that yields the 'default' ASCII and
    /// C++11-like literal escape of a character, as `char`s.
    ///
    /// The default is chosen with a bias toward producing literals that are
    /// legal in a variety of languages, including C++11 and similar C-family
    /// languages. The exact rules are:
    ///
    /// * Tab, CR and LF are escaped as '\t', '\r' and '\n' respectively.
    /// * Single-quote, double-quote and backslash chars are backslash-
    ///   escaped.
    /// * Any other chars in the range [0x20,0x7e] are not escaped.
    /// * Any other chars are given hex Unicode escapes; see `escape_unicode`.
    ///
    /// # Examples
    ///
    /// ```
    /// for i in '"'.escape_default() {
    ///     println!("{}", i);
    /// }
    /// ```
    ///
    /// This prints:
    ///
    /// ```text
    /// \
    /// "
    /// ```
    ///
    /// Collecting into a `String`:
    ///
    /// ```
    /// let quote: String = '"'.escape_default().collect();
    ///
    /// assert_eq!(quote, "\\\"");
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn escape_default(self) -> EscapeDefault { C::escape_default(self) }

    /// Returns the number of bytes this character would need if encoded in
    /// UTF-8.
    ///
    /// # Examples
    ///
    /// ```
    /// let n = 'ß'.len_utf8();
    ///
    /// assert_eq!(n, 2);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn len_utf8(self) -> usize { C::len_utf8(self) }

    /// Returns the number of 16-bit code units this character would need if
    /// encoded in UTF-16.
    ///
    /// # Examples
    ///
    /// ```
    /// let n = 'ß'.len_utf16();
    ///
    /// assert_eq!(n, 1);
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn len_utf16(self) -> usize { C::len_utf16(self) }

    /// Encodes this character as UTF-8 into the provided byte buffer, and then
    /// returns the number of bytes written.
    ///
    /// If the buffer is not large enough, nothing will be written into it and a
    /// `None` will be returned. A buffer of length four is large enough to
    /// encode any `char`.
    ///
    /// # Examples
    ///
    /// In both of these examples, 'ß' takes two bytes to encode.
    ///
    /// ```
    /// #![feature(unicode)]
    ///
    /// let mut b = [0; 2];
    ///
    /// let result = 'ß'.encode_utf8(&mut b);
    ///
    /// assert_eq!(result, Some(2));
    /// ```
    ///
    /// A buffer that's too small:
    ///
    /// ```
    /// #![feature(unicode)]
    ///
    /// let mut b = [0; 1];
    ///
    /// let result = 'ß'.encode_utf8(&mut b);
    ///
    /// assert_eq!(result, None);
    /// ```
    #[unstable(feature = "unicode",
               reason = "pending decision about Iterator/Writer/Reader")]
    #[inline]
    pub fn encode_utf8(self, dst: &mut [u8]) -> Option<usize> {
        C::encode_utf8(self, dst)
    }

    /// Encodes this character as UTF-16 into the provided `u16` buffer, and
    /// then returns the number of `u16`s written.
    ///
    /// If the buffer is not large enough, nothing will be written into it and a
    /// `None` will be returned. A buffer of length 2 is large enough to encode
    /// any `char`.
    ///
    /// # Examples
    ///
    /// In both of these examples, 'ß' takes one `u16` to encode.
    ///
    /// ```
    /// #![feature(unicode)]
    ///
    /// let mut b = [0; 1];
    ///
    /// let result = 'ß'.encode_utf16(&mut b);
    ///
    /// assert_eq!(result, Some(1));
    /// ```
    ///
    /// A buffer that's too small:
    ///
    /// ```
    /// #![feature(unicode)]
    ///
    /// let mut b = [0; 0];
    ///
    /// let result = 'ß'.encode_utf8(&mut b);
    ///
    /// assert_eq!(result, None);
    /// ```
    #[unstable(feature = "unicode",
               reason = "pending decision about Iterator/Writer/Reader")]
    #[inline]
    pub fn encode_utf16(self, dst: &mut [u16]) -> Option<usize> {
        C::encode_utf16(self, dst)
    }

    /// Returns whether the specified character is considered a Unicode
    /// alphabetic code point.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_alphabetic(self) -> bool {
        match self {
            'a' ... 'z' | 'A' ... 'Z' => true,
            c if c > '\x7f' => derived_property::Alphabetic(c),
            _ => false
        }
    }

    /// Returns whether the specified character satisfies the 'XID_Start'
    /// Unicode property.
    ///
    /// 'XID_Start' is a Unicode Derived Property specified in
    /// [UAX #31](http://unicode.org/reports/tr31/#NFKC_Modifications),
    /// mostly similar to ID_Start but modified for closure under NFKx.
    #[unstable(feature = "unicode",
               reason = "mainly needed for compiler internals")]
    #[inline]
    pub fn is_xid_start(self) -> bool { derived_property::XID_Start(self) }

    /// Returns whether the specified `char` satisfies the 'XID_Continue'
    /// Unicode property.
    ///
    /// 'XID_Continue' is a Unicode Derived Property specified in
    /// [UAX #31](http://unicode.org/reports/tr31/#NFKC_Modifications),
    /// mostly similar to 'ID_Continue' but modified for closure under NFKx.
    #[unstable(feature = "unicode",
               reason = "mainly needed for compiler internals")]
    #[inline]
    pub fn is_xid_continue(self) -> bool { derived_property::XID_Continue(self) }

    /// Indicates whether a character is in lowercase.
    ///
    /// This is defined according to the terms of the Unicode Derived Core
    /// Property `Lowercase`.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_lowercase(self) -> bool {
        match self {
            'a' ... 'z' => true,
            c if c > '\x7f' => derived_property::Lowercase(c),
            _ => false
        }
    }

    /// Indicates whether a character is in uppercase.
    ///
    /// This is defined according to the terms of the Unicode Derived Core
    /// Property `Uppercase`.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_uppercase(self) -> bool {
        match self {
            'A' ... 'Z' => true,
            c if c > '\x7f' => derived_property::Uppercase(c),
            _ => false
        }
    }

    /// Indicates whether a character is whitespace.
    ///
    /// Whitespace is defined in terms of the Unicode Property `White_Space`.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_whitespace(self) -> bool {
        match self {
            ' ' | '\x09' ... '\x0d' => true,
            c if c > '\x7f' => property::White_Space(c),
            _ => false
        }
    }

    /// Indicates whether a character is alphanumeric.
    ///
    /// Alphanumericness is defined in terms of the Unicode General Categories
    /// 'Nd', 'Nl', 'No' and the Derived Core Property 'Alphabetic'.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_alphanumeric(self) -> bool {
        self.is_alphabetic() || self.is_numeric()
    }

    /// Indicates whether a character is a control code point.
    ///
    /// Control code points are defined in terms of the Unicode General
    /// Category `Cc`.
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_control(self) -> bool { general_category::Cc(self) }

    /// Indicates whether the character is numeric (Nd, Nl, or No).
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn is_numeric(self) -> bool {
        match self {
            '0' ... '9' => true,
            c if c > '\x7f' => general_category::N(c),
            _ => false
        }
    }

    /// Converts a character to its lowercase equivalent.
    ///
    /// This performs complex unconditional mappings with no tailoring.
    /// See `to_uppercase()` for references and more information.
    ///
    /// # Return value
    ///
    /// Returns an iterator which yields the characters corresponding to the
    /// lowercase equivalent of the character. If no conversion is possible then
    /// an iterator with just the input character is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(Some('c'), 'C'.to_lowercase().next());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_lowercase(self) -> ToLowercase {
        ToLowercase(CaseMappingIter::new(conversions::to_lower(self)))
    }

    /// Converts a character to its uppercase equivalent.
    ///
    /// This performs complex unconditional mappings with no tailoring:
    /// it maps one Unicode character to its uppercase equivalent
    /// according to the Unicode database [1]
    /// and the additional complex mappings [`SpecialCasing.txt`].
    /// Conditional mappings (based on context or language) are not considerd here.
    ///
    /// A full reference can be found here [2].
    ///
    /// # Return value
    ///
    /// Returns an iterator which yields the characters corresponding to the
    /// uppercase equivalent of the character. If no conversion is possible then
    /// an iterator with just the input character is returned.
    ///
    /// [1]: ftp://ftp.unicode.org/Public/UNIDATA/UnicodeData.txt
    ///
    /// [`SpecialCasing.txt`]: ftp://ftp.unicode.org/Public/UNIDATA/SpecialCasing.txt
    ///
    /// [2]: http://www.unicode.org/versions/Unicode7.0.0/ch03.pdf#G33992
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(Some('C'), 'c'.to_uppercase().next());
    /// ```
    #[stable(feature = "rust1", since = "1.0.0")]
    #[inline]
    pub fn to_uppercase(self) -> ToUppercase {
        ToUppercase(CaseMappingIter::new(conversions::to_upper(self)))
    }
}
