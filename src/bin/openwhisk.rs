#![feature(async_await)]

#[macro_use] extern crate failure;

use std::env;
use serde_json;
use serde::{Serialize, Deserialize};
use chrono::{NaiveDateTime, DateTime, Utc};
use failure::Error;
use tokio::runtime::Runtime;

use rusoto_s3::S3Client;
use rusoto_s3::{PutObjectRequest, GetObjectRequest, GetObjectError};
use rusoto_s3::S3;
use rusoto_credential::StaticProvider;
use rusoto_core::RusotoError;
use rusoto_core::Region;
use rusoto_core::request::HttpClient;

use futures::stream::{StreamExt, Stream};

use futures::compat::{Stream01CompatExt, Future01CompatExt};

use mc_creation_date::{make_https, simple_created_date};

#[derive(Serialize, Deserialize)]
struct Request {
	name: String,
    cos_endpoint: String,
    bucket: String,
    __bx_creds: Creds
}

#[derive(Serialize, Deserialize)]
struct Creds {
    #[serde(rename = "cloud-object-storage")]
    cloud_object_storage: CloudCreds
}

#[derive(Serialize, Deserialize)]
struct CloudCreds {
    apikey: String,
    cos_hmac_keys: HMACKeys
}

#[derive(Serialize, Deserialize)]
struct HMACKeys {
    access_key_id: String,
    secret_access_key: String
}


#[derive(Debug, Fail)]
enum MyError {
    #[fail(display = "Missing JSON from OpenWhisk")]
    MissingJSON
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
	date: DateTime<Utc> 
}


fn main_inner() -> Result<Response, Error>  {
    if let Some(arg) = env::args().nth(1) {
		let request: Request = serde_json::from_str(&arg)?;
        let rt = Runtime::new().unwrap();

        let ibm_region = Region::Custom {
            name: "us-south".to_string(),
            endpoint: request.cos_endpoint.to_string()
        };

        let hmac = &request.__bx_creds.cloud_object_storage.cos_hmac_keys;

        let creds = StaticProvider::new(hmac.access_key_id.to_string(), hmac.secret_access_key.to_string(), None, None);


        let (builder, https) = make_https();

        let http_client = HttpClient::from_builder(builder, https);
        let s3 = S3Client::new_with(http_client, creds, ibm_region);

        let key = request.name.to_string();

        let response: Result<Response, failure::Error> = rt.block_on(async move {
            match s3.get_object(GetObjectRequest {
                bucket: request.bucket.to_string(),
                key: key.clone(),
                ..Default::default()
            }).compat().await {
                Ok(output) => {
                    let res: Vec<u8> = output.body.unwrap().compat().map(|b| b.unwrap().as_ref().to_owned()).concat().await;
                    let response: Response = serde_json::from_str(&String::from_utf8(res)?)?;
                    println!("Got cached response: {:?}", response);
                    Ok(response)
                },
                Err(RusotoError::Service(GetObjectError::NoSuchKey(_))) => {
                    let response = Response { date: simple_created_date(request.name.to_string()).await? };
                    let encoded = serde_json::to_string(&response)?;

                    s3.put_object(PutObjectRequest {
                        bucket: request.bucket.to_string(),
                        key: key.clone(),
                        body: Some(encoded.into_bytes().into()),
                        ..Default::default()
                    }).compat().await?;

                    println!("Saved new response: {:?}", response);

                    Ok(response)
                },
                Err(e) => Err(e)?
            }
        });
        response
    } else {
        bail!(MyError::MissingJSON)
    }
}

fn main() {
    std::env::set_var("RUST_BACKTRACE", "1");
    println!("{}", serde_json::to_string(&main_inner().expect("Error!")).expect("Failed to convert to JSON!"));
}
