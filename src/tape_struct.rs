use std::ops::{Index, IndexMut};

pub struct Tape {
    array: Option<Vec<u8>>,
}

impl Tape {
    pub fn new() -> Self {
        Tape { array: None }
    }

    pub fn init(&mut self, size: usize) {
        self.array = Some(vec![0u8; size]);
    }

    pub fn size(&self) -> usize {
        self.array.as_ref().unwrap().capacity()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.array.as_ref().unwrap().len()
    }
}

impl Index<usize> for Tape {
    type Output = u8;
    fn index(&self, index: usize) -> &Self::Output {
        &self.array.as_ref().unwrap()[index]
    }
}

impl IndexMut<usize> for Tape {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.array.as_mut().unwrap()[index]
    }
}
