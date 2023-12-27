use log::kv::ToValue;
use rbatis::error::Error;
use rbatis::rbatis::Rbatis;
use rbdc::datetime::FastDateTime ;//DateTimeNative;
use rbson::Bson;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

use super::RB;
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct TableInfo {
    pub table_catalog: Option<String>,
    pub table_schema: Option<String>,
    pub table_type: Option<String>,
    pub table_name: Option<String>,
    pub table_collation: Option<String>,
    pub table_comment: Option<String>,
    pub create_time: Option<FastDateTime>,
    pub update_time: Option<FastDateTime>,
}

impl TableInfo {
    pub async fn load_table(
        rb: &Rbatis,
        table_schema: &str,
        table_name: &str,
    ) -> Result<Option<TableInfo>, Error> {
        // log::info!("TS: {}, TN: {}", table_schema.clone(), table_name.clone());
        let mut rb_args = vec![];
        rb_args.push(rbs::to_value!(table_schema));
        rb_args.push(rbs::to_value!(table_name));

        return rb.query_decode(&
        "SELECT table_catalog as table_catalog, table_schema as table_schema, table_type as table_type, 
            table_name as table_name, table_collation as table_collation, table_comment as table_comment, 
            create_time as create_time, update_time as update_time
            FROM INFORMATION_SCHEMA.TABLES WHERE table_schema = ? and table_name = ?",
        rb_args).await ;
    }

    pub async fn load_tables_by_schema(
        rb: &Rbatis,
        table_schema: &str,
    ) -> Result<Option<TableInfo>, Error> {
        //  table_name:&str
        log::info!("TS: {}, TN: {}", table_schema.clone(), "table_name.clone()");
        let mut rb_args = vec![];
        rb_args.push(rbs::to_value!(table_schema));
        // rb_args.push(rbson::to_bson(table_name).unwrap_or_default()) ;

        return rb.query_decode(&
        "SELECT table_catalog as table_catalog, table_schema as table_schema, table_type as table_type, 
            table_name as table_name, table_collation as table_collation, table_comment as table_comment, 
            create_time as create_time, update_time as update_time
            FROM INFORMATION_SCHEMA.TABLES WHERE table_schema = ? ", //and table_name = ?
        rb_args).await ;
    }

    // pub async fn load_tables_coloums(rb:&Rbatis, table_schema:&str, table_name:&str) -> Result<Option<TableInfo>, Error> {
    //     // log::info!("TS: {}, TN: {}", table_schema.clone(), table_name.clone());
    //     let mut rb_args = vec! [] ;
    //     rb_args.push(rbson::to_bson(table_schema).unwrap_or_default()) ;
    //     rb_args.push(rbson::to_bson(table_name).unwrap_or_default()) ;

    //     return rb.fetch::<Option<TableInfo>>(&
    //     "SELECT
    //         TABLE_SCHEMA AS '库名',
    //         TABLE_NAME AS '表名',
    //         COLUMN_NAME AS '列名',
    //         ORDINAL_POSITION AS '列的排列顺序',
    //         COLUMN_DEFAULT AS '默认值',
    //         IS_NULLABLE AS '是否为空',
    //         DATA_TYPE AS '数据类型',
    //         CHARACTER_MAXIMUM_LENGTH AS '字符最大长度',
    //         NUMERIC_PRECISION AS '数值精度(最大位数)',
    //         NUMERIC_SCALE AS '小数精度',
    //         COLUMN_TYPE AS '列类型',
    //         COLUMN_KEY 'KEY',
    //         EXTRA AS '额外说明',
    //         COLUMN_COMMENT AS '注释'

    //     FROM
    //         information_schema.`COLUMNS`
    //     WHERE
    //         TABLE_SCHEMA = ?
    //     ORDER BY
    //         TABLE_NAME,
    //         ORDINAL_POSITION;",
    //     rb_args).await ;
    // }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ColumnInfo {
    pub table_schema: Option<String>,
    pub table_name: Option<String>,
    pub column_name: Option<String>,
    pub column_type: Option<String>,
    pub column_comment: Option<String>,
    pub column_key: Option<String>,
    pub column_default: Option<String>,
    pub data_type: Option<String>,
    pub extra: Option<String>,
    pub ordinal_position: Option<i32>,
    pub character_maximum_length: Option<i32>,
    pub is_nullable: Option<String>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
}

impl ColumnInfo {
    //#[sql("SELECT table_schema, table_name,  column_name, column_type, column_comment, column_key,
    //        column_default, data_type, ordinal_position, character_maximum_length, is_nullable, numeric_precision, numeric_scale,
    //        FROM INFORMATION_SCHEMA.COLUMNS WHERE table_schema = ? and table_name = ?")]
    pub async fn load_columns(rb: &Rbatis, ts: &str, tn: &str) -> Result<Vec<Self>, Error> {
        let mut rb_args = vec![];
        rb_args.push(rbs::to_value!(ts));
        rb_args.push(rbs::to_value!(tn));
        // rb.update_by_wrapper(table, w, skips);
        let con = redis::Client::open("");
        return rb.query_decode(&
        "SELECT table_schema as table_schema, table_name as table_name,  column_name as column_name, column_type as column_type, column_comment as column_comment, column_key as column_key,
        column_default as column_default, data_type as data_type, ordinal_position as ordinal_position, character_maximum_length as character_maximum_length, 
        is_nullable as is_nullable, numeric_precision as numeric_precision, numeric_scale as numeric_scale, extra as extra
        FROM INFORMATION_SCHEMA.COLUMNS WHERE table_schema = ? and table_name = ?",
        rb_args).await ;
    }
}
