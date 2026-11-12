use anyhow::Result;
use clap::Args;

use crate::app::App;

#[derive(Args, Clone, Debug, PartialEq)]
pub struct AppCommand {}

impl AppCommand {
    pub fn run(self) -> Result<()> {
        let app = App::new();

        app.run()
    }
}
