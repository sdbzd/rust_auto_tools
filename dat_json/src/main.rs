#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(clippy::cargo)]

use quicli::fs::read_file;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, ErrorKind};

use quicli::prelude::*;
use serde_json::Map;
use serde_json::Value;
use std::collections::BTreeMap;

use std::fs::create_dir_all;
// use walkdir::Result;
pub mod error;

pub use error::{Error, Result};
// pub mod package_json;
// pub use package_json::*;
// #[cfg(feature = "validate")]
// pub use validator;
// extern crate error;
// user serde_json::error;

// use package_json::*;
use json_typegen::json_typegen;
use json_typegen_shared::internal_util::display_error_with_causes;
use json_typegen_shared::{codegen, codegen_from_macro, parse, Options, OutputMode};
use std::sync::Mutex;
use std::{env, path::PathBuf};
use walkdir::WalkDir;
// use crate::package_json::PackageJson;
fn inner_main() -> anyhow::Result<PathBuf> {
    let exe = env::current_exe()?;
    let dir = exe.parent().expect("Executable must be in some directory");
    let dir = dir.join("/");
    Ok(dir)
}
fn main() {
    test();
    println!("hello");
    // json_typegen!("Point", r#"{ "x": 1, "y": 2 }"#);
    // let mut p: Point = serde_json::from_str(r#"{ "x": 3, "y": 5 }"#).unwrap();
    // println!("deserialized = {:?}", p);
    // p.x = 4;
    // let serialized = serde_json::to_string(&p).unwrap();
    // println!("serialized = {}", serialized);
}
#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref HOSTNAME: Mutex<String> = Mutex::new(String::new());
    static ref FILE_STRUCTS: Mutex<BTreeMap<String, BTreeMap<String, String>>> =
        Mutex::new(BTreeMap::new());
}
//must convert the &str reference returned from lines() to the owned type String, using .to_string() and String::from respectively.
fn read_lines(filename: &str) -> Vec<String> {
    std::fs::read_to_string(filename)
        .unwrap() // panic on possible file-reading errors
        .lines() // split the string into an iterator of string slices
        .map(String::from) // make each slice into a string
        .collect() // gather them together into a vector
}
// The output is wrapped in a Result to allow matching on errors
// Returns an Iterator to the Reader of the lines of the file.
fn read_lines_wrap<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn json_to_file(line: &str) {
    //  println!("#start:{}\r\n\r\n#end", line);
    // let json_obj: Value = if  let ok = Ok(serde_json::from_str(&line)).expect(msg).err();
    // if let Ok(lines) = read_lines_wrap(file_name.clone()) {
    // let res: Result<Person, serde_json::Error> = serde_json::from_str("{"age":30}");
    // if let Ok(ss ::i8) = serde_json::from_str("&line"){

    // }
    // if let Ok(json_objects) = serde_json::from_str(&line)
    // if let Err(e) = res {
    //     println!("Error: {}", e);
    // }
    // let _: Value = match serde_json::from_str(line) {
    //     Ok(value) => value,
    //     Err(error) => return e,
    //     // {
    //     //     println!("error: {error:#}");

    //     // }
    // };
    // let res=  serde_json::from_str(&line)  ;
    // match res {
    //     Ok(v) => { do_something(v) },
    //     Err(e) => { //handle error },
    // }
    let json_obj: Value = match serde_json::from_str(line) {
        //空行有换行符
        Ok(val) => val,
        Err(err) => {
            // format!("parsing was unsuccessful");
            serde_json::Value::Null
        }
    };
    // let json: Vec<String> = serde_json::from_str(&line).expect("Failed to parse json.");
    if !json_obj.is_null() {
        //
        // let json_obj = json_objects.u
        let url = json_obj.get("url").unwrap().to_string();

        let t2 = url.rfind('?').unwrap_or(url.len() - 1); //
        let t1 = url[0..t2].rfind('/').unwrap_or(0) + 1; // 从后向前查找 / 符号
        let name = &url[t1..t2];

        let response_data = json_obj.get("response_data").unwrap();
        FILE_STRUCTS.lock().unwrap().clear();
        // println!("{}", file_name);
        let _btm_strut: BTreeMap<String, String> = BTreeMap::new();
        // println!("new line::{},len{}\r\n", ip.trim(),ip.trim().len());
        // println!("#start:{}\r\n\r\n#end", ip);
        let output_file_name = env::current_dir()
            .unwrap()
            .join("output")  ; //[name.clone(),".json".to_string()].join());

        // write_to_file(output_file_name, &response_data.get("data").unwrap().to_string());
        // let json_file = output_file_name
        //     .clone()
        //     .into_os_string()
        //     .into_string()
        //     .unwrap();
        let _ = write_to_file(
            output_file_name.join(format!("{name}.json")),
            &serde_json::to_string_pretty(&response_data.get("data")).unwrap(),
        );

        let code = {
            //  let name = matches.value_of("name").unwrap_or("Root");  
            let mut _options = Options::default();
            _options.use_default_for_missing_fields= true; 
            // _options.
            // options.import_style = ImportStyle::AssumeExisting;
            codegen(
                name,
                &serde_json::to_string_pretty(&response_data.get("data")).unwrap(),
                _options,
            )
        };
        match code {
            Ok(v)=>{
                write_to_file(
                            format!("{}.rs", output_file_name.join(name).to_string_lossy()),
                            &v,
                        );

            },
            _ =>{},
            
        }
       

        // println!("{:#}",code.unwrap(""));

        // json_typegen!("Root",json_file);

        // parse_object("data", response_data.get("data").unwrap());
        // let content = FILE_STRUCTS.lock().unwrap().clone();
        // write_to_file(output_file_name, &serde_json::to_string(&content).unwrap());

        // println!("{}:{:?}", name," response_data.to_string()");
        // println!("struct {} {{", name.to_owned());
        // parse_object("ROOT", json_obj);
        // println!("{:#?}",FILE_STRUCTS.lock().unwrap());
        // let content = FILE_STRUCTS.lock().unwrap().clone();

        // write_to_file(output_file_name, &serde_json::to_string(&content).unwrap());
        // // println!("{}", line?);
        // println!("struct {} {{", name);
        // parse_object("ROOT", json_obj);
        // // println!("{:#?}",FILE_STRUCTS.lock().unwrap());
        // let content = FILE_STRUCTS.lock().unwrap().clone();

        // write_to_file(output_file_name, &serde_json::to_string(&content).unwrap());
    }
    {}
}

fn test() {
    // let current_dir = env::current_dir()?;
    // println!(
    //     "Entries modified in the last 24 hours in {:?}:",
    //     current_dir
    // );
    let path = env::current_dir().unwrap();
    println!("The current directory is {}", path.display());
    let _dir = path.as_path().read_dir().unwrap();
    // let mut current_dir:String = inner_main().unwrap().to_str().unwrap().to_string();
    // let mut current_dir = path.into_os_string().into_string().unwrap();
    let output_dir = path.join("output");
    let json_dir = env::current_dir()
        .unwrap()
        .join("json")
        .into_os_string()
        .into_string()
        .unwrap();
    println!("{json_dir:#?}");
    let entries = WalkDir::new(json_dir)
        .into_iter()
        // .filter_entry(|e|is_json(e))
        .filter_map(std::result::Result::ok);
    for entry in entries {
        let file_name = entry.path().to_str().expect("REASON").to_string();
        // let name = entry.file_name().to_str().expect("REASON").to_string() + ".rs";
        if std::path::Path::new(&file_name)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("dat"))
        {
            // if file_name.ends_with(".dat") { //  let file_content = read_file(file_name.clone()).unwrap();
            if let Ok(lines) = read_lines_wrap(file_name.clone()) {
                // Consumes the iterator, returns an (Optional) String
                for line in lines {
                    if let Ok(ip) = line {
                        if !ip.trim().is_empty() {
                            json_to_file(&ip[0..ip.len()]);
                        }
                    }
                }
            }
            // let lines  = read_lines(&file_name);
            // {
            //     // Consumes the iterator, returns an (Optional) String
            //     for line in lines {
            //         let ip = line;
            //         {
            //             if !ip.trim().is_empty() {
            //                // println!("new line::{},len{}\r\n", ip.trim(),ip.trim().len());
            //                 println!("#start:{}#end", ip);
            //                 json_to_file(ip);
            //             }
            //         }
            //     }
            // }
            // let file = File::open(file_name).unwrap();
            // let reader = BufReader::new(file);

            // for line in reader.lines() {
            //     if !line.unwrap().is_empty(){
            //     }
            // }
            // for line in file_content{
            //     let json_obj: Value = serde_json::from_str(&file_content).unwrap();
            //     // println!("{:#?}", Value);
            //     // let annts = json_obj.as_object().unwrap();
            //     println!("struct {} {{", name);
            //     parse_object("ROOT", json_obj);
            //     // println!("{:#?}",FILE_STRUCTS.lock().unwrap());
            //     let content = FILE_STRUCTS.lock().unwrap().clone();
            //     write_to_file(output_file_name, &serde_json::to_string(&content).unwrap());
            // }
        }
    }
    // let markdown_files = glob("*.json")?;
    // println!("{} files",markdown_files.len() );
    // let image_files = glob("**/*.{png,jpg,gif}");
    println!("Hello, world! end app ");
    // Ok(())
}

// impl Value{
fn match_value(_v: Value) -> i32 {
    match _v {
        Value::Null => 1,
        Value::Bool(_v) => 1,
        Value::Number(_v) => 1,
        Value::String(_v) => 1,
        Value::Array(_v) => 2,
        Value::Object(_v) => 3,
    }
}
// }

fn parse_object(obj_key: &str, objs: &Value) {
    // let obj = objs.as_object();
    match objs.as_object() {
        Some(obj) => {
            // let mut keys_vec: Vec<String> = Vec::new();
            let _arr_vec: Vec<Vec<Value>> = Vec::new();
            let mut obj_vec: Vec<Map<String, Value>> = Vec::new();
            let mut btm_strut: BTreeMap<String, String> = BTreeMap::new();
            let keys = obj.keys();
            // json_typegen!(obj.to_string());
            for key in keys {
                println!("{}", key)
            }
            // for (key, v) in obj {
            //     match v {
            //         Value::Null => {
            //             // println!("    {}", "null");

            //             if !btm_strut.contains_key(&key.to_string()) {
            //                 btm_strut.insert(key, "Null".to_string());
            //             }
            //         }
            //         Value::Bool(_v) => {
            //             // println!("    {}", "bool");
            //             if !btm_strut.contains_key(&key.to_string()) {
            //                 btm_strut.insert(key, "Null".to_string());
            //             };
            //         }
            //         Value::Number(_v) => {
            //             // println!("    {}", "num");
            //             if !btm_strut.contains_key(&key.to_string()) {
            //                 btm_strut.insert(key, "num".to_string());
            //             }
            //         }
            //         Value::String(_v) => {
            //             // println!("    {}", "String");
            //             if !btm_strut.contains_key(&key.to_string()) {
            //                 btm_strut.insert(key, "String".to_string());
            //             }
            //         }
            //         Value::Array(_v) => {
            //             let re = &key.to_string();
            //             if !_v.is_empty() && !btm_strut.contains_key(&key.to_string()) {
            //                 btm_strut.insert(key.to_string(), re.to_string());
            //                 parse_arr(key.to_string(), serde_json::Value::Array(_v.clone()));
            //             }
            //         }
            //         Value::Object(v) => {
            //             let len = v.clone().len();
            //             if len != 0 && !btm_strut.contains_key(&key.to_string()) {
            //                 btm_strut.insert(key.to_string(), "Vec<None>".to_string());

            //                 obj_vec.push(v.clone());
            //                 parse_object(key.as_str(), &serde_json::Value::Object(v));
            //             }
            //         }
            //     }
            // }
            // for arr_items in &arr_vec {
            //     if !arr_items.is_empty()
            //     {
            //         // let arr = arr_items.as_array().unwrap().clone();
            //         println!("arry {} param{:#?}",obj_key,"arr_items");
            //         parse_arr(obj_key.to_string(),serde_json::Value::Array(arr_items.to_vec()));
            //     }
            // }
            // for v in &obj_vec {
            //     parse_object(obj_key, serde_json::Value::Object(v.clone()));
            // }
            let mut tree = FILE_STRUCTS.lock().unwrap();
            if !tree.contains_key(obj_key) {
                tree.insert(obj_key.to_string(), btm_strut.clone());
            }
        }
        _ => {}
    }
    // println!("{}__btree:{:#?}", obj_key, btm_strut);
    // keys_vec
}

// fn parse_arr(key: String, arr_param: Value) {
//     let arr = arr_param.as_array().unwrap().clone();
//     if !arr.is_empty() {
//         let _arr_vec: Vec<Vec<Value>> = Vec::new();
//         // let mut obj_vec: Vec<serde_json::Map<String, Value>> = Vec::new();
//         // let mut keys_vec: Vec<String> = Vec::new();
//         let _arr_struct: Vec<Map<String, String>> = Vec::new();
//         // let mut root_iter: Vec<_> = arr
//         //     .iter()
//         //     .filter(|item| match_value((*item).clone()) == 1)
//         //     // .map(|p| p.0.clone();p.1.clone()))
//         //     .collect();
//         let arry_iter: Vec<_> = arr.iter().filter(|item| item.is_array()).collect();
//         let obj_iter: Vec<_> = arr.iter().filter(|item| item.is_object()).collect();

//         for item in arry_iter {
//             // println!("arr{}:{}", key, match_value(item.clone()));
//             parse_arr(key.to_string(), item.clone());
//         }
//         for item in obj_iter {
//             parse_object(&key, &item);
//             // println!("ojb{}:{}", key, match_value(item.clone()));
//         }
//     }

//     // let mut root_iter: Vec<_> = arr
//     //             .iter()
//     //             .filter(|item|match_value(item.1.clone())==1)
//     //             // .map(|p| p.0.clone();p.1.clone()))
//     //             .collect();
//     //         let mut arry_iter:Vec<_> = arr
//     //             .iter()
//     //             .filter(|data| match_value(data.1.clone()) == 2).collect();
//     //         let mut obj_iter:Vec<_> = arr
//     //             .iter()
//     //             .filter(|data| match_value(data.1.clone()) == 3).collect();
//     // for k,v in arr {
//     //     // println!("{:#?}:{:#?}", key, v);
//     //     // print!("{:#?}:", key);
//     //     match v {
//     //         Value::Null => println!("    {}", v),
//     //         Value::Bool(_v) => println!("    {}", "bool"),
//     //         Value::Number(_v) => println!("    {}", "num"),
//     //         Value::String(_v) => println!("    {}", "String"),
//     //         Value::Array(v) => {
//     //             arr_vec.push(v);
//     //         }
//     //         Value::Object(v) => {
//     //             obj_vec.push(v);
//     //         }
//     //     }
//     // }
//     // for v in &arr_vec {
//     //     // parse_arr(v.clone());
//     // }

//     // for v in &obj_vec {
//     //     // let _keys = parse_object(v.clone(), &mut keys_vec);
//     // }
//     // print!("NewObject:{:#?}:", &keys_vec);
// }
