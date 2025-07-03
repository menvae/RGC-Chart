#![allow(clippy::explicit_write)]

use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::str;

pub fn read_file_to_string(file_path: &str) -> io::Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(String::from_utf8_lossy(&contents).to_string())
}

pub fn write_to_file(file_path: &str, content: &str) -> std::io::Result<()> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut file = fs::File::create(file_path)?;
    file.write_all(content.as_bytes())?;
    println!("File written successfully to {}", file_path);
    Ok(())
}

pub fn sanitize_str(filename: &str) -> String {
    const FORBIDDEN_CHARS: &[char] = &['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>', '\0', '\n'];
    
    let mut sanitized = String::with_capacity(filename.len());
    
    for ch in filename.chars() {
        if FORBIDDEN_CHARS.contains(&ch) {
            sanitized.push('_');
        } else if ch.is_control() {
            sanitized.push('_');
        } else {
            sanitized.push(ch);
        }
    }
    
    sanitized = sanitized.trim_matches(|c| c == '.' || c == ' ').to_string();
    
    if sanitized.is_empty() {
        sanitized = "untitled".to_string();
    }
    
    if sanitized.len() > 255 {
        sanitized.truncate(255);
    }
    
    sanitized
}

#[inline]
pub fn println_test(func_name: &str, color_code: &str, message: &str) {
    writeln!(io::stdout(), "{} || {}{}\x1b[0m", func_name, color_code, message).unwrap();
}

#[macro_export]
macro_rules! parse_and_convert {
    ($name:ident, $file_path:expr, $parse_fn:expr, $convert_fn:expr, $write_to_file:expr) => {{
        use std::time::Instant;
        use std::io::{self, Write};
        use std::path::Path;
        use self::Chart;
        
        println_test(stringify!($name), "\x1b[34m", "Started running test");

        let result = (|| -> Result<(Chart, String), Box<dyn std::error::Error>> {
            let input_path = $file_path;
            
            let raw_chart = read_file_to_string(input_path).map_err(|e| {
                println_test(stringify!($name), "\x1b[31m", &format!("ERROR: Failed to read file '{}': {:?}", input_path, e));
                e
            })?;

            let mut start = Instant::now();
            let chart = $parse_fn(&raw_chart).map_err(|e| {
                println_test(stringify!($name), "\x1b[31m", &format!("ERROR: Parsing failed: {:?}", e));
                e
            })?;
            let parse_duration = start.elapsed();

            start = Instant::now();
            let converted_chart = $convert_fn(&chart).map_err(|e| {
                println_test(stringify!($name), "\x1b[31m", &format!("ERROR: Conversion failed: {:?}", e));
                e
            })?;
            let convert_duration = start.elapsed();
            
            #[allow(clippy::explicit_write)]
            writeln!(
                io::stdout(),
                "{} || \x1b[36mParsing Time taken: {:?}\x1b[0m\n{} || \x1b[33mConverting Time taken: {:?}\x1b[0m\n{} || \x1b[32mTotal Time taken: {:?}\x1b[0m",
                stringify!($name), parse_duration,
                stringify!($name), convert_duration,
                stringify!($name), parse_duration + convert_duration
            ).unwrap();
            
            if $write_to_file {
                let function_name = stringify!($name);
                let extension = function_name.split('_').last().unwrap_or("txt");
                
                let sanitized_title = sanitize_str(&chart.metadata.title);
                let sanitized_difficulty = sanitize_str(&chart.chartinfo.difficulty_name);
                
                let filename = format!("{}[{}].{}", 
                    sanitized_title, 
                    sanitized_difficulty, 
                    extension
                );
                
                let output_path = format!("test_export/{}", filename);
                let output_dir = Path::new("test_export");
                if !output_dir.exists() {
                    std::fs::create_dir_all(output_dir)?;
                }
                
                write_to_file(&output_path, &converted_chart).map_err(|e| {
                    println_test(stringify!($name), "\x1b[31m", &format!("ERROR: Failed to write to file '{}': {:?}", output_path, e));
                    e
                })?;
            }
            
            Ok((chart, converted_chart))
        })();

        match result {
            Ok(val) => val,
            Err(e) => {
                println_test(stringify!($name), "\x1b[31m", &format!("ERROR: Test execution failed: {:?}", e));
                return;
            }
        }
    }};
}