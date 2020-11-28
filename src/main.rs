use anyhow::Result;
use argopt::cmd;
use regex::Regex;
use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;

#[cmd]
fn main(
    #[opt(short, long)] pattern: Option<String>,
    #[opt(short, long)] delimiter: Option<String>,
    output_format: String,
) -> Result<()> {
    let scanner: Box<dyn Scanner> = if pattern.is_some() && delimiter.is_some() {
        panic!("Give pattern xor delimiter.");
    } else if let Some(pattern) = pattern {
        let pattern = Regex::new(&pattern)?;
        Box::new(PatternScanner { pattern: pattern })
    } else if let Some(delimiter) = delimiter {
        let delimiter = Regex::new(&delimiter)?;
        Box::new(DelimiterScanner {
            delimiter: delimiter,
        })
    } else {
        let delimiter = Regex::new(r"\s")?;
        Box::new(DelimiterScanner {
            delimiter: delimiter,
        })
    };
    let formatter = Formatter {
        format: output_format,
    };

    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;

    let scan_line_results = scanner.scan(input);
    let output = formatter.format(scan_line_results);

    std::io::stdout().write_all(output.as_ref())?;

    Ok(())
}

struct ScanLineResult {
    values: BTreeMap<String, String>, // "1" => "aaaa", "captured_name" => "xxxx"
}

trait Scanner {
    fn scan(&self, input: String) -> Vec<ScanLineResult>;
}

struct PatternScanner {
    pattern: Regex,
}

impl Scanner for PatternScanner {
    fn scan(&self, input: String) -> Vec<ScanLineResult>;
}

struct DelimiterScanner {
    delimiter: Regex,
}

impl Scanner for DelimiterScanner {
    fn scan(&self, input: String) -> Vec<ScanLineResult>;
}

struct Formatter {
    format: String,
}

impl Formatter {
    fn format(&self, scan_line_results: Vec<ScanLineResult>) -> String;
}
