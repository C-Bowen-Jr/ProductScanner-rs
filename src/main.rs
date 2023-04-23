#![allow(unused_imports, dead_code)]
use std::io::stdin;
use std::fs;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
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
    fn sell_product(&mut self, negative: i32) {
        self.stock += negative;
        self.sold -= negative;
        self.print_product(TransactionType::Sell);
    }
    fn stock_product(&mut self, positive: i32) {
        self.stock += positive;
        self.print_product(TransactionType::Stock);
    }
    fn gift_product(&mut self) {
        self.stock -= 1;
        self.print_product(TransactionType::Gift);
    }

    fn print_product(&self, trans_type: TransactionType) {
        match trans_type {
            TransactionType::Stock => println!("{} [In Stock: {} Sold: {}]", Paint::green(&self.name), Paint::green(&self.stock), self.sold),
            TransactionType::Sell => println!("{} [In Stock: {} Sold: {}]", Paint::red(&self.name), self.stock, Paint::red(&self.sold)),
            TransactionType::Gift => println!("{} [In Stock: {} Sold: {}]", self.name, self.stock, self.sold),
        }
    }
}

#[derive(Debug)]
struct InventoryApp {
    product_list: HashMap<String,Product>,
}

impl InventoryApp {
    fn new() -> Self {
         Self {
            product_list: json_object("./src/Products.json".to_owned()),
        }
    }
    fn result_stock_or_sell(&self, code: String) -> Option<Vec<String>> {
        let re = Regex::new(STOCKSELL_REGEX).unwrap();
        if re.is_match(code.as_str()) {
            let caps = re.captures(code.as_str()).unwrap();
            return Some(vec![caps.get(1).unwrap().as_str().to_string(), caps.get(2).unwrap().as_str().to_string()]);
        }
        None
    }
    fn product_by_sku(&mut self, sku: &str) -> Option<&mut Product> {
        self.product_list.get_mut(sku)
    }
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

fn json_object(json_string: String) -> HashMap<String, Product> {
    let product_vector: Vec<Product> = serde_json::from_str(&load_json_to_string(json_string))
    .unwrap_or_default();
    let mut prehash_map: Vec<(String,Product)> = vec![];

    for each_product in product_vector {
        prehash_map.push((each_product.sku.clone(),each_product));
    }
    println!("load from json: {:?}", prehash_map);
    prehash_map.into_iter().collect()
}

fn main() {
    let _time_thread = thread::spawn(|| {
        loop {
            //println!("{}", Paint::green("Color and time check"));
            thread::sleep(Duration::from_secs(60));
        }
    });

    let mut product_scanner = InventoryApp::new();
    println!("\n---{}--- {}{}", Paint::green("Inventory Server Product Scanner"), Paint::yellow("V."), Paint::yellow(VERSION));
    println!("Scan '{}' to add a new product.", Paint::blue("Q+[SKU](Product Name)#"));
    println!("Scan '{}' with +/- numbers for stock/sell respectively.", Paint::blue("[SKU]*#"));

    loop {
        let choice = user_input().trim().replace("\n","");
        if choice == "quit" {
            return ();
        }

        if let Some(found_sell_stock) = product_scanner.result_stock_or_sell(choice.clone()) {
            if let Some(found_product) = product_scanner.product_by_sku(found_sell_stock[1].as_str()) {
                let quantity: i32 = found_sell_stock[2].parse().unwrap();
                match quantity {
                    x if x > 0 => found_product.stock_product(quantity),
                    x if x < 0 => found_product.sell_product(quantity),
                    _ => found_product.gift_product(),
                }
            }
            else {
                println!("found product check invalid");
            }
        }
        else {
            println!("found_sell_stock check invalid");
        }
    }

}
