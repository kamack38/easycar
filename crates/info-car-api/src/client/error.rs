use regex::Regex;
use reqwest::Response;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Error ({0}): {1} ({2})")]
pub struct JWTError(String, String, String);

fn extract_all_quoted_strings(input: &str) -> Vec<String> {
    let re = Regex::new(r#""(.*?)""#).unwrap();

    re.captures_iter(input)
        .map(|cap| cap[1].to_string()) // Extract the matched group inside the quotes
        .collect()
}

pub fn handle_response(response: Response) -> Result<Response, JWTError> {
    if response.status() == 401 {
        let error_header = response
            .headers()
            .get("www-authenticate")
            .expect("Header not found")
            .to_str()
            .unwrap_or("Failed to convert HeaderValue to string");
        let strings = extract_all_quoted_strings(error_header);
        return Err(JWTError(
            strings[0].clone(),
            strings[1].clone(),
            strings[2].clone(),
        ));
    }
    Ok(response)
}
