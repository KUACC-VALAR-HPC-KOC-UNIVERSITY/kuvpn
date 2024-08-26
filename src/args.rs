use clap::{Parser, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum Browser {
    Chrome,
    Gecko,
    None,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = "https://vpn.ku.edu.tr")]
    pub url: String,

    #[arg(short, long, value_enum, default_value_t = Browser::Chrome)]
    pub browser: Browser,

    #[arg(short, long, default_value_t = 9515)]
    pub port: u16,
}
