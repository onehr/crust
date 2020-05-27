//     Copyright 2019 Haoran Wang
//
//     Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
//     You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
//     distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//     See the License for the specific language governing permissions and
//     limitations under the License.

// -----------------------------------------------------------------------------
// cpp.rs: Simple c preprocessor
// -----------------------------------------------------------------------------

use lazy_static::lazy_static;
use log::{debug, error};
use regex::Regex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::{error, fs, path::Path};

#[derive(Debug)]
struct Macro {
    replacement: String,
    params: Vec<String>,
}
lazy_static! {
    static ref MACROS: Mutex<HashMap<String, Macro>> = {
        let m = HashMap::new();
        Mutex::new(m)
    };
}

fn trigraph_processor(input: String) -> Result<String, String> {
    // Trigraph:       ??(  ??)  ??<  ??>  ??=  ??/  ??'  ??!  ??-
    // Replacement:      [    ]    {    }    #    \    ^    |    ~
    let mut res = "".to_string();

    let mut idx = 0;
    while idx < input.len() {
        if idx > input.len() - 3 {
            res.push(char::from(input.as_bytes()[idx]));
            idx = idx + 1;
            continue;
        }
        let mut combine = String::new();
        combine.push(char::from(input.as_bytes()[idx]));
        combine.push(char::from(input.as_bytes()[idx + 1]));
        combine.push(char::from(input.as_bytes()[idx + 2]));
        match combine.as_ref() {
            "??(" => {
                res.push('[');
                idx += 3;
            }
            "??)" => {
                res.push(']');
                idx += 3;
            }
            "??<" => {
                res.push('{');
                idx += 3;
            }
            "??>" => {
                res.push('}');
                idx += 3;
            }
            "??=" => {
                res.push('#');
                idx += 3;
            }
            "??/" => {
                res.push('\\');
                idx += 3;
            }
            "??'" => {
                res.push('^');
                idx += 3;
            }
            "??!" => {
                res.push('|');
                idx += 3;
            }
            "??-" => {
                res.push('~');
                idx += 3;
            }
            _ => {
                res.push(char::from(input.as_bytes()[idx]));
                idx += 1;
            }
        }
    }
    return Ok(res);
}

fn line_concat(input: String) -> Result<String, String> {
    let mut res = String::new();
    let mut it = input.chars().peekable();
    while let Some(&c) = it.peek() {
        if c == '\\' {
            it.next();
            if let Some(&nc) = it.peek() {
                match nc {
                    '\n' => {
                        // remove this `\` and `\n`
                        // so nothing here, just skip
                    }
                    _ => {
                        res.push(c);
                        res.push(nc);
                    }
                }
            } else {
                // no other characters, at the end of file
                // Should give warning, but continue compile.
                // cause crust now has no warning option, so just remove this `\`
                break;
            }
        } else {
            res.push(c);
        }
        it.next();
    }

    return Ok(res);
}

fn remove_comment(input: String) -> Result<String, String> {
    fn in_comment(single: bool, multi: bool) -> bool {
        return single || multi;
    }
    let mut res = String::new();

    let mut idx = 0;
    let mut single_line_in_comment = false;
    let mut multi_line_in_comment = false;
    while idx < input.len() {
        if idx == input.len() - 1 {
            if single_line_in_comment || multi_line_in_comment {
                // skip
                break;
            } else {
                res.push(char::from(input.as_bytes()[idx]));
                break;
            }
        }
        let b1 = char::from(input.as_bytes()[idx]);
        let b2 = char::from(input.as_bytes()[idx + 1]);

        let mut combine = String::new();
        combine.push(b1);
        combine.push(b2);
        match combine.as_ref() {
            "//" => {
                if !in_comment(single_line_in_comment, multi_line_in_comment) {
                    single_line_in_comment = true;
                }
                idx = idx + 2;
            }
            "/*" => {
                if !in_comment(single_line_in_comment, multi_line_in_comment) {
                    multi_line_in_comment = true;
                }
                idx = idx + 2;
            }
            "*/" => {
                if multi_line_in_comment {
                    multi_line_in_comment = false;
                    idx = idx + 2;
                } else {
                    res.push(b1);
                    idx = idx + 1;
                }
            }
            _ => {
                if b1 == '\n' && single_line_in_comment {
                    single_line_in_comment = false;
                    idx = idx + 1;
                } else {
                    if in_comment(single_line_in_comment, multi_line_in_comment) {
                        idx = idx + 1;
                    } else {
                        res.push(b1);
                        idx = idx + 1;
                    }
                }
            }
        }
    }
    return Ok(res);
}

fn include_headers(input: String, parent: Option<&Path>) -> Result<String, Box<dyn error::Error>> {
    // TODO: now only support "header.h", system header file not supported now
    //       should add system header support when the macro expension was finished and
    //       other directives are supported.
    fn include_file_path(relational_name: &str) -> Result<&Path, String> {
        if char::from(relational_name.as_bytes()[0]) == '<' {
            return Err("Crust do not support system header now".to_string());
        }
        let name = Path::new(&relational_name[1..(relational_name.len() - 1)]);
        return Ok(name);
    }

    let mut res = String::new();

    for line in input.lines() {
        if line.is_empty() {
            // empty line
            res.push_str("\n");
            continue;
        }
        match char::from(line.trim_start().as_bytes()[0]) {
            '#' => {
                let a: Vec<&str> = line.split_whitespace().collect();
                let directive = *a.get(0).unwrap();
                match directive {
                    "#include" => {
                        let file_name = include_file_path(*a.get(1).unwrap())?;
                        let full_relational_path = file_name;

                        match parent {
                            None => {
                                let header_contents = fs::read_to_string(full_relational_path)?;
                                res.push_str(include_headers(header_contents, None)?.as_ref());
                            }
                            Some(p_dir) => {
                                let full_relational_path = p_dir.join(file_name);
                                let header_contents = fs::read_to_string(full_relational_path)?;
                                res.push_str(include_headers(header_contents, parent)?.as_ref());
                            }
                        }
                    }
                    _ => {
                        // just leave other directives to be handled by the directive_handler
                        res.push_str(line);
                        res.push_str("\n");
                    }
                }
            }
            _ => {
                res.push_str(line);
                res.push_str("\n");
            }
        }
    }

    return Ok(res);
}

fn replace(input: String) -> String {
    // This is a function, which replace the string if contains defined identifier.
    let mut it = input.chars().peekable();
    let mut res = String::new();
    while let Some(&c) = it.peek() {
        match c {
            'a'..='z' | 'A'..='Z' | '_' => {
                it.next();
                let mut id = String::new();
                id.push(c);
                while let Some(&tmp) = it.peek() {
                    match tmp {
                        'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                            id.push(tmp);
                            it.next();
                        }
                        _ => {
                            break;
                        }
                    }
                }
                let mut get_val = String::new();
                match MACROS.lock().unwrap().get(&id) {
                    Some(s) => {
                        // XXX: I use this method to get rid of deadlock.
                        //      cause call replace function in this scope will cause the
                        //      child function to lock the MACROS, which is locked here.

                        let mut replacement = s.replacement.clone();
                        if !s.params.is_empty() {
                            let re_args: Regex =
                                Regex::new(&format!(r"{}\(([^)]+)\)", id)).unwrap();

                            if re_args.is_match(&input) {
                                let caps = re_args.captures(&input).unwrap();
                                let matched = caps[0].to_string();
                                let all_args = caps[1].to_string();
                                let args: Vec<&str> = all_args.split(",").collect();
                                if args.len() != s.params.len() {
                                    error!("Not matched with macro arguments{:?}", id);
                                }

                                for (i, p) in s.params.iter().enumerate() {
                                    replacement = replacement.replace(p, args[i]);
                                }

                                // Skip matched text
                                for _ in 0..matched.len() - id.len() {
                                    it.next();
                                }
                            } else {
                                error!("Unable to find macro's parameter, {:?}", id);
                            }
                        }
                        get_val.push_str(&replacement);
                    }
                    None => {
                        res.push_str(&id);
                    }
                }
                // after the lock was released, can lock in child replace function
                res.push_str(&replace(get_val));
            }
            _ => {
                res.push(c);
                it.next();
            }
        }
    }
    return res;
}

fn directive_handler(input: String) -> Result<String, Box<dyn error::Error>> {
    // TODO: now only support #define directive
    let mut res = String::new();

    let mut lines = input.lines();
    loop {
        let iter = lines.next();
        match iter {
            Some(line) => {
                if line.trim_start().is_empty() {
                    // empty line
                    res.push_str("\n");
                    continue;
                }
                match char::from(line.trim_start().as_bytes()[0]) {
                    '#' => {
                        lazy_static! {
                            static ref RE_MACRO: Regex = Regex::new(
                                r"^[ \t]*#define[ \t]+([A-Za-z0-9_]+)(\(.*\))?[ \t]*((?:.*\\\r?\n)*.*)"
                            )
                            .unwrap();
                        }
                        if RE_MACRO.is_match(line) {
                            let caps = RE_MACRO.captures(line).unwrap();
                            let name = caps[1].to_string();
                            let params = caps
                                .get(2)
                                .map_or("", |m| m.as_str())
                                .to_string()
                                .replace(&['(', ')', ' ', '\t'][..], "");
                            let replacement = caps[3].to_string();

                            let mut param_list = Vec::new();
                            for t in params.split(",") {
                                if !t.is_empty() {
                                    param_list.push(t.to_string());
                                }
                            }

                            let m = Macro {
                                replacement: replacement,
                                params: param_list,
                            };
                            debug!("Macro detected: name:{:?} => {:?}", name, m);

                            MACROS.lock().unwrap().insert(name, m);
                        }
                    }
                    _ => {
                        // not directive starting sentence, so replace the token if it's defined before.
                        // check every identifier name
                        res.push_str(&replace(line.to_string()));
                        res.push('\n');
                    }
                }
            }
            None => break,
        }
    }
    return Ok(res);
}

pub fn cpp_driver(input: String, path: PathBuf) -> Result<String, Box<dyn error::Error>> {
    let parent = path.parent();
    // include the header files in the source file
    let after_cpp_str = include_headers(input, parent)?;
    // first translate trigraph into chars
    let after_cpp_str = trigraph_processor(after_cpp_str)?;
    // concatenate lines
    let after_cpp_str = line_concat(after_cpp_str)?;
    // remove comment
    let after_cpp_str = remove_comment(after_cpp_str)?;
    // directives handler
    let after_cpp_str = directive_handler(after_cpp_str)?;

    Ok(after_cpp_str)
}
