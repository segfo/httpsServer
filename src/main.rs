#![allow(non_snake_case)]
extern crate iron;
extern crate hyper_native_tls;
extern crate router;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

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
use segfo::exception::Exception::ConfigException;

fn loadConfig()->Result<ServerConfig,ConfigException>{
    let conf = ServerConfig::new();
    match conf.loadConfig(){
        Err(e)=>{
            println!("設定ファイルの読み込みに失敗したため、新しく作成します。");
            println!("古いファイルは保持されています。\n");
            if let Err(e) = conf.storeConfig(){
                println!("設定ファイルの生成に一部失敗しました。({})",e.description());
            };
            println!("設定ファイルを作成しました。");
            println!(" ==>{}",e.description());
            Err(e)
        },
        Ok(conf)=>Ok(conf)
    }
}

fn main(){
    let conf = match loadConfig(){
        Ok(conf)=>conf,
        Err(_)=>return
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
    let ref q1 = req.extensions.get::<Router>().unwrap().find("para1").unwrap_or("/");
    let ref q2 = req.extensions.get::<Router>().unwrap().find("para2").unwrap_or("/");
    let res=format!("{}:{}",*q1,*q2);
    Ok(Response::with((status::Ok, res)))
}

fn setupServer()->iron::Iron<router::Router>{
    let mut router = Router::new();
    // router , method , router_id
    router.get("/", index, "index");
    router.get("/req/:para1/:para2", query, "query");
    Iron::new(router)
}

fn runHttpServer(conf:&ServerConfig) {
    println!("httpサーバ：動作開始(非推奨)");

    let bindAddr=format!("{}:{}", conf.interface, conf.port);
    match setupServer().http(&bindAddr) {
        Ok(listening) => println!("{:?}", listening),
        Err(err) => panic!("{:?}", err),
    }
}

fn runHttpsServer(conf:&ServerConfig) {
    let ssl = NativeTlsServer::new(
        &conf.certificate.filePath,
        &conf.certificate.passphrase).unwrap();

    println!("httpsサーバ：動作開始");

    let bindAddr=format!("{}:{}", conf.interface, conf.port);
    match setupServer().https(&bindAddr, ssl) {
        Ok(listening) => println!("{:?}", listening),
        Err(err) => panic!("{:?}", err),
    }
}
