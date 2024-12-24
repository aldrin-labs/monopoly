use bolt_lang::*;

declare_id!("BKFBy41BBf45X2RQZCBP7Jk3SH6FpXtnN4M91pwM7xDo");

#[component]
#[derive(Default)]
pub struct Position {
    pub x: i64,
    pub y: i64,
    pub z: i64,
    #[max_len(20)]
    pub description: String,
}