use std::error::Error;
use std::io::Read;

use self::error::FormatError;
use self::header::{INESHeader, Mirroring};

mod error
{
    use std::{fmt::Display, error::Error};

    #[derive(Debug)]
    pub struct FormatError(pub String);
    
    impl Display for FormatError
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result 
        {
            write!(f, "INES format error: {}", self.0)
        }
    }
    
    impl Error for FormatError {}
}

mod header
{
    use std::{io::Read, error::Error};

    pub enum Mirroring
    {
        Horizontal,
        Vertical
    }

    mod flag6
    {
        pub const MIRRORING: u8 = 0b00000001;
        pub const PERSISTENT_MEMEORY: u8 = 0b00000010;
        pub const TRAINER: u8 = 0b00000100;
        pub const IGNORE_MIRRORING: u8 = 0b00001000;
        pub const MAPPER_LOWER: u8 = 0b11110000; 
    }
    
    mod flag7
    {
        pub const VS_UNISYSTEM: u8 = 0b00000001;
        pub const PLAY_CHOICE_10: u8 = 0b00000010;
        pub const NES2_FORMAT: u8 = 0b00001100;
        pub const MAPPER_UPPER: u8 = 0b11110000;
    }

    #[repr(C, packed)]
    pub struct INESHeader
    {
        pub format: [u8; 4],
        pub prg_rom_banks: u8,
        pub chr_rom_banks: u8,
        pub flag6: u8,
        pub flag7: u8,
        pub prg_ram_banks: u8,
        pub flag9: u8
    }

    impl INESHeader 
    {
        pub fn from_reader(reader: &mut dyn Read) -> Result<INESHeader, Box<dyn Error>>
        {
            let mut buf = [0; std::mem::size_of::<INESHeader>()];
            reader.read_exact(&mut buf)?;
            unsafe {
                Ok(std::mem::transmute(buf))
            }
        }

        pub fn has_trainer(&self) -> bool
        {
            self.flag6 & flag6::TRAINER > 0
        }

        pub fn has_play_choice_10(&self) -> bool
        {
            self.flag7 & flag7::PLAY_CHOICE_10 > 0
        }

        pub fn has_persistent_memory(&self) -> bool
        {
            self.flag6 & flag6::PERSISTENT_MEMEORY > 0
        }

        pub fn has_vs_unisystem(&self) -> bool
        {
            self.flag7 & flag7::VS_UNISYSTEM > 0
        }

        pub fn is_nes2_format(&self) -> bool
        {
            self.flag7 & flag7::NES2_FORMAT == 0b1000
        }

        pub fn get_mirroring(&self) -> Mirroring
        {
            if self.flag6 & flag6::MIRRORING == 0 {
                Mirroring::Horizontal
            } 
            else {
                Mirroring::Vertical
            }        
        }

        pub fn get_ignore_mirroring(&self) -> bool
        {
            self.flag6 & flag6::IGNORE_MIRRORING > 0
        }

        pub fn get_mapper(&self) -> u8
        {
            self.flag7 & flag7::MAPPER_UPPER | (self.flag6 & flag6::MAPPER_LOWER) >> 4
        }
    }

    #[cfg(test)]
    mod tests
    {
        use super::*;

        fn header_with_flag6(flag6: u8) -> [u8; 10]
        {
            [0x4E, 0x45, 0x53, 0x1A, 0x0, 0x0, flag6, 0x0, 0x0, 0x0]
        }

        fn header_with_flag7(flag7: u8) -> [u8; 10]
        {
            [0x4E, 0x45, 0x53, 0x1A, 0x0, 0x0, 0x0, flag7, 0x0, 0x0]
        }

        #[test]
        fn read()
        {
            let header_bytes = [0x4E, 0x45, 0x53, 0x1A, 0x1, 0x1, 0x0, 0x0, 0x1, 0x0];
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();

            assert_eq!(header.format, [0x4E, 0x45, 0x53, 0x1A]);
            assert_eq!(header.prg_rom_banks, 1);
            assert_eq!(header.chr_rom_banks, 1);
            assert_eq!(header.flag6, 0);
            assert_eq!(header.flag7, 0);
            assert_eq!(header.prg_ram_banks, 1);
            assert_eq!(header.flag9, 0);
        }

        #[test]
        fn has_trainer()
        {
            let header_bytes = [
                0x4E, 0x45, 0x53, 0x1A, // format
                0x0, // prg_rom_banks
                0x0, // chr_rom_banks
                0b00000100, // flag 6
                0x0, // flag 7
                0x0, // prg_ram_banks
                0x0, // flag 9
            ];
            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
            
            assert!(header.has_trainer())
        }

        #[test]
        fn doesnt_have_trainer()
        {
            let header_bytes = header_with_flag6(0);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(!header.has_trainer())
        }

        #[test]
        fn has_play_choice_10()
        {
            let header_bytes = header_with_flag7(0b00000010);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(header.has_play_choice_10())
        }

        #[test]
        fn doesnt_have_play_choice_10()
        {
            let header_bytes = header_with_flag7(0);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(!header.has_play_choice_10())
        }

        #[test]
        fn read_mirroring_horizontal()
        {
            let header_bytes = header_with_flag6(0);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(matches!(header.get_mirroring(), Mirroring::Horizontal))
        }

        #[test]
        fn read_mirroring_vertiacal()
        {
            let header_bytes = header_with_flag6(0b00000001);
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(matches!(header.get_mirroring(), Mirroring::Vertical))
        }

        #[test]
        fn has_persistent_memory()
        {
            let header_bytes = header_with_flag6(0b00000010);       
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(header.has_persistent_memory())
        }

        #[test]
        fn doesnt_have_persistent_memory()
        {
            let header_bytes = header_with_flag6(0);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(!header.has_persistent_memory());
        }

        #[test]
        fn has_vs_unisystem()
        {
            let header_bytes = header_with_flag7(0b00000001);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(header.has_vs_unisystem());
        }

        #[test]
        fn doesnt_have_vs_unisystem()
        {
            let header_bytes = header_with_flag7(0);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(!header.has_vs_unisystem());
        }

        #[test]
        fn is_nes2_foramt()
        {
            let header_bytes = header_with_flag7(0b00001000);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(header.is_nes2_format());
        }

        #[test]
        fn is_not_nes2_foramt()
        {
            let header_bytes = header_with_flag7(0b00001100);            
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();
                    
            assert!(!header.is_nes2_format());
        }

        #[test]
        fn get_mapper()
        {
            let header_bytes = [
                0x4E, 0x45, 0x53, 0x1A, // format
                0x0, // prg_rom_banks
                0x0, // chr_rom_banks
                0b01100000, // flag 6
                0b10010000, // flag 7
                0x0, // prg_ram_banks
                0x0, // flag 9
            ];
            let header = INESHeader::from_reader(&mut &header_bytes[..]).unwrap();

            assert_eq!(header.get_mapper(), 0b10010110);
        }     
    }
}

const TRAINER_SIZE: usize = 0x200;
const PRG_ROM_BANK_SIZE: usize = 0x4000;
const CHR_ROM_BANK_SIZE: usize = 0x2000;
const PLAY_CHOICE_10_SIZE: usize = 0x2000;

pub struct INESRom
{
    header: INESHeader,
    trainer: Option<Vec<u8>>,
    play_chouice_10: Option<Vec<u8>>,
    prg_banks: Vec<Vec<u8>>,
    chr_banks: Vec<Vec<u8>>,
}

impl INESRom
{
    pub fn from_reader(mut reader: impl Read) -> Result<Self, Box<dyn Error>>
    {
        let header = INESHeader::from_reader(&mut reader)?;
        if header.format != [0x4E, 0x45, 0x53, 0x1A]
        {
            return Err(Box::new(FormatError("Invalid format name".into())));
        }

        let mut trainer = None;
        if header.has_trainer()
        {
            trainer = Some(INESRom::read_bank(&mut reader, TRAINER_SIZE)?);
        }

        let mut prg_banks = Vec::<Vec::<u8>>::new();
        for _ in 0..header.prg_rom_banks
        {
            prg_banks.push(INESRom::read_bank(&mut reader, PRG_ROM_BANK_SIZE)?);
        }

        let mut chr_banks = Vec::<Vec::<u8>>::new();
        for _ in 0..header.chr_rom_banks 
        {
            chr_banks.push(INESRom::read_bank(&mut reader, CHR_ROM_BANK_SIZE)?);
        }

        let mut play_choice_bank = None;
        if header.has_play_choice_10()
        {
            play_choice_bank = Some(INESRom::read_bank(&mut reader, PLAY_CHOICE_10_SIZE)?);
        }

        Ok(INESRom { 
            header: header,
            trainer: trainer,
            play_chouice_10: play_choice_bank,
            prg_banks: prg_banks,
            chr_banks: chr_banks
        })
    }

    pub fn has_persistent_memory(&self) -> bool
    {
        self.header.has_persistent_memory()
    }

    pub fn has_vs_unisystem(&self) -> bool
    {
        self.header.has_vs_unisystem()
    }

    pub fn is_nes2_foramt(&self) -> bool
    {
        self.header.is_nes2_format()
    }

    pub fn get_mirroring(&self) -> Mirroring
    {
        self.header.get_mirroring()
    }

    pub fn get_ignore_mirroring(&self) -> bool
    {
        self.header.get_ignore_mirroring()
    }

    pub fn get_mapper(&self) -> u8
    {
        self.header.get_mapper()
    }

    pub fn get_trainer(&self) -> Option<&Vec<u8>>
    {
        self.trainer.as_ref()
    }

    pub fn get_prg_bank(&self, index: usize) -> Option<&Vec<u8>>
    {
        self.prg_banks.get(index)
    }

    pub fn get_chr_bank(&self, index: usize) -> Option<&Vec<u8>>
    {
        self.chr_banks.get(index)
    }

    pub fn get_play_choise_10(&self) -> Option<&Vec<u8>>
    {
        self.play_chouice_10.as_ref()
    }

    fn read_bank(reader: &mut dyn Read, size: usize) -> Result<Vec<u8>, Box<dyn Error>>
    {
        let mut buf = Vec::with_capacity(size);
        buf.resize(size, 0);
        reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests
{
    
}