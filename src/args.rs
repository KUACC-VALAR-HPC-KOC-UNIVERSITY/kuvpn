use clap::Parser;

/// Simple program to retrieve DSID cookie and execute OpenConnect command
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The URL to the page where we will start logging in and looking for DSID
    #[arg(short, long, default_value = "https://vpn.ku.edu.tr")]
    pub url: String,

    /// Gives the user the dsid without running openconnect
    #[arg(short, long, default_value_t = false)]
    pub dsid: bool,

    /// Delete session information
    #[arg(short, long, default_value_t = false)]
    pub clean: bool,
}
