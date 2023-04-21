use std::ffi::OsStr;

pub(crate) trait OsStrExt: private::Sealed {
    /// Converts to a string slice.
    fn try_str(&self) -> Result<&str, std::str::Utf8Error>;
    /// Returns `true` if the given pattern matches a sub-slice of
    /// this string slice.
    ///
    /// Returns `false` if it does not.
    fn contains(&self, needle: &str) -> bool;
    /// Returns the byte index of the first character of this string slice that
    /// matches the pattern.
    ///
    /// Returns [`None`] if the pattern doesn't match.
    fn find(&self, needle: &str) -> Option<usize>;
    /// Returns a string slice with the prefix removed.
    ///
    /// If the string starts with the pattern `prefix`, returns substring after the prefix, wrapped
    /// in `Some`.
    ///
    /// If the string does not start with `prefix`, returns `None`.
    fn strip_prefix(&self, prefix: &str) -> Option<&OsStr>;
    /// Returns `true` if the given pattern matches a prefix of this
    /// string slice.
    ///
    /// Returns `false` if it does not.
    fn starts_with(&self, prefix: &str) -> bool;
    /// An iterator over substrings of this string slice, separated by
    /// characters matched by a pattern.
    fn split<'s, 'n>(&'s self, needle: &'n str) -> Split<'s, 'n>;
    /// Splits the string on the first occurrence of the specified delimiter and
    /// returns prefix before delimiter and suffix after delimiter.
    fn split_once(&self, needle: &'_ str) -> Option<(&OsStr, &OsStr)>;
}

impl OsStrExt for OsStr {
    fn try_str(&self) -> Result<&str, std::str::Utf8Error> {
        let bytes = to_bytes(self);
        std::str::from_utf8(bytes)
    }

    fn contains(&self, needle: &str) -> bool {
        self.find(needle).is_some()
    }

    fn find(&self, needle: &str) -> Option<usize> {
        let bytes = to_bytes(self);
        (0..=self.len().checked_sub(needle.len())?)
            .find(|&x| bytes[x..].starts_with(needle.as_bytes()))
    }

    fn strip_prefix(&self, prefix: &str) -> Option<&OsStr> {
        let bytes = to_bytes(self);
        bytes.strip_prefix(prefix.as_bytes()).map(|s| {
            // SAFETY:
            // - This came from `to_bytes`
            // - Since `prefix` is `&str`, any split will be along UTF-8 boundarie
            unsafe { to_os_str_unchecked(s) }
        })
    }
    fn starts_with(&self, prefix: &str) -> bool {
        let bytes = to_bytes(self);
        bytes.starts_with(prefix.as_bytes())
    }

    fn split<'s, 'n>(&'s self, needle: &'n str) -> Split<'s, 'n> {
        assert_ne!(needle, "");
        Split {
            haystack: Some(self),
            needle,
        }
    }

    fn split_once(&self, needle: &'_ str) -> Option<(&OsStr, &OsStr)> {
        let start = self.find(needle)?;
        let end = start + needle.len();
        let haystack = to_bytes(self);
        let first = &haystack[0..start];
        let second = &haystack[end..];
        // SAFETY:
        // - This came from `to_bytes`
        // - Since `needle` is `&str`, any split will be along UTF-8 boundarie
        unsafe { Some((to_os_str_unchecked(first), to_os_str_unchecked(second))) }
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for std::ffi::OsStr {}
}

/// Allow access to raw bytes
///
/// As the non-UTF8 encoding is not defined, the bytes only make sense when compared with
/// 7-bit ASCII or `&str`
///
/// # Compatibility
///
/// There is no guarantee how non-UTF8 bytes will be encoded, even within versions of this crate
/// (since its dependent on rustc)
fn to_bytes(s: &OsStr) -> &[u8] {
    // SAFETY:
    // - Lifetimes are the same
    // - Types are compatible (`OsStr` is effectively a transparent wrapper for `[u8]`)
    // - The primary contract is that the encoding for invalid surrogate code points is not
    //   guaranteed which isn't a problem here
    //
    // There is a proposal to support this natively (https://github.com/rust-lang/rust/pull/95290)
    // but its in limbo
    unsafe { std::mem::transmute(s) }
}

/// Restore raw bytes as `OsStr`
///
/// # Safety
///
/// - `&[u8]` must either by a `&str` or originated with `to_bytes` within the same binary
/// - Any splits of the original `&[u8]` must be done along UTF-8 boundaries
unsafe fn to_os_str_unchecked(s: &[u8]) -> &OsStr {
    // SAFETY:
    // - Lifetimes are the same
    // - Types are compatible (`OsStr` is effectively a transparent wrapper for `[u8]`)
    // - The primary contract is that the encoding for invalid surrogate code points is not
    //   guaranteed which isn't a problem here
    //
    // There is a proposal to support this natively (https://github.com/rust-lang/rust/pull/95290)
    // but its in limbo
    std::mem::transmute(s)
}

pub struct Split<'s, 'n> {
    haystack: Option<&'s OsStr>,
    needle: &'n str,
}

impl<'s, 'n> Iterator for Split<'s, 'n> {
    type Item = &'s OsStr;

    fn next(&mut self) -> Option<Self::Item> {
        let haystack = self.haystack?;
        match haystack.split_once(self.needle) {
            Some((first, second)) => {
                if !haystack.is_empty() {
                    debug_assert_ne!(haystack, second);
                }
                self.haystack = Some(second);
                Some(first)
            }
            None => {
                self.haystack = None;
                Some(haystack)
            }
        }
    }
}

/// Split an `OsStr`
///
/// # Safety
///
/// `index` must be at a valid UTF-8 boundary
pub(crate) unsafe fn split_at(os: &OsStr, index: usize) -> (&OsStr, &OsStr) {
    let bytes = to_bytes(os);
    let (first, second) = bytes.split_at(index);
    (to_os_str_unchecked(first), to_os_str_unchecked(second))
}
