#![allow(non_snake_case)]
#![allow(unused_imports)]
extern crate iron;
extern crate hyper_native_tls;
extern crate router;

use std::result::Result;
use std::io::{Read,BufReader,Write,BufWriter};
use std::fs::File;
use std::error::*;
use std::fmt::{Debug, Display};
use std::collections::HashMap;

use router::Router;
use iron::prelude::*;
use iron::status;
use iron::{Iron, Request, Response,Handler};
use hyper_native_tls::NativeTlsServer;


extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Deserialize)]
struct HTTPServerConfig {
    interface: String,
    port: String,
    https:bool,
    certificate:Certificate
}
#[derive(Serialize, Deserialize)]
struct Certificate{
    filePath:String,
    passphrease:String,
}

/////////////////////////////////////////////////
#[derive(Debug)]
enum ConfigException{
    ParserException(serde_json::Error),
    IoException(std::io::Error),
}

impl From<serde_json::Error> for ConfigException {
    fn from(err: serde_json::Error) -> ConfigException {
        ConfigException::ParserException(err)
    }
}

impl From<std::io::Error> for ConfigException {
    fn from(err: std::io::Error) -> ConfigException {
        ConfigException::IoException(err)
    }
}

impl std::fmt::Display for ConfigException {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            // 下層のエラーは両方ともすでに `Display` を実装しているので、
            // それらの実装に従います。
            ConfigException::ParserException(ref err) => write!(f, "Parse Exception: {}", err),
            ConfigException::IoException(ref err) => write!(f, "IO Exception: {}", err),
        }
    }
}

impl std::error::Error for ConfigException {
    fn description(&self) -> &str {
        // 下層のエラーは両方ともすでに `Error` を実装しているので、
        // それらの実装に従います。
        match *self {
            ConfigException::ParserException(ref err) => err.description(),
            ConfigException::IoException(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            // 注意：これらは両方とも `err` を、その具象型（`&io::Error` か
            // `&num::ParseIntError` のいずれか）から、トレイトオブジェクト
            // `&Error` へ暗黙的にキャストします。どちらのエラー型も `Error` を
            // 実装しているので、問題なく動きます。
            ConfigException::ParserException(ref err) => Some(err),
            ConfigException::IoException(ref err) => Some(err),
        }
    }
}
////////////////////////

fn generateConfig() -> Result<(), ConfigException> {
    let conf = HTTPServerConfig {
        interface: "::0".to_owned(),
        port: "443".to_owned(),
        https:true,
        certificate: Certificate{
            filePath:"identity.p12".to_owned(),
            passphrease:"password".to_owned()
        }
    };
    // jsonファイルを保存
    let json = serde_json::to_string(&conf)?;
    let mut writer = BufWriter::new(File::create("serverconfig.json")?);
    let json = json.into_bytes();
    writer.write_all(& json)?;
    
    Ok(())
}

fn loadConfig()-> Result<HTTPServerConfig, ConfigException >{
    let mut reader = BufReader::new(File::open("serverconfig.json")?);
    let mut json = String::new();
    reader.read_to_string(&mut json)?;

    let config: HTTPServerConfig = match serde_json::from_str(&json){
        Ok(config)=>config,
        Err(e)=>{
            let mut writer = BufWriter::new(File::create("serverconfig.json.old")?);
            let oldJson=json.to_string();
            writer.write_all(&oldJson.into_bytes())?;
            return Err(std::convert::From::from(e));
        }
    };
    Ok(config)
}

fn main(){
    let conf = match loadConfig(){
        Ok(r)=>r,
        Err(e)=>{
            println!("設定ファイルの読み込みに失敗したため、新しく作成します。\n古いファイルは保持されています。\n ==>{}",e.description());
            match generateConfig(){
                Ok(_)=>{},
                Err(e)=>{
                    println!("設定ファイルの生成に一部失敗しました。\n詳細：{}",e.description())
                }
            }
            println!("設定ファイルを作成しました。");
            return;
        }
    };
    println!("{}",conf.certificate.filePath);
    runHttpsServer(&conf);
}

fn index(req: &mut Request) -> IronResult<Response> {
    let ref query = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");

    Ok(Response::with((status::Ok, "index")))
}

fn query(req: &mut Request) -> IronResult<Response> {
    let ref q1 = req.extensions.get::<Router>().unwrap().find("query").unwrap_or("/");
        let ref q2 = req.extensions.get::<Router>().unwrap().find("q1").unwrap_or("/");
    let res=format!("{}:{}",*q1,*q2);
    Ok(Response::with((status::Ok, res)))
}

fn runHttpsServer(conf:&HTTPServerConfig) {
    let bindAddr=format!("{}:{}", conf.interface, conf.port);

    let ssl = NativeTlsServer::new(
        &conf.certificate.filePath,
        &conf.certificate.passphrease).unwrap();

    let mut router = Router::new();
    // router , method , router_id
    router.get("/", index, "index");
    router.get("/req/:query/:q1", query, "query");

    match Iron::new(router).https(&bindAddr, ssl) {
        Ok(listening) => println!("{:?}", listening),
        Err(err) => panic!("{:?}", err),
    }
}
