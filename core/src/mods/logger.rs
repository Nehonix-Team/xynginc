use colored::*;

/// Log a message with color
pub fn log_info(message: &str) {
    println!("{}", message.normal());
}

pub fn log_success(message: &str) {
    println!("{}", message.green());
}

pub fn log_warning(message: &str) {
    println!("{}", message.yellow());
}

pub fn log_error(message: &str) {
    eprintln!("{}", message.red());
}

pub fn log_step(message: &str) {
    println!("{}", message.blue().bold());
}
