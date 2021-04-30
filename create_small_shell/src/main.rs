//use std::env;
use std::io::{stdin/*, stdout, Write*/};
//use std::path::Path;
use std::process::{/*Child,*/ Command/*, Stdio*/};

fn main() {
    // 新しい空のStringを作成
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();

    // `read_line`は末尾に改行を残しますが、`trim`はこれを削除します。
    let command = input.trim();

    // 
    Command::new(command).spawn().unwrap();
}
