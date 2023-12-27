use super::schema::{ColumnInfo, TableInfo};
use fast_log::print;
// use rbatis::crud::{Skip, CRUD};
// use rbatis::crud_table;
use rbatis::error::Error;
use rbatis::rbatis::Rbatis;
use rbatis::sql::{Page,PageRequest};
// use rbatis::PageRequest;
use rbson::Bson;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use sqlx::pool::PoolConnection;
// use serde_derive::{Deserialize, Serialize};
use super::utils;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::string;
// use super::config::get_rbatis;
use super::config;
use crate::config::{get_rbatis, TableConfig};
use crate::RB;
extern crate rbatis;
use regex::Regex;
use sqlx::MySqlConnection;
use sqlx::Row;
use sqlx::{mysql::MySqlColumn, mysql::MySqlPool, mysql::MySqlRow, Pool};
use std::fs::File;
use std::io::Write;
// use sqlx::{mysql::MySqlPool, Pool};
pub async fn init_db() -> Rbatis {
    let rb = Rbatis::new();
    // ------------choose database driver------------
    // rb.init(rbdc_mysql::driver::MysqlDriver {}, "mysql://root:123456@localhost:3306/test").unwrap();
    // rb.init(rbdc_pg::driver::PgDriver {}, "postgres://postgres:123456@localhost:5432/postgres").unwrap();
    // rb.init(rbdc_mssql::driver::MssqlDriver {}, "mssql://SA:TestPass!123456@localhost:1433/test").unwrap();
    // rb.init(
    //     rbdc_mysql::driver::MysqlDriver {},
    //     "mysql://postgres:postgres@localhost:3306/ruoyi_web",
    // )
    // .unwrap();

    // ------------create tables way 2------------
    // let sql = std::fs::read_to_string("example/table_sqlite.sql").unwrap();
    // let raw = fast_log::LOGGER.get_level().clone();
    // fast_log::LOGGER.set_level(LevelFilter::Off);
    // let _ = rb.exec(&sql, vec![]).await;
    // fast_log::LOGGER.set_level(raw);
    // ------------create tables way 2 end------------
    return rb;
}
// lazy_static! {
//     // Rbatis is thread-safe, and the runtime method is Send+Sync. there is no need to worry about thread contention
//     static ref RB:Rbatis=Rbatis::new();
//   }

// #[crud_table(table_name:"TABLES" | table_columns:"id,name,delete_flag" | formats_pg:"id:{}::uuid")]
// #[derive(Clone, Debug)]
// pub struct BizActivity {
//     //  pub id: Option<String>,
//     pub table_name: Option<String>,
//     pub table_comment: Option<String>,
// }
pub async fn GenYaml()->Vec<TableInfo> {
    // You have some type.
    // let mut map = BTreeMap::new();
    // map.insert("x".to_string(), 1.0);
    // map.insert("y".to_string(), 2.0);

    // // Serialize it to a YAML string.
    // let yaml = serde_yaml::to_string(&map).unwrap();
    //    assert_eq!(yaml, "x: 1.0\ny: 2.0\n");

    // Deserialize it back to a Rust type.
    // let deserialized_map: BTreeMap<String, f64> = serde_yaml::from_str(&yaml).unwrap();
    // let conf = std::env::args().nth(1);
    // let conf_path = if conf.is_none() {
    //     std::env::current_dir().unwrap().as_os_str().to_str().unwrap().to_owned() + "/conf/rbatis.yml"
    // } else {
    //     let conf_base = conf.unwrap();
    //     if conf_base.starts_with("/") {
    //         conf_base
    //     } else {
    //         std::env::current_dir().unwrap().as_os_str().to_str().unwrap().to_owned() + "/" + conf_base.as_str()
    //     }
    // };
    // let rb = Rbatis::new();
    // rb.link("mysql://postgres:postgres@localhost:3306/ruoyi_web");
    // / connect to database
    // sqlite
    // RB.link( "mysql://postgres:postgres@localhost:3306/ruoyi_web");//.await().unwrap();
    // let rb1 = config::get_rbatis();
    // let mut tbi =   TableInfo::default();
    // let tbls = TableInfo::load_tables_by_schema(&RB, "ruoyi_web");
    //   let rb = Rbatis::new();
    // log.info("{}","ready");

    //   let rb = Rbatis::new();
    //  rb.link("mysql://postgres:postgres@localhost:3306/ruoyi_web")
    //      .await
    //      .unwrap();

    //  // Execute a query to retrieve all table names and their schemas
    //  let tables: Vec<(String, String)> = rb.fetch_list("SELECT table_schema, table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_schema = 'ruoyi_web'")
    //      .await
    //      .unwrap()
    //      .into_iter()
    //      .map(|table| (table["table_schema"].to_owned(), table["table_name"].to_owned()))
    //      .collect();

    //  // Print out the table names and their schemas
    //  println!("Tables in your_database_name:");
    //  for (schema, table_name) in tables {
    //      println!("- {}.{}", schema, table_name);
    //  }
    // let rb = Rbatis::new();
    // rb.link("mysql://postgres:postgres@localhost:3306/ruoyi_web")
    //     .await
    //     .unwrap();

    let database_url = "mysql://postgres:postgres@localhost:3306/ruoyi_web";

    let pool = MySqlPool::connect(database_url).await.unwrap();
    //

    // let pool =MySqlPoolOptions::new()
    //     .max_connections(5)
    //     .connect("postgres://postgres:password@localhost/test").await?;

    let tables = read_table_names(&pool, database_url).await;

    let colums = read_table_columns(&pool, &tables).await;
    
    // write_table_names(&tables);
    // let mut list: Vec<BTreeMap<String,String>> = Vec::new();
    let mut list = Vec::new();
    for table in tables {
        let mut pri_key = "".to_string();
        let mut fileds = "".to_string();
        println!("table result: {:#?}", table.0.to_string());

        let intersect = colums
            .iter()
            .filter(|u| u.table_name == table.0)
            .collect::<Vec<_>>();
        // println!("column result: {:#?}", intersect);
        let files_columns = intersect
            .iter()
            .filter(|u| u.table_name == table.0)
            .collect::<Vec<_>>();

        let pri_keys = intersect
            .iter()
            .filter(|u| u.table_name == table.0 && u.column_key == "PRI")
            .collect::<Vec<_>>();

        for column in files_columns {
            fileds.push_str(&",".to_string());
            fileds.push_str(&column.column_name);
            //  pri_key.push_str(&key.column_name);
        }
        // println!("fields result: {:#?}", fileds.trim_matches(','));

        for key in pri_keys {
            // concat!(pri_key, "-", "key");
            pri_key.push_str(&key.column_name);
            pri_key.push_str(&",".to_string());
        }
        // println!("primary keys result: {:#?}", pri_key.trim_matches(','));
        // for key in pri_keys
        // {
        //     // concat!(pri_key, "-", "key");
        //      fileds.push_str(&key.column_name);
        // }

        let mut  tbi = TableInfo::default();

        tbi.table_comment= Some(table.1.to_string());
        tbi.table_name = Some(table.0.clone());  

        list.push(tbi);

     
            
    //    return retu;rn list;

        // let mut map = BTreeMap::new();
        // map.insert("name".to_string(), table.0.clone());
        // map.insert("comment".to_string(), table.1.to_string());
        // map.insert("struct-name".to_string(), snake_to_camel_case(&table.0.to_string()));
        // map.insert("tree-parent-field".to_string(), String::new());
        // map.insert("all-field-option".to_string(),true.to_string());
        // map.insert("auto-generate-key".to_string(), String::new());
        // map.insert("update-skip-fields".to_string(), String::new());
        // map.insert("update-seletive".to_string(), true.to_string());
        // map.insert("page-query".to_string(), true.to_string());
        // map.insert("logic-deletion".to_string(), true.to_string());
        // map.insert("api-handler-name".to_string(), table.0.clone());

        // println!("result {:#?}", map);
        // list.push(map);
    }

    list
    // println!("result: {:#?}", map);
    // let yaml = serde_yaml::to_string(&list).unwrap();
    // println!("result: {:#?}", yaml);


    // let mut file = File::create("table_names.txt").unwrap();

    // // Write the table names and their schemas to the file
   
    // writeln!(file, "{}", yaml).unwrap();
   
    //    // 取交集
    //    let intersect = a.iter().filter(|&u| b.contains(u)).collect::<Vec<_>>();
    //    println!("a 和 b 交集是：{:?}", intersect);

    //     // 取交集
    // let intersect = a.iter().filter(|&u| b.contains(u)).collect::<Vec<_>>();
    // println!("a 和 b 交集是：{:?}", intersect);

    // // 取差集
    // let minusion = a.iter().filter(|&u| !b.contains(u)).collect::<Vec<_>>();
    // println!("a 和 b 差集是：{:?}", minusion);

    // // 取并集
    // let union = a
    //     .iter()
    //     .filter(|&u| !b.contains(u))
    //     .chain(&b)
    //     .collect::<Vec<_>>();
    // println!("a 和 b 并集是：{:?}", union);

    // // 取补集
    // let complement = a
    //     .iter()
    //     .filter(|&u| !b.contains(u))
    //     .chain(b.iter().filter(|&u| !a.contains(u)))
    //     .collect::<Vec<_>>();
    // println!("a 和 b 补集是：{:?}", complement);
    // write_table_names(&colums);
}

pub async fn read_table_names(pool: &MySqlPool, database_url: &str) -> Vec<(String, String)> {
    let mut conn = pool.acquire().await.unwrap();
    let rows = sqlx::query("SELECT table_name,table_comment FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_schema = 'ruoyi_web'")
        .fetch_all( &mut conn )
        .await
        .unwrap();

    // let row: (i64,) = sqlx::query_as("SELECT $1")
    //     .bind(150_i64)
    //     .fetch_one(&pool).await?;
    //  rows.into_iter()
    //      .map(|row| (row.try_get(0), row.try_get(1)))
    //      .collect()

    rows.into_iter()
        .map(|row| (row.try_get(0).unwrap(), row.try_get(1).unwrap()))
        .collect()
}

pub async  fn read_table_columns(pool: &MySqlPool, tables: &[(String, String)]) -> Vec<RawRawData> {
    // let pool = MySqlPool::connect(database_url).await.unwrap();
    let mut conn = pool.acquire().await.unwrap();

    let query = "SELECT table_schema,table_name,   column_name as column_name, column_type as column_type, column_comment as column_comment, column_key as column_key,
        column_default as column_default, data_type as data_type, ordinal_position as ordinal_position, character_maximum_length as character_maximum_length,
        is_nullable as is_nullable, numeric_precision as numeric_precision, numeric_scale as numeric_scale, extra as extra
        FROM INFORMATION_SCHEMA.COLUMNS WHERE table_schema = ? "; //and table_name = ?

    // let rows = sqlx::query(query)
    //     .bind("ruoyi_web")
    //     .bind("sys_notice")
    //     .fetch_all( &mut conn)
    //     .await
    //     .unwrap();
    let rows = sqlx::query_as::<_, RawRawData>(query)
        .bind("ruoyi_web")
        .fetch_all(&mut conn)
        .await
        .unwrap();

    // let rows = fetch_all_rows(pool, "sys_notice").await;

    if rows.is_empty() {
        println!("No rows found");
        //  return  ;
    }

    rows
    // rows.get::<i32, _>(0);
    // let name = rows.get::<String, _>("column_name");
    // println!(" rows found{}", name);
    // rows.get::<f64, _>(2);
    // println!("{:#?}", rows);
    // for row in rows {
    //     // let columns = row.columns();
    //     // for column in columns {
    //     //     //   print!("{:#?}", column);
    //     // }

    //    let t = row as RawRawData;

    // //    println!("comment{:#?}", t.column_comment);
    // //    println!("comment{:#?}", t.column_comment);
    // //    println!("comment{:#?}", t.column_comment);
    // //    println!("comment{:#?}", t.column_comment);
    // //    println!("comment{:#?}", t.column_comment);
    // //    println!("comment{:#?}", t.column_comment);
    // //    - name: chimes_dept
    // //    comment: 部门
    // //    struct-name: ChimesDeptInfo
    // //    primary-key: dept_id
    // //    tree-parent-field: pid
    // //    all-field-option: true
    // //    auto-generate-key: dept_id
    // //    update-skip-fields: dept_id, create_time, update_time
    // //    update-seletive: true
    // //    page-query: true
    // //    logic-deletion: true
    // //    api-handler-name: dept
    //    // for column in row.columns() {
    //         // Get the name of the column
    //         // let column_name = column.name();

    //         // Get the value of the column in the current row
    //         // let column_value = row.get::<MySqlColumn, _>(&column.name());

    //         // println!("{}: {:?}",  row.column_name, "column_value");
    //   //  }
    // }
    // for row in &mut rows {
    //     let columns = row.columns();

    //     println!("{}",columns[0].name());

    // for column in columns {
    //     println!("Column name: {}", column.name());
    //     println!("Column type: {:?}", column.type_info());
    //     println!("Column table: {:?}", column.table());
    // }
    // let column_names = row
    //     .columns()
    //     .iter()
    //     .map(|column| column.name().clone())
    //     .collect::<Vec<String>>();
    // let column_values: Vec<Option<String>> = (0..row.len())
    //     .map(|i| row.try_get(i))
    //     .collect::<Result<_, _>>()
    //     .unwrap();
    // for (name, value) in column_names.into_iter().zip(column_values.into_iter()) {
    //     println!("{}: {:?}", name, value);
    // }
    // }

    //     for row in &mut rows {
    //         let column_names = row
    //             .columns()
    //             .iter()
    //             .map(|column| column.name().to_owned())
    //             .collect::<Vec<String>>();
    //         let column_values: Vec<Option<String>> = (0..row.len())
    //             .map(|i| row.try_get(i))
    //             .collect::<Result<_, _>>()
    //             .unwrap();
    //         for (name, value) in column_names.into_iter().zip(column_values.into_iter()) {
    //             println!("{}: {:?}", name, value);
    //         }
    // }
    // Get the column names
    // let column_names: Vec<String> = rows[0]
    //     .columns()
    //     .iter()
    //     .map(|c| c.name.to_owned())
    //     .collect();

    // // Print the column names
    // for name in &column_names {
    //     print!("{:<20}", name);
    // }
    // println!();
    // for row in rows {
    //     // let column_names = row
    //     //     .columns()
    //     //     .into_iter()
    //     //     .map(|column| column.name().to_owned())
    //     //     .collect::<Vec<String>>();
    //     // let column_values: Vec<Option<String>> = (0..row.len()).map(|i| row.get(i)).collect();
    //     let column_names = row.columns().into_iter().map(|column| column.name().to_owned()).collect::<Vec<String>>();
    //     // let column_values: Vec<Option<String>> = (0..row.len()).map(|i| row.get::<Option<String>, _>(i)).collect();

    //     // for (name, value) in column_names.into_iter().zip(column_values.into_iter()) {
    //     //     println!("{}: {:?}", name, value);
    //     // }
    // }
    // for row in rows {
    //      let columns = row.columns_ref();
    //     for i in 0..columns.len() {
    //         let column_name = row.columns[i].name();
    //         let column_value: Option<String> = row.try_get(i).unwrap_or(None);
    //         println!("{}: {:?}", column_name, column_value);
    //     }
    // }
    //  rows.into_iter()
    //      .map(|row| (row.try_get(0), row.try_get(1)))
    //      .collect()

    // rows.into_iter()
    //     .map(|row| (row.try_get(0).unwrap(), row.try_get(1).unwrap()))
    //     .collect()

    // for row in rows {
    //     for (column_name, column_value) in row.columns() {
    //         println!("values:{}: {}", column_name, column_value);
    //     }
    // }
}

fn snake_to_camel_case(input: &str) -> String {
    let re = Regex::new(r"(?:^|_)([a-z])").unwrap();
    let result = re.replace_all(input, |caps: &regex::Captures| caps[1].to_ascii_uppercase());
    result.to_string()
}

fn camel_to_snake_case(input: &str) -> String {
    let re = Regex::new(r"(?<!^)[A-Z]").unwrap();
    let result = re.replace_all(input, |caps: &regex::Captures| {
        format!("_{}", caps[0].to_ascii_lowercase())
    });
    result.to_string()
}
#[derive(Debug, sqlx::FromRow,Serialize,Deserialize)]
pub struct RawRawData {
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub column_type: String,
    pub column_comment: String,
    pub column_key: String,
    pub data_type: String,
}

pub async fn fetch_all_rows(pool: &MySqlPool, table_name: &str) -> Vec<MySqlRow> {
    let mut conn = pool.acquire().await.unwrap();

    //let query = format!("SELECT * FROM {}", table_name);
    let query = "SELECT table_schema as table_schema, table_name as table_name,  column_name as column_name, column_type as column_type, column_comment as column_comment, column_key as column_key,
        column_default as column_default, data_type as data_type, ordinal_position as ordinal_position, character_maximum_length as character_maximum_length, 
        is_nullable as is_nullable, numeric_precision as numeric_precision, numeric_scale as numeric_scale, extra as extra
        FROM INFORMATION_SCHEMA.COLUMNS WHERE table_schema = ? and table_name = ?";
    let rows = sqlx::query(&query)
        .bind("ruoyi_web")
        .bind(table_name)
        .fetch_all(&mut conn)
        .await
        .unwrap();

    rows
}

pub fn write_table_names(tables: &[(String, String)]) {
    //  let mut file = std::fs::File::create("tables.txt").unwrap();

    for (schema, table) in tables {
        //   writeln!(file, "{}.{}", schema, table).unwrap();
        // console.writeln!( "{}.{}", schema, table).unwrap();
        println!("{}.{}", schema, table);

        // let mut map = BTreeMap::new();
        // // map.insert("name".to_string(), t.column_name.clone());
        // // map.insert("comment".to_string(), t.column_comment);
        // // map.insert("struct-name".to_string(), t.column_name.clone());
        // // map.insert("tree-parent-field".to_string(),String::new() );
        // // map.insert("all-field-option".to_string(), String::new());
        // // map.insert("auto-generate-key".to_string(), String::new());
        // // map.insert("update-skip-fields".to_string(), String::new());
        // // map.insert("update-seletive".to_string(), "true".to_string());
        // // map.insert("page-query".to_string(), "true".to_string());
        // // map.insert("logic-deletion".to_string(), "true".to_string());
        // // map.insert("api-handler-name".to_string(), t.column_name);

        // println!("result {:#?}", map);
    }
}

pub fn write_table_columns(tables: &[(String, String)]) {
    //  let mut file = std::fs::File::create("tables.txt").unwrap();

    for (schema, table) in tables {
        //   writeln!(file, "{}.{}", schema, table).unwrap();
        // console.writeln!( "{}.{}", schema, table).unwrap();
        println!("{}---{}", schema, table);
    }
}

// pub async fn read_table_names(rb: &Rbatis) -> Vec<(String, String)> {
//     // Create a new Rbatis instance and configure it to connect to your MySQL database

//     // Execute a query to retrieve all table names and their schemas
//     let query = "SELECT table_schema, table_name FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_schema = 'ruoyi_web'";
//     rb.fetch_list(&query)
//         .await
//         .unwrap()
//         .into_iter()
//         .map(|table| (table["table_schema"].to_owned(), table["table_name"].to_owned()))
//         .collect()
// }

// pub fn write_table_names(tables: &[(String, String)]) {
//     // Open a new file for writing
//     let mut file = File::create("table_names.txt").unwrap();

//     // Write the table names and their schemas to the file
//     for (schema, table_name) in tables {
//         writeln!(file, "{}.{}", schema, table_name).unwrap();
//     }
// }

fn write(file: &mut String) {
    //    for ln in self.annotations.clone() {
    //        ro.write_line(&ln);
    //    }
    //    if self.is_pub {
    //        ro.write_line(&format!("pub struct {} {{", self.struct_name.clone()));
    //    } else {
    //  ro.write_line(&format!("struct {} {{", file));
    //    }
}

// pub async fn link_db(&self) {
//    //连接数据库
//    println!("[abs_admin] rbatis connect database ({})...", self.config.database_url);
//    self.rbatis
//        .link(&self.config.database_url)
//        .await
//        .expect("[abs_admin] rbatis connect database fail!");
//    println!("[abs_admin] rbatis connect database success!");
//    log::info!(
//    " - Local:   http://{}",
//    self.config.server_url.replace("0.0.0.0", "127.0.0.1")
// );
// }


// SELECT table_name,table_comment,
// (SELECT count(COLUMN_NAME ) FROM information_schema.COLUMNS c WHERE  c.table_schema = 'ruoyi_web' AND c.COLUMN_KEY='PRI' AND c.TABLE_NAME=t.table_name  ) AS column_key_count,
// (SELECT group_concat(COLUMN_NAME SEPARATOR '-') FROM information_schema.COLUMNS c WHERE  c.table_schema = 'ruoyi_web' AND c.COLUMN_KEY='PRI' AND c.TABLE_NAME=t.table_name  ) AS column_key,
// (SELECT group_concat(COLUMN_NAME SEPARATOR ',') FROM information_schema.COLUMNS c WHERE  c.table_schema = 'ruoyi_web'  AND c.TABLE_NAME=t.table_name  ) AS fields
//  FROM information_schema.tables  t WHERE  t.table_type = 'BASE TABLE' AND t.table_schema = 'ruoyi_web'
 