#![feature(async_await)]


use tokio::runtime::Runtime;
use hyper_tls::HttpsConnector;
use hyper::Client;


use mc_creation_date::simple_created_date;
use mc_creation_date::ExecutorCompat;



fn main() {
    let rt = Runtime::new().unwrap();

    let name = std::env::args().nth(1).unwrap_or_else(|| "Aaorn1011".to_string());

    let date = rt.block_on(simple_created_date(name));

	println!("Date: {:?}", date);

}
