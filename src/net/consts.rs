// src/consts.rs
// 1:1 översättning av const.h till Rust

// --------- Global constants ----------
pub const STATUS_SERVER_NAME: &str = "The Forgotten Server";
pub const STATUS_SERVER_VERSION: &str = "1.4.2";
pub const STATUS_SERVER_DEVELOPERS: &str = "The Forgotten Server Team";

pub const CLIENT_VERSION_MIN: u16 = 1097;
pub const CLIENT_VERSION_MAX: u16 = 1098;
pub const CLIENT_VERSION_STR: &str = "10.98";

pub const AUTHENTICATOR_DIGITS: u32 = 6;
pub const AUTHENTICATOR_PERIOD: u32 = 30;

pub const NETWORKMESSAGE_MAXSIZE: i32 = 24590;
pub const MIN_MARKET_FEE: u32 = 20;
pub const MAX_MARKET_FEE: u32 = 100_000;

pub const CHANNEL_GUILD: u16 = 0x00;
pub const CHANNEL_PARTY: u16 = 0x01;
pub const CHANNEL_PRIVATE: u16 = 0xFFFF;

pub const PSTRG_RESERVED_RANGE_START: u32 = 10_000_000;
pub const PSTRG_RESERVED_RANGE_SIZE: u32 = 10_000_000;

// Helper macro from TFS: IS_IN_KEYRANGE
pub fn is_in_keyrange(key: i32, range_start: i32, range_size: i32) -> bool {
    key >= range_start && (key - range_start) <= range_size
}

// --------- Fluid Maps ----------
pub static REVERSE_FLUID_MAP: [u8; 11] = [
    FluidTypes::Empty as u8,
    FluidTypes::Water as u8,
    FluidTypes::Mana as u8,
    FluidTypes::Beer as u8,
    FluidTypes::Empty as u8,
    FluidTypes::Blood as u8,
    FluidTypes::Slime as u8,
    FluidTypes::Empty as u8,
    FluidTypes::Lemonade as u8,
    FluidTypes::Milk as u8,
    FluidTypes::Ink as u8,
];

pub static CLIENT_TO_SERVER_FLUID_MAP: [u8; 19] = [
    FluidTypes::Empty as u8,
    FluidTypes::Water as u8,
    FluidTypes::Mana as u8,
    FluidTypes::Beer as u8,
    FluidTypes::Mud as u8,
    FluidTypes::Blood as u8,
    FluidTypes::Slime as u8,
    FluidTypes::Rum as u8,
    FluidTypes::Lemonade as u8,
    FluidTypes::Milk as u8,
    FluidTypes::Wine as u8,
    FluidTypes::Life as u8,
    FluidTypes::Urine as u8,
    FluidTypes::Oil as u8,
    FluidTypes::FruitJuice as u8,
    FluidTypes::CoconutMilk as u8,
    FluidTypes::Tea as u8,
    FluidTypes::Mead as u8,
    FluidTypes::Ink as u8,
];

pub static FLUID_MAP: [u8; 9] = [
    ClientFluidTypes::Empty as u8,
    ClientFluidTypes::Blue as u8,
    ClientFluidTypes::Red as u8,
    ClientFluidTypes::Brown1 as u8,
    ClientFluidTypes::Green as u8,
    ClientFluidTypes::Yellow as u8,
    ClientFluidTypes::White as u8,
    ClientFluidTypes::Purple as u8,
    ClientFluidTypes::Black as u8,
];

// --------- Example Enums ----------
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FluidTypes {
    Empty = 0,
    Water = 1,
    Blood = 2,
    Beer = 3,
    Slime = 4,
    Lemonade = 5,
    Milk = 6,
    Mana = 7,
    Ink = 8,
    Life = 10,
    Oil = 11,
    Urine = 12,
    CoconutMilk = 13,
    Wine = 14,
    Mud = 19,
    FruitJuice = 20,
    Lava = 26,
    Rum = 27,
    Swamp = 28,
    Tea = 35,
    Mead = 43,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientFluidTypes {
    Empty = 0,
    Blue = 1,
    Purple = 2,
    Brown1 = 3,
    Brown2 = 4,
    Red = 5,
    Green = 6,
    Brown = 7,
    Yellow = 8,
    White = 9,
    Black = 18,
}

// Exempel: TextColor
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextColor {
    Blue = 5,
    LightGreen = 30,
    LightBlue = 35,
    DarkGrey = 86,
    MayaBlue = 95,
    DarkRed = 108,
    LightGrey = 129,
    SkyBlue = 143,
    Purple = 154,
    ElectricPurple = 155,
    Red = 180,
    PastelRed = 194,
    Orange = 198,
    Yellow = 210,
    WhiteExp = 215,
    None = 255,
}

// TODO: Lägg in resterande enums från const.h (MagicEffectClasses, SpeakClasses, Skulls, Shields, osv)
