use anyhow::Result;
use rustyline::{error::ReadlineError, DefaultEditor};

pub fn run_shell() -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    println!("🐱  Meow shell activated.");
    println!("Type 'exit' or 'quit' to leave.\n");

    loop {
        let line = rl.readline("meow> ");

        match line {
            Ok(input) => {
                let input = input.trim();

                if input.is_empty() {
                    continue;
                }

                // Add to history (ignore return bool)
                let _ = rl.add_history_entry(input);

                // Exit commands
                if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                    println!("👋 Bye, human.");
                    break;
                }

                // TEMP: just echo for now
                println!("You said: {input}");
            }

            Err(ReadlineError::Interrupted) => {
                // Ctrl+C
                println!("\n(Interrupted) Bye 🐾");
                break;
            }

            Err(ReadlineError::Eof) => {
                // Ctrl+D
                println!("\n(EOF) Bye 🐾");
                break;
            }

            Err(err) => {
                eprintln!("Error reading line: {err}");
                break;
            }
        }
    }

    Ok(())
}
