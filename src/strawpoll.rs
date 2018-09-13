use std::io::Read;
use std::convert::From;
use curl;
use curl::easy::{Easy, List};
use serde_json;

pub fn create_poll(title: &str, options: &[&str]) -> Result<Poll, PollError> {

    let options_len = options.len();
    if options_len < 2 || options_len > 30 {
        return Err(PollError::new("Wrong number of options", None))
    }

    let request = format!("{{ \"title\": \"{}\", \"options\": {:?} }}", title, options);
    let mut byte_request = request.as_bytes();
    let mut result = Vec::new();

    let mut headers = List::new();
    headers.append("Content-Type: application/json").unwrap();

    let mut handle = Easy::new();
    handle.url("https://www.strawpoll.me/api/v2/polls")?;
    handle.post(true)?;
    handle.post_field_size(byte_request.len() as u64)?;
    handle.http_headers(headers)?;

    { // Perform transfer
        let mut transfer = handle.transfer();
        transfer.read_function(|buf| {
            Ok(byte_request.read(buf).unwrap_or(0))
        })?;
        transfer.write_function(|buf| {
            result.extend_from_slice(buf);
            Ok(buf.len())
        })?;

        transfer.perform()?;
    }

    Ok(serde_json::from_slice::<Poll>(&result)?)
}

pub fn get_poll(id: u32) -> Result<Poll, PollError> {

    let mut result = Vec::new();

    let mut handle = Easy::new();
    handle.url(&format!("https://www.strawpoll.me/api/v2/polls/{}", id))?;

    { // Perform transfer
        let mut transfer = handle.transfer();
        transfer.write_function(|buf| {
            result.extend_from_slice(buf);
            Ok(buf.len())
        })?;

        transfer.perform()?;
    }

    Ok(serde_json::from_slice::<Poll>(&result)?)
}

// 
//  Poll struct
//
#[derive(Debug, Deserialize)]
pub struct Poll {
    pub id:       u32,
    pub title:    String,
    pub options:  Vec<String>,
    pub votes:    Option<Vec<u32>>,
    pub multi:    bool,
    pub dupcheck: String,
    pub captcha:  bool,
}


//
//  PollError struct
// Make a separate module just for errors: error.rs
pub struct PollError {
    description: String,
    cause: Option<String>,
}

impl PollError {
    pub fn new(desc: &str, cause: Option<String>) -> Self {
        PollError {
            description: String::from(desc),
            cause,
        }
    }
}

impl From<curl::Error> for PollError {
    fn from(err: curl::Error) -> Self {
        Self {
            description: String::from("cURL error encountered"),
            cause: None,
        }
    }
}

impl From<serde_json::Error> for PollError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            description: String::from("Error deserializing JSON"),
            cause: None,
        }
    }
}
