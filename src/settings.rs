use argh::FromArgs;

#[derive(FromArgs)]
/// Prints how many unread emails you have in Thunderbird.
pub struct Settings {
    /// do not print results to standard output
    #[argh(switch, short = 'q')]
    pub quiet: bool,

    /// file to write the results to
    #[argh(option)]
    pub output: Option<String>,

    /// interval in seconds to scan for the Thunderbird process
    #[argh(option, default = "5")]
    pub interval: u64,
}
