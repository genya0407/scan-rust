use anyhow::Result;
use argopt::cmd;
use regex::Regex;
use std::collections::BTreeMap;
use std::io::Read;
use std::io::Write;
extern crate maplit;

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
    std::io::stdout().write_all("\n".as_ref())?;

    Ok(())
}

#[derive(Debug)]
struct ScanLineResult {
    captures: BTreeMap<String, String>, // "1" => "aaaa", "captured_name" => "xxxx"
}

impl ScanLineResult {
    fn get(&self, key: &str) -> String {
        self.captures.get(key).cloned().unwrap_or(String::new())
    }
}

trait Scanner {
    fn scan(&self, input: String) -> Vec<ScanLineResult>;
}

#[cfg(test)]
mod test_pattern_scanner {
    use super::{PatternScanner, Scanner};
    use regex::Regex;

    #[test]
    fn test_scan_named_and_unnamed_patterns() {
        let input = "yusuke.sangenya age:25
unmatched.line age:aaa
taro.tanaka   age:55"
            .to_string();
        let scanner = PatternScanner {
            pattern: Regex::new(r"(?P<first>.+?)\.(?P<last>.+?)\s+age:(\d+)").unwrap(),
        };
        let results = scanner.scan(input);

        assert_eq!(results[0].get("first"), String::from("yusuke"));
        assert_eq!(results[0].get("last"), String::from("sangenya"));
        assert_eq!(results[0].get("3"), String::from("25"));

        // unmatched line
        assert_eq!(results[1].get("1"), String::from(""));

        assert_eq!(results[2].get("first"), String::from("taro"));
        assert_eq!(results[2].get("last"), String::from("tanaka"));
        assert_eq!(results[2].get("3"), String::from("55"));
    }
}

struct PatternScanner {
    pattern: Regex,
}

impl Scanner for PatternScanner {
    fn scan(&self, input: String) -> Vec<ScanLineResult> {
        let mut results = vec![];
        for line in input.lines() {
            let mut line_result = ScanLineResult {
                captures: BTreeMap::new(),
            };
            if let Some(caps) = self.pattern.captures(&line) {
                for (i, name_opt) in self.pattern.capture_names().enumerate() {
                    if let Some(name) = name_opt {
                        // named capture
                        if let Some(value) = caps.name(name) {
                            line_result
                                .captures
                                .insert(name.into(), value.as_str().into());
                        }
                    }

                    // unnamed or named capture
                    // treat named capture as being also unnamed capture
                    if let Some(value) = caps.get(i) {
                        line_result
                            .captures
                            .insert(i.to_string(), value.as_str().into());
                    }
                }
            }
            results.push(line_result);
        }
        results
    }
}

#[cfg(test)]
mod test_delimiter_scanner {
    use super::{DelimiterScanner, Scanner};
    use regex::Regex;

    #[test]
    fn test_scan_named_and_unnamed_patterns() {
        let input = "aaa,bbb,,ccc
a a a
xxx,,yyy,zzz
"
        .to_string();
        let scanner = DelimiterScanner {
            delimiter: Regex::new(r",+").unwrap(),
        };
        let results = scanner.scan(input);

        assert_eq!(results[0].get("1"), String::from("aaa"));
        assert_eq!(results[0].get("2"), String::from("bbb"));
        assert_eq!(results[0].get("3"), String::from("ccc"));

        assert_eq!(results[1].get("1"), String::from("a a a"));
        assert_eq!(results[1].get("2"), String::from(""));

        assert_eq!(results[2].get("1"), String::from("xxx"));
        assert_eq!(results[2].get("2"), String::from("yyy"));
        assert_eq!(results[2].get("3"), String::from("zzz"));
    }
}

struct DelimiterScanner {
    delimiter: Regex,
}

impl Scanner for DelimiterScanner {
    fn scan(&self, input: String) -> Vec<ScanLineResult> {
        let mut results = vec![];
        for line in input.lines() {
            let mut line_result = ScanLineResult {
                captures: BTreeMap::new(),
            };

            // any lines matches as 0
            line_result.captures.insert("0".to_string(), line.into());

            for (i, part) in self.delimiter.split(&line).enumerate() {
                line_result
                    .captures
                    .insert((i + 1).to_string(), part.into());
            }
            results.push(line_result);
        }
        results
    }
}

#[cfg(test)]
mod test_formatter {
    use super::{Formatter, ScanLineResult};

    #[test]
    fn test_scan_named_and_numbered_format() {
        let formatter = Formatter {
            format: "{1}: {some_name}, {non_existent_name}".to_string(),
        };
        let scan_line_results = vec![
            ScanLineResult {
                captures: btreemap! {
                    String::from("1") => String::from("hoge"),
                    String::from("some_name") => String::from("fuga"),
                },
            },
            ScanLineResult {
                captures: btreemap! {},
            },
            ScanLineResult {
                captures: btreemap! {
                    String::from("1") => String::from("hoge"),
                },
            },
        ];
        let output = formatter.format(scan_line_results);
        let expected_output = "hoge: fuga, 
: , 
hoge: , "
            .to_string();
        assert_eq!(output, expected_output);
    }
}

struct Formatter {
    format: String,
}

impl Formatter {
    fn format(&self, scan_line_results: Vec<ScanLineResult>) -> String {
        let targets = Regex::new(r"\{(.+?)\}")
            .unwrap()
            .captures_iter(&self.format)
            .map(|caps| {
                (
                    caps.get(1).unwrap().as_str().into(),
                    Regex::new(&["\\{", caps.get(1).unwrap().as_str(), "\\}"].join("")).unwrap(),
                )
            })
            .collect::<Vec<(String, Regex)>>();
        let mut lines = vec![];
        for result in scan_line_results {
            let mut line = self.format.clone();
            for (target, target_regex) in targets.iter() {
                line = target_regex
                    .replace_all(&line, result.get(&target).as_str())
                    .into();
            }
            lines.push(line);
        }
        lines.join("\n")
    }
}
