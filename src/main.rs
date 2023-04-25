#![allow(unused_imports, dead_code)]
use std::io::stdin;
use std::fs;
use std::fs::File;
use std::thread;
use std::time::Duration;
use chrono;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use yansi::{Paint,Color};
use regex::Regex;
//use serde_json::{Key, Value};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STOCKSELL_REGEX: &str = r"^\[(.+)\]\*(-?\d+)$";
const NEWPRODUCT_REGEX: &str = r"^Q\+(\[[\w]+\])\(([\w\s]+)\)(\d)$";

enum TransactionType {
    Sell,
    Stock,
    Gift,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    fn result_new_product(&self, code: String) -> Option<Vec<String>> {
        let re = Regex::new(NEWPRODUCT_REGEX).unwrap();
        if re.is_match(code.as_str()) {
            let caps = re.captures(code.as_str()).unwrap();
            let cap_sku = caps.get(1).unwrap().as_str().to_string();
            let cap_title = caps.get(2).unwrap().as_str().to_string();
            let cap_quantity = caps.get(3).unwrap().as_str().to_string();
            return Some(vec![cap_sku, cap_title, cap_quantity]);
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
    prehash_map.into_iter().collect()
}

fn save_to_json(product_hashmap: &HashMap<String, Product>) {
    let extracted_products: Vec<&Product> = product_hashmap.values().collect();
    //let file = File::open("./src/Products.json".to_owned()).unwrap();
    //let data = serde_json::to_string(&extracted_products).unwrap();
    //file.write(data.as_bytes());
    fs::write("./src/Products.json", serde_json::to_string_pretty(&extracted_products).unwrap());
}

fn main() {
    let _time_thread = thread::spawn(|| {
        loop {
            //println!("{}", Paint::green("Color and time check"));
            thread::sleep(Duration::from_secs(60));
        }
    });

    let mut product_scanner = InventoryApp::new();
    let mut email_log: Vec<String> = vec![];
    println!("\n---{}--- {}{}", Paint::green("Inventory Server Product Scanner"), Paint::yellow("V."), Paint::yellow(VERSION));
    println!("Scan '{}' to add a new product.", Paint::blue("Q+[SKU](Product Name)#"));
    println!("Scan '{}' with +/- numbers for stock/sell respectively.", Paint::blue("[SKU]*#"));

    loop {
        let now = format!("{}", chrono::offset::Local::now().format("%m/%d/%Y"));
        let choice = user_input().trim().replace("\n","");
        if choice == "quit" {
            return ();
        }

        // Action code on SELL, STOCK, or GIFT
        if let Some(found_sell_stock) = product_scanner.result_stock_or_sell(choice.clone()) {
            if let Some(found_product) = product_scanner.product_by_sku(found_sell_stock[0].as_str()) {
                let quantity: i32 = found_sell_stock[1].parse().unwrap();
                match quantity {
                    x if x > 0 => found_product.stock_product(quantity),
                    x if x < 0 => found_product.sell_product(quantity),
                    _ => found_product.gift_product(),
                }
                save_to_json(&product_scanner.product_list);
            }
            else {
                println!("'{}' is not a product", Paint::red(found_sell_stock[0].as_str()));
                email_log.push(format!("'{}' is not a product",found_sell_stock[0].as_str()));
            }
        }
        // Action code on ADD PRODUCT
        else if let Some(new_product) = product_scanner.result_new_product(choice.clone()) {
            if let Some(already_exists) = product_scanner.product_by_sku(new_product[0].as_str()) {
                println!("'{}' already exiists", Paint::red(new_product[0].as_str()));
                email_log.push(format!("'{}' already exiists", new_product[0].as_str()));
            }
            else {
                let new_sku = new_product[0].clone();
                let new_name = new_product[1].clone();
                let new_quantity = new_product[2].clone();
                let new_release_date = now.clone();
                let build_product: Product = Product{
                    name: new_name,
                    sku: new_sku,
                    stock: new_quantity.parse().unwrap(),
                    sold: 0,
                    released: new_release_date,
                    retired: false,
                };
                product_scanner.product_list.insert(new_product[0].clone(), build_product);
                save_to_json(&product_scanner.product_list);
            }
        }
        else {
            println!("'{}' is not an action", Paint::red(choice.clone()));
            email_log.push(format!("'{}' is not an action",choice.clone()));
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_action() {
        let test_app = InventoryApp::new();
        assert_eq!(test_app.result_stock_or_sell("something".to_string()), None);
    }
    #[test]
    fn test_invalid_product() {
        let mut test_app = InventoryApp::new();
        assert_eq!(test_app.product_by_sku("NOT_REAL"), None);
    }
    #[test]
    fn test_valid_product() {
        let mut test_app = InventoryApp::new();
        assert_eq!(test_app.product_by_sku("NOSKU").is_some(), true);
    }
    #[test]
    fn test_invalid_new_product() {
        let mut test_app = InventoryApp::new();
        assert_eq!(test_app.result_new_product("not valid".to_string()), None);
    }
    #[test]
    fn test_valid_new_product() {
        let mut test_app = InventoryApp::new();
        let expected: Vec<String> = vec!["NEW".to_string(),"New product".to_string(),"1".to_string()];
        if let Some(new_product) = test_app.result_new_product("Q+[NEW](New product)1".to_string()) {
            assert_eq!(new_product, expected);
        }
    }
}
