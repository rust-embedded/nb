use core::cmp;

use Result;

/// Non-blocking reader trait
pub trait Read {
    /// An enumeration of possible errors
    ///
    /// May be `!` (`never_type`) for infallible implementations
    type Error;

    /// Pull some bytes from this source into the specified buffer, returning how many bytes were
    /// read.
    ///
    /// If an object needs to block for a read it will return an `Err(nb::Error::WouldBlock)`
    /// return value.
    ///
    /// If the return value of this method is `Ok(n)`, then it must be guaranteed that `0 <= n <=
    /// buf.len()`. The `n` value indicates that the buffer `buf` has been filled in with `n` bytes
    /// of data from this source. If `n == 0 && buf.len() > 0` then it can be assumed that this
    /// reader has run out of data and will not be able service any future read calls.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error>;
}

impl<'a, R: ?Sized + Read> Read for &'a mut R {
    type Error = R::Error;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        (**self).read(buf)
    }
}

impl<'a> Read for &'a [u8] {
    type Error = !;

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = self.split_at(len);
        buf[..len].copy_from_slice(head);
        *self = tail;
        Ok(len)
    }
}
