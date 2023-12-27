//pub mod index;
//pub use index::Index;

/// 依据条件获取结果记录
/// ```
/// record_list!(); //后台模型默认
/// ```
// #[macro_export]
// macro_rules! record_list {
//     () => {
//         /// 用户列表
//         pub async fn list(db: &Pool) -> anyhow::Result<JsonPager<ModelRow>> {
//             Self::set_pager::<ModelRow>(
//                 sqlx::query(&Self::get_query())
//                     .map(|r: PgRow| Self::get_row_filter(&r))
//                     .fetch_all(db.as_ref())
//                     .await,
//             )
//         }
//     };
// }
// #[macro_export]
// macro_rules! record_list_page {
//     () => {
//         pub async fn list_list_page(
//             page: Option<u16>,
//             size: Option<u16>,
//             db: &Pool,
//         ) -> anyhow::Result<JsonPager<ModelRow>> {
//             Self::set_pager::<ModelRow>(
//                 sqlx::query(&Self::get_query())
//                     .map(|r: PgRow| Self::get_row_filter(&r))
//                     .fetch_all(db.as_ref())
//                     .await,
//             )
//         }
//     };
// }
// #[macro_export]
// macro_rules! unfold_fields_my3 {
//     ($r: expr, {$($key: ident => $type: tt,)+}) => {{
//         $(let $key: $type = if let Ok(v) = $r.try_get::<$type, &'static str>(stringify!($key)) { v } else { $type::default() };)+
//         ModelRow {
//             $($key,)+
//         }
//     }};
//     ($r: expr, {$($key: ident => $type: tt,)+}, {$($float_field: ident,)+}) => {{
//         $(let $key: $type = if let Ok(v) = $r.try_get::<$type, &'static str>(stringify!($key)) { v } else { $type::default() };)+
//         $(let $float_field: i8 = if let Ok(v) = $r.try_get::<i8, &'static str>(stringify!($float_field)) {
//             // v.to_owned()
//             v
//         } else { 2 };)+
//         ModelRow {
//             $($key,)+
//             $($float_field,)+
//         }
//     }};
// }

/// 展开数据库字段, 生成默认模型
/// (单行记录, 字段列表)
/// ```
/// unfold_fields!(row, {id => i32, name => String,})
/// ```
/// (单行记录, 字段列表, 浮点字段)
/// ```
/// unfold_fields!(row, {id => i32, name => String,}, {money,})
/// ```
/// (单行记录, 字段列表, 浮点字段, 日期时间字段)
/// ```
/// unfold_fields!(row, {id => i32, name => String,}, {money,}, {created, updated,})
/// ```
/// (单行记录, 字段列表, 浮点字段, 日期时间字段, 日期字段)
/// ```
/// unfold_fields!(row, {id => i32, name => Stirng,}, {money,}, {created, updated,}, {birth,})
/// ```
/// (单行记录, 字段列表, 浮点字段, 日期时间字段, 日期字段, ip字段)
/// ```
/// unfold_fields!(row, {id => i32, name => Stirng,}, {money,}, {created, updated,}, {birth,}, {login_ip,})
/// ```
#[macro_export]
macro_rules! unfold_fields {
    ($r: expr, {$($key: ident => $type: tt,)+}) => {{
        $(let $key: $type = if let Ok(v) = $r.try_get::<$type, &'static str>(stringify!($key)) { v } else { $type::default() };)+
        ModelRow {
            $($key,)+
        }
    }};
    ($r: expr, {$($key: ident => $type: tt,)+}, {$($float_field: ident,)+}) => {{
        $(let $key: $type = if let Ok(v) = $r.try_get::<$type, &'static str>(stringify!($key)) { v } else { $type::default() };)+
        $(let $float_field: f32 = if let Ok(v) = $r.try_get::<sqlx::types::BigDecimal, &'static str>(stringify!($float_field)) {
            v.to_f32().unwrap()
        } else { 0.0 };)+
        ModelRow {
            $($key,)+
            $($float_field,)+
        }
    }};
    ($r: expr, {$($key: ident => $type: tt,)+}, {$($float_field: ident,)*}, {$($datetime_field: ident,)+}) => {{
        $(let $key: $type = if let Ok(v) = $r.try_get::<$type, &'static str>(stringify!($key)) { v } else { $type::default() };)+
        $(let $float_field: f32 = if let Ok(v) = $r.try_get::<sqlx::types::BigDecimal, &'static str>(stringify!($float_field)) {
            v.to_f32().unwrap()
        } else { 0.0 };)*
        $(let $datetime_field: String = if let Ok(v) = $r.try_get::<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>, &'static str>(stringify!($datetime_field)) {
            v.format("%Y-%m-%d %H:%M").to_string()
        } else { "".to_owned() };)+
        ModelRow {
            $($key,)+
            $($float_field,)+
            $($datetime_field,)+
        }
    }};
    ($r: expr, {$($key: ident => $type: tt,)+}, {$($float_field: ident,)*}, {$($datetime_field: ident,)*}, {$($date_field: ident,)+}) => {{
        $(let $key: $type = if let Ok(v) = $r.try_get::<$type, &'static str>(stringify!($key)) { v } else { $type::default() };)+
        $(let $float_field: f32 = if let Ok(v) = $r.try_get::<sqlx::types::BigDecimal, &'static str>(stringify!($float_field)) {
            v.to_f32().unwrap()
        } else { 0.0 };)*
        $(let $datetime_field: String = if let Ok(v) = $r.try_get::<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>, &'static str>(stringify!($datetime_field)) {
            v.format("%Y-%m-%d %H:%M").to_string()
        } else { "".to_owned() };)*
        $(let $date_field: String = if let Ok(v) = $r.try_get::<sqlx::types::chrono::NaiveDate, &'static str>(stringify!($date_field)) {
            v.to_string()
        } else { "".to_owned() };)+
        ModelRow {
            $($key,)+
            $($float_field,)*
            $($datetime_field,)*
            $($date_field,)+
        }
    }};
    ($r: expr, {$($key: ident => $type: tt,)+}, {$($float_field: ident,)*}, {$($datetime_field: ident,)*}, {$($date_field: ident,)*}, {$($ip_field: ident,)+}) => {{
        $(let $key: $type = if let Ok(v) = $r.try_get::<$type, &'static str>(stringify!($key)) { v } else { $type::default() };)+
        $(let $float_field: f32 = if let Ok(v) = $r.try_get::<sqlx::types::BigDecimal, &'static str>(stringify!($float_field)) {
            v.to_f32().unwrap()
        } else { 0.0 };)*
        $(let $datetime_field: String = if let Ok(v) = $r.try_get::<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>, &'static str>(stringify!($datetime_field)) {
            v.format("%Y-%m-%d %H:%M").to_string()
        } else { "".to_owned() };)*
        $(let $date_field: String = if let Ok(v) = $r.try_get::<sqlx::types::chrono::NaiveDate, &'static str>(stringify!($date_field)) {
            v.to_string()
        } else { "".to_owned() };)*
        $(let $ip_field: String = if let Ok(v) = $r.try_get::<sqlx::types::ipnetwork::IpNetwork, &'static str>(stringify!($ip_field)) {
            v.to_string()
        } else { "".to_owned() };)+
        ModelRow {
            $($key,)+
            $($float_field,)*
            $($datetime_field,)*
            $($date_field,)*
            $($ip_field,)+
        }
    }};
}
