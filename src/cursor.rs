use ufmt::uWrite;
use core::convert::Infallible;

pub struct Cursor<B>
where
    B: ?Sized,
{
    pos: usize,
    buffer: B,
}

impl<B> Cursor<B> {
    pub fn new(buffer: B) -> Self {
        Cursor { pos: 0, buffer }
    }
}

impl uWrite for Cursor<[u8]> {
    type Error = Infallible;

    fn write_str(&mut self, s: &str) -> Result<(), Infallible> {
        let bytes = s.as_bytes();
        let len = bytes.len();
        let start = self.pos;
        if let Some(buffer) = self.buffer.get_mut(start..start + len) {
            buffer.copy_from_slice(bytes);
            self.pos += len;
        }

        Ok(())
    }
}

/*impl<B> Into<dyn Iterator<Item = B>> for Cursor<B> {
    fn into(self) -> dyn Iterator<Item = B> {
        self.buffer.iter()
    }
}*/

/*impl<B> IntoIterator for Cursor<B> {
    type Item = u8;
    //type IntoIter = B::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.buffer.into_iter()
    }
}*/

/*impl Iterator for Cursor<[u8]> {
    type Item = Option<&'a u8>;

    fn next(self) -> Self::Item {
        self.buffer.iter().next()
    }
}*/

/*impl Iterator for Cursor<[u8]> {
    // we will be counting with usize
    type Item = &u8;

    // next() is the only required method
    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.iter().next()
    }
}*/

