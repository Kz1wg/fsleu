mod filefinder;

use crossterm::ExecutableCommand;
use crossterm::terminal::{Clear, ClearType};
use rustyline::DefaultEditor;
use std::io::stdout;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut my_readline = DefaultEditor::new()?;
    let mut stdout = stdout();
    stdout.execute(Clear(ClearType::All))?;
    println!("Fsleu -File Finder-");

    let mut app = filefinder::FileFinder::new();

    app.extention = app.set_extension(&mut my_readline)?;
    app.manage_token(&mut my_readline)?;

    Ok(())
}
