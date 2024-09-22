use core::mem;
use super::EMPTY;
use super::bitmask::BitMask;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as x86;

#[derive(Copy, Clone)]
pub(crate) struct Group(x86::__m128i);

impl Group {

    // loads 128-bit data from given pointer
    pub(crate) unsafe fn load(ptr: *const u8) -> Self {
        Group(x86::_mm_loadu_si128(ptr.cast()))
    }

    pub(crate) fn match_byte(self, byte: u8) -> BitMask {
        unsafe {
            // _mm_set1_epi8 creates a vector for given value
            // _mm_set1_epi8(42) -> [42; 16]
            // _mm_cmpeq_epi8 conpares both vector
            // returns high bit values for each matching data
            // [42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42, 42]
            // [41, 40, 39, 42, 44, 45, 44, 42, 47, 52, 56, 64, 67, 80, 22, 26]
            // [ 0,  0,  0, FF,  0,  0,  0, FF,  0,  0,  0,  0,  0,  0,  0,  0]
            let cmp = x86::_mm_cmpeq_epi8(self.0, x86::_mm_set1_epi8(byte as i8));
            // sets 1 if most significant bit is set and returns i32 of it
            // [ 0,  0,  0, FF,  0,  0,  0, FF,  0,  0,  0,  0,  0,  0,  0,  0]
            // 0001_0001_0000_0000 integer value
            let res = x86::_mm_movemask_epi8(cmp) as u16;
            BitMask(res)
        }
    }

    pub(crate) fn match_empty(self) -> BitMask {
        // compares loaded data with EMPTY bytes
        self.match_byte(EMPTY)
    }

    pub(crate) fn match_empty_or_deleted(self) -> BitMask {
        unsafe {
            // A byte is EMPTY or DELETED if the high bit is set
            BitMask(x86::_mm_movemask_epi8(self.0) as u16)
        }
    }
}
