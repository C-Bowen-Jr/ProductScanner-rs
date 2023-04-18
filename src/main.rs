use std::io::stdin;
use std::thread;
use std::time::Duration;

fn user_input() -> String {
    let mut buffer = String::new();
    match stdin().read_line(&mut buffer) {
        Ok(_x) => return buffer.to_string(),
        Err(_e) => return "Error".to_string()
    }
}

fn main() {
    let time_thread = thread::spawn(|| {
        for _i in 0..10 {
            println!("Time check");
            thread::sleep(Duration::from_secs(1));
        }
    });

    let mut choice = String::new();
    while choice != "quit" {
        choice = user_input().trim().to_lowercase().replace("\n","");
        println!(" > {}", choice);
    }

    //time_thread.join().unwrap();
}
