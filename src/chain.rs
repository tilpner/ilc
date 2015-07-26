use std::io::{ Result, Read, Write };

pub struct Chain<T> {
    elem: Vec<T>,
    index: usize
}

impl<T: Read> Read for Chain<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        loop {
            match self.elem.get_mut(self.index) {
                Some(ref mut r) => match try!(r.read(buf)) {
                    0 => self.index += 1,
                    n => return Ok(n)
                },
                None => return Ok(0)
            }
        }
    }
}

impl<T: Write> Write for Chain<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        loop {
            match self.elem.get_mut(self.index) {
                Some(ref mut r) => match try!(r.write(buf)) {
                    0 => self.index += 1,
                    n => return Ok(n)
                },
                None => return Ok(0)
            }
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self.elem.get_mut(self.index) {
            Some(ref mut r) => r.flush(),
            None => Ok(())
        }
    }

}

impl<T> Chain<T> {
    pub fn new(elem: Vec<T>) -> Chain<T> {
        Chain { index: 0, elem: elem }
    }
}
