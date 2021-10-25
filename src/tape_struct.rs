#![allow(unused)]

use std::mem::MaybeUninit;
use std::ops::{Drop, Index, IndexMut};

pub struct Tape {
    // SAFETY: the compiler always produce a first Make OpCode,
    // so it will always be initialized before used
    array: MaybeUninit<Vec<u8>>,
}

impl Tape {
    pub fn new() -> Self {
        Tape {
            array: MaybeUninit::uninit(),
        }
    }

    pub fn init(&mut self, size: usize) {
        self.array.write(vec![0u8; size]);
    }

    pub fn size(&self) -> usize {
        unsafe { self.array.assume_init_ref() }.capacity()
    }

    pub fn len(&self) -> usize {
        unsafe { self.array.assume_init_ref() }.len()
    }
}

impl Index<usize> for Tape {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        let arr = unsafe { self.array.assume_init_ref() };
        &arr[index]
    }
}

impl IndexMut<usize> for Tape {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let arr = unsafe { self.array.assume_init_mut() };
        &mut arr[index]
    }
}

impl Drop for Tape {
    fn drop(&mut self) {
        unsafe { self.array.assume_init_drop() };
    }
}
