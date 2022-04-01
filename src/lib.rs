use std::io;
use std::io::Write;

/// Generic function that takes in a prompt and converts the string to a type.
pub fn get_input<U: std::str::FromStr>(prompt: &str) -> U {
    loop {
        let mut input = String::new();

        // Print prompt to the screen and flush output.
        print!("{}", prompt);
        let _ = io::stdout().flush().expect("Failed to flush stdout.");

        // Read in the string from stdin.
        io::stdin().read_line(&mut input).expect("Failed to read input.");

        // Convert to specified type.
        // If successful, bind it to the variable input
        // If not, loop continuously.
        let input = match input.trim().parse::<U>() {
            Ok(parsed_input) => parsed_input,
            Err(_) => continue,
        };

        return input;
    }
}