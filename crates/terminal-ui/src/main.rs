use std::error::Error;

use terminal_ui::async_main;

fn main() -> Result<(), Box<dyn Error>> {
    async_std::task::block_on(async_main(None))?;

    Ok(())
}

