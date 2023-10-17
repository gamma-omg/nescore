use std::{ffi::CStr, error::Error};

pub struct Memory
{
    data: Vec<u8>
}

impl Memory
{
    pub fn new() -> Memory
    {
        let mut buffer = Vec::new();
        buffer.resize(0xFFFF, 0);
        Memory { data: buffer }
    }

    pub fn from_buffer(mut initial: Vec<u8>) -> Memory 
    {
        initial.resize(0xFFFF, 0);
        Memory { data: initial }
    }

    #[inline(always)]
    pub fn read8(&self, addr: u16) -> u8
    {
        let addr = addr as usize;

        // Mirrors of $0000–$07FF
        if addr >= 0x800 && addr < 0x2000 {
            let addr = addr % 0x800;
            return self.data[addr];
        }

        // PPU registers mirrors ($2000–$2007)
        if addr >= 0x2000 && addr < 0x4000 {
            let addr = addr + (addr - 0x2000) % 8;
            return self.data[addr];
        }

        self.data[addr]
    }

    #[inline(always)]
    pub fn write8(&mut self, addr: u16, val: u8)
    {
        let addr = addr as usize;
        self.data[addr] = val;

        // Mirrors of $0000–$07FF
        if addr >= 0x800 && addr < 0x2000 {
            let addr = addr % 0x800;
            self.data[addr] = val;
            self.data[addr + 0x800] = val;
            self.data[addr + 0x1000] = val;
            self.data[addr + 0x1800] = val;
            return;
        }

        // PPU registers mirrors ($2000–$2007)
        if addr >= 0x2000 && addr < 0x4000 {
            let mut addr = addr + (addr - 0x2000) % 8;
            while addr < 0x4000 {
                self.data[addr] = val;
                addr += 8;
            }
            return;
        }

        self.data[addr] = val;
    }

    #[inline(always)]
    pub fn read16(&self, addr: u16) -> u16 
    {
        let l:u16 = self.read8(addr) as u16;
        let h:u16 = self.read8(addr + 1) as u16;
        l | (h << 8)
    }

    #[inline(always)]
    pub fn write16(&mut self, addr: u16, val: u16)
    {
        self.write8(addr, (val & 0xFF) as u8);
        self.write8(addr + 1, (val >> 8) as u8);
    }

    #[inline(always)]
    pub fn read_str(&self, addr: usize) -> Result<&str, Box<dyn Error>>
    {
        let cstr = CStr::from_bytes_until_nul(&self.data[addr..])?;
        let str = cstr.to_str()?;
        Ok(str)
    }
    
    #[inline(always)]
    pub fn write_buffer(&mut self, addr: u16, buffer: &[u8])
    {        
        for i in 0..buffer.len() {
            let addr = addr + i as u16;
            self.write8(addr, buffer[i])
        }
    }

    #[inline(always)]
    pub fn read_buffer(&self, addr: u16, out_buffer: &mut Vec<u8>)
    {
        for i in 0..out_buffer.len() {
            let addr = addr + i as u16;
            out_buffer[i] = self.read8(addr);
        }
    }
}

#[cfg(test)]
mod tests
{
    use std::ffi::CString;

    use super::Memory;

    #[test]
    fn read8()
    {
        let mut mem = Memory::new();
        mem.write8(0x100, 42);
        assert_eq!(42, mem.read8(0x100))
    }

    #[test]
    fn read8_mirror()
    {
        let mut mem = Memory::new();
        mem.write8(0x100, 42);

        assert_eq!(42, mem.read8(0x100));
        assert_eq!(42, mem.read8(0x100 + 0x800));
        assert_eq!(42, mem.read8(0x100 + 0x1000));
        assert_eq!(42, mem.read8(0x100 + 0x1800));
    }

    #[test]
    fn read8_ppu_mirror()
    {
        let mut mem = Memory::new();
        mem.write8(0x2002, 42);

        let mut addr = 0x2002;
        while addr < 0x4000 {
            assert_eq!(42, mem.read8(addr));
            addr += 8;
        }
    }

    #[test]
    fn read16()
    {
        let mut mem = Memory::new();
        mem.write16(0x400, 0x1234);

        assert_eq!(0x1234, mem.read16(0x400))
    }

    #[test]
    fn write16()
    {
        let mut mem = Memory::new();
        mem.write16(0x400, 0x1234);

        assert_eq!(0x34, mem.read8(0x400));
        assert_eq!(0x12, mem.read8(0x401))
    }

    #[test]
    fn read_string()
    {
        let mut mem = Memory::new();
        let c_string = CString::new("Hello world!").unwrap();
        mem.write_buffer(0x200, c_string.as_bytes());

        assert_eq!("Hello world!", mem.read_str(0x200).unwrap())
    }

    #[test]
    fn write_buffer()
    {
        let mut mem = Memory::new();
        mem.write_buffer(0x600, &vec![0x01, 0x02, 0x03]);
        
        let mut out = vec![0; 3];
        mem.read_buffer(0x600, &mut out);
        assert_eq!(vec![0x01, 0x02, 0x03], out);

        mem.read_buffer(0x600 + 0x800, &mut out);
        assert_eq!(vec![0x01, 0x02, 0x03], out);

        mem.read_buffer(0x600 + 0x1000, &mut out);
        assert_eq!(vec![0x01, 0x02, 0x03], out);

        mem.read_buffer(0x600 + 0x1800, &mut out);
        assert_eq!(vec![0x01, 0x02, 0x03], out);
    }
}