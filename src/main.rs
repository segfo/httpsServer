#![allow(non_snake_case)]
extern crate iron;
extern crate hyper_native_tls;
extern crate router;

use std::io::{BufReader,BufWriter};
use std::fs::File;
use std::error::*;

use router::Router;
use iron::prelude::*;
use iron::status;
use iron::{Iron, Request, Response};
use iron::response::HttpResponse;
use hyper_native_tls::NativeTlsServer;
mod segfo;
use segfo::configure::Config::ServerConfig;

extern crate serde_json;
#[macro_use]
extern crate serde_derive;

fn main(){
    let conf = match ServerConfig::new(){
        Ok(conf)=>{
            match conf.loadConfig(){
                Ok(r)=>r,
                Err(e)=>{
                    println!("設定ファイルの読み込みに失敗したため、新しく作成します。\n古いファイルは保持されています。\n ==>{}",e.description());
                    match conf.generateConfig(){
                        Ok(_)=>{},
                        Err(e)=>{
                            println!("設定ファイルの生成に一部失敗しました。\n ==>{}",e.description())
                        }
                    }
                    println!("設定ファイルを作成しました。");
                    return;
                }
            }
        },
        Err(e)=>{
            println!("致命的なエラー：設定ファイル構造体を初期化できませんでした。\n ==>{}",e.description());
            return;
        }
    };
    if conf.https {
        runHttpsServer(&conf);
    }else{
        runHttpServer(&conf);
    }
}

fn index(req: &mut Request) -> IronResult<Response> {
    let ref q1 = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");
    Ok(Response::with((status::Ok, "index")))
}

fn query(req: &mut Request) -> IronResult<Response> {
    let ref q1 = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");
    let ref q2 = req.extensions.get::<Router>().unwrap().find("q1").unwrap_or("/");
    let res=format!("{}:{}",*q1,*q2);
    Ok(Response::with((status::Ok, res)))
}


fn runHttpServer(conf:&ServerConfig) {
    let bindAddr=format!("{}:{}", conf.interface, conf.port);
    let mut router = Router::new();

    println!("httpサーバ：動作開始(非推奨)");

    // router , method , router_id
    router.get("/", index, "index");
    router.get("/req/:query/:q1", query, "query");

    match Iron::new(router).http(&bindAddr) {
        Ok(listening) => println!("{:?}", listening),
        Err(err) => panic!("{:?}", err),
    }
}

fn runHttpsServer(conf:&ServerConfig) {
    let bindAddr=format!("{}:{}", conf.interface, conf.port);
    let ssl = NativeTlsServer::new(
        &conf.certificate.filePath,
        &conf.certificate.passphrase).unwrap();
    let mut router = Router::new();

    println!("httpsサーバ：動作開始");

    // router , method , router_id
    router.get("/", index, "index");
    router.get("/req/:query/:q1", query, "query");

    match Iron::new(router).https(&bindAddr, ssl) {
        Ok(listening) => println!("{:?}", listening),
        Err(err) => panic!("{:?}", err),
    }
}
