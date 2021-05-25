use argh::FromArgs;

#[derive(FromArgs)]
/// Prints various metrics of your system.
pub struct Settings {
    /// file to write the results to
    #[argh(option)]
    pub output: Option<String>,
}
