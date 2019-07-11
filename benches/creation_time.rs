#![feature(async_await)]

#[macro_use]
extern crate criterion;

use tokio::runtime::Runtime;
use hyper_tls::HttpsConnector;
use hyper::Client;


use mc_verifier::created_date;
use mc_verifier::ExecutorCompat;

use criterion::Criterion;
use criterion::black_box;

use std::rc::Rc;
use std::cell::RefCell;

fn criterion_benchmark(c: &mut Criterion) {
    let mut rt = Runtime::new().unwrap();

    rt.block_on(async {

        let https = HttpsConnector::new(4).unwrap();
        // TODO: re-enable keep-alive when Hyper is using std-futures tokio
        let mut client = Client::builder().keep_alive(false).executor(ExecutorCompat).build::<_, hyper::Body>(https);


        println!("Creation date: {:?}", created_date(&mut client, "Aaron1011".to_string()).await);
    });

    /*c.bench_function("created_date", move |b| b.iter(|| {
        let date = black_box(;
    }));*/
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
