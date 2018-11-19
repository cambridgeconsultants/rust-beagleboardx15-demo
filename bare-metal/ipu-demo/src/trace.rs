//! # Trace Buffers
//!
//! Copyright (c) 2018, Cambridge Consultants Ltd.
//! See the top-level README.md for licence details.
//!
//! This module is for emitting text to
//! `/sys/kernel/debug/remoteproc/remoteproc0/trace`.

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

/// Represents our tracebuffer. Uses a shared mutable buffer,
/// so only one of these can exist at any one time.
pub struct Trace<'a> {
    out_idx: usize,
    buffer: &'a mut [u8; 16384],
}

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

// None

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

/// Our output text buffer we share with the kernel. Must must must be linked
/// at address 0x9F000000 and the size and address must match that given in
/// the `rt::Trace` part of `main::RESOURCE_TABLE`.
#[link_section = ".tracebuffer"]
static mut TRACE_BUFFER: [u8; 16384] = [0u8; 16384];

// ****************************************************************************
//
// Public Functions
//
// ****************************************************************************

/// The first time you call this you'll get Some(t), where t
/// can be passed to `writeln!` and friends. The second
/// time you'll get None, so only call it once!
pub fn get_trace() -> Option<&'static mut Trace<'static>> {
    singleton!(: Trace = Trace {
        out_idx: 0,
        buffer: unsafe { &mut TRACE_BUFFER },
    })
}

/// Only call this from a panic handler.
pub unsafe fn steal_trace() -> Trace<'static> {
    let mut used_space = 0_usize;
    for (idx, ch) in TRACE_BUFFER.iter().enumerate() {
        if *ch == 0 {
            used_space = idx;
            break;
        }
    }
    Trace {
        out_idx: used_space,
        buffer: &mut TRACE_BUFFER,
    }
}

impl<'a> ::core::fmt::Write for Trace<'a> {
    fn write_str(&mut self, s: &str) -> Result<(), ::core::fmt::Error> {
        // Can never fit (with the null), so return an error.
        if (s.len() + 1) > self.buffer.len() {
            return Err(::core::fmt::Error);
        }

        let space = self.buffer.len() - self.out_idx;

        // Doesn't fit (with the null), let's wrap to make us some more space.
        if (s.len() + 1) > space {
            self.out_idx = 0;
        }

        for (s, d) in s
            .bytes()
            .zip(self.buffer[self.out_idx..self.out_idx + s.len()].iter_mut())
        {
            *d = s;
        }
        self.out_idx += s.len();
        self.buffer[self.out_idx] = 0;
        Ok(())
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
