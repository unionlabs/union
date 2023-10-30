pub trait _StrExt {
    fn _is_char_boundary(&self, index: usize) -> bool;
}

impl _StrExt for str {
    #[inline]
    fn _is_char_boundary(&self, index: usize) -> bool {
        if index == self.len() {
            return true;
        }
        match self.as_bytes().get(index) {
            None => false,
            Some(&b) => !(128..192).contains(&b),
        }
    }
}
