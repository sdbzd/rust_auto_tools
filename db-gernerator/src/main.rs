#![cfg_attr(
    debug_assertions,
    allow(
        non_camel_case_types,
        dead_code,
        unused_imports,
        unused_variables,
        unused_mut,
        // unused_value,
        unused_imports,
        dead_code,
    )
)]
#![allow(dead_code, unused_imports)]
#![allow(unused_variables)] //允许未使用的变量
#![allow(unused_must_use)]
#![allow(non_snake_case)]
#![allow(unused_features)]
#![allow(unused_macros)]
#![allow(unused_must_use)]
// #![allow(unused_value)]
#![allow(dead_code)]
#[allow(dead_code)]
#[macro_use]
extern crate lazy_static;

extern crate rbatis;

mod codegen;
mod config;
// mod permission;
mod schema;
mod tmpl;
mod utils;
mod yaml_helper;
use crate::codegen::{CodeGenerator, GenerateContext};
use crate::config::AppConfig;

use chrono::NaiveDateTime;
use rbatis::RbatisOption;
// use rbatis::db::DBPoolOptions;
use rbatis::rbatis::Rbatis;
use sqlx_sqlhelper::{common_fields, SqlHelper};
use std::time::Duration;

use std::sync::Arc;
// use poem_openapi::Object;
//#[actix_web::main]
// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
// async fn main() -> std::io::Result<()> {
// #[macro_use]

// extern crate rbatis;

// #[macro_use]

// extern crate lazy_static;

// use rbatis::rbatis::Rbatis;
lazy_static! {
    // Rbatis是线程、协程安全的，运行时的方法是Send+Sync，无需担心线程竞争
    pub static ref RB:Rbatis=Rbatis::new();
}
//fn main(){
#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    // 加载配置文件
    std::env::set_var("RUST_LOG", "rbatis=warn");
    match fast_log::init(fast_log::config::Config::new().console()) {
        Ok(_) => {}
        Err(err) => {
            log::info!("An error occurred on the Logger initializing. {}", err);
        }
    };

    let conf = std::env::args().nth(1);
    let conf_path = if conf.is_none() {
        std::env::current_dir()
            .unwrap()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_owned()
            + "/conf/rbatis.yml"
    } else {
        let conf_base = conf.unwrap();
        if conf_base.starts_with("/") {
            conf_base
        } else {
            std::env::current_dir()
                .unwrap()
                .as_os_str()
                .to_str()
                .unwrap()
                .to_owned()
                + "/"
                + conf_base.as_str()
        }
    };

    log::info!("Parsing rust-generator config file: {}", conf_path);

    // 加载配置信息
    let mut webc = AppConfig::get().lock().unwrap();
    webc.load_yaml(&conf_path.clone());
    log::info!("MySQL: {}", webc.mysql_conf.url);
    let conf = webc.clone();
    drop(webc);

    // let mut rcnn = AppConfig::redis();

    // match redis::cmd("GET").arg("h").query::<String>(&mut rcnn) {
    //     Ok(r) => {
    //         log::info!("Get the redis value: {}", r);
    //     }
    //     Err(err) => {
    //         log::info!("The error is {}", err.to_string());
    //     }
    // };

    let mut cgconf = conf.codegen_conf.clone();
    cgconf.database_url = conf.mysql_conf.url.clone();

    let ctx = GenerateContext::create(&cgconf.clone(), &conf.redis_conf);

    let mut cg = CodeGenerator::new(&ctx);

    // RB.link("mysql://postgres:postgres@localhost:3306/ruoyi_web")
    //     .await
    //     .unwrap();
    // let rb = Arc::new(&RB);

    cg.load_tables().await;
    cg.generate();
    cg.write_out();

    
    // yaml_helper::GenYaml().await;

    // cg.write_permission().await;

     std::thread::sleep_ms(500u32);
    Ok(())
}

pub async fn rd_init(link: String, max_connections: u32) -> bool {
    // let mut opt = RbatisOption::default();
    // //最小连接数
    // opt. = 1;
    // //连接池中允许的最大连接数 默认值为10
    // opt.max_connections = max_connections;
    // //等待连接池分配连接的最大时长 超时5秒
    // opt.connect_timeout = Duration::new(5, 0);
    // //连接的生命时长 超时而且没被使用则被释放（retired），缺省:30分钟，
    // opt.max_lifetime = Some(Duration::from_secs(1800));
    // //一个连接idle状态的最大时长（秒），超时则被释放（retired），缺省:10分钟
    // opt.idle_timeout = Some(Duration::new(600, 0));

    // match RB.link_opt(link.as_str(), opt).await {
    //     Ok(_t) => true,
    //     Err(_e) => false,
    // }
    true
}
