#![allow(unused_imports, dead_code)]
use std::io::stdin;
use std::fs;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use yansi::{Paint,Color};
use regex::Regex;
//use serde_json::{Key, Value};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STOCKSELL_REGEX: &str = r"^\[(.+)\]\*(-?\d+)$";

enum TransactionType {
    Sell,
    Stock,
    Gift,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
struct Product {
    name: String,
    sku: String,
    stock: i32,
    sold: i32,
    released: String,
    retired: bool,
}

impl Product {
    fn new() -> Self {
        Self {
            name: "Default product".to_string(),
            sku: "NOSKU".to_string(),
            stock: 0,
            sold: 0,
            released: "01/01/1900".to_string(),
            retired: false,
        }
    }
    pub fn sell_product(&mut self, negative: i32) {
        //self.stock += negative;
        //self.sold -= negative;
        self.print_product(TransactionType::Sell);
    }
    pub fn stock_product(&mut self, positive: i32) {
        //self.stock += positive;
        self.print_product(TransactionType::Stock);
    }
    pub fn gift_product(&mut self) {
        //self.stock -= 1;
        self.print_product(TransactionType::Gift);
    }

    pub fn print_product(&self, trans_type: TransactionType) {
        match trans_type {
            TransactionType::Stock => println!("{} [In Stock: {} Sold: {}]", Paint::green(&self.name), Paint::green(&self.stock), self.sold),
            TransactionType::Sell => println!("{} [In Stock: {} Sold: {}]", Paint::red(&self.name), self.stock, Paint::red(&self.sold)),
            TransactionType::Gift => println!("{} [In Stock: {} Sold: {}]", self.name, self.stock, self.sold),
        }
    }
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
    pub fn result_stock_or_sell(&self, code: String) -> Option<Vec<String>> {
        let re = Regex::new(STOCKSELL_REGEX).unwrap();
        if re.is_match(code.as_str()) {
            let caps = re.captures(code.as_str()).unwrap();
            if self.valid_sku(caps.get(1).unwrap().as_str().to_string()){
                return Some(vec![caps.get(1).unwrap().as_str().to_string(), caps.get(2).unwrap().as_str().to_string()]);
            }
        }
        None
    }
    pub fn action_stock_or_sell(&self, action: Vec<String>) {
        let sku = &action[0];
        let quantity: i32 = action[1].parse().unwrap();

        for each_product in self.product_list.iter_mut() {
            if *each_product.sku == sku.to_string() {
                match quantity {
                    x if x > 0 => each_product.stock_product(quantity),
                    x if x < 0 => each_product.sell_product(quantity),
                    _ => each_product.gift_product(),
                };
            }
        }
    }
    pub fn valid_action_code(&self, code: String) -> bool {
        let re = Regex::new(STOCKSELL_REGEX).unwrap();
        //self.regex_quantity_result(code.clone());

        if re.is_match(code.as_str()) {
            return true;
        }
        return false;
    }
    pub fn valid_sku(&self, sku_to_check: String) -> bool {
        for each_product in self.product_list.iter() {
            if each_product.sku == sku_to_check {
                //println!("Product found");
                return true;
            }
        }
        false
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
    println!("\n---{}--- {}{}", Paint::green("Inventory Server Product Scanner"), Paint::yellow("V."), Paint::yellow(VERSION));
    println!("Scan '{}' to add a new product.", Paint::blue("Q+[SKU](Product Name)#"));
    println!("Scan '{}' with +/- numbers for stock/sell respectively.", Paint::blue("[SKU]*#"));

    loop {
        let choice = user_input().trim().replace("\n","");
        if choice == "quit" {
            return ();
        }
        match product_scanner.result_stock_or_sell(choice.clone()) {
            Some(x) => product_scanner.action_stock_or_sell(x),
            None => println!("Not valid"),
        }
    }

}
