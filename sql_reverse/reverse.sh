./target/debug/sql_reverse mysql -f reverse_mysql.yml -s .go

target\release\sql_reverse.exe  mysql -f reverse_mysql.yml -s rs -n sqlx.tera

target\debug\sql_reverse.exe  mysql -f reverse_mysql.yml -s rs -n rbatis.tera

target\debug\sql_reverse.exe  mysql -f reverse_mysql.yml -s rs -n sea-orm-handler.tera
#sql_reverse postgres
#cargo install sql_reverse


sql_reverse.exe  mysql -f reverse_mysql.yml -s rs -n r.tera -m 1


use rbatis::rbdc::datetime::DateTime;
use serde_json::json;
use serde::{Serialize,Deserialize}