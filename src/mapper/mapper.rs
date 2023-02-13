use int_enum::IntEnum;

#[repr(u16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntEnum)]
pub enum Mapper {
    NROM = 0,
    MMC1 = 1,
    UxROM = 2,
    CNROM = 3,
    MMC3 = 4,
    MMC5 = 5,
    NotSupported = 0xFFFF
}

impl Mapper {
    pub fn get_mapper(mapper: u16) -> Mapper {
        match Mapper::from_int(mapper) {
            Ok(m) => m,
            Err(_) => Mapper::NotSupported
        }
    }
}