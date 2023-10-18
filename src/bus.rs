use std::{ffi::{CStr, CString, OsString}, error::Error, vec, os::unix::prelude::{OsStrExt, OsStringExt}};

pub struct Bus
{
    ram: Vec<u8>,
    ppu: Vec<u8>,
    apu: Vec<u8>
}

impl Bus
{
    pub fn new() -> Bus
    {
        Bus {
            ram: vec![0; 0x800],
            ppu: vec![0; 8],
            apu: vec![0; 18]
        }        
    }

    #[inline(always)]
    pub fn read8(&self, addr: u16) -> u8
    {
        let addr = addr as usize;

        // RAM
        if addr < 0x2000 {
            let addr = addr % 0x800;
            return self.ram[addr];
        }

        // PPU
        if addr >= 0x2000 && addr < 0x4000 {
            let addr = (addr - 0x2000) % 8;
            return self.ppu[addr];
        }

        // APU & I/O
        if addr >= 0x4000 && addr < 0x4018 {
            return self.apu[addr - 0x4000]
        }

        // CPU Test Mode
        if addr >= 0x4018 && addr < 0x4020 {
            todo!("CPU Test Mode is not implemented")
        }

        // Cartridge space
        if addr >= 0x4020 {
            todo!("Cartrige is not implemented")
        }

        panic!("Invalud address: {}", addr)
    }

    #[inline(always)]
    pub fn write8(&mut self, addr: u16, val: u8)
    {
        let addr = addr as usize;

        // RAM
        if addr < 0x2000 {
            let addr = addr % 0x800;
            self.ram[addr] = val;
            return;
        }

        // PPU
        if addr >= 0x2000 && addr < 0x4000 {
            let addr = (addr - 0x2000) % 8;
            self.ppu[addr] = val;           
            return;
        }

        // APU & I/O
        if addr >= 0x4000 && addr < 0x4018 {
            self.apu[addr - 0x4000] = val;
            return;
        }

        // CPU Test Mode
        if addr >= 0x4018 && addr < 0x4020 {
            todo!("CPU Test Mode is not implemented")
        }

        // Cartridge space
        if addr >= 0x4020 {
            todo!("Cartrige is not implemented")
        }

        panic!("Invalud address: {}", addr)
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
    pub fn read_str(&self, addr: u16) -> Result<String, Box<dyn Error>>
    {
        let mut buf = Vec::<u8>::new();
        let mut offset: usize = 0;
        loop {
            let read_addr:usize = addr as usize + offset;
            if read_addr > 0xFFFF {
                panic!("Failed to read null terminated string at {}", addr)
            }

            let byte = self.read8(read_addr as u16);
            if byte == 0 {
                break;
            }
            
            buf.push(byte);
            offset += 1;
        }

        let ostr:OsString = OsStringExt::from_vec(buf);
        let str = ostr.into_string().unwrap();
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

    use super::Bus;

    #[test]
    fn read8()
    {
        let mut mem = Bus::new();
        mem.write8(0x100, 42);
        assert_eq!(42, mem.read8(0x100))
    }

    #[test]
    fn read8_mirror()
    {
        let mut mem = Bus::new();
        mem.write8(0x100, 42);

        assert_eq!(42, mem.read8(0x100));
        assert_eq!(42, mem.read8(0x100 + 0x800));
        assert_eq!(42, mem.read8(0x100 + 0x1000));
        assert_eq!(42, mem.read8(0x100 + 0x1800));
    }

    #[test]
    fn read8_ppu_mirror()
    {
        let mut mem = Bus::new();
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
        let mut mem = Bus::new();
        mem.write16(0x400, 0x1234);

        assert_eq!(0x1234, mem.read16(0x400))
    }

    #[test]
    fn write16()
    {
        let mut mem = Bus::new();
        mem.write16(0x400, 0x1234);

        assert_eq!(0x34, mem.read8(0x400));
        assert_eq!(0x12, mem.read8(0x401))
    }

    #[test]
    fn read_string()
    {
        let mut mem = Bus::new();
        let c_string = CString::new("Hello world!").unwrap();
        mem.write_buffer(0x200, c_string.as_bytes());

        assert_eq!("Hello world!", mem.read_str(0x200).unwrap())
    }

    #[test]
    fn write_buffer()
    {
        let mut mem = Bus::new();
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