// Simple chat file system for Claude-SAGE communication
use std::fs::{OpenOptions, File};
use std::io::{Write, BufRead, BufReader};
use std::path::Path;

const CHAT_FILE: &str = "/tmp/sage_chat.txt";
const LAST_LINE_FILE: &str = "/tmp/sage_last_line";

/// Write a message to the shared chat file
pub fn write_message(sender: &str, content: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(CHAT_FILE)
        .map_err(|e| format!("Failed to open chat file: {}", e))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let formatted = format!("[{}] {}: {}\n", timestamp, sender, content);

    file.write_all(formatted.as_bytes())
        .map_err(|e| format!("Failed to write to chat file: {}", e))?;

    Ok(())
}

/// Read new messages from the chat file (since last read)
pub fn read_new_messages() -> Result<Vec<(String, String)>, String> {
    if !Path::new(CHAT_FILE).exists() {
        return Ok(Vec::new());
    }

    // Get last line count
    let last_line = if Path::new(LAST_LINE_FILE).exists() {
        std::fs::read_to_string(LAST_LINE_FILE)
            .unwrap_or_else(|_| "0".to_string())
            .trim()
            .parse::<usize>()
            .unwrap_or(0)
    } else {
        0
    };

    // Read all lines
    let file = File::open(CHAT_FILE)
        .map_err(|e| format!("Failed to open chat file: {}", e))?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

    // Get new lines
    let new_lines: Vec<(String, String)> = lines
        .iter()
        .skip(last_line)
        .filter_map(|line| {
            // Parse: [timestamp] sender: message
            if let Some(rest) = line.strip_prefix("[") {
                if let Some(idx) = rest.find("] ") {
                    let after_timestamp = &rest[idx + 2..];
                    if let Some(colon_idx) = after_timestamp.find(": ") {
                        let sender = after_timestamp[..colon_idx].to_string();
                        let message = after_timestamp[colon_idx + 2..].to_string();
                        return Some((sender, message));
                    }
                }
            }
            None
        })
        .collect();

    // Update last line count
    if !new_lines.is_empty() {
        let _ = std::fs::write(LAST_LINE_FILE, lines.len().to_string());
    }

    Ok(new_lines)
}
