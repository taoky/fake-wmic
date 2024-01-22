use std::env;
use std::io::{self, BufRead, Write};
use std::process::exit;

use regex::Regex;

#[derive(Copy, Clone)]
enum ExecuteStateEnum {
    Command,
    SubStart,
    FilterInside,
    ActionInside,
}

fn execute(command: Vec<String>, output: &mut impl Write) {
    macro_rules! output {
        ($($arg:tt)*) => {
            output.write_all(format!($($arg)*).as_bytes()).unwrap();
        }
    }
    macro_rules! outputln {
        ($($arg:tt)*) => {
            output.write_all(format!($($arg)*).as_bytes()).unwrap();
            output.write_all(b"\n").unwrap();
        }
    }
    if command.is_empty() {
        return;
    }

    let disk_re = Regex::new(r#"name\s*=\s*['"]([a-zA-Z]:)['"]"#).unwrap();

    let mut state = ExecuteStateEnum::Command;
    let mut cmd: Option<String> = None;
    let mut drive_list: Vec<String> = ["C:".to_owned(), "Z:".to_owned()].to_vec();
    let mut get_attr_vector: Vec<String> = Vec::new();

    for item in command.iter() {
        let item = item.to_lowercase();
        match state {
            ExecuteStateEnum::Command => {
                match cmd {
                    None => {
                        if item == "quit" {
                            exit(0);
                        }
                        if item != "logicaldisk" {
                            eprintln!("fake-wmic only supports logicaldisk as cmd now");
                            return;
                        }
                        cmd = Some(item.to_string())
                    }
                    Some(_) => {
                        eprintln!("Cannot parse cmd for command");
                        return;
                    }
                }
                state = ExecuteStateEnum::SubStart;
            }
            ExecuteStateEnum::SubStart => {
                state = match item.as_str() {
                    "where" => ExecuteStateEnum::FilterInside,
                    "get" => ExecuteStateEnum::ActionInside,
                    "set" => return,
                    _ => {
                        eprintln!("Unsupported action");
                        return;
                    }
                };
            }
            ExecuteStateEnum::FilterInside => {
                let caps = disk_re.captures(&item);
                match caps {
                    None => {}
                    Some(caps) => {
                        let drive = caps[1].to_owned().to_uppercase();
                        drive_list = [drive].to_vec();
                    }
                }
                state = ExecuteStateEnum::SubStart
            }
            ExecuteStateEnum::ActionInside => {
                let attr_vec: Vec<String> = item
                    .split(',')
                    .map(|x| x.trim().to_string())
                    .filter(|x| !x.is_empty())
                    .collect();
                get_attr_vector.extend(attr_vec);
            }
        }
    }

    get_attr_vector.sort();

    outputln!("{}", get_attr_vector.join("\t"));

    for drive in drive_list {
        for (idx, attr) in get_attr_vector.iter().enumerate() {
            let outstr = match attr.as_str() {
                "drivetype" => "3",             // "normal disk"
                "freespace" => "1000000000000", // 1 TB
                "size" => "1000000000000",      // 1 TB
                "name" => drive.as_str(),
                _ => unimplemented!("unknown attr"),
            };
            if idx != get_attr_vector.len() - 1 {
                output!("{}\t", outstr);
            } else {
                output!("{}\n", outstr);
            }
        }
    }
}

fn repl() {
    let stdin = io::stdin();
    let mut stdin_iter = stdin.lock().lines();
    loop {
        print!(r"wmic:root\cli>");
        io::stdout().flush().unwrap();
        let line = stdin_iter.next();
        let line = match line {
            None => return,
            Some(line) => line.unwrap(),
        };
        let command = shlex::split(&line);
        match command {
            Some(command) => execute(command, &mut io::stdout()),
            None => eprintln!("Unknown input"),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 => {
            panic!("Unexpected args len")
        }
        1 => repl(),
        _ => {
            // Here we don't handle "global switch" (like /?, /namespace, etc) of wmic
            // too lazy to do that
            let wmic_command = &args[1..];
            execute(wmic_command.to_vec(), &mut io::stdout());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_payload(command: &str, expected: &str) {
        let command = shlex::split(command).unwrap();
        let mut output = Vec::new();
        execute(command, &mut output);
        let output = String::from_utf8(output).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_payload1() {
        let command = r#"logicaldisk where "not size=null and name = 'c:'" get name, drivetype, size, freespace"#;
        let expected = r#"drivetype	freespace	name	size
3	1000000000000	C:	1000000000000
"#;
        test_payload(command, expected);
    }

    #[test]
    fn test_payload2() {
        let command = r#"logicaldisk where "not size=null" get name, drivetype, size, freespace"#;
        let expected = r#"drivetype	freespace	name	size
3	1000000000000	C:	1000000000000
3	1000000000000	Z:	1000000000000
"#;
        test_payload(command, expected);
    }
}
