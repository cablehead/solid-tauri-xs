// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use xs::store::Store;

fn main() {
    let store = Store::spawn("./store".into());
    stacks2_lib::run()
}
