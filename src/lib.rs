#![recursion_limit="256"]
#![feature(async_await)]
use hyper::Client;

use chrono::{NaiveDateTime, DateTime, Utc};
use futures::future;
use hyper::StatusCode;
use futures::future::FutureExt;
use hyper::client::connect::Connect;
use hyper::client::HttpConnector;
use futures::compat::Future01CompatExt;
use futures::select;
use std::pin::Pin;

use hyper::client::Builder;
use hyper_tls::HttpsConnector;


use hyper::Response;
use hyper::Body;

use std::future::Future;
use failure::Error;


pub struct ExecutorCompat;

impl futures01::future::Executor<Box<dyn futures01::Future<Item = (), Error = ()> + Send + 'static>> for ExecutorCompat {
    fn execute(&self, future: Box<dyn futures01::Future<Item = (), Error = ()> + Send>) -> Result<(), futures01::future::ExecuteError<Box<dyn futures01::Future<Item = (), Error = ()> + Send>>> {
        tokio::spawn(future.compat().map(|r| r.unwrap()));
        Ok(())
    }
}

pub fn make_https() -> (Builder, HttpsConnector<HttpConnector>) {
    let https = HttpsConnector::new(4).unwrap();
    // TODO: re-enable keep-alive when Hyper is using std-futures tokio
    let mut builder = Client::builder();
    builder.keep_alive(false).executor(ExecutorCompat);
    (builder, https)
}

pub async fn simple_created_date(name: String) ->  Result<DateTime<Utc>, Error>  {
    let (builder, https) = make_https();
    let mut client = builder.build(https);
    Ok(created_date(&mut client, name).await?)
}


// Based on https://gist.github.com/jomo/be7dbb5228187edbb993
pub async fn created_date<T: Connect + Sync + 'static>(client: &mut Client<T>, name: String) -> Result<DateTime<Utc>, Error> {
    let mut start = 1263146630; // notch sign-up;
    let mut end = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();

    let check = |name, time| {
        println!("Checking: {:?}", time);
        let boxed: Box<dyn Send + Future<Output = Result<Response<Body>, Error>>> = Box::new(client
            .get(format!("https://api.mojang.com/users/profiles/minecraft/{}?at={}", name, time).parse().unwrap())
            .compat().map(|r| r.map_err(|e| Error::from_boxed_compat(Box::new(e) as Box<dyn std::error::Error + Send + Sync>))));
        Pin::from(boxed)
    };

    let calc_mid = |start, end| {
        start + ((end - start) / 2)
    };

    let mut cur_fut = check(&name, calc_mid(start, end)).fuse();

    loop {
        if start == end {
            println!("Finished!");
            return Ok(DateTime::from_utc(NaiveDateTime::from_timestamp(start as i64, 0), Utc))
        }
        let mid = calc_mid(start, end);
        let mut left_fut = check(&name, calc_mid(start, mid)).fuse();
        let mut right_fut = check(&name, calc_mid(mid + 1, end)).fuse();

        //let res = cur_fut.await?;


        let mut left_done = None;
        let mut right_done = None;

        loop {
            select! {
                cur_res = cur_fut => {
                    if cur_res?.status() == StatusCode::OK {
                        end = mid;
                        cur_fut = left_done.unwrap_or(left_fut);
                    } else {
                        start = mid + 1;
                        cur_fut = right_done.unwrap_or(right_fut);
                    }
                    println!("Resolved: {}", calc_mid(start, end));
                    break;
                },
                left_res = left_fut => {
                    println!("Got left future!");
                    left_done = Some(Pin::from(Box::new(future::ready(left_res)) as Box<dyn Send + Future<Output = _>> ).fuse());
                }
                right_res = right_fut => {
                    println!("Got right future!");
                    right_done = Some(Pin::from(Box::new(future::ready(right_res)) as Box<dyn Send + Future<Output = _>> ).fuse());
                }
            }
        }
    }
}
