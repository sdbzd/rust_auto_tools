use lazy_static::lazy_static;
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use std::env::var;


lazy_static! {
    pub static ref POOL: Pool<MySql> = MySqlPoolOptions::new()
        .max_connections(5)
        .connect_lazy(&format!(
            "mysql://{}:{}@{}/{}",
            var("db_user").expect("配置文件db_user错误"),
            var("db_pass").expect("配置文件db_pass错误"),
            var("db_host").expect("配置文件db_host错误"),
            var("db_name").expect("配置文件db_host错误"),
        ))
        .unwrap();
}
