#![allow(unused_imports, dead_code)]
use std::io::stdin;
use std::io::Write;
use std::fmt;
use std::fs;
use std::fs::File;
use std::thread;
use std::time::Duration;
use std::collections::HashMap;
use chrono;
use serde::{Serialize, Deserialize};
use yansi::{Paint,Color};
use regex::Regex;
use askama::Template;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const STOCKSELL_REGEX: &str = r"^(\w+)\*(-?\d+)$";
const NEWPRODUCT_REGEX: &str = r"^Q\+\[([\w]+)\]\(([\w\s]+)\)(\d)$";
const INSPECT_REGEX: &str = r"^inspect:(\w+)$";
const RETIRE_REGEX: &str = r"^(retire|restore):(\w+)$";
#[cfg(debug_assertions)]
const SAVEFILE: &str = "./src/Products.json";
#[cfg(not(debug_assertions))]
const SAVEFILE: &str = "./Products.json";

#[derive(Template)]
#[template(path = "report_template.html")]

struct ServerTemplate<'a> {
    report_product_list: &'a Vec<Product>,
    report_weekly_sold: &'a i32,
    report_weekly_produced: &'a i32,
    report_total_sold: &'a i32,
    report_total_produced: &'a i32,
    report_currently_stocked: &'a i32,
    report_terminal_log: &'a Vec<LogType>,
}

enum TransactionType {
    Sell,
    Stock,
    Gift,
}

#[derive(Clone)]
enum LogType {
    Info(String),
    Error(String),
}

#[derive(PartialEq)]
enum MenuItems {
    Report,
    Email,
    Help,
    Invalid,
    Quit,
}

impl From<String> for MenuItems {
    fn from(command: String) -> Self {
        // We use &str from command
        match command.to_lowercase().as_str() {
            "report" => Self::Report,
            "email" => Self::Email,
            "help" => Self::Help,
            "quit" => Self::Quit,
            _ => Self::Invalid,
        }
    }
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

impl fmt::Display for LogType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LogType::Info(text) => write!(f, "{}", text),
            LogType::Error(text) => write!(f, "{}", text),
        }
    }
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
    product_list: HashMap<String,Product>, // SKU string, Product struct
    weekly_produced: i32,
    weekly_sold: i32,
}

impl InventoryApp {
    fn new() -> Self {
         Self {
            product_list: json_object(SAVEFILE.to_owned()),
            weekly_produced: 0,
            weekly_sold: 0,
        }
    }
    fn result_stock_or_sell(&self, code: String) -> Option<Vec<String>> {
        let re = Regex::new(STOCKSELL_REGEX).unwrap();
        if re.is_match(code.as_str()) {
            let caps = re.captures(code.as_str()).unwrap();
            let cap_sku = caps.get(1).unwrap().as_str().to_string();
            let cap_quantity = caps.get(2).unwrap().as_str().to_string();
            return Some(vec![cap_sku, cap_quantity]);
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
    fn result_inspect(&self, code: String) -> Option<String> {
        let re = Regex::new(INSPECT_REGEX).unwrap();
        if re.is_match(code.as_str()) {
            let caps = re.captures(code.as_str()).unwrap();
            let cap_sku = caps.get(1).unwrap().as_str().to_string();
            return Some(cap_sku);
        }
        None
    }
    fn result_retire(&self, code: String) -> Option<Vec<String>> {
        let re = Regex::new(RETIRE_REGEX).unwrap();
        if re.is_match(code.as_str()) {
            let caps = re.captures(code.as_str()).unwrap();
            let cap_sku = caps.get(2).unwrap().as_str().to_string();
            let cap_retire = caps.get(1).unwrap().as_str().to_string();
            // Keep sku consistently as the first element in the result_inspect
            return Some(vec![cap_sku, cap_retire]);
        }
        None
    }
    fn product_by_sku(&mut self, sku: &str) -> Option<&mut Product> {
        self.product_list.get_mut(sku)
    }
    fn build_server_report(&self, log: Vec<LogType>) {
        // Calculate values
        let mut sorted_products: Vec<_> = self.product_list
            .clone()
            .into_iter()
            .map(|(_, v)| v)
            .collect();
        sorted_products.sort_by(|a,b| a.sku.cmp(&b.sku));

        // Build
        let report_gen = ServerTemplate {
            report_product_list: &sorted_products,
            report_weekly_sold: &self.weekly_sold,
            report_weekly_produced: &self.weekly_produced,
            report_total_sold: &self.total_sold(),
            report_total_produced: &self.total_produced(),
            report_currently_stocked: &self.currently_stocked(),
            report_terminal_log: &log,

        };
        // Output is either debug to output.html or release to str?String?
        //println!("{}", report_gen.render().unwrap());
        let mut file = File::create("./report.html").unwrap();
        let file_ref = file.by_ref();
        report_gen.write_into(file_ref).unwrap();
    }
    fn total_sold(&self) -> i32 {
        let mut sum_of_sold = 0;
        self.product_list.values().for_each(|v| sum_of_sold += v.sold);
        sum_of_sold
    }
    fn total_produced(&self) -> i32 {
        let mut sum_of_produced = 0;
        self.product_list.values().for_each(|v| sum_of_produced += v.sold + v.stock);
        sum_of_produced
    }
    fn currently_stocked(&self) -> i32 {
        let mut sum_of_stocked = 0;
        self.product_list.values().for_each(|v| sum_of_stocked += v.stock);
        sum_of_stocked
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
    fs::write(SAVEFILE, serde_json::to_string_pretty(&extracted_products).unwrap());
}

fn main() {
    let _time_thread = thread::spawn(|| {
        loop {
            //println!("{}", Paint::green("Color and time check"));
            thread::sleep(Duration::from_secs(60));
        }
    });

    let mut product_scanner = InventoryApp::new();
    let mut report_log: Vec<LogType> = vec![];
    Paint::enable_windows_ascii();

    println!("\n---{}--- {}{}", Paint::green("Inventory Server Product Scanner"), Paint::yellow("V."), Paint::yellow(VERSION));
    println!("Scan '{}' to add a new product.", Paint::blue("Q+[SKU](Product Name)#"));
    println!("Scan '{}' with +/- numbers for stock/sell respectively.", Paint::blue("SKU*#"));

    loop {
        let now = format!("{}", chrono::offset::Local::now().format("%m/%d/%Y"));
        let choice = user_input().trim().replace("\n","");
        let menu_option = MenuItems::from(choice.clone());
        /*if choice == "quit" {
            return ();
        }
        if choice == "report" {
            println!("Generating report");
            product_scanner.build_server_report(report_log.clone());
        }*/

        // Action code on SELL, STOCK, or GIFT
        if let Some(found_sell_stock) = product_scanner.result_stock_or_sell(choice.clone()) {
            if let Some(found_product) = product_scanner.product_by_sku(found_sell_stock[0].as_str()) {
                let quantity: i32 = found_sell_stock[1].parse().unwrap();
                match quantity {
                    x if x > 0 => {
                        report_log.push(LogType::Info(format!("[{}] {}", now, choice)));
                        found_product.stock_product(quantity);
                        product_scanner.weekly_produced += quantity;
                    },
                    x if x < 0 => {
                        report_log.push(LogType::Info(format!("[{}] {}", now, choice)));
                        found_product.sell_product(quantity);
                        product_scanner.weekly_sold -= quantity;
                    },
                    _ => {
                        found_product.gift_product();
                        report_log.push(LogType::Info(format!("[{}] {}", now, choice)));
                    },
                }
                save_to_json(&product_scanner.product_list);
            }
            else {
                println!("'{}' is not a product", Paint::red(found_sell_stock[0].as_str()));
                report_log.push(LogType::Error(format!("[{}] '{}' is not a product", now, found_sell_stock[0].as_str())));
            }
        }
        // Action code on ADD PRODUCT
        else if let Some(new_product) = product_scanner.result_new_product(choice.clone()) {
            if let Some(_already_exists) = product_scanner.product_by_sku(new_product[0].as_str()) {
                println!("'{}' already exiists", Paint::red(new_product[0].as_str()));
                report_log.push(LogType::Error(format!("[{}] '{}' already exiists", now, new_product[0].as_str())));
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
                product_scanner.weekly_produced += &build_product.stock;
                product_scanner.product_list.insert(new_product[0].clone(), build_product);
                save_to_json(&product_scanner.product_list);
            }
        }
        // Action code on INSPECT
        else if let Some(inspect_product) = product_scanner.result_inspect(choice.clone()) {
            if let Some(found_product) = product_scanner.product_by_sku(inspect_product.as_str()) {
                found_product.print_product(TransactionType::Gift);
            }
            else {
                println!("{} is not a product", Paint::red(inspect_product.as_str()));
            }
        }
        // Action code on RETIRE/RESTORE
        else if let Some(retire_product) = product_scanner.result_retire(choice.clone()) {
            if let Some(found_product) = product_scanner.product_by_sku(retire_product[0].as_str()) {
                let currently_retired: bool = found_product.retired;
                match (retire_product[1].as_str(), currently_retired) {
                    ("retire", true) => println!("{} is already retired", Paint::red(retire_product[0].as_str())),
                    ("retire", false) => found_product.retired = true,
                    ("restore", false) => println!("{} is already active", Paint::red(retire_product[0].as_str())),
                    ("restore", true) => found_product.retired = false,
                    _ => println!("Something went wrong in retire/stock \n{:?}", retire_product),
                }
                save_to_json(&product_scanner.product_list);
            }
            else {
                println!("'{}' is not a product", Paint::red(retire_product[0].as_str()));
                report_log.push(LogType::Error(format!("[{}] '{}' is not a product", now, retire_product[0].as_str())));
            }
        }
        else if menu_option != MenuItems::Invalid {
            match menu_option {
                MenuItems::Report => { println!("Generating report"); product_scanner.build_server_report(report_log.clone()); },
                MenuItems::Email => println!("Not implemented"),
                MenuItems::Help => println!("\n\nMenu options:\nReport - Will generate local .html report\nEmail - Will force Email send\nQuit - Exit program"),
                MenuItems::Quit => return (),
                MenuItems::Invalid => println!("Unrepresentable"),
            }
        }
        else {
            println!("'{}' is not an action", Paint::red(choice.clone()));
            report_log.push(LogType::Error(format!("[{}] '{}' is not an action", now, choice.clone())));
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
        let test_app = InventoryApp::new();
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
    #[test]
    fn test_invalid_inspect() {
        let test_app = InventoryApp::new();
        assert_eq!(test_app.result_inspect("inspect:cant be sku".to_string()),None);
    }
    #[test]
    fn test_valid_inspect() {
        let test_app = InventoryApp::new();
        assert_eq!(test_app.result_inspect("inspect:NOSKU".to_string()).is_some(),true);
    }
}
