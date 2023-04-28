use std::io::{self, Read};

pub struct ReadHelper<'a> {
    pub read: &'a mut dyn Read,
}

impl<'a> ReadHelper<'a> {
    pub fn new(read: &'a mut impl Read) -> Self {
        Self { read }
    }

    pub fn skip_bytes(&mut self, n: usize) -> io::Result<usize> {
        let mut ni = 0;
        for _ in 0..n {
            let mut arr = [0; 1];
            ni += self.read.read(&mut arr)?;
        }
        Ok(ni)
    }

    pub fn read_bool(&mut self) -> io::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_i8(&mut self) -> io::Result<i8> {
        let mut arr = [0; 1];
        self.read.read_exact(&mut arr)?;
        Ok(i8::from_be_bytes(arr))
    }

    pub fn read_u8(&mut self) -> io::Result<u8> {
        let mut arr = [0; 1];
        self.read.read_exact(&mut arr)?;
        Ok(arr[0])
    }

    pub fn read_i16(&mut self) -> io::Result<i16> {
        let mut arr = [0; 2];
        self.read.read_exact(&mut arr)?;
        Ok(i16::from_be_bytes(arr))
    }

    pub fn read_u16(&mut self) -> io::Result<u16> {
        let mut arr = [0; 2];
        self.read.read_exact(&mut arr)?;
        Ok(u16::from_be_bytes(arr))
    }

    pub fn read_char(&mut self) -> io::Result<char> {
        let mut arr = [0; 4];
        self.read.read_exact(&mut arr)?;
        char::from_u32(u32::from_be_bytes(arr)).map_or(
            Err(io::Error::new(io::ErrorKind::Other, "Unable to get char")),
            |ok| Ok(ok),
        )
    }

    pub fn read_i32(&mut self) -> io::Result<i32> {
        let mut arr = [0; 4];
        self.read.read_exact(&mut arr)?;
        Ok(i32::from_be_bytes(arr))
    }

    pub fn read_i64(&mut self) -> io::Result<i64> {
        let mut arr = [0; 8];
        self.read.read_exact(&mut arr)?;
        Ok(i64::from_be_bytes(arr))
    }

    pub fn read_f32(&mut self) -> io::Result<f32> {
        let mut arr = [0; 4];
        self.read.read_exact(&mut arr)?;
        Ok(f32::from_be_bytes(arr))
    }

    pub fn read_f64(&mut self) -> io::Result<f64> {
        let mut arr = [0; 8];
        self.read.read_exact(&mut arr)?;
        Ok(f64::from_be_bytes(arr))
    }

    pub fn read_utf(&mut self) -> io::Result<String> {
        let utf_len = self.read_u16()? as usize;
        let mut vec = Vec::with_capacity(utf_len);
        for _ in 0..utf_len {
            vec.push(self.read_u8()?)
        }
        String::from_utf8(vec).map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
    }
}

impl Read for ReadHelper<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read.read(buf)
    }
}
