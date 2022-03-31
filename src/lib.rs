use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use std::io;
use std::io::Write;

const AUTH_PROMPT: &str = "Enter your smash.gg authentication token: ";

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

/// Reads in user input to grab their smash.gg authentication token.
/// Assigns the AUTHORIZATION header to Bearer [auth_token] and assigns the
/// CONTENT_TYPE header so we're taking in json on our post request.
pub fn construct_headers() -> HeaderMap{
    let mut headers = HeaderMap::new();
    let mut auth_token: String = get_input(AUTH_PROMPT);
    auth_token = "Bearer ".to_owned() + &auth_token;

    headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_token).unwrap());
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    return headers;
}