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

pub fn cpp_driver(input: String) -> Result<String, String> {
    // first translate trigraph into chars
    let after_cpp_str = trigraph_processor(input)?;

    Ok(after_cpp_str)
}

