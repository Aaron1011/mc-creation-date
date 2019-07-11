#![feature(async_await)]


use tokio::runtime::Runtime;
use hyper_tls::HttpsConnector;
use hyper::Client;


use mc_creation_date::created_date;
use mc_creation_date::ExecutorCompat;

fn main() {
    let rt = Runtime::new().unwrap();

    let name = std::env::args().nth(1).unwrap_or_else(|| "Aaorn1011".to_string());

    rt.block_on(async move {

        let https = HttpsConnector::new(4).unwrap();
        // TODO: re-enable keep-alive when Hyper is using std-futures tokio
        let mut client = Client::builder().keep_alive(false).executor(ExecutorCompat).build::<_, hyper::Body>(https);


        println!("Creation date: {:?}", created_date(&mut client, name).await);
    });

}
