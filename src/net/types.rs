#[derive(Debug)]
pub struct AccountLogin {
    pub account_name: String,
    pub password: String,
    pub xtea: [u32; 4],
    pub os: u16,
    pub version: u16,
}
