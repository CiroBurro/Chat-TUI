/// Args module
// Necessary imports
use structopt::StructOpt;

/// Args struct representing cli arguments
#[derive(StructOpt, Debug)]
#[structopt(name = "Chat-TUI")]
pub struct Args {
    #[structopt(
        short,
        long,
        about = "Specify a different ip address than default (localhost 127.0.0.1)"
    )]
    /// Ip: Specify a different ip address than default (localhost 127.0.0.1)
    pub ip: Option<String>,

    /// Port: Specify a different port than default (8080)
    #[structopt(short, long, about = "Specify a different port than default (8080)")]
    pub port: Option<String>,
}
