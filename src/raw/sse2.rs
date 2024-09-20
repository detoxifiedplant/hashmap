use super::EMPTY;
use core::mem;
use super::bitmask::BitMask;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64 as x86;

#[derive(Copy, Clone)]
pub(crate) struct Group(x86::__m128i);

impl Group {

    pub(crate) unsafe fn load(ptr: *const u8) -> Self {
        Group(x86::_mm_loadu_si128(ptr.cast()))
    }

    pub(crate) fn match_byte(self, byte: u8) -> BitMask {
        unsafe {
            let cmp = x86::_mm_cmpeq_epi8(self.0, x86::_mm_set1_epi8(byte as i8));
            let res = x86::_mm_movemask_epi8(cmp) as u16;
            BitMask(res)
        }
    }

    pub(crate) fn match_empty(self) -> BitMask {
        self.match_byte(EMPTY)
    }

    pub(crate) fn match_empty_or_deleted(self) -> BitMask {
        unsafe {
            // A byte is EMPTY or DELETED iff the high bit is set
            BitMask(x86::_mm_movemask_epi8(self.0) as u16)
        }
    }
}

