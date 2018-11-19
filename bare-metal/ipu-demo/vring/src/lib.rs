//! # vring - VirtIO support for P3642 (I) Cerberust
//!
//! Copyright (c) 2018, Cambridge Consultants Ltd.
//! See the top-level README.md for licence details.
//!
//! Implements the Linux kernel vring interface
//! See http://docs.oasis-open.org/virtio/virtio/v1.0/virtio-v1.0.pdf

// ****************************************************************************
//
// Crates
//
// ****************************************************************************

#![cfg_attr(not(test), no_std)]

// ****************************************************************************
//
// Imports
//
// ****************************************************************************

#[cfg(test)]
use std as core;

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

/// Represents a Host view of a Vring. Holds no data itself, but instead points to an area
/// of statically allocated RAM.
pub struct HostVring {
    descriptors: &'static mut DescriptorRing,
    head: Option<usize>,
    available: &'static mut AvailableRing,
    used: &'static mut UsedRing,
    entries: usize,
    last_seen_used: u16,
    addr_map: &'static Fn(u64) -> u64
}

/// Represents a Guest view of a Vring. Holds no data itself, but instead points to an area
/// of statically allocated RAM.
pub struct GuestVring {
    descriptors: &'static mut DescriptorRing,
    available: &'static mut AvailableRing,
    used: &'static mut UsedRing,
    entries: usize,
    last_seen_available: u16,
    addr_map: &'static Fn(u64) -> u64
}

/// A ring of buffers. Indexes to these buffer descriptors are placed in the
/// other two rings.
#[repr(C)]
pub struct DescriptorRing {
    /// The ring of all entries. We put a single entry in the type and use
    /// unsafe code to access the run-time sized array behind it.
    pub ring: DescriptorEntry,
}

/// Describes an entry in the vring.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DescriptorEntry {
    /// Physical address of this buffer
    addr: u64,
    /// Length of this buffer.
    len: u32,
    /// Flags for this buffer.
    pub flags: DescriptorFlags,
    /// Only valid if `flags.is_set(DescriptorFlag::Next)`
    pub next: u16,
}

/// Bitmask of flags set on a 'DescriptorEntry`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DescriptorFlags(u16);

/// The individual flags set on a 'DescriptorEntry`
#[derive(Debug, Clone, Copy)]
pub enum DescriptorFlag {
    /// Marks a buffer as continuing via the `next` field.
    Next = 1,
    /// Marks a buffer as device write-only (else device read-only).
    Write = 2,
    /// This buffer contains a list of buffer descriptors.
    Indirect = 4,
}

/// A ring of 'available' entries. The host driver uses the available ring to
/// offer buffers to the device. Each ring entry refers to the head of a
/// descriptor chain. The driver writes to a buffer and puts it on this
/// available ring. The device then reads/writes the buffer and puts it on the
/// used ring.
#[repr(C)]
pub struct AvailableRing {
    /// Flags for this ring.
    pub flags: AvailableFlags,
    /// Where in this ring the host should put the next available buffer.
    pub idx: u16,
    /// The ring of available entries. We put a single entry in the type and use
    /// unsafe code to access the run-time sized array behind it.
    pub ring: AvailableEntry,
}

/// An entry in the available ring.
/// Points to a descriptor.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AvailableEntry {
    /// Index
    pub idx: u16,
}

/// Bitmask of flags set on the `AvailableRing`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AvailableFlags(u16);

/// The individual flags set on the `AvailableRing`.
#[derive(Debug, Clone, Copy)]
pub enum AvailableFlag {
    // Tells the device - do not interrupt me when you consume this
    NoInterrupt = 1,
}

/// A ring of 'used' entries. The host driver uses the used ring to
/// receive buffers from the device. Each ring entry refers to the head of a
/// descriptor chain. The driver writes to a buffer and puts it on the
/// available ring. The device then reads/writes the buffer and puts it on this
/// used ring.
#[repr(C)]
pub struct UsedRing {
    /// Flags for this ring.
    pub flags: UsedFlags,
    /// Where in this ring the device should put the next used buffer.
    pub idx: u16,
    /// The ring of used entries. We put a single entry in the type and use
    /// unsafe code to access the run-time sized array behind it.
    pub ring: UsedEntry,
}

/// An entry in the 'used' ring. Points to a descriptor.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UsedEntry {
    /// Index of start of chain
    pub idx: u32,
    /// Total length of chain
    pub len: u32,
}

/// Bitmask of flags set on the `UsedRing`
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct UsedFlags(u16);

/// Individual flags set on the `UsedRing`
#[derive(Debug, Clone, Copy)]
pub enum UsedFlag {
    // Tells the host - do not kick me when you make a buffer available
    NoNotify = 1,
}

/// Errors that can occur
#[derive(Debug, Clone, Copy)]
pub enum Error {
    OutOfMemory,
    NoData,
    InternalError,
    PayloadTooLarge
}

// ****************************************************************************
//
// Public Data
//
// ****************************************************************************

pub const VIRTIO_CONFIG_S_ACKNOWLEDGE: u8 =  1;
pub const VIRTIO_CONFIG_S_DRIVER: u8 =  2;
pub const VIRTIO_CONFIG_S_DRIVER_OK: u8 =  4;

// Virtio Ids: keep in sync with the linux "include/linux/virtio_ids.h"

/// virtio console
pub const VIRTIO_ID_CONSOLE: u32 = 3;

/// virtio remote processor messaging
pub const VIRTIO_ID_RPMSG: u32 = 7;

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
// Public Functions / Impl
//
// ****************************************************************************

impl HostVring {
    /// Creates a new `Vring` from an address. Unsafe because you need to
    /// ensure the address actually points at a valid vring structure from a
    /// resource table.
    ///
    /// Note there is assumed to be some alignment padding between
    /// the available ring and the used ring based on the value of `align`.
    ///
    /// This currently assumes that the descriptors array is pre-filled with
    /// buffers to be used. This is probably incorrect - we should add an API
    /// for allocating new buffers.
    ///
    /// We also need to support chaining multiple buffers.
    pub unsafe fn new<F>(addr: usize, entries: usize, align: usize, addr_map: &'static F) -> HostVring
    where
        F: Fn(u64) -> u64
    {
        let available_addr = addr + (16 * entries);
        let available_end = available_addr + 6 + (2 * entries);
        let used_addr = align_address(available_end, align);
        HostVring {
            descriptors: &mut *(addr as *mut DescriptorRing),
            available: &mut *(available_addr as *mut AvailableRing),
            used: &mut *(used_addr as *mut UsedRing),
            entries,
            head: Some(0),
            last_seen_used: 0,
            addr_map,
        }
    }

    /// Pop a descriptor off the linked list and make it available to the guest.
    /// TODO: Add support for pulling off multiple linked descriptors.
    pub fn give_to_guest<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnOnce(&mut DescriptorEntry),
    {
        match self.head {
            Some(head) => {
                if head < self.entries {
                    let descriptor_table: *mut DescriptorEntry =
                        &mut self.descriptors.ring as *mut DescriptorEntry;
                    let e = unsafe { &mut *(descriptor_table.offset(head as isize)) };
                    if e.flags.is_set(DescriptorFlag::Next) {
                        // New head of list
                        self.head = Some(e.next as usize);
                        // Disconnect this descriptor from the list
                        e.flags.clear(DescriptorFlag::Next);
                        e.next = 0;
                    } else {
                        // No more descriptors in the list
                        self.head = None;
                    }

                    let mut e_copy = *e;
                    e_copy.addr = (self.addr_map)(e_copy.addr);

                    callback(&mut e_copy);

                    e.len = e_copy.len;
                    e.flags = e_copy.flags;
                    e.next = e_copy.next;

                    // Now push e on to the available list
                    // Impossible to over-fill this list as we only have exactly enough buffers to go on it

                    let available_table: *mut AvailableEntry =
                        &mut self.available.ring as *mut AvailableEntry;
                    let slot = (self.available.idx as usize) % (self.entries as usize);
                    let available_slot = unsafe { &mut *(available_table.offset(slot as isize)) };
                    *available_slot = AvailableEntry { idx: head as u16 };

                    // Need a memory barrier here

                    // Always goes up by one, wraps at 65536
                    self.available.idx = self.available.idx.wrapping_add(1);

                    // Need a memory barrier here

                    // Need to check the notification flags here and, if set,
                    // notify the device through some mechanism.

                    Ok(())
                } else {
                    Err(Error::InternalError)
                }
            }
            None => Err(Error::OutOfMemory),
        }
    }

    /// Take an item from the used ring and put it back on the free list.
    pub fn take_from_guest<F>(&mut self, _callback: F) -> Result<(), Error>
    where
        F: FnOnce(&mut DescriptorEntry),
    {
        /// Check last_seen_used against used.idx
        unimplemented!();
    }
}

impl GuestVring {
    /// Creates a new `Vring` from an address. Unsafe because you need to
    /// ensure the address actually points at a valid vring structure from a
    /// resource table.
    ///
    /// Note there is assumed to be some alignment padding between
    /// the available ring and the used ring.
    pub unsafe fn new<F>(addr: usize, entries: usize, align: usize, addr_map: &'static F) -> GuestVring
    where
        F: Fn(u64) -> u64
    {
        let available_addr = addr + (16 * entries);
        let available_end = available_addr + 6 + (2 * entries);
        let used_addr = align_address(available_end, align);
        GuestVring {
            descriptors: &mut *(addr as *mut DescriptorRing),
            available: &mut *(available_addr as *mut AvailableRing),
            used: &mut *(used_addr as *mut UsedRing),
            entries,
            last_seen_available: 0,
            addr_map,
        }
    }

    /// Take an item from the available ring and put it back on the used ring.
    pub fn process<F>(&mut self, callback: F) -> Result<(), Error>
    where
        F: FnOnce(&DescriptorEntry),
    {
        if self.last_seen_available != self.available.idx {
            // Must have new stuff to play with
            let available_table: *mut AvailableEntry =
                &mut self.available.ring as *mut AvailableEntry;
            let slot = self.last_seen_available as usize % self.entries;
            let available_idx = unsafe { &mut *(available_table.offset(slot as isize)) };

            let descriptor_table: *mut DescriptorEntry =
                &mut self.descriptors.ring as *mut DescriptorEntry;
            let e = unsafe { &mut *(descriptor_table.offset(available_idx.idx as isize)) };


            let mut e_copy = *e;
            e_copy.addr = (self.addr_map)(e_copy.addr);

            callback(&e_copy);

            // Move to used

            let used_table: *mut UsedEntry = &mut self.used.ring as *mut UsedEntry;
            let used_slot = self.used.idx as usize % self.entries;
            let used_entry = unsafe { &mut *(used_table.offset(used_slot as isize)) };
            *used_entry = UsedEntry {
                idx: available_idx.idx as u32,
                len: e.len as u32,
            };

            self.last_seen_available = self.last_seen_available.wrapping_add(1);
            self.used.idx = self.used.idx.wrapping_add(1);

            Ok(())
        } else {
            Err(Error::NoData)
        }
    }

    pub fn transmit<P1, P2>(&mut self, payload1: &P1, payload2: &P2) -> Result<(), Error> {
        if self.last_seen_available != self.available.idx {
            // Must have new stuff to play with
            let available_table: *mut AvailableEntry =
                &mut self.available.ring as *mut AvailableEntry;
            let slot = self.last_seen_available as usize % self.entries;
            let available_idx = unsafe { &mut *(available_table.offset(slot as isize)) };

            let descriptor_table: *mut DescriptorEntry =
                &mut self.descriptors.ring as *mut DescriptorEntry;
            let e = unsafe { &mut *(descriptor_table.offset(available_idx.idx as isize)) };

            let addr = (self.addr_map)(e.addr) as *mut u8;

            let length1 = ::core::mem::size_of::<P1>();
            let length2 = ::core::mem::size_of::<P2>();
            let length = length1 + length2;

            if length > e.len as usize {
                return Err(Error::PayloadTooLarge)
            }

            unsafe {
                core::ptr::copy_nonoverlapping(payload1 as *const P1 as *const u8, addr, length1);
                core::ptr::copy_nonoverlapping(payload2 as *const P2 as *const u8, addr.offset(length1 as isize), length2);
            };

            e.len = length as u32;
            e.flags = DescriptorFlags(0);
            e.next = 0;

            // Move to used
            let used_table: *mut UsedEntry = &mut self.used.ring as *mut UsedEntry;
            let used_slot = self.used.idx as usize % self.entries;
            let used_entry = unsafe { &mut *(used_table.offset(used_slot as isize)) };
            *used_entry = UsedEntry {
                idx: available_idx.idx as u32,
                len: e.len as u32,
            };

            self.last_seen_available = self.last_seen_available.wrapping_add(1);
            self.used.idx = self.used.idx.wrapping_add(1);

            Ok(())
        } else {
            Err(Error::NoData)
        }
    }
}

/// Align a value. `alignment` must be a power of 2.
fn align_address(input: usize, alignment: usize) -> usize {
    (input + alignment - 1) & !(alignment - 1)
}

impl ::core::fmt::Debug for HostVring {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        writeln!(fmt, "HostVring {{")?;
        writeln!(
            fmt,
            "    address: 0x{:08x}",
            self.descriptors as *const _ as usize
        )?;
        writeln!(fmt, "    num_descriptors: {}", self.entries)?;
        writeln!(
            fmt,
            "    available: 0x{:08x}",
            self.available as *const _ as usize
        )?;
        writeln!(fmt, "    available_flags: {:?}", self.available.flags)?;
        writeln!(fmt, "    available_idx: 0x{:04x}", self.available.idx)?;
        writeln!(fmt, "    used: 0x{:08x}", self.used as *const _ as usize)?;
        writeln!(fmt, "    used_flags: {:?}", self.used.flags)?;
        writeln!(fmt, "    used_idx: 0x{:04x}", self.used.idx)?;
        writeln!(fmt, "    last: {}", self.last_seen_used)?;
        writeln!(fmt, "}}")?;
        Ok(())
    }
}

impl ::core::fmt::Debug for GuestVring {
    fn fmt(&self, fmt: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        writeln!(fmt, "GuestVring {{")?;
        writeln!(
            fmt,
            "    address: 0x{:08x}",
            self.descriptors as *const _ as usize
        )?;
        writeln!(fmt, "    num_descriptors: {}", self.entries)?;
        writeln!(
            fmt,
            "    available: 0x{:08x}",
            self.available as *const _ as usize
        )?;
        writeln!(fmt, "    available_flags: {:?}", self.available.flags)?;
        writeln!(fmt, "    available_idx: 0x{:04x}", self.available.idx)?;
        writeln!(fmt, "    used: 0x{:08x}", self.used as *const _ as usize)?;
        writeln!(fmt, "    used_flags: {:?}", self.used.flags)?;
        writeln!(fmt, "    used_idx: 0x{:04x}", self.used.idx)?;
        writeln!(fmt, "    last: 0x{:04x}", self.last_seen_available)?;
        writeln!(fmt, "}}")?;
        Ok(())
    }
}

impl DescriptorEntry {
    pub fn get_buffer_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.addr as *mut u8, self.len as usize) }
    }

    pub fn get_buffer(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.addr as *const u8, self.len as usize) }
    }
}

impl DescriptorFlags {
    pub fn is_set(&self, flag: DescriptorFlag) -> bool {
        self.0 & (flag as u16) != 0
    }

    pub fn is_clear(&self, flag: DescriptorFlag) -> bool {
        !self.is_set(flag)
    }

    pub fn set(&mut self, flag: DescriptorFlag) {
        self.0 |= flag as u16;
    }

    pub fn clear(&mut self, flag: DescriptorFlag) {
        self.0 &= !(flag as u16);
    }
}

impl AvailableFlags {
    pub fn is_set(&self, flag: AvailableFlag) -> bool {
        self.0 & (flag as u16) != 0
    }

    pub fn is_clear(&self, flag: AvailableFlag) -> bool {
        !self.is_set(flag)
    }

    pub fn set(&mut self, flag: AvailableFlag) {
        self.0 |= flag as u16;
    }

    pub fn clear(&mut self, flag: AvailableFlag) {
        self.0 &= !(flag as u16);
    }
}

impl UsedFlags {
    pub fn is_set(&self, flag: UsedFlag) -> bool {
        self.0 & (flag as u16) != 0
    }

    pub fn is_clear(&self, flag: UsedFlag) -> bool {
        !self.is_set(flag)
    }

    pub fn set(&mut self, flag: UsedFlag) {
        self.0 |= flag as u16;
    }

    pub fn clear(&mut self, flag: UsedFlag) {
        self.0 &= !(flag as u16);
    }
}

// ****************************************************************************
//
// Private Functions
//
// ****************************************************************************

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_align() {
        for bits in 2..30 {
            let a = 1 << bits;
            assert_eq!(align_address(0, a), 0);
            assert_eq!(align_address(1, a), a);
            assert_eq!(align_address(a - 1, a), a);
            assert_eq!(align_address(a, a), a);
            assert_eq!(align_address(a + 1, a), a * 2);
        }
    }

    #[test]
    fn test_flags() {
        let flags = DescriptorFlags(3);
        assert_eq!(flags.is_set(DescriptorFlag::Next), true);
        assert_eq!(flags.is_set(DescriptorFlag::Write), true);
        assert_eq!(flags.is_set(DescriptorFlag::Indirect), false);

        let flags = DescriptorFlags(5);
        assert_eq!(flags.is_set(DescriptorFlag::Next), true);
        assert_eq!(flags.is_set(DescriptorFlag::Write), false);
        assert_eq!(flags.is_set(DescriptorFlag::Indirect), true);

        let mut flags = UsedFlags(0);
        assert_eq!(flags.is_set(UsedFlag::NoNotify), false);
        flags.set(UsedFlag::NoNotify);
        assert_eq!(flags.is_set(UsedFlag::NoNotify), true);
        flags.clear(UsedFlag::NoNotify);
        assert_eq!(flags.is_set(UsedFlag::NoNotify), false);

        let mut flags = AvailableFlags(0);
        assert_eq!(flags.is_set(AvailableFlag::NoInterrupt), false);
        flags.set(AvailableFlag::NoInterrupt);
        assert_eq!(flags.is_set(AvailableFlag::NoInterrupt), true);
        flags.clear(AvailableFlag::NoInterrupt);
        assert_eq!(flags.is_set(AvailableFlag::NoInterrupt), false);
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    struct Buffer {
        data: [u8; 16],
    }

    #[repr(C)]
    #[derive(Debug)]
    struct VirtQueue {
        descriptors: [DescriptorEntry; 8],
        available_flags: AvailableFlags,
        available_idx: u16,
        available_ring: [AvailableEntry; 8],
        used_flags: UsedFlags,
        used_idx: u16,
        used_ring: [UsedEntry; 8],
        buffers: [Buffer; 8],
    }

    fn make_virtqueue() -> Box<VirtQueue> {
        let mut v = Box::new(VirtQueue {
            descriptors: [DescriptorEntry {
                addr: 0,
                len: 0,
                flags: DescriptorFlags(0),
                next: 0,
            }; 8],
            available_flags: AvailableFlags(0),
            available_idx: 0,
            available_ring: [AvailableEntry { idx: 0 }; 8],
            used_flags: UsedFlags(0),
            used_idx: 0,
            used_ring: [UsedEntry { idx: 0, len: 0 }; 8],
            buffers: [Buffer { data: [0u8; 16] }; 8],
        });

        // See http://docs.oasis-open.org/virtio/virtio/v1.0/virtio-v1.0.pdf
        // section 3.2.1.1 Placing Buffers Into The Descriptor Table
        for idx in 0..8 {
            v.descriptors[idx].addr = &v.buffers[idx].data[0] as *const _ as u64;
            v.descriptors[idx].len = v.buffers[idx].data.len() as u32;
            v.descriptors[idx].flags.set(DescriptorFlag::Write);
            if idx < 7 {
                v.descriptors[idx].next = (idx + 1) as u16;
                v.descriptors[idx].flags.set(DescriptorFlag::Next);
            }
        }

        v
    }

    #[test]
    fn get_descriptors() {
        let backing = make_virtqueue();
        let backing_pointer = Box::into_raw(backing);
        let mut hq = unsafe { HostVring::new(backing_pointer as usize, 8, 1) };

        for i in 0..8 {
            hq.give_to_guest(|entry| {
                assert!(entry.flags.is_clear(DescriptorFlag::Next));
                // Buffer is not device writable, we (the host) are writing it
                entry.flags.clear(DescriptorFlag::Write);
                {
                    let buffer = entry.get_buffer();
                    buffer[0] = i as u8;
                    buffer[1] = (i + 1) as u8;
                    buffer[2] = (i + 2) as u8;
                }
                entry.len = 3;
            }).unwrap();
        }

        // We should now be out of memory
        assert!(hq.give_to_guest(|_| {}).is_err());

        // Now pretend we are the guest, processing these packets.
        let mut vq = unsafe { GuestVring::new(backing_pointer as usize, 8, 1) };
        for i in 0..8 {
            vq.process(|entry| {
                assert!(entry.flags.is_clear(DescriptorFlag::Write));
                assert!(entry.flags.is_clear(DescriptorFlag::Next));
                let buffer = entry.get_buffer();
                assert_eq!(buffer.len(), 3);
                assert_eq!(buffer[0], i);
                assert_eq!(buffer[1], i + 1);
                assert_eq!(buffer[2], i + 2);
                0
            }).unwrap();
        }

        // for _i in 0..8 {
        //     hq.take_from_guest(|entry| {
        //         assert!(entry.flags.is_clear(DescriptorFlag::Next));
        //     }).unwrap();
        // }

        let _backing = unsafe { Box::from_raw(backing_pointer) };
    }
}

// ****************************************************************************
//
// End Of File
//
// ****************************************************************************
