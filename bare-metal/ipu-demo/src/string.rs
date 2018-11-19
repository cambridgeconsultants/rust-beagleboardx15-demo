//! # Fixed-length Strings
//!
//! Copyright (c) 2018, Cambridge Consultants Ltd.
//! See the top-level README.md for licence details.
//!
//! This module is for wrapping strings into fixed-length 32-byte buffers, as
//! used by Remote Proc Resource Tables.

// ****************************************************************************
//
// Crates
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Sub-modules
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Macros
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Types / Traits
//
// ****************************************************************************

/// Represents a UTF-8 string in a fixed-length 32-byte buffer.
pub struct String32 {
    pub buffer: [u8; BUFFER_LEN],
}

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

pub const BUFFER_LEN: usize = 32;

// ****************************************************************************
//
// Private Types / Traits
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Private Data
//
// ****************************************************************************

// None

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

impl<'a> ::core::convert::From<&'a str> for String32 {
    /// Converts an `&str` to a fixed 32-byte buffer. If the given string is
    /// too long, it is truncated. The buffer is null padded, not null
    /// terminated.
    fn from(src: &str) -> String32 {
        let mut string = String32 {
            buffer: [0u8; BUFFER_LEN],
        };
        for (d, s) in string.buffer.iter_mut().zip(src.bytes()) {
            *d = s;
        }
        string
    }
}

impl ::core::fmt::Debug for String32 {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        let mut len = BUFFER_LEN;
        // Check for null termination
        for (idx, ch) in self.buffer.iter().enumerate() {
            if *ch == 0u8 {
                len = idx;
                break;
            }
        }
        let s = unsafe { ::core::str::from_utf8_unchecked(&self.buffer[0..len]) };
        write!(fmt, "\"{}\"", s)
    }
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************

// None

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
