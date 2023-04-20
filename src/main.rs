use std::io::stdin;
use std::fs;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use yansi::{Paint,Color};

//use serde_json::{Key, Value};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
struct Product {
    name: String,
    sku: String,
    stock: u32,
    sold: u32,
    released: String,
    retired: bool,
}

#[derive(Debug)]
struct InventoryApp {
    debug_state: bool,
    product_list: Vec<Product>,
}

fn user_input() -> String {
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_x) => return buffer.to_string(),
        Err(_e) => return "Error".to_string()
    }
}

fn load_json_to_string(file_path: String) -> String {
    fs::read_to_string(file_path).unwrap_or_default()
}

fn json_object(json_string: String) -> Vec<Product> {
    serde_json::from_str(&load_json_to_string(json_string))
    .unwrap_or_default()
}

impl InventoryApp {
    fn new() -> Self {
         Self {
            debug_state: true,
            product_list: json_object("./src/Products.json".to_owned()),
        }
    }
    fn valid_action_code(code: String) -> bool {
        //println!("validator got: [{}]", code.clone().replace("\n",""));
        if code == "test" {
            return true;
        }
        return false;
    }
}

fn main() {
    let _time_thread = thread::spawn(|| {
        loop {
            //println!("{}", Paint::green("Color and time check"));
            thread::sleep(Duration::from_secs(60));
        }
    });

    let product_scanner = InventoryApp::new();

    loop {
        let choice = user_input().trim().replace("\n","");
        if choice == "quit" {
            return ();
        }
        else if InventoryApp::valid_action_code(choice) {
            println!("valid");
        }
        else {
            println!("invalid");
        }
    }

}
