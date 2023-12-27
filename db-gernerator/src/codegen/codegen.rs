use crate::codegen::parse_table_as_struct;
use crate::config::{
    get_rbatis, safe_struct_field_name, CodeGenConfig, QueryConfig, RedisConfig, RelationConfig,
    TableConfig,
};
// use crate::permission::ChimesPermissionInfo;
use crate::schema::{ColumnInfo, TableInfo};
use crate::tmpl::{format_conf_tmpl, format_redis_conf_tmpl};
use crate::yaml_helper::RawRawData;
use change_case::{pascal_case, snake_case};
use chrono::format::format;
use fast_log::print;
use rbatis::dark_std::sync::SyncBtreeMap;
use rbatis::py_sql;
use rbatis::rbatis::Rbatis;
use rbdc::db::Row;
use rbs::Error;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::{create_dir, create_dir_all, OpenOptions};
use std::io::{self, ErrorKind, Write};
use std::path::Path;
use std::path::PathBuf;
use substring::Substring;

use super::{
    execute_sql, generate_actix_handler_for_table, parse_query_as_file, parse_query_as_struct,
    parse_query_handler_as_file, parse_relation_as_file, parse_relation_handlers_as_file,
    parse_table_as_request_param_struct, parse_table_as_value_object_struct, parse_yaml_as_file,
};

pub trait CodeWriter {
    fn write(&self, ro: &mut RustOutput);
}

/**
 * 代码生成的上下文
 * 它的主要功能是解释配置文件，并根据配置文件来准备代码生成所需要对应的一些参数
 * 这些参数如：生成后所存放的路径等
 */
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GenerateContext {
    pub codegen_conf: CodeGenConfig,
    pub redis_conf: RedisConfig,
    pub tables: Vec<TableInfo>,
    pub columns: HashMap<String, Vec<ColumnInfo>>,
    pub structs: Vec<RustStruct>,
    pub queries: Vec<RustStruct>,
    pub permissions: Vec<RustPermission>,
}

impl GenerateContext {
    pub fn create(cgconf: &CodeGenConfig, redisconf: &RedisConfig) -> Self {
        Self {
            codegen_conf: cgconf.clone(),
            redis_conf: redisconf.clone(),
            tables: vec![],
            columns: HashMap::new(),
            structs: vec![],
            queries: vec![],
            permissions: vec![],
        }
    }

    pub fn get_root_path(&self) -> &str {
        self.codegen_conf.output_path.as_str()
    }

    pub fn get_entity_path(&self) -> String {
        self.get_root_path().to_owned() + "/entity"
    }

    pub fn get_controller_path(&self) -> String {
        self.get_root_path().to_owned() + "/controller"
    }

    pub fn get_facade_path(&self) -> String {
        self.get_root_path().to_owned() + "/facade"
    }

    pub fn is_all_entity_in_one_file(&self) -> bool {
        self.codegen_conf.entity_in_one_file
    }

    pub fn is_generate_lib(&self) -> bool {
        self.codegen_conf.generate_for_lib
    }

    pub fn add_struct(&mut self, st: &RustStruct) {
        self.structs.push(st.clone());
    }

    pub fn add_query(&mut self, st: &RustStruct) {
        self.queries.push(st.clone());
    }

    pub fn add_table(&mut self, tb: &TableInfo, cols: &Vec<ColumnInfo>) {
        if tb.table_name.is_some() {
            // log::info!("Add the table {} into the context.", tb.table_name.clone().unwrap_or_default());
            self.tables.push(tb.clone());
            self.columns
                .insert(tb.table_name.clone().unwrap(), cols.clone());
        }
    }

    pub fn add_permission(&mut self, tb: &TableInfo, funclist: &Vec<RustFunc>) {
        let tbname = tb.table_name.clone().unwrap_or_default();
        let tbc = self.get_table_conf(&tbname.clone()).unwrap();
        let alias = tbc.api_handler_name.clone();
        let name = if tbc.comment.is_empty() {
            let tbcmt = tb.table_comment.clone().unwrap();
            if tbcmt.trim().len() > 0 {
                tbcmt.trim().to_string()
            } else {
                tbc.struct_name.to_uppercase()
            }
        } else {
            tbc.comment.clone()
        };

        let mut children = vec![];
        for mk in funclist.clone() {
            if mk.api_method.is_some() {
                let child = RustPermission {
                    name: mk.comment.clone().unwrap_or_default(),
                    alias: mk.func_name.to_uppercase(),
                    service_id: self.codegen_conf.app_name.clone(),
                    module_id: alias.clone(),
                    api_pattern: mk.api_pattern.clone(),
                    api_method: mk.api_method.clone(),
                    api_bypass: Some("user".to_string()),
                    children: vec![],
                };
                children.push(child);
            }
        }

        let perm = RustPermission {
            name: name,
            alias: alias.to_uppercase(),
            service_id: self.codegen_conf.app_name.clone(),
            module_id: alias.clone(),
            api_pattern: Some(format!(
                "{}/{}",
                self.codegen_conf.api_handler_prefix.clone(),
                tbc.api_handler_name.clone()
            )),
            api_method: None,
            api_bypass: None,
            children: children,
        };
        self.permissions.push(perm);
    }

    pub fn add_permission_for_relation(&mut self, tb: &RelationConfig, funclist: &Vec<RustFunc>) {
        let alias = tb.api_handler_name.clone().unwrap_or_default();
        let name = if tb.comment.trim().is_empty() {
            tb.struct_name.to_uppercase()
        } else {
            tb.comment.clone()
        };

        let mut children = vec![];
        for mk in funclist.clone() {
            if mk.api_method.is_some() {
                let child = RustPermission {
                    name: mk.comment.clone().unwrap_or_default(),
                    alias: mk.func_name.to_uppercase(),
                    service_id: self.codegen_conf.app_name.clone(),
                    module_id: alias.clone(),
                    api_pattern: mk.api_pattern.clone(),
                    api_method: mk.api_method.clone(),
                    api_bypass: Some("user".to_string()),
                    children: vec![],
                };
                children.push(child);
            }
        }

        let perm = RustPermission {
            name: name,
            alias: alias.to_uppercase(),
            service_id: self.codegen_conf.app_name.clone(),
            module_id: alias.clone(),
            api_pattern: Some(format!(
                "{}/{}",
                self.codegen_conf.api_handler_prefix.clone(),
                tb.api_handler_name.clone().unwrap_or_default()
            )),
            api_method: None,
            api_bypass: None,
            children: children,
        };
        self.permissions.push(perm);
    }

    pub fn add_permission_for_query(&mut self, tb: &QueryConfig, funclist: &Vec<RustFunc>) {
        let alias = tb.api_handler_name.clone();
        let name = if tb.comment.trim().is_empty() {
            tb.struct_name.to_uppercase()
        } else {
            tb.comment.clone()
        };

        let mut children = vec![];
        for mk in funclist.clone() {
            if mk.api_method.is_some() {
                let child = RustPermission {
                    name: mk.comment.clone().unwrap_or_default(),
                    alias: mk.func_name.to_uppercase(),
                    service_id: self.codegen_conf.app_name.clone(),
                    module_id: alias.clone(),
                    api_pattern: mk.api_pattern.clone(),
                    api_method: mk.api_method.clone(),
                    api_bypass: Some("user".to_string()),
                    children: vec![],
                };
                children.push(child);
            }
        }

        let perm = RustPermission {
            name: name,
            alias: alias.to_uppercase(),
            service_id: self.codegen_conf.app_name.clone(),
            module_id: alias.clone(),
            api_pattern: Some(format!(
                "{}/{}",
                self.codegen_conf.api_handler_prefix.clone(),
                tb.api_handler_name.clone()
            )),
            api_method: None,
            api_bypass: None,
            children: children,
        };
        self.permissions.push(perm);
    }

    pub fn table_for_each<F>(&mut self, func: &mut F)
    where
        Self: Sized,
        F: FnMut((TableInfo, Vec<ColumnInfo>)),
    {
        self.tables.clone().into_iter().for_each(|f| {
            let cols = self.columns.get(&f.table_name.clone().unwrap_or_default());

            match cols {
                Some(cs) => {
                    func((f, cs.to_vec()));
                }
                None => {}
            }
        });
    }

    pub fn get_struct_name(&self, tbl: &String) -> Option<String> {
        for tc in self.codegen_conf.tables.clone() {
            if tc.name == tbl.clone() {
                if tc.struct_name.is_empty() {
                    return Some(pascal_case(tc.name.clone().as_str()));
                } else {
                    return Some(tc.struct_name.clone());
                }
            }
        }
        None
    }

    pub fn get_value_object_struct_name(&self, tbl: &String) -> Option<String> {
        for tc in self.codegen_conf.tables.clone() {
            if tc.name == tbl.clone() {
                if tc.struct_name.is_empty() {
                    return Some(format!("{}Value", pascal_case(tc.name.clone().as_str())));
                } else {
                    return Some(format!("{}Value", tc.struct_name.clone().as_str()));
                }
            }
        }
        None
    }

    pub fn get_table_conf(&self, tbl: &String) -> Option<TableConfig> {
        for tc in self.codegen_conf.tables.clone() {
            if tc.name == tbl.clone() {
                return Some(tc.clone());
            }
        }
        None
    }

    pub fn get_table_info(&self, tbl: &String) -> Option<TableInfo> {
        for tc in self.tables.clone() {
            if tc.table_name.clone().unwrap_or_default() == tbl.clone() {
                return Some(tc.clone());
            }
        }
        None
    }

    pub fn get_table_columns(&self, tbl: &String) -> Vec<ColumnInfo> {
        match self.columns.get(&tbl.clone()) {
            Some(st) => st.to_owned(),
            None => {
                vec![]
            }
        }
    }

    pub fn find_table_column(&self, tbl: &String, col: &String) -> Option<ColumnInfo> {
        match self.columns.get(&tbl.clone()) {
            Some(st) => {
                for xs in st.clone() {
                    if xs.column_name.clone().unwrap_or_default().to_lowercase()
                        == col.to_lowercase()
                    {
                        return Some(xs.clone());
                    }
                }
                None
            }
            None => None,
        }
    }

    /**
     * 从TableConfig中的主键来解决
     */
    pub fn get_table_column_by_primary_key(&self, tbl: &String) -> Vec<ColumnInfo> {
        let mut pkeys = vec![];
        let cols = self.columns.get(&tbl.clone());
        let tbconf = self.get_table_conf(tbl);
        if tbconf.is_some() {
            let tbc = tbconf.unwrap();
            match cols {
                Some(tcls) => {
                    tbc.primary_key.split(&",".to_string()).for_each(|f| {
                        for cl in tcls {
                            if cl.column_name.clone().unwrap_or_default() == f.trim().to_string() {
                                pkeys.push(cl.clone());
                                // break;
                            }
                        }
                    });
                }
                None => {}
            };
        }

        pkeys
    }

    pub fn get_table_pkey_column(&self, tbl: &String) -> Vec<ColumnInfo> {
        let mut pkeys = vec![];
        let cols = self.get_table_columns(tbl);
        for cl in cols {
            if cl.column_key.clone().unwrap_or_default().to_lowercase() == "pri" {
                pkeys.push(cl.clone());
                break;
            }
        }

        pkeys
    }

    pub fn get_table_auto_incremnt_column(&self, tbl: &String) -> Option<ColumnInfo> {
        let cols = self.get_table_columns(tbl);
        for cl in cols {
            if cl.extra.clone().unwrap_or_default().to_lowercase() == "auto_increment" {
                return Some(cl.clone());
            }
        }

        None
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustFunc {
    pub is_struct_fn: bool,
    pub is_self_fn: bool,
    pub is_self_mut: bool,
    pub is_pub: bool,
    pub is_async: bool,
    pub func_name: String,
    pub return_is_option: bool,
    pub return_is_result: bool,
    pub return_type: Option<String>,
    pub params: Vec<(String, String)>,
    pub bodylines: Vec<String>,
    pub macros: Vec<String>,
    pub comment: Option<String>,
    pub api_method: Option<String>,
    pub api_pattern: Option<String>,
}

impl RustFunc {
    pub fn add_params(&mut self, name: &String, rtype: &String) {
        self.params.push((name.clone(), rtype.clone()));
    }

    pub fn add_bodyline(&mut self, line: &String) {
        self.bodylines.push(line.clone());
    }

    pub fn add_bodylines(&mut self, lines: &mut Vec<String>) {
        self.bodylines.append(lines);
    }
}

impl CodeWriter for RustFunc {
    fn write(&self, ro: &mut RustOutput) {
        let fnname = if self.is_pub {
            if self.is_async {
                format!("pub async fn {}(", self.func_name)
            } else {
                format!("pub fn {}(", self.func_name)
            }
        } else {
            if self.is_async {
                format!("async fn {}(", self.func_name)
            } else {
                format!("fn {}(", self.func_name)
            }
        };

        let mut first = format!("{}", fnname);

        if self.is_struct_fn && self.is_self_fn {
            // Should be in an struct, the self fn will valid
            if self.is_self_mut {
                first.push_str("&mut self,");
            } else {
                first.push_str("&self,");
            }
        }

        for pm in self.params.clone() {
            first.push_str(&format!("{}: {},", pm.0.to_string(), pm.1.to_string()));
        }

        if first.ends_with(",") {
            // do sub string process
            first = first.substring(0, first.len() - 1).to_string();
        }
        first.push(')');
        if self.return_type.is_some() {
            if self.return_is_result {
                if self.return_is_option {
                    first.push_str(&format!(
                        " -> Result<Option<{}>, Error> {{",
                        self.return_type.clone().unwrap_or_default()
                    ));
                } else {
                    first.push_str(&format!(
                        " -> Result<{}, Error> {{",
                        self.return_type.clone().unwrap_or_default()
                    ));
                }
            } else {
                if self.return_is_option {
                    first.push_str(&format!(
                        " -> Option<{}> {{",
                        self.return_type.clone().unwrap_or_default()
                    ));
                } else {
                    first.push_str(&format!(
                        " -> {} {{",
                        self.return_type.clone().unwrap_or_default()
                    ));
                }
            }
        } else {
            first.push_str(" {");
        }
        let mut space = 0i32;
        if self.is_struct_fn {
            for mc in self.macros.clone() {
                ro.write_line(&format!("    {}", mc));
            }
            ro.write_line(&format!("    {}", first));
            space = 2;
        } else {
            for mc in self.macros.clone() {
                ro.write_line(&format!("{}", mc));
            }
            ro.write_line(&format!("{}", first));
            space = 1;
        }

        for ln in self.bodylines.clone() {
            if ln.trim().starts_with("}") {
                space -= 1;
            }
            let mut blankspace = String::new();
            let mut t = 0;
            while t < space {
                blankspace.push_str("    ");
                t += 1;
            }
            ro.write_line(&format!("{}{}", blankspace, ln));
            if ln.trim().ends_with("{") {
                space += 1;
            }
        }

        if self.is_struct_fn {
            ro.write_line("    }");
        } else {
            ro.write_line("}");
        }
        ro.write_line("");
        ro.write_line("");
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustStructField {
    pub is_pub: bool,
    pub column_name: String,
    pub field_name: String,
    pub orignal_field_name: Option<String>,
    pub field_type: String,
    pub is_option: bool,
    pub annotations: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustStruct {
    pub is_pub: bool,
    pub has_paging: bool,
    pub struct_name: String,
    pub annotations: Vec<String>,
    pub fields: Vec<RustStructField>,
    pub funclist: Vec<RustFunc>,
}

impl RustStruct {
    pub fn add_field(&mut self, fd: &RustStructField) {
        self.fields.push(fd.clone())
    }

    pub fn add_func(&mut self, fd: &RustFunc) {
        self.funclist.push(fd.clone())
    }

    pub fn add_annotation(&mut self, fd: &String) {
        if !self.annotations.contains(fd) {
            self.annotations.push(fd.clone())
        }
    }
}

impl CodeWriter for RustStruct {
    fn write(&self, ro: &mut RustOutput) {
        for ln in self.annotations.clone() {
            ro.write_line(&ln);
        }
        if self.is_pub {
            ro.write_line(&format!("pub struct {} {{", self.struct_name.clone()));
        } else {
            ro.write_line(&format!("struct {} {{", self.struct_name.clone()));
        }

        for fd in self.fields.clone() {
            let ret = if fd.is_option {
                format!("Option<{}>", fd.field_type.clone())
            } else {
                format!("{}", fd.field_type.clone())
            };

            for annt in fd.annotations.clone() {
                if !annt.is_empty() {
                    ro.write_line(&format!("    {}", annt));
                }
            }

            if fd.orignal_field_name.is_none() {
                if fd.column_name.len() > 0 && fd.column_name != fd.field_name {
                    ro.write_line(&format!(
                        "    #[serde(rename(deserialize=\"{}\"))]",
                        fd.column_name.clone()
                    ));
                }
            }

            if fd.is_pub {
                ro.write_line(&format!("    pub {}: {},", fd.field_name.clone(), ret));
            } else {
                ro.write_line(&format!("    {}: {},", fd.field_name.clone(), ret));
            }
        }
        ro.write_line("}");
        ro.write_line("");
        ro.write_line("");
        if !self.funclist.is_empty() {
            ro.write_line(&format!("impl {} {{", self.struct_name.clone()));

            for func in self.funclist.clone() {
                func.write(ro);
            }

            ro.write_line("}");
            ro.write_line("");
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustOutput {
    outputs: Vec<String>,
}

impl RustOutput {
    pub fn write_line(&mut self, line: &str) {
        let newline = line.to_string() + "\n";
        self.outputs.push(newline);
    }

    pub fn print_out(&self) {
        for ln in self.outputs.clone() {
            println!("{}", ln);
        }
    }
}

/**
 * Rust程序文件结构
 */
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustFileImpl {
    pub file_name: String,
    pub mod_name: String, // Save into a folder
    pub caretlist: Vec<String>,
    pub usinglist: Vec<String>,
    pub structlist: Vec<RustStruct>,
    pub funclist: Vec<RustFunc>,
}

impl RustFileImpl {
    pub fn add_using(&mut self, us: &String) {
        self.usinglist.push(us.clone());
    }

    pub fn add_caret(&mut self, us: &String) {
        self.caretlist.push(us.clone());
    }

    pub fn add_struct(&mut self, us: &RustStruct) {
        self.structlist.push(us.clone());
    }

    pub fn add_func(&mut self, us: &RustFunc) {
        self.funclist.push(us.clone());
    }

    pub fn write_out(&self, filename: &String) -> std::io::Result<()> {
        let mut ro = RustOutput::default();
        ro.write_line("/**");
        ro.write_line(format!(" * Generate the file for {}, ", self.file_name.clone()).as_str());
        ro.write_line(" */");
        for crt in self.caretlist.clone() {
            ro.write_line(format!("extern caret {};", crt).as_str());
        }
        ro.write_line("");
        for usingline in self.usinglist.clone() {
            ro.write_line(format!("use {};", usingline).as_str());
        }
        ro.write_line("");

        for st in self.structlist.clone() {
            st.write(&mut ro);
        }
        for func in self.funclist.clone() {
            func.write(&mut ro);
        }

        let mut file = OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .truncate(true)
            .open(filename)?;
        for lt in ro.outputs.clone() {
            file.write_all(lt.as_bytes())?;
        }

        file.flush()?;

        Ok(())
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RustPermission {
    pub name: String,
    pub alias: String,
    pub service_id: String,
    pub module_id: String,
    pub api_pattern: Option<String>,
    pub api_method: Option<String>,
    pub api_bypass: Option<String>,
    pub children: Vec<RustPermission>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CodeGenerator {
    pub ctx: GenerateContext,
    pub files: Vec<RustFileImpl>,
    //pub default_entity_using: Vec<String>,
    //pub default_handler_using: Vec<String>,
}

impl CodeGenerator {
    /**
     * Step 1
     * 创建代码生成器实例
     */
    pub fn new(ctx: &GenerateContext) -> Self {
        Self {
            ctx: ctx.clone(),
            files: vec![],
            // default_entity_using: Self::get_default_entity_using(true),
            // default_handler_using: Self::get_default_handler_using(true),
        }
    }

    pub fn get_default_entity_using(ctx: &GenerateContext, paging: bool) -> Vec<String> {
        let mut list = vec![];
        list.push("std::fmt::{Debug}".to_string());
        list.push("serde_derive::{Deserialize, Serialize}".to_string());
        list.push("rbatis::crud_table".to_string());
        list.push("rbatis::rbatis::{Rbatis}".to_string());
        // list.push("rbatis::executor::{ Executor, ExecutorMut }".to_string());
        list.push("rbatis::error::Error".to_string());
        if paging {
            list.push("rbatis::Page".to_string());
            list.push("rbatis::PageRequest".to_string());
            list.push("rbson::Bson".to_string());
        }
        list.push("rbatis::crud::{CRUD, Skip}".to_string());
        if ctx.codegen_conf.allow_bool_widecard {
            list.push("crate::utils::bool_from_str".to_string());
        }
        if ctx.codegen_conf.allow_number_widecard {
            list.push(
                "crate::utils::{i64_from_str,u64_from_str,f64_from_str,f32_from_str}".to_string(),
            );
        }

        list
    }

    pub fn get_default_handler_using(_paging: bool) -> Vec<String> {
        let mut list = vec![];

        list.push("crate::utils::get_rbatis".to_string());
        list.push("chimes_auth::ApiResult".to_string());
        list.push("actix_web::{web, HttpResponse, Result}".to_string());
        list
    }

    /**
     *
     * 从数据库中查询所有表名，列名
     *
     */

    pub async fn read_table_names(rb: &Rbatis, schema: &'static str)->Vec<String> {
        // let mut conn = pool.acquire().await.unwrap();
        // let rows = sqlx::query("SELECT table_name,table_comment FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_schema = 'ruoyi_web'")
        // .fetch_all( &mut conn )
        // .await
        // .unwrap();

        let query_str = "SELECT table_name as name,table_comment as comment FROM information_schema.tables WHERE table_type = 'BASE TABLE' AND table_schema = ?";

        let mut rb_args = vec![];
        rb_args.push(rbson::to_bson(schema).unwrap_or_default());

        //let v =  crate::RB::qu(query_str).await;
        let r = rb
            .query(query_str, vec![rbs::to_value!(schema)])
            .await.unwrap(); //crate::RB::fetch_by_wrapper(query_str,&w).await;
                    //处理返回值是错误的时候转换成String
                    // rows.into_iter()
                    //     .map(|row| (row.t ry_get(0).unwrap(), row.try_get(1).unwrap()))
                    //     .collect()
                    // let row: (i64,) = sqlx::query_as("SELECT $1")
                    //     .bind(150_i64)
                    //     .fetch_one(&pool).await?;
                    //  rows.into_iter()
                    //      .map(|row| (row.try_get(0), row.try_get(1)))
                    //      .collect()
                    //    let c = Vec::new();
                    //     // r.unwrap().into_iter()
                    //     //     .map(|row| c.push(row)).collect();

        //    r.into_iter().map(|row| {
        //         // Use `unwrap_or` to provide a default empty Vec<TableConfig> if the input is `None`
        //        c.push(row.unwrap());
        //        c
        //     });

        // c
        // match r {
        //     Ok(r) => r,
        //     Err(e) => {
        //         eprintln!("Error: {:?}", e);
        //         vec![]
        //     }
        // }
        // let r=vec![("a".to_string(),"b".to_string())];

        // r
        // match r {
        //     Ok(r) => r,
        //     Err(e) => {
        //         eprintln!("Error: {:?}", e);  
        //         rbs::to_value!(e)
        //     }
        // }
        let mut  res = vec![];
        for row in r
        {

            println!("{},{}",row.0,row.1);
            res.push(row.0.to_string());

        }
        res
    }

    // #[py_sql("select * from biz_activity where delete_flag = 0
    // if name != '':
    //   `and name=#{name}`")]
    // #[rbatis::sql("select * from biz_activity where id = ?")]
    pub async fn read_table_columns(rb: &Rbatis, schema: &str) -> Vec<RawRawData> {
        // let pool = MySqlPool::connect(database_url).await.unwrap();
        // let mut conn = pool.acquire();

        let query = "SELECT table_schema,table_name,   column_name as column_name, column_type as column_type, column_comment as column_comment, column_key as column_key,
        column_default as column_default, data_type as data_type, ordinal_position as ordinal_position, character_maximum_length as character_maximum_length,
        is_nullable as is_nullable, numeric_precision as numeric_precision, numeric_scale as numeric_scale, extra as extra
        FROM INFORMATION_SCHEMA.COLUMNS WHERE table_schema = ? "; //and table_name = ?

        let rows = rb
            .query_decode(query, vec![rbs::to_value!(schema)])
            .await
            .unwrap();
        // let rows = sqlx::query(query)
        //     .bind("ruoyi_web")
        //     .bind("sys_notice")
        //     .fetch_all( &mut conn)
        //     .await
        //     .unwrap();
        // let rows = sqlx::query_as::<_, RawRawData>(query)
        //     .bind("ruoyi_web")
        //     .fetch_all(&mut conn)
        //     // .await
        //     .unwrap();

        // let rows = fetch_all_rows(pool, "sys_notice").await;

        // if rowsis_empty() {
        //     println!("No rows found");
        //     //  return  ;
        // }

        rows
    }

    pub async fn load_tables_config_from_db() -> Vec<TableConfig> {
        // let database_url = "mysql://postgres:postgres@localhost:3306/ruoyi_web";

        // let pool = sqlx::MySqlPool::connect(database_url);
        // //

        // let pool =MySqlPoolOptions::new()
        //     .max_connections(5)
        //     .connect("postgres://postgres:password@localhost/test").await?;
        let ts = "ruoyi_web"; //ctx.codegen_conf.schema_name.clone();

        let rb = get_rbatis();
        let tables = Self::read_table_names(rb, &ts).await;

        let colums = Self::read_table_columns(rb, &ts).await;

        // write_table_names(&tables);
        // let mut list: Vec<BTreeMap<String,String>> = Vec::new();
        let mut list = Vec::new();
/*   for (name,comment) in tables.unwrap() {
            // let name =  row.0;
            // let comment = row.1;
            let mut pri_key = "".to_string();
            let mut fileds = "".to_string();
            println!("table result: {:#?}", name.to_string());

            let intersect = colums
                .iter()
                .filter(|u| u.table_name == name)
                .collect::<Vec<_>>();
            // println!("column result: {:#?}", intersect);
            let files_columns = intersect
                .iter()
                .filter(|u| u.table_name == name)
                .collect::<Vec<_>>();

            let pri_keys = intersect
                .iter()
                .filter(|u| u.table_name == name && u.column_key == "PRI")
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

            let mut tbi = TableConfig::default();
            tbi.comment = comment.to_string(); //注，对于该业务对象的备注，建议该字段都应该给出相应的代表准确含义的内容
            tbi.name = name.clone(); //出来的结构名称，如没有，则会根据name(表名)的PascalCase来自动产生。
            tbi.api_handler_name = snake_case(&name);
            // tbi.primary_key = pri_key; //主键，如果主键没有被定义，则会尝试从表结构来进行解析。
            tbi.page_query = true;
            tbi.all_field_option = true; //所有的字段都是Option的。缺省为true。
            tbi.generate_handler = true;
            // update-skip-fields: create_date                               # 执行更新操作时，需要进行跳过的字段。
            // update-seletive: true                                         # 执行有选择地更新操作，该属性为true时，会生成一个update_selective方法，更新时，将只更新有值的字段
            // page-query: true                                              # 是否生成分页查询
            // default-sort-field: job_sort asc                              # 缺省的排序字段，如果有多个，可以以半角逗号隔开。所有生成出来的查询（列表和分页）都会加入这个缺省的排序。
            // generate-param-struct: true                                   # 是否生成用于查询的结构，这个生成的用于查询的结构名称由实体的结构名+Query，如MorinkhuurUserQuery，生成出来的结构有以下两个特点：
            //                                                               # 1. 日期/时间类型字段，生成出与实体中相对应的字段名一样的字段，但其类型为Vec<...>，如果在该Vec中不包含值，则该字段不参与查询，如果只包含一个值，
            //                                                               # 则是按大于值进行查询，如果包含2个或以后，则是按第1个值和第2个值进行between .. and .. 查询。
            //                                                               # 2. 如果字段名以_id, _code, _status, category, catalog, _type为结尾，则会在原基础上加多一个列表字段，如果列表字段不为空，则会对其执行IN (...) 查询。
            // tree-parent-field: pid                                        # 是否支持有树形查询，且指定了用于树形查询的parent字段
            // tree-root-value: "null"                                       # 表式为树形查询中为ROOT节点的数据条件，其值通常为"null"或为0，则会为其生成对应的查询语句，如 pid is null 或 pid = 0等。
            // api-handler-name: user                                        # 生成actix-web的handler时，所对应的映射的url的主要部分，通常一个url为
            //                                                                # ${api-handler-prefix}/${api-handler-name}/.. 组成，
            //                                                                # api-handler-name中不要包含有非字母及数字之外的值，且不要以数字开头。
            // simple-funclist:                    # 定义一些常用的简单的查询方法，通常在实际使用过程中，有些实体需要根据少数的一个或几个字段进行查询，我们就可以定义简单函数列表
            // - func-name: load_username          # 定义函数的名称
            //   condition: username               # 查询条件的字段
            //   list: false                       # 返回列表吗？
            //   self-func: false                  # 定义为需要引用self吗
            //   paged: false                      # 是否支持分页查询。 如果list以及paged都为false，则会生成返回单条记录的查询
            list.push(tbi);
        } */
        list
    }

    /**
     * Step 2
     * 加载数据库表
     * 根据从Yaml文件中加载的配置来进行处理
     */
    pub async fn load_tables(&mut self) {
        let rb = get_rbatis();
        let ts = self.ctx.codegen_conf.schema_name.clone();
        let tables1 = self.ctx.codegen_conf.tables.clone();

        let tables = Self::load_tables_config_from_db().await;
        for f in tables {
            let tn = f.name.clone();
            let tbi = TableInfo::load_table(rb, &ts.clone(), &tn.clone()).await;

            match tbi {
                Ok(tbop) => {
                    match tbop {
                        Some(tb) => {
                            log::info!(
                                "Columns of table {} {} {} will be fetching.",
                                tb.table_name.clone().unwrap_or_default(),
                                tb.table_schema.clone().unwrap_or_default(),
                                tb.table_catalog.clone().unwrap_or_default()
                            );
                            match ColumnInfo::load_columns(rb, &ts.clone(), &tn.clone()).await {
                                Ok(cols) => {
                                    // log::info!("The table {} will be added.", tb.table_name.clone().unwrap_or_default());
                                    self.ctx.add_table(&tb, &cols);
                                }
                                Err(err) => {
                                    log::info!(
                                        "Load the columns for table {} with an error {}",
                                        &f.name,
                                        err
                                    );
                                }
                            };
                        }
                        None => {
                            log::info!("Could not found the table {}", &f.name);
                        }
                    }
                }
                Err(err) => {
                    log::info!("Load the table {} with an error {}", &f.name, err);
                }
            };
            // log::info!("Table: {}, PK: {}", f.name, f.primary_key);
        }

        for qry in self.ctx.codegen_conf.queries.clone() {
            let mut fds = Vec::new();
            for st in qry.params.clone() {
                fds.push(st.default_value.clone().unwrap_or_default());
            }

            match execute_sql(&self.ctx, qry.base_sql.as_str(), &fds).await {
                Ok(rt) => {
                    let st = parse_query_as_file(&self.ctx, &qry, &rt);
                    self.files.push(st);
                    if qry.generate_handler {
                        let hl = parse_query_handler_as_file(&mut self.ctx, &qry, &rt);
                        self.files.push(hl);
                    }
                }
                Err(err) => {
                    log::info!("Execute the query with an error {}", err);
                }
            }
        }
    }

    /**
     * Step 3
     * 根据Table来进行代码生成
     */
    pub fn generate(&mut self) {
        let mut hashm = HashMap::new();
        let mut paramhm = HashMap::new();
        for tbl in self.ctx.tables.clone() {
            let columns = self
                .ctx
                .get_table_columns(&tbl.table_name.clone().unwrap_or_default());
            let st = parse_table_as_struct(&self.ctx, &tbl, &columns);
            self.ctx.add_struct(&st);

            let tbcc = self
                .ctx
                .get_table_conf(&tbl.table_name.clone().unwrap_or_default())
                .unwrap();
            if tbcc.tree_parent_field.is_some() {
                let stvo = parse_table_as_value_object_struct(&self.ctx, &tbl, &columns);
                hashm.insert(st.struct_name.to_string(), stvo);
            }
            if tbcc.generate_param_struct {
                let mtvo = parse_table_as_request_param_struct(&self.ctx, &tbl, &columns);
                paramhm.insert(st.struct_name.to_string(), mtvo);
            }
        }

        // 组织文件结构
        for sts in self.ctx.structs.clone() {
            let mut stlist = vec![];
            stlist.push(sts.clone());
            if hashm.contains_key(&sts.struct_name.clone()) {
                let mx = hashm[&sts.struct_name.clone()].clone();
                stlist.push(mx.clone());
            }

            if paramhm.contains_key(&sts.struct_name.clone()) {
                let mx = paramhm[&sts.struct_name.clone()].clone();
                stlist.push(mx.clone());
            }

            let rfi = RustFileImpl {
                file_name: format!("{}.rs", snake_case(sts.struct_name.clone().as_str())),
                mod_name: "entity".to_string(),
                caretlist: vec![],
                usinglist: Self::get_default_entity_using(&self.ctx, sts.has_paging),
                structlist: stlist,
                funclist: vec![],
            };
            self.files.push(rfi);
        }

        for sts in self.ctx.queries.clone() {
            let rfi = RustFileImpl {
                file_name: format!("{}.rs", snake_case(sts.struct_name.clone().as_str())),
                mod_name: "query".to_string(),
                caretlist: vec![],
                usinglist: Self::get_default_entity_using(&self.ctx, sts.has_paging),
                structlist: vec![sts.clone()],
                funclist: vec![],
            };
            self.files.push(rfi);
        }

        for tbl in self.ctx.tables.clone() {
            let mut usinglist = vec![];
            let tbc = self
                .ctx
                .get_table_conf(&tbl.table_name.clone().unwrap_or_default())
                .unwrap();
            if tbc.generate_handler {
                let funclist =
                    generate_actix_handler_for_table(&mut self.ctx, &tbl.clone(), &mut usinglist);

                usinglist.append(&mut Self::get_default_handler_using(tbc.page_query));
                // let tbc =  self.ctx.get_table_conf(&tbl.table_name.clone().unwrap_or_default()).unwrap();
                let rfi = RustFileImpl {
                    file_name: format!(
                        "{}.rs",
                        snake_case(
                            self.ctx
                                .get_struct_name(&tbl.table_name.clone().unwrap_or_default())
                                .unwrap()
                                .as_str()
                        )
                        .to_string()
                    ),
                    mod_name: "handler".to_string(),
                    caretlist: vec![],
                    usinglist: usinglist,
                    structlist: vec![],
                    funclist: funclist,
                };
                self.files.push(rfi);
            }
        }

        for rel in self.ctx.codegen_conf.relations.clone() {
            match parse_relation_as_file(&self.ctx, &rel) {
                Some(rfi) => {
                    self.files.push(rfi);
                }
                None => {
                    log::info!(
                        "Could not generated relation entity for {}",
                        rel.major_table
                    );
                }
            }

            if rel.generate_handler {
                match parse_relation_handlers_as_file(&mut self.ctx, &rel) {
                    Some(rfi) => {
                        self.files.push(rfi);
                    }
                    None => {
                        log::info!(
                            "Could not generated relation handler for {}",
                            rel.major_table
                        );
                    }
                }
            }
        }

        match self.ctx.codegen_conf.config_template_generate.clone() {
            // should generate the config template parse
            Some(fl) => {
                let rfi = parse_yaml_as_file(&fl, &"app_config.rs".to_string());
                if !rfi.structlist.is_empty() {
                    self.files.push(rfi);
                }
            }
            None => {}
        };
    }

    /**
     * Step 4
     * 写到文件
     * |--entity
     * |--handler
     * |--utils
     * |--main.rs
     * |--cargo.toml
     */
    pub fn write_out(&self) -> std::io::Result<()> {
        let str = self
            .ctx
            .codegen_conf
            .output_path
            .clone()
            .as_str()
            .to_owned();
        let root_path = Path::new(&str);
        if !root_path.exists() {
            // should create the path
            create_dir_all(root_path)?;
        }
        let src = root_path.join("src");
        if !src.exists() {
            // should create the path
            create_dir(src.clone())?;
        }
        let utils = src.join("utils");
        if !utils.exists() {
            // should create the path
            create_dir(utils.clone())?;
        }

        let utilsfile = utils.join("mod.rs");
        self.write_content(
            &utilsfile.to_str().unwrap_or_default().to_string(),
            crate::tmpl::UTILS_TMPL,
        )?;

        let conf = root_path.join("conf");
        if !conf.exists() {
            // should create the path
            create_dir(conf.clone())?;
        }

        let cargotoml = root_path.join("Cargo.toml");
        self.write_content(
            &cargotoml.to_str().unwrap_or_default().to_string(),
            &crate::tmpl::replace_cargo_toml(&self.ctx.codegen_conf),
        )?;

        let scoconf = conf.join("app.yml");
        let conftext = format_conf_tmpl(
            &self.ctx.codegen_conf.database_url.clone(),
            &self.ctx.codegen_conf.webserver_port.clone(),
        );

        if self.ctx.redis_conf.has_redis {
            let redisconf = format_redis_conf_tmpl(
                &self.ctx.redis_conf.host,
                self.ctx.redis_conf.port.clone(),
                &self.ctx.redis_conf.username,
                &self.ctx.redis_conf.password,
                self.ctx.redis_conf.db.clone(),
            );
            let wholeconf = conftext + redisconf.as_str();
            self.write_content(
                &scoconf.as_path().to_str().unwrap_or_default().to_string(),
                wholeconf.as_str(),
            )?;
        } else {
            self.write_content(
                &scoconf.as_path().to_str().unwrap_or_default().to_string(),
                conftext.as_str(),
            )?;
        }
        let mut modmap = HashMap::<String, Vec<String>>::new();
        let mut service_func: Vec<String> = Vec::new();

        for fl in self.files.clone() {
            let modpath = src.join(fl.mod_name.clone());
            if !modmap.contains_key(&fl.mod_name) {
                modmap.insert(fl.mod_name.clone(), vec![]);
            }
            let mut modfiles = modmap.get(&fl.mod_name).unwrap().clone();
            modfiles.push(fl.file_name.clone());
            modmap.insert(fl.mod_name.clone(), modfiles);

            if !modpath.exists() {
                // create the path
                create_dir(modpath.clone())?;
            }

            let filename = modpath.join(fl.file_name.clone());
            fl.write_out(&filename.to_str().unwrap_or_default().to_string())?;

            if fl.mod_name == "handler" {
                for func in fl.funclist {
                    service_func
                        .push(format!("crate::{}::{}", fl.mod_name, func.func_name).to_string());
                }
            }
        }

        let mut mainmods: Vec<String> = Vec::new(); //生成用于main.rs的mod声明

        for mkey in modmap {
            let mn = mkey.0.clone();
            mainmods.push(mn.clone());
            let tj = src.join(mkey.0.clone()).join("mod.rs"); // Generate the mod.rs for each folder
            let tjfile = if self.ctx.codegen_conf.always_override {
                OpenOptions::new()
                    .write(true)
                    .append(false)
                    .create(true)
                    .truncate(true)
                    .open(tj.as_path())
            } else {
                OpenOptions::new()
                    .write(true)
                    .append(false)
                    .create_new(true)
                    .truncate(true)
                    .open(tj.as_path())
            };
            match tjfile {
                Ok(mjfile) => {
                    let mut mutfile = mjfile;
                    for ln in mkey.1.clone() {
                        let nameonly = ln.substring(0, ln.len() - 3);
                        let modfmt = format!("mod {};\n", nameonly.clone());
                        let usingfmt = format!("pub use {}::*;\n", nameonly.clone());
                        mutfile.write_all(modfmt.as_bytes())?;
                        mutfile.write_all(usingfmt.as_bytes())?;
                        mutfile.write_all("\r\n".as_bytes())?;
                    }
                    mutfile.flush()?;
                }
                Err(err) => {
                    if err.kind() == ErrorKind::AlreadyExists {
                        log::info!("Skipped the existed file.");
                    } else {
                        log::info!("File was not create/opened. Becuase {}", err);
                    }
                }
            }
        }

        let main = src.join("main.rs");
        self.write_content(
            &main.to_str().unwrap_or_default().to_string(),
            crate::tmpl::format_main_template(mainmods, service_func).as_str(),
        )?;

        Ok(())
    }

    fn write_content(&self, filename: &String, content: &str) -> std::io::Result<()> {
        let file = if self.ctx.codegen_conf.always_override {
            OpenOptions::new()
                .write(true)
                .append(false)
                .create(true)
                .truncate(true)
                .open(filename)
        } else {
            OpenOptions::new()
                .write(true)
                .append(false)
                .create_new(true)
                .truncate(true)
                .open(filename)
        };
        match file {
            Ok(mjfile) => {
                let mut mutfile = mjfile;
                mutfile.write_all(content.as_bytes())?;
                mutfile.flush()?;
            }
            Err(err) => {
                if err.kind() == ErrorKind::AlreadyExists {
                    log::info!("Skipped the existed file.");
                } else {
                    log::info!("File was not create/opened. Becuase {}", err);
                }
            }
        };
        Ok(())
    }

    /**
     * 将Permission写入到数据库
     */
    // pub async fn write_permission(&self) {
    //     let rb = get_rbatis();
    //     for ele in self.ctx.permissions.clone() {
    //         let mut perm = ChimesPermissionInfo {
    //             id: None,
    //             alias: Some(ele.alias.clone()),
    //             create_time: None,
    //             name: Some(ele.name.clone()),
    //             pid: Some(0i64),
    //             api_pattern: ele.api_pattern,
    //             service_id: Some(ele.service_id.clone()),
    //             api_method: ele.api_method,
    //             api_bypass: ele.api_bypass,
    //         };
    //         // log::info!("Permission: {} {}", ele.name.clone(), ele.alias.clone());

    //         let mut query = ChimesPermissionInfo::default();
    //         query.alias = Some(ele.alias.clone());
    //         query.service_id = Some(ele.service_id.clone());
    //         let stid = match query.query_list(rb).await {
    //             Ok(rs) => {
    //                 if rs.len() > 0 {
    //                     let mut mp = rs[0].clone();
    //                     mp.name = perm.name.clone();
    //                     // mp.api_bypass = perm.api_bypass.clone();
    //                     mp.api_method = perm.api_method.clone();
    //                     mp.api_pattern = perm.api_pattern.clone();
    //                     match mp.update(rb).await {
    //                         Ok(r) => rs[0].id.unwrap_or_default(),
    //                         Err(_) => rs[0].id.unwrap_or_default(),
    //                     }
    //                 } else {
    //                     match perm.save(rb).await {
    //                         Ok(r) => perm.id.unwrap(),
    //                         Err(err) => {
    //                             log::info!("Error: {}", err.to_string());
    //                             0i64
    //                         }
    //                     }
    //                 }
    //             }
    //             Err(err) => {
    //                 log::info!("Error: {}", err.to_string());
    //                 0i64
    //             }
    //         };
    //         if stid != 0i64 {
    //             for chl in ele.children.clone() {
    //                 let mut chperm = ChimesPermissionInfo {
    //                     id: None,
    //                     alias: Some(chl.alias.clone()),
    //                     create_time: None,
    //                     name: Some(chl.name.clone()),
    //                     pid: Some(stid),
    //                     api_pattern: chl.api_pattern,
    //                     service_id: Some(chl.service_id.clone()),
    //                     api_method: chl.api_method,
    //                     api_bypass: chl.api_bypass,
    //                 };
    //                 match chperm.save_or_update(rb).await {
    //                     Ok(_) => {}
    //                     Err(err) => {
    //                         log::info!("Error: {}", err.to_string());
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    /**
     * Step 6
     * 加载数据库表
     * 根据从Yaml文件中加载的配置来进行处理
     */
    pub async fn load_tables_name(&mut self) {
        let rb = get_rbatis();
        let ts = self.ctx.codegen_conf.schema_name.clone();
        let tables = self.ctx.codegen_conf.tables.clone();
        for f in tables {
            let tn = f.name.clone();
            match TableInfo::load_table(rb, &ts.clone(), &tn.clone()).await {
                Ok(tbop) => {
                    match tbop {
                        Some(tb) => {
                            // log::info!("Columns of table {} {} {} will be fetching.", tb.table_name.clone().unwrap_or_default(), tb.table_schema.clone().unwrap_or_default(), tb.table_catalog.clone().unwrap_or_default());
                            match ColumnInfo::load_columns(rb, &ts.clone(), &tn.clone()).await {
                                Ok(cols) => {
                                    // log::info!("The table {} will be added.", tb.table_name.clone().unwrap_or_default());
                                    self.ctx.add_table(&tb, &cols);
                                }
                                Err(err) => {
                                    log::info!(
                                        "Load the columns for table {} with an error {}",
                                        &f.name,
                                        err
                                    );
                                }
                            };
                        }
                        None => {
                            log::info!("Could not found the table {}", &f.name);
                        }
                    }
                }
                Err(err) => {
                    log::info!("Load the table {} with an error {}", &f.name, err);
                }
            };
            // log::info!("Table: {}, PK: {}", f.name, f.primary_key);
        }

        for qry in self.ctx.codegen_conf.queries.clone() {
            let mut fds = Vec::new();
            for st in qry.params.clone() {
                fds.push(st.default_value.clone().unwrap_or_default());
            }

            match execute_sql(&self.ctx, qry.base_sql.as_str(), &fds).await {
                Ok(rt) => {
                    let st = parse_query_as_file(&self.ctx, &qry, &rt);
                    self.files.push(st);
                    if qry.generate_handler {
                        let hl = parse_query_handler_as_file(&mut self.ctx, &qry, &rt);
                        self.files.push(hl);
                    }
                }
                Err(err) => {
                    log::info!("Execute the query with an error {}", err);
                }
            }
        }
    }
}

pub fn parse_data_type_as_rust_type(dt: &String) -> String {
    match dt.as_str() {
        "smallint" => "i16".to_string(),
        "smallint unsigned" => "u16".to_string(),
        "int" => "i32".to_string(),
        "int unsigned" => "u32".to_string(),
        "bigint" => "i64".to_string(),
        "bigint unsigned" => "u64".to_string(),
        "tinyint" => "bool".to_string(),
        "tinyint unsigned" => "bool".to_string(),
        "tinyint signed" => "bool".to_string(),
        "boolean" => "bool".to_string(),
        "bit" => "i32".to_string(),
        "longtext" => "String".to_string(),
        "text" => "String".to_string(),
        "mediumtext" => "String".to_string(),
        "char" => "String".to_string(),
        "varchar" => "String".to_string(),
        "float" => "f32".to_string(),
        "double" => "f64".to_string(),
        "decimal" => "rbatis::Decimal".to_string(),
        "datetime" => "rbatis::DateTimeNative".to_string(),
        "date" => "rbatis::DateNative".to_string(),
        "timestamp" => "rbatis::Timestamp".to_string(),
        "time" => "rbatis::TimeNative".to_string(),
        "blob" => "rbatis::Bytes".to_string(),
        "binary" => "rbatis::Bytes".to_string(),
        "varbinary" => "rbatis::Bytes".to_string(),
        "mediumblob" => "rbatis::Bytes".to_string(),
        "longblob" => "rbatis::Bytes".to_string(),
        "json" => "rbatis::Json".to_string(),
        _ => "String".to_string(),
    }
}

pub fn parse_data_type_annotions(ctx: &GenerateContext, field_type: &String) -> Vec<String> {
    let mut annts = vec![];
    if field_type == "bool" {
        if ctx.codegen_conf.allow_bool_widecard {
            annts.push("#[serde(default)]".to_string());
            annts.push("#[serde(deserialize_with=\"bool_from_str\")]".to_string());
        }
    }
    if field_type == "i64" {
        if ctx.codegen_conf.allow_number_widecard {
            annts.push("#[serde(default)]".to_string());
            annts.push("#[serde(deserialize_with=\"i64_from_str\")]".to_string());
        }
    }
    if field_type == "u64" {
        if ctx.codegen_conf.allow_number_widecard {
            annts.push("#[serde(default)]".to_string());
            annts.push("#[serde(deserialize_with=\"u64_from_str\")]".to_string());
        }
    }
    if field_type == "f64" {
        if ctx.codegen_conf.allow_number_widecard {
            annts.push("#[serde(default)]".to_string());
            annts.push("#[serde(deserialize_with=\"f64_from_str\")]".to_string());
        }
    }
    if field_type == "f32" {
        if ctx.codegen_conf.allow_number_widecard {
            annts.push("#[serde(default)]".to_string());
            annts.push("#[serde(deserialize_with=\"f32_from_str\")]".to_string());
        }
    }
    annts
}

pub fn parse_column_as_field(
    ctx: &GenerateContext,
    tbl: &TableConfig,
    col: &ColumnInfo,
    rename_id: bool,
) -> RustStructField {
    let field_type =
        parse_data_type_as_rust_type(&col.data_type.clone().unwrap_or_default().to_lowercase());

    let annts = parse_data_type_annotions(ctx, &field_type);

    // log::info!("{} is {} -- {}.", col.column_name.clone().unwrap_or_default(), col.extra.clone().unwrap_or_default().to_lowercase(), col.column_key.clone().unwrap_or_default());

    let mut opt_field_name = None;
    let original_field_name =
        safe_struct_field_name(&col.column_name.clone().unwrap_or_default().to_lowercase());
    let field_name = if rename_id {
        if col.extra.clone().unwrap_or_default().to_lowercase() == "auto_increment"
            && col.column_key.clone().unwrap_or_default().to_lowercase() == "pri"
        {
            opt_field_name = Some(original_field_name.clone());
            "id".to_string()
        } else {
            original_field_name.clone()
        }
    } else {
        original_field_name.clone()
    };

    RustStructField {
        is_pub: true,
        column_name: col.column_name.clone().unwrap_or_default(),
        field_name: field_name,
        field_type: field_type.clone(),
        orignal_field_name: opt_field_name,
        is_option: if tbl.all_field_option {
            true
        } else {
            col.is_nullable.clone().unwrap_or_default().to_lowercase() == "yes"
        },
        annotations: annts,
    }
}

pub fn parse_column_list(
    ctx: &GenerateContext,
    tbl: &TableConfig,
    cols: &Vec<ColumnInfo>,
    columns: &mut String,
    rename_id: bool,
) -> Vec<RustStructField> {
    let mut fields = vec![];

    for col in cols {
        let colname = col.column_name.clone().unwrap_or_default();
        columns.push_str(colname.as_str());
        columns.push(',');
        fields.push(parse_column_as_field(ctx, tbl, &col, rename_id));
    }
    fields
}

pub fn make_skip_columns(ctx: &GenerateContext, tbl: &TableConfig) -> String {
    let mut skips = String::new();
    match tbl.update_skip_fields.clone() {
        Some(sk) => {
            for fd in sk.split(",").into_iter() {
                skips.push_str(format!("Skip::Column(\"{}\"),", fd.trim()).as_str());
            }
        }
        None => {}
    };

    skips
}
