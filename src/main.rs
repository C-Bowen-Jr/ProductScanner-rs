use std::io::stdin;
use std::fs;
use std::thread;
use std::time::Duration;
use serde::{Serialize, Deserialize};
use yansi::{Paint,Color};
//use serde_json::{Key, Value};

#[derive(Debug, Serialize, Deserialize)]
struct JsonObject {
    name: String,
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

fn main() {
    let _time_thread = thread::spawn(|| {
        for _i in 0..10 {
            println!("{}", Paint::green("Color and time check"));
            thread::sleep(Duration::from_secs(1));
        }
    });

    let mut choice = String::new();
    while choice != "quit" {
        choice = user_input().trim().to_lowercase().replace("\n","");
        println!(" > {}", load_json_to_string("./src/some.json".to_owned()));
    }

    //time_thread.join().unwrap();
}
