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

use std::process::id;

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
        combine.push(char::from(input.as_bytes()[idx+1]));
        combine.push(char::from(input.as_bytes()[idx+2]));
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

fn in_comment(single: bool, multi: bool) -> bool {
    return single || multi;
}

fn remove_comment(input: String) -> Result<String, String> {
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
        let b2 = char::from(input.as_bytes()[idx+1]);

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

pub fn cpp_driver(input: String) -> Result<String, String> {
    // first translate trigraph into chars
    let after_cpp_str = trigraph_processor(input)?;
    // concatenate lines
    let after_cpp_str = line_concat(after_cpp_str)?;
    // remove comment
    let after_cpp_str = remove_comment(after_cpp_str)?;

    Ok(after_cpp_str)
}

