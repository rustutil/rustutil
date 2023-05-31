use std::io::{self, Write};

pub fn prompt(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    print!("{}: ", prompt);
    io::stdout().flush()?; // flushes the buffer so it is always before the readline
    let buf = &mut String::new();
    io::stdin().read_line(buf)?;
    Ok(buf.trim_end().to_owned())
}
