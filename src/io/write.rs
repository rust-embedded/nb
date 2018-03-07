use core::{cmp, mem};

use Result;

/// Non-blocking writer trait
pub trait Write {
    /// An enumeration of possible errors
    ///
    /// May be `!` (`never_type`) for infallible implementations
    type Error;

    /// Push some bytes into this source from the specified buffer, returning how many bytes were
    /// written.
    ///
    /// If an object needs to block for a write it will return an `Err(nb::Error::WouldBlock)`
    /// return value.
    ///
    /// If the return value of this method is `Ok(n)`, then it must be guaranteed that `0 <= n <=
    /// buf.len()`. The `n` value indicates that `n` bytes from the buffer `buf` have been written
    /// to this source. If `n == 0 && buf.len() > 0` then it can be assumed that this writer has
    /// run out of space and will not be able to service future writes.
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error>;

    /// Attempt to flush the object, ensuring that any buffered data reach their destination.
    ///
    /// On success, returns `Ok(())`.
    ///
    /// If flushing cannot immediately complete, this method returns `Err(nb::Error::WouldBlock)`.
    fn flush(&mut self) -> Result<(), Self::Error>;

    /// Attempt to close the object.
    ///
    /// On success, returns `Ok(())`.
    ///
    /// If closing cannot immediately complete, this method returns `Err(nb::Error::WouldBlock)`.
    fn close(&mut self) -> Result<(), Self::Error>;
}

impl<'a, W: ?Sized + Write> Write for &'a mut W {
    type Error = W::Error;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        (**self).write(buf)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        (**self).flush()
    }

    fn close(&mut self) -> Result<(), Self::Error> {
        (**self).close()
    }
}

impl<'a> Write for &'a mut [u8] {
    type Error = !;

    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let len = cmp::min(self.len(), buf.len());
        let (head, tail) = mem::replace(self, &mut []).split_at_mut(len);
        head.copy_from_slice(&buf[..len]);
        *self = tail;
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn close(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
