#[macro_use] extern crate failure;

use std::env;
use serde_json;
use serde::{Serialize, Deserialize};
use chrono::{NaiveDateTime, DateTime, Utc};
use failure::Error;
use tokio::runtime::Runtime;

use mc_creation_date::simple_created_date;

#[derive(Serialize, Deserialize)]
struct Request<'a> {
	name: &'a str
}

#[derive(Debug, Fail)]
enum MyError {
    #[fail(display = "Missing JSON from OpenWhisk")]
    MissingJSON
}

#[derive(Serialize, Deserialize)]
struct Response {
	date: DateTime<Utc> 
}


fn main_inner() -> Result<Response, Error>  {
    if let Some(arg) = env::args().nth(1) {
		let request: Request = serde_json::from_str(&arg)?;
        let rt = Runtime::new().unwrap();
        let date = rt.block_on(simple_created_date(request.name.to_string()))?;
        Ok(Response { date })
    } else {
        bail!(MyError::MissingJSON)
    }
}

fn main() {
    println!("{}", serde_json::to_string(&main_inner().expect("Error!")).expect("Failed to convert to JSON!"));
}
