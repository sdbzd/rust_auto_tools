#[cfg(feature = "async-std")]
use async_std::{
    fs::{OpenOptions, Path},
    prelude::*,
};

#[cfg(feature = "tokio")]
use tokio::fs::OpenOptions;

#[cfg(feature = "tokio")]
use tokio::io::AsyncWriteExt;

const FLAG: &'static str = "// ***************************************以下是自定义代码区域******************************************";
const FLAG2: &'static str = r#"
/*
*/
// *************************************************************************************************"#;

use crate::table::Table;
use async_trait::async_trait;
use quicli::prelude::*;
use serde::{Deserialize, Serialize};
use sql_reverse_error::result::Result;
use std::path::Path;
use tera::{Context, Tera};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilterFields {
    pub skip_fields: Option<Vec<String>>,
    pub contain_fields: Option<Vec<String>>,
    pub filename: String,
}

async fn filter_fields(table: &Table, params: Vec<FilterFields>, ) -> Result<Vec<(Table, String)>> {
    let mut list = vec![];
    for field in params.iter() {
        if field.skip_fields.is_some() {
            let data = table
                .skip_fields(field.skip_fields.to_owned().unwrap())
                .await;
            list.push((data, field.filename.to_owned()));
        } else if field.contain_fields.is_some() {
            let data = table
                .contain_fields(field.contain_fields.to_owned().unwrap())
                .await;
            list.push((data, field.filename.to_owned()));
        }
    }
    Ok(list)
}

#[async_trait]
pub trait Render {
    async fn render_rust(
        template_path: &str,
        template_name: &str,
        suffix: &str,
        output_dir: &str,
        tables: &Vec<Table>,
        multifile: &str,
    ) -> Result<()> {
        create_dir(output_dir)?;
        let tera = Tera::new(template_path)?;
        let mut mods = vec![];
        if multifile=="1"{
            //单文件输出  
            mods.push(format!("pub mod {};\n", "tables"));
            mods.push(format!("pub use {}::*;\n", "tables"));
            for table in tables {
              
                let (mut struct_str, mut custom,filepath) = Self::render_table(&tera, table, template_name, suffix, output_dir, &table.table_name).await?;
                if custom != "" {
                    let data: Vec<&str> = custom.split("*/").collect();
                    let data = data.get(0).unwrap_or(&"").to_string();
                    let data = data.replace("/*", "");
                    let data = data.trim();
                    let params: Vec<FilterFields> = serde_json::from_str(&data).unwrap_or(vec![]);
                    let filters = filter_fields(table, params).await?;
                  
                    for filter in filters.iter() {
                     
                        let (mut struct_str, custom,filepath) = Self::render_table(&tera, &filter.0, template_name, suffix, output_dir, &filter.1).await?;
                        // context.insert("template", &filter.0); // 兼容之前的版本
                        // context.insert("table", &filter.0);
                        // let mut struct_str = tera.render(template_name, &context)?;
                        // let filepath = format!("{}/{}.{}", output_dir, filter.1, suffix);
                        // let content = read_file(&filepath).unwrap_or_default();
                        // let vv: Vec<&str> = content.split(FLAG).collect();
                        // let custom = vv.get(1).unwrap_or(&"").to_string();
                        struct_str = struct_str + "\n" + FLAG + custom.as_str();
                        // async_ok!(Self::write_to_file(&filepath, &struct_str))?;
                        async_ok!(Self::append_str_to_file(&output_dir, &struct_str,suffix))?;
                    }
                }
                // if custom.trim() == "" {
                //     custom = FLAG2.to_owned();
                // }
                // struct_str = struct_str + "\n" + FLAG + custom.as_str();
                // async_ok!(Self::write_to_file(&filepath, &struct_str))?;
                async_ok!(Self::append_str_to_file(&output_dir, &struct_str,suffix))?;
            }  
        }
        else {
            // 多文件输出
            for table in tables {
                mods.push(format!("pub mod {};\n", table.table_name));
                mods.push(format!("pub use {}::*;\n", "tables"));
                let (mut struct_str, mut custom,filepath) = Self::render_table(&tera, table, template_name, suffix, output_dir, &table.table_name).await?;
                if custom != "" {
                    let data: Vec<&str> = custom.split("*/").collect();
                    let data = data.get(0).unwrap_or(&"").to_string();
                    let data = data.replace("/*", "");
                    let data = data.trim();
                    let params: Vec<FilterFields> = serde_json::from_str(&data).unwrap_or(vec![]);
                    let filters = filter_fields(table, params).await?;
                    for filter in filters.iter() {
                        mods.push(format!("pub mod {};\n", filter.1));
                        let (mut struct_str, custom,filepath) = Self::render_table(&tera, &filter.0, template_name, suffix, output_dir, &filter.1).await?;
                        // context.insert("template", &filter.0); // 兼容之前的版本
                        // context.insert("table", &filter.0);
                        // let mut struct_str = tera.render(template_name, &context)?;
                        // let filepath = format!("{}/{}.{}", output_dir, filter.1, suffix);
                        // let content = read_file(&filepath).unwrap_or_default();
                        // let vv: Vec<&str> = content.split(FLAG).collect();
                        // let custom = vv.get(1).unwrap_or(&"").to_string();
                        struct_str = struct_str + "\n" + FLAG + custom.as_str();
                        async_ok!(Self::write_to_file(&filepath, &struct_str))?;
                    }
                }
                if custom.trim() == "" {
                    custom = FLAG2.to_owned();
                }
                struct_str = struct_str + "\n" + FLAG + custom.as_str();
                async_ok!(Self::write_to_file(&filepath, &struct_str))?;
            }  
        }

        if suffix == "rs" {
            let mod_path = format!("{}/mod.{}", output_dir, suffix);
            async_ok!(Self::append_to_file(mods, &mod_path))?;
        }

        Ok(())
    }

    async fn render_table(
        tera: &Tera,
        table: &Table,
        template_name: &str,
        suffix: &str,
        output_dir: &str,
        filename: &str
    ) -> Result<(String, String, String)> {
        let mut context = Context::new();
        context.insert("template", table); // 兼容之前的版本
        context.insert("table", table);
        let struct_str = tera.render(template_name, &context)?;
        let filepath = format!("{}/{}.{}", output_dir, filename, suffix);
        let content = read_file(&filepath).unwrap_or_default();
        let vv: Vec<&str> = content.split(FLAG).collect();
        let custom = vv.get(1).unwrap_or(&"").to_string();
        Ok((struct_str, custom, filepath))
    }

    async fn write_to_file(filepath: &str, content: &str) -> Result<()> {
        let filepath = Path::new(&filepath);
        let s = write_to_file(filepath, content)?;
        Ok(s)
    }

    async fn append_to_file(mods: Vec<String>, filepath: &str) -> Result<()> {
        let file_content = read_file(filepath).unwrap_or_default();
        for v in mods.iter() {
            if !file_content.contains(v) {
                let mut file =
                    async_ok!(OpenOptions::new().append(true).create(true).open(filepath))?;
                async_ok!(file.write(v.as_bytes()))?;
            };
        }
        Ok(())
    }
    async fn append_str_to_file(filepath: &str,file_content: &String,  suffix: &str) -> Result<()> {
         let file_full_name = format!("{}/{}.{}", filepath, "tables", suffix);
         log::info!("输出文件：{}",file_full_name);
        //  let file_content = read_file(filepath).unwrap_or_default();
        //  let file_full_name = format!("{}/{}.{}", filepath, "tables", suffix);
        //     // if !file_content.contains(v)          
         let mut file = async_ok!(OpenOptions::new().append(true).create(true).open(file_full_name))?;
         async_ok!(file.write(file_content.as_bytes()))?;
            
        Ok(())
    }
}
