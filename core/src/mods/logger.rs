use colored::*;

/// Helper function to process messages and color the ">" symbol red
fn process_arrow_message(message: &str) -> String {
    // Check if message starts with ">" (with or without spaces)
    let trimmed = message.trim_start();
    if trimmed.starts_with('>') {
        // Find the position of the first ">"
        if let Some(pos) = message.find('>') {
            // Split the message at the ">" symbol
            let (before_arrow, after_arrow) = message.split_at(pos);
            let rest = &after_arrow[1..]; // Remove the ">"
            
            // Return message with ">" in red
            return format!("{}{}{}", before_arrow, ">".red(), rest);
        }
    }
    message.to_string()
}

/// Log a message with color
pub fn log_info(message: &str) {
    let processed = process_arrow_message(message);
    println!("{}", processed.normal());
}

pub fn log_success(message: &str) {
    let processed = process_arrow_message(message);
    println!("{}", processed.green());
}

pub fn log_warning(message: &str) {
    let processed = process_arrow_message(message);
    println!("{}", processed.yellow());
}

pub fn log_error(message: &str) {
    let processed = process_arrow_message(message);
    eprintln!("{}", processed.red());
}

pub fn log_step(message: &str) {
    let processed = process_arrow_message(message);
    println!("{}", processed.blue().bold());
}
