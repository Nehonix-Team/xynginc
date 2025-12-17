use colored::*;

/// Helper function to process messages and color the ">" symbol red
fn process_arrow_message(message: &str, color_fn: impl Fn(&str) -> ColoredString) -> String {
    // Check if message starts with ">" (with or without leading spaces)
    let trimmed = message.trim_start();
    if trimmed.starts_with('>') {
        // Find the position of the first ">"
        if let Some(pos) = message.find('>') {
            // Split the message at the ">" symbol
            let (before_arrow, after_arrow) = message.split_at(pos);
            let rest = &after_arrow[1..]; // Remove the ">"
            
            // Return message with ">" in red and rest in specified color
            return format!("{}{}{}", before_arrow, ">".red().bold(), color_fn(rest));
        }
    }
    color_fn(message).to_string()
}

/// Log a message with color
pub fn log_info(message: &str) {
    let processed = process_arrow_message(message, |s| s.normal());
    println!("{}", processed);
}

pub fn log_success(message: &str) {
    let processed = process_arrow_message(message, |s| s.green());
    println!("{}", processed);
}

pub fn log_warning(message: &str) {
    let processed = process_arrow_message(message, |s| s.yellow());
    println!("{}", processed);
}

pub fn log_error(message: &str) {
    let processed = process_arrow_message(message, |s| s.red());
    eprintln!("{}", processed);
}

pub fn log_step(message: &str) {
    let processed = process_arrow_message(message, |s| s.blue().bold());
    println!("{}", processed);
}
