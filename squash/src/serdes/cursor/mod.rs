use core::fmt::Debug;
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};

import!(bytes, object);

pub trait SquashCursor: Read + Write + Seek {
    fn realloc(&mut self, size: u64);
    fn print_cursor(&self);

    fn seek_end(&mut self) -> io::Result<u64> {
        self.seek(SeekFrom::End(0))
    }
    
    fn remaining(&mut self) -> io::Result<u64> {
        self.seek(SeekFrom::Current(0))
    }
    
    fn pop_read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        let ret = -(buf.len() as i64);
        self.seek(SeekFrom::Current(ret))?;
        let x = self.read(buf)? as i64;
        self.seek(SeekFrom::Current(ret))?;
        Ok(x as usize)
    }

    fn push<'a, T>(&mut self, value: T) -> crate::Result<usize>
    where
        T: SquashObject,
        Self: Sized,
    {
        value.push_obj(self)
    }

    fn pop<T>(&mut self) -> crate::Result<T>
    where
        T: SquashObject,
        Self: Sized,
    {
        T::pop_obj(self)
    }
}

impl<A> SquashCursor for Cursor<A>
where
    A: AsMut<Vec<u8>> + AsRef<Vec<u8>> + Debug,
    Cursor<A>: Read + Write + Seek,
{
    fn realloc(&mut self, size: u64) {
        let position = self.position();
        let buf = self.get_mut().as_mut();
        let len = buf.len() as u64;
        if len < position + size {
            buf.resize((position + size) as usize, 0);
        }
    }

    fn print_cursor(&self) {
        let buf = self.get_ref();
        println!("{:?} - {}", buf, self.position());
    }
}

pub fn serialize<T: SquashObject>(value: T) -> crate::Result<Vec<u8>> {
    let mut cursor = Cursor::new(Vec::new());
    cursor.push(value)?;
    Ok(cursor.into_inner())
}

pub fn deserialize<T: SquashObject>(data: Vec<u8>) -> crate::Result<T> {
    let mut cursor = Cursor::new(data);
    cursor.seek_end()?;
    cursor.pop()
}
