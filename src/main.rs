use dirs;
use std::path::{PathBuf};
use std::io;
use std::fs;
use std::collections::HashMap;
use anyhow::{anyhow};


fn get_config_file_location() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap();
    return home_dir.join(".ssh").join("config");
}


fn read_file(path: &PathBuf) -> io::Result<String> {
    return fs::read_to_string(path);
}


#[derive(Debug)]
struct PatternEntry {
    patterns: Vec<String>,
    options: HashMap<String, String>,
}


impl PatternEntry {
    pub fn is_tunnel(self: &PatternEntry) -> bool {
        return self.options.contains_key("LocalForward");
    }
}


fn parse_config(config: &str) -> anyhow::Result<Vec<PatternEntry>> {
    let mut current_host: Option<PatternEntry> = None;

    let mut result: Vec<PatternEntry> = vec![];

    for line in config.lines() {
        let parsed_line = parse_config_line(line)?;
        if parsed_line.is_none() {
            continue;
        }

        let (name, value) = parsed_line.unwrap();
        match name {
            "Host" => {
                if current_host.is_some() {
                    result.push(current_host.take().unwrap())
                }

                current_host = Some(PatternEntry { patterns: vec![value.to_string()], options: HashMap::default() })
            }
            _ => {
                match current_host {
                    Some(ref mut current_host) => {
                        current_host.options.insert(name.to_string(), value.to_string());
                    }
                    None => return Err(anyhow!("No Host directive before parameter {}", name))
                }
            }
        }
    }

    if current_host.is_some() {
        result.push(current_host.take().unwrap())
    }

    return Ok(result);
}


fn parse_config_line(line: &str) -> anyhow::Result<Option<(&str, &str)>> {
    let line = line.trim();
    if line.starts_with('#') || line.len() == 0 {
        return Ok(None);
    }

    let space_index = line.find(' ');
    if space_index.is_none() {
        return Err(anyhow!("No parameter value found in line: {}", line));
    }

    let param_name = &line[0..space_index.unwrap()];
    let param_value = &line[space_index.unwrap()..line.len()].trim_start();

    return Ok(Some((param_name, param_value)));
}


fn main() {
    let config_location = get_config_file_location();
    let config_contents = read_file(&config_location).unwrap();
    let config = parse_config(config_contents.as_str()).unwrap();
    for entry in config {
        if entry.is_tunnel() {
            println!("[ ] {}", entry.patterns.get(0).unwrap());
        }
    }
}
