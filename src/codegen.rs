//! Generate client code for a request in several languages (Postman-style snippets).
use crate::model::{Body, KV, Request}; // 引入依赖
use base64::Engine as _; // 引入依赖

fn headers(req: &Request) -> Vec<(String, String)> { // 函数 headers
    let mut hs: Vec<(String, String)> = req // 变量 hs
        .header // 语句
        .iter() // 语句
        .filter(|h| !h.disabled) // 语句
        .map(|h| (h.key.clone(), h.value.clone())) // 语句
        .collect(); // 语句
    if let Some(auth) = &req.auth { // 条件分支
        match auth.kind.to_ascii_lowercase().as_str() { // 分支匹配
            "bearer" => { // 匹配分支
                if let Some(t) = auth.bearer_token() { // 条件分支
                    hs.push(("Authorization".to_string(), format!("Bearer {}", t))); // 语句
                } // 块结束
            } // 块结束
            "basic" => { // 匹配分支
                let u = auth.basic_user().unwrap_or(""); // 变量 u
                let p = auth.basic_pass().unwrap_or(""); // 变量 p
                let enc = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", u, p)); // 变量 enc
                hs.push(("Authorization".to_string(), format!("Basic {}", enc))); // 语句
            } // 块结束
            "apikey" => { // 匹配分支
                if !auth.apikey_in().eq_ignore_ascii_case("query") { // 条件分支
                    hs.push(( // 语句
                        auth.apikey_key().unwrap_or("").to_string(), // 语句
                        auth.apikey_value().unwrap_or("").to_string(), // 语句
                    )); // 语句
                } // 块结束
            } // 块结束
            _ => {} // 匹配分支
        } // 块结束
    } // 块结束
    hs // 语句
} // 块结束

/// A field of a multipart/form-data body, normalized for code generation.
struct FormField { // 结构体 FormField
    key: String, // 字段 key
    value: String, // 字段 value
    /// `Some(path)` when the row is a file upload (`kv.typ == "file"`).
    file: Option<String>, // 字段 file
} // 块结束

/// Normalized view of a request body so every language generator can see the
/// full `Body` mode (raw, graphql, urlencoded, formdata, file).
enum BodyData { // 枚举 BodyData
    None, // 语句
    Raw(String), // 调用 Raw
    Graphql(String), // 调用 Graphql
    Urlencoded(Vec<(String, String)>), // 调用 Urlencoded
    Formdata(Vec<FormField>), // 调用 Formdata
    File(String), // 调用 File
} // 块结束

/// Resolve the local path for a form-data file row: prefer `src` (string or
/// array), otherwise fall back to the row's `value`.
fn file_path(kv: &KV) -> String { // 函数 file_path
    if let Some(src) = &kv.src { // 条件分支
        match src { // 分支匹配
            serde_json::Value::String(s) => return s.clone(), // 匹配分支
            serde_json::Value::Array(a) => { // 匹配分支
                if let Some(first) = a.first() { // 条件分支
                    if let Some(s) = first.as_str() { // 条件分支
                        return s.to_string(); // 返回
                    } // 块结束
                    return first.to_string(); // 返回
                } // 块结束
            } // 块结束
            other => return other.to_string(), // 匹配分支
        } // 块结束
    } // 块结束
    kv.value.clone() // 语句
} // 块结束

fn body_data(body: &Option<Body>) -> BodyData { // 函数 body_data
    let b = match body.as_ref() { // 变量 b
        Some(b) => b, // 匹配分支
        None => return BodyData::None, // 匹配分支
    }; // 块结束
    match b.mode.as_str() { // 分支匹配
        "raw" => match &b.raw { // 匹配分支
            Some(r) => BodyData::Raw(r.clone()), // 匹配分支
            None => BodyData::None, // 匹配分支
        }, // 块结束
        "graphql" => match &b.graphql { // 匹配分支
            Some(g) => BodyData::Graphql(format!( // 匹配分支
                "{{\"query\":{},\"variables\":{}}}", // 语句
                serde_json::to_string(&g.query).unwrap_or_default(), // 字段 serde_json
                if g.variables.trim().is_empty() { // 条件分支
                    "{}".to_string() // 语句
                } else { // 语句
                    g.variables.clone() // 语句
                } // 块结束
            )), // 语句
            None => BodyData::None, // 匹配分支
        }, // 块结束
        "urlencoded" => BodyData::Urlencoded( // 匹配分支
            b.urlencoded // 语句
                .iter() // 语句
                .filter(|kv| !kv.disabled) // 语句
                .map(|kv| (kv.key.clone(), kv.value.clone())) // 语句
                .collect(), // 语句
        ), // 语句
        "formdata" => BodyData::Formdata( // 匹配分支
            b.formdata // 语句
                .iter() // 语句
                .filter(|kv| !kv.disabled) // 语句
                .map(|kv| { // 语句
                    let is_file = kv.typ.as_deref() == Some("file"); // 变量 is_file
                    FormField { // 语句
                        key: kv.key.clone(), // 字段 key
                        value: kv.value.clone(), // 字段 value
                        file: if is_file { Some(file_path(kv)) } else { None }, // 字段 file
                    } // 块结束
                }) // 语句
                .collect(), // 语句
        ), // 语句
        "file" => match &b.file { // 匹配分支
            Some(f) => BodyData::File(f.src.clone().unwrap_or_default()), // 匹配分支
            None => BodyData::None, // 匹配分支
        }, // 块结束
        _ => BodyData::None, // 匹配分支
    } // 块结束
} // 块结束

fn has_content_type(hs: &[(String, String)]) -> bool { // 函数 has_content_type
    hs.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type")) // 语句
} // 块结束

fn curl_escape(s: &str) -> String { // 函数 curl_escape
    s.replace('\'', "'\\''") // 语句
} // 块结束

/// Legacy content used by fallback languages: only raw/graphql have a plain
/// string payload.
fn body_string_for_legacy(body: &BodyData) -> Option<String> { // 函数 body_string_for_legacy
    match body { // 分支匹配
        BodyData::Raw(s) | BodyData::Graphql(s) => Some(s.clone()), // 匹配分支
        _ => None, // 匹配分支
    } // 块结束
} // 块结束

/// For languages without full body support, include the structured field data
/// as comments (or an equivalent representation) so nothing is silently lost.
fn body_fallback_lines(body: &BodyData, comment: &str) -> String { // 函数 body_fallback_lines
    match body { // 分支匹配
        BodyData::None | BodyData::Raw(_) | BodyData::Graphql(_) => String::new(), // 匹配分支
        BodyData::Urlencoded(pairs) => { // 匹配分支
            let mut s = format!("{} urlencoded body:\n", comment); // 变量 s
            for (k, v) in pairs { // 循环
                s.push_str(&format!("{}   {}={}\n", comment, k, v)); // 语句
            } // 块结束
            s // 语句
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            let mut s = format!("{} multipart/form-data body:\n", comment); // 变量 s
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => s.push_str(&format!("{}   {}=@{} (file)\n", comment, f.key, p)), // 匹配分支
                    None => s.push_str(&format!("{}   {}={}\n", comment, f.key, f.value)), // 匹配分支
                } // 块结束
            } // 块结束
            s // 语句
        } // 块结束
        BodyData::File(p) => format!("{} binary file body: @{}\n", comment, p), // 匹配分支
    } // 块结束
} // 块结束

pub fn generate(req: &Request, lang: &str) -> anyhow::Result<String> { // 函数 generate
    let method = req.method.to_uppercase(); // 变量 method
    let url = req.url.effective_url(); // 变量 url
    let hs = headers(req); // 变量 hs
    let body = body_data(&req.body); // 变量 body
    let out = match lang.to_ascii_lowercase().as_str() { // 变量 out
        "curl" => gen_curl(&method, &url, &hs, &body, req), // 匹配分支
        "rust" | "reqwest" => gen_rust(&method, &url, &hs, &body), // 匹配分支
        "python" | "requests" => gen_python(&method, &url, &hs, &body), // 匹配分支
        "js" | "javascript" | "fetch" => gen_js(&method, &url, &hs, &body), // 匹配分支
        "go" | "golang" => gen_go(&method, &url, &hs, &body), // 匹配分支
        "nodejs" | "node" | "axios" => gen_node(&method, &url, &hs, &body), // 匹配分支
        "java" | "okhttp" => gen_java(&method, &url, &hs, &body), // 匹配分支
        "csharp" | "c#" | "dotnet" => gen_csharp(&method, &url, &hs, &body), // 匹配分支
        "php" => gen_php(&method, &url, &hs, &body), // 匹配分支
        "ruby" => gen_ruby(&method, &url, &hs, &body), // 匹配分支
        "powershell" | "ps" => gen_powershell(&method, &url, &hs, &body), // 匹配分支
        "swift" => gen_swift(&method, &url, &hs, &body), // 匹配分支
        "http" => gen_http(&method, &url, &hs, &body), // 匹配分支
        other => anyhow::bail!("unsupported codegen language: {}", other), // 匹配分支
    }; // 块结束
    Ok(out) // 调用 Ok
} // 块结束

fn gen_curl( // 函数 gen_curl
    method: &str, // 字段 method
    url: &str, // 字段 url
    hs: &[(String, String)], // 字段 hs
    body: &BodyData, // 字段 body
    req: &Request, // 字段 req
) -> String { // 语句
    let mut s = format!("curl --location --request {} '{}'", method, url); // 变量 s
    for (k, v) in hs { // 循环
        s.push_str(&format!(" \\\n  --header '{}: {}'", k, v)); // 语句
    } // 块结束
    if let Some(auth) = &req.auth { // 条件分支
        match auth.kind.to_ascii_lowercase().as_str() { // 分支匹配
            "bearer" => { // 匹配分支
                if let Some(t) = auth.bearer_token() { // 条件分支
                    s.push_str(&format!(" \\\n  --header 'Authorization: Bearer {}'", t)); // 语句
                } // 块结束
            } // 块结束
            "basic" => { // 匹配分支
                let u = auth.basic_user().unwrap_or(""); // 变量 u
                let p = auth.basic_pass().unwrap_or(""); // 变量 p
                s.push_str(&format!(" \\\n  --user '{}:{}'", u, p)); // 语句
            } // 块结束
            _ => {} // 匹配分支
        } // 块结束
    } // 块结束
    match body { // 分支匹配
        BodyData::None => {} // 匹配分支
        BodyData::Raw(b) => s.push_str(&format!(" \\\n  --data '{}'", curl_escape(b))), // 匹配分支
        BodyData::Graphql(j) => { // 匹配分支
            if !has_content_type(hs) { // 条件分支
                s.push_str(" \\\n  --header 'Content-Type: application/json'"); // 语句
            } // 块结束
            s.push_str(&format!(" \\\n  --data '{}'", curl_escape(j))); // 语句
        } // 块结束
        BodyData::Urlencoded(pairs) => { // 匹配分支
            for (k, v) in pairs { // 循环
                s.push_str(&format!( // 语句
                    " \\\n  --data-urlencode '{}={}'", // 语句
                    curl_escape(k), // 调用 curl_escape
                    curl_escape(v) // 调用 curl_escape
                )); // 语句
            } // 块结束
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => s.push_str(&format!( // 匹配分支
                        " \\\n  -F '{}=@{}'", // 语句
                        curl_escape(&f.key), // 调用 curl_escape
                        curl_escape(p) // 调用 curl_escape
                    )), // 语句
                    None => s.push_str(&format!( // 匹配分支
                        " \\\n  -F '{}={}'", // 语句
                        curl_escape(&f.key), // 调用 curl_escape
                        curl_escape(&f.value) // 调用 curl_escape
                    )), // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        BodyData::File(p) => s.push_str(&format!(" \\\n  --data-binary '@{}'", curl_escape(p))), // 匹配分支
    } // 块结束
    s // 语句
} // 块结束

fn gen_rust(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_rust
    let mut header_lines = String::new(); // 变量 header_lines
    for (k, v) in hs { // 循环
        header_lines.push_str(&format!("        .header({:?}, {:?})\n", k, v)); // 语句
    } // 块结束
    let mut body_lines = String::new(); // 变量 body_lines
    match body { // 分支匹配
        BodyData::None => {} // 匹配分支
        BodyData::Raw(b) | BodyData::Graphql(b) => { // 匹配分支
            body_lines.push_str(&format!("        .body({:?})\n", b)); // 语句
        } // 块结束
        BodyData::Urlencoded(pairs) => { // 匹配分支
            let items: Vec<String> = pairs // 变量 items
                .iter() // 语句
                .map(|(k, v)| format!("({:?}, {:?})", k, v)) // 语句
                .collect(); // 语句
            body_lines.push_str(&format!("        .form(&[{}])\n", items.join(", "))); // 语句
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            let mut form = String::from("reqwest::blocking::multipart::Form::new()"); // 变量 form
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => form.push_str(&format!( // 匹配分支
                        "\n            .part({:?}, reqwest::blocking::multipart::Part::bytes(std::fs::read({:?})?).file_name({:?}))", // 语句
                        f.key, p, f.key // 语句
                    )), // 语句
                    None => form.push_str(&format!("\n            .text({:?}, {:?})", f.key, f.value)), // 匹配分支
                } // 块结束
            } // 块结束
            body_lines.push_str(&format!("        .multipart({})\n", form)); // 语句
        } // 块结束
        BodyData::File(p) => { // 匹配分支
            body_lines.push_str(&format!("        .body(std::fs::read({:?})?)\n", p)); // 语句
        } // 块结束
    } // 块结束
    format!( // 语句
        "// Cargo.toml: reqwest = {{ version = \"0.12\", features = [\"blocking\", \"rustls-tls\", \"multipart\"] }}\n\
use reqwest::blocking::Client;\n\n\
fn main() -> Result<(), Box<dyn std::error::Error>> {{\n    \
let client = Client::new();\n    \
let resp = client\n        \
.request(reqwest::Method::from_bytes(b\"{}\")?, \"{}\")\n{}{}        \
.send()?;\n    \
println!(\"{{}}\", resp.status());\n    \
println!(\"{{}}\", resp.text()?);\n    \
Ok(())\n}}\n", // 调用 Ok
        method, url, header_lines, body_lines // 语句
    ) // 块结束
} // 块结束

fn gen_python(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_python
    let mut h = String::new(); // 变量 h
    if !hs.is_empty() { // 条件分支
        h.push_str("headers = {\n"); // 语句
        for (k, v) in hs { // 循环
            h.push_str(&format!("    {:?}: {:?},\n", k, v)); // 语句
        } // 块结束
        h.push_str("}\n"); // 语句
    } // 块结束
    let mut setup = String::new(); // 变量 setup
    let mut call_args = String::new(); // 变量 call_args
    match body { // 分支匹配
        BodyData::None => {} // 匹配分支
        BodyData::Raw(b) | BodyData::Graphql(b) => { // 匹配分支
            setup.push_str(&format!("data = {:?}\n", b)); // 语句
            call_args.push_str(", data=data"); // 语句
        } // 块结束
        BodyData::Urlencoded(pairs) => { // 匹配分支
            setup.push_str("data = {\n"); // 语句
            for (k, v) in pairs { // 循环
                setup.push_str(&format!("    {:?}: {:?},\n", k, v)); // 语句
            } // 块结束
            setup.push_str("}\n"); // 语句
            call_args.push_str(", data=data"); // 语句
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            let mut data_fields = String::new(); // 变量 data_fields
            let mut file_fields = String::new(); // 变量 file_fields
            let mut has_data = false; // 变量 has_data
            let mut has_files = false; // 变量 has_files
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => { // 匹配分支
                        file_fields.push_str(&format!("    {:?}: open({:?}, \"rb\"),\n", f.key, p)); // 语句
                        has_files = true; // 语句
                    } // 块结束
                    None => { // 匹配分支
                        data_fields.push_str(&format!("    {:?}: {:?},\n", f.key, f.value)); // 语句
                        has_data = true; // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            if has_files { // 条件分支
                setup.push_str(&format!("files = {{\n{}}}\n", file_fields)); // 语句
            } // 块结束
            if has_data { // 条件分支
                setup.push_str(&format!("data = {{\n{}}}\n", data_fields)); // 语句
            } // 块结束
            if has_files { // 条件分支
                call_args.push_str(", files=files"); // 语句
            } // 块结束
            if has_data { // 条件分支
                call_args.push_str(", data=data"); // 语句
            } // 块结束
        } // 块结束
        BodyData::File(p) => { // 匹配分支
            setup.push_str(&format!("data = open({:?}, \"rb\")\n", p)); // 语句
            call_args.push_str(", data=data"); // 语句
        } // 块结束
    } // 块结束
    format!( // 语句
        "import requests\n\n{}{}\nresponse = requests.request({:?}, {:?}, headers=headers{})\nprint(response.status_code)\nprint(response.text)\n", // 语句
        h, setup, method, url, call_args // 语句
    ) // 块结束
} // 块结束

fn js_str_comment(s: &str) -> String { // 函数 js_str_comment
    s.replace("*/", "*\\/").replace('\n', " ") // 语句
} // 块结束

fn gen_js(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_js
    let mut h = String::from("{\n"); // 变量 h
    for (k, v) in hs { // 循环
        h.push_str(&format!("  {:?}: {:?},\n", k, v)); // 语句
    } // 块结束
    h.push('}'); // 语句
    let mut prelude = String::new(); // 变量 prelude
    let body_part = match body { // 变量 body_part
        BodyData::None => String::new(), // 匹配分支
        BodyData::Raw(b) | BodyData::Graphql(b) => format!(",\n  body: {:?}", b), // 匹配分支
        BodyData::Urlencoded(pairs) => { // 匹配分支
            let items: Vec<String> = pairs // 变量 items
                .iter() // 语句
                .map(|(k, v)| format!("{:?}: {:?}", k, v)) // 语句
                .collect(); // 语句
            format!(",\n  body: new URLSearchParams({{{}}})", items.join(", ")) // 语句
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            prelude.push_str("const formData = new FormData();\n"); // 语句
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => prelude.push_str(&format!( // 匹配分支
                        "formData.append({:?}, /* file: {} */ fileFromPath);\n", // 语句
                        f.key, // 语句
                        js_str_comment(p) // 调用 js_str_comment
                    )), // 语句
                    None => prelude.push_str(&format!( // 匹配分支
                        "formData.append({:?}, {:?});\n", // 语句
                        f.key, f.value // 语句
                    )), // 语句
                } // 块结束
            } // 块结束
            ",\n  body: formData".to_string() // 语句
        } // 块结束
        BodyData::File(p) => format!(",\n  body: new Blob([/* contents of {} */])", js_str_comment(p)), // 匹配分支
    }; // 块结束
    format!( // 语句
        "{}const options = {{\n  method: {:?},\n  headers: {}{}\n}};\n\nfetch({:?}, options)\n  .then(res => res.text())\n  .then(console.log);\n", // 匹配分支
        prelude, method, h, body_part, url // 语句
    ) // 块结束
} // 块结束

fn gen_go(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_go
    let mut imports: Vec<&str> = vec!["fmt", "io", "net/http"]; // 变量 imports
    let mut prelude = String::new(); // 变量 prelude
    let mut content_type: Option<&str> = None; // 变量 content_type
    match body { // 分支匹配
        BodyData::None => { // 匹配分支
            prelude.push_str("\tvar payload io.Reader\n"); // 语句
        } // 块结束
        BodyData::Raw(b) | BodyData::Graphql(b) => { // 匹配分支
            imports.push("strings"); // 语句
            prelude.push_str(&format!("\tpayload := strings.NewReader({:?})\n", b)); // 语句
            content_type = Some("application/json"); // 语句
        } // 块结束
        BodyData::Urlencoded(pairs) => { // 匹配分支
            imports.push("net/url"); // 语句
            imports.push("strings"); // 语句
            prelude.push_str("\tdata := url.Values{}\n"); // 语句
            for (k, v) in pairs { // 循环
                prelude.push_str(&format!("\tdata.Set({:?}, {:?})\n", k, v)); // 语句
            } // 块结束
            prelude.push_str("\tpayload := strings.NewReader(data.Encode())\n"); // 语句
            content_type = Some("application/x-www-form-urlencoded"); // 语句
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            imports.push("bytes"); // 语句
            imports.push("mime/multipart"); // 语句
            prelude.push_str("\tvar buf bytes.Buffer\n\tw := multipart.NewWriter(&buf)\n"); // 语句
            let mut need_os = false; // 变量 need_os
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => { // 匹配分支
                        need_os = true; // 语句
                        prelude.push_str(&format!( // 语句
                            "\tfw, _ := w.CreateFormFile({:?}, {:?})\n\tfr, _ := os.Open({:?})\n\tio.Copy(fw, fr)\n\tfr.Close()\n", // 语句
                            f.key, p, p // 语句
                        )); // 语句
                    } // 块结束
                    None => prelude.push_str(&format!("\tw.WriteField({:?}, {:?})\n", f.key, f.value)), // 匹配分支
                } // 块结束
            } // 块结束
            if need_os { // 条件分支
                imports.push("os"); // 语句
            } // 块结束
            prelude.push_str("\tw.Close()\n\tpayload := io.Reader(&buf)\n"); // 语句
            content_type = Some("multipart/form-data"); // 语句
        } // 块结束
        BodyData::File(p) => { // 匹配分支
            imports.push("os"); // 语句
            prelude.push_str(&format!( // 语句
                "\tfr, _ := os.Open({:?})\n\tpayload := io.Reader(fr)\n", // 语句
                p // 语句
            )); // 语句
        } // 块结束
    } // 块结束
    let mut uniq: Vec<&str> = Vec::new(); // 变量 uniq
    for i in imports { // 循环
        if !uniq.contains(&i) { // 条件分支
            uniq.push(i); // 语句
        } // 块结束
    } // 块结束
    let mut import_block = String::new(); // 变量 import_block
    for i in &uniq { // 循环
        import_block.push_str(&format!("\t{:?}\n", i)); // 语句
    } // 块结束
    let mut header_lines = String::new(); // 变量 header_lines
    if !has_content_type(hs) { // 条件分支
        if let Some(ct) = content_type { // 条件分支
            if ct != "multipart/form-data" { // 条件分支
                header_lines.push_str(&format!("\treq.Header.Set(\"Content-Type\", {:?})\n", ct)); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    for (k, v) in hs { // 循环
        header_lines.push_str(&format!("\treq.Header.Set({:?}, {:?})\n", k, v)); // 语句
    } // 块结束
    let mut post = String::new(); // 变量 post
    if matches!(body, BodyData::Formdata(_)) && !has_content_type(hs) { // 条件分支
        post.push_str("\treq.Header.Set(\"Content-Type\", w.FormDataContentType())\n"); // 语句
    } // 块结束
    format!( // 语句
        "package main\n\nimport (\n{}\n)\n\nfunc main() {{\n{}{}\treq, _ := http.NewRequest({:?}, {:?}, payload)\n{}{}\tresp, err := http.DefaultClient.Do(req)\n\tif err != nil {{ panic(err) }}\n\tdefer resp.Body.Close()\n\tb, _ := io.ReadAll(resp.Body)\n\tfmt.Println(resp.StatusCode)\n\tfmt.Println(string(b))\n}}\n", // 语句
        import_block, prelude, "", method, url, header_lines, post // 语句
    ) // 块结束
} // 块结束

fn gen_node(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_node
    let mut h = String::from("{\n"); // 变量 h
    for (k, v) in hs { // 循环
        h.push_str(&format!("    {:?}: {:?},\n", k, v)); // 语句
    } // 块结束
    h.push_str("  }"); // 语句
    let mut prelude = String::new(); // 变量 prelude
    let (headers_expr, data_expr) = match body { // 调用 let
        BodyData::None => (h.clone(), None), // 匹配分支
        BodyData::Raw(b) | BodyData::Graphql(b) => (h.clone(), Some(format!("{:?}", b))), // 匹配分支
        BodyData::Urlencoded(pairs) => { // 匹配分支
            let items: Vec<String> = pairs // 变量 items
                .iter() // 语句
                .map(|(k, v)| format!("{:?}: {:?}", k, v)) // 语句
                .collect(); // 语句
            (h.clone(), Some(format!("{{{}}}", items.join(", ")))) // 语句
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            prelude.push_str( // 语句
                "const FormData = require('form-data');\nconst fs = require('fs');\nconst form = new FormData();\n", // 语句
            ); // 块结束
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => prelude.push_str(&format!( // 匹配分支
                        "form.append({:?}, fs.createReadStream({:?}));\n", // 语句
                        f.key, p // 语句
                    )), // 语句
                    None => prelude.push_str(&format!( // 匹配分支
                        "form.append({:?}, {:?});\n", // 语句
                        f.key, f.value // 语句
                    )), // 语句
                } // 块结束
            } // 块结束
            ( // 语句
                format!("Object.assign({{}}, {}, form.getHeaders())", h), // 语句
                Some("form".to_string()), // 调用 Some
            ) // 块结束
        } // 块结束
        BodyData::File(p) => { // 匹配分支
            prelude.push_str("const fs = require('fs');\n"); // 语句
            (h.clone(), Some(format!("fs.createReadStream({:?})", p))) // 语句
        } // 块结束
    }; // 块结束
    let data_line = data_expr // 变量 data_line
        .map(|d| format!(",\n  data: {}", d)) // 语句
        .unwrap_or_default(); // 语句
    format!( // 语句
        "const axios = require('axios');\n{}\naxios({{\n  method: {:?},\n  url: {:?},\n  headers: {}{}\n}}).then(r => console.log(r.status, r.data)).catch(console.error);\n", // 匹配分支
        prelude, // 语句
        method.to_lowercase(), // 语句
        url, // 语句
        headers_expr, // 语句
        data_line // 语句
    ) // 块结束
} // 块结束

fn esc(s: &str) -> String { // 函数 esc
    s.replace('\\', "\\\\").replace('"', "\\\"") // 语句
} // 块结束

fn gen_java(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_java
    let mut h = String::new(); // 变量 h
    for (k, v) in hs { // 循环
        h.push_str(&format!("        .header(\"{}\", \"{}\")\n", esc(k), esc(v))); // 语句
    } // 块结束
    let legacy = body_string_for_legacy(body); // 变量 legacy
    let method_line = match legacy { // 变量 method_line
        Some(b) => format!( // 匹配分支
            "        .method(\"{}\", RequestBody.create(\"{}\", MediaType.parse(\"application/json\")))\n", // 语句
            method, // 语句
            esc(&b) // 调用 esc
        ), // 语句
        None => format!("        .method(\"{}\", null)\n", method), // 匹配分支
    }; // 块结束
    let fallback = body_fallback_lines(body, "// "); // 变量 fallback
    format!( // 语句
        "// OkHttp (Java)\nimport okhttp3.*;\n\n{}OkHttpClient client = new OkHttpClient();\nRequest request = new Request.Builder()\n        .url(\"{}\")\n{}{}        .build();\nResponse response = client.newCall(request).execute();\nSystem.out.println(response.body().string());\n", // 语句
        fallback, url, h, method_line // 语句
    ) // 块结束
} // 块结束

fn gen_csharp(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_csharp
    let mut h = String::new(); // 变量 h
    for (k, v) in hs { // 循环
        h.push_str(&format!( // 语句
            "request.Headers.TryAddWithoutValidation(\"{}\", \"{}\");\n", // 语句
            esc(k), // 调用 esc
            esc(v) // 调用 esc
        )); // 语句
    } // 块结束
    let legacy = body_string_for_legacy(body); // 变量 legacy
    let body_line = match legacy { // 变量 body_line
        Some(b) => format!( // 匹配分支
            "request.Content = new StringContent(\"{}\", Encoding.UTF8, \"application/json\");\n", // 语句
            esc(&b) // 调用 esc
        ), // 语句
        None => String::new(), // 匹配分支
    }; // 块结束
    let fallback = body_fallback_lines(body, "// "); // 变量 fallback
    format!( // 语句
        "// C# HttpClient\nusing System.Net.Http;\nusing System.Text;\n\n{}var client = new HttpClient();\nvar request = new HttpRequestMessage(HttpMethod.{}, \"{}\");\n{}{}var response = await client.SendAsync(request);\nConsole.WriteLine(await response.Content.ReadAsStringAsync());\n", // 语句
        fallback, method, url, h, body_line // 语句
    ) // 块结束
} // 块结束

fn gen_php(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_php
    let hdrs: Vec<String> = hs // 变量 hdrs
        .iter() // 语句
        .map(|(k, v)| format!("\"{}\"", esc(&format!("{}: {}", k, v)))) // 语句
        .collect(); // 语句
    let legacy = body_string_for_legacy(body); // 变量 legacy
    let body_line = match legacy { // 变量 body_line
        Some(b) => format!("curl_setopt($ch, CURLOPT_POSTFIELDS, \"{}\");\n", esc(&b)), // 匹配分支
        None => String::new(), // 匹配分支
    }; // 块结束
    let fallback = body_fallback_lines(body, "// "); // 变量 fallback
    format!( // 语句
        "<?php\n{}$ch = curl_init();\ncurl_setopt($ch, CURLOPT_URL, \"{}\");\ncurl_setopt($ch, CURLOPT_RETURNTRANSFER, true);\ncurl_setopt($ch, CURLOPT_CUSTOMREQUEST, \"{}\");\ncurl_setopt($ch, CURLOPT_HTTPHEADER, array({}));\n{}$response = curl_exec($ch);\ncurl_close($ch);\necho $response;\n", // 语句
        fallback, url, method, hdrs.join(", "), body_line // 语句
    ) // 块结束
} // 块结束

fn gen_ruby(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_ruby
    let mut h = String::new(); // 变量 h
    for (k, v) in hs { // 循环
        h.push_str(&format!("  request[\"{}\"] = \"{}\"\n", esc(k), esc(v))); // 语句
    } // 块结束
    let legacy = body_string_for_legacy(body); // 变量 legacy
    let body_line = match legacy { // 变量 body_line
        Some(b) => format!("  request.body = \"{}\"\n", esc(&b)), // 匹配分支
        None => String::new(), // 匹配分支
    }; // 块结束
    let fallback = body_fallback_lines(body, "# "); // 变量 fallback
    format!( // 语句
        "# Ruby net/http\n{}require 'net/http'\nrequire 'uri'\n\nuri = URI('{}')\nrequest = Net::HTTP::{} .new(uri)\n{}{}response = Net::HTTP.start(uri.host, uri.port, use_ssl: uri.scheme == 'https') do |http|\n  http.request(request)\nend\nputs response.body\n", // 语句
        fallback, url, method, h, body_line // 语句
    ) // 块结束
} // 块结束

fn gen_powershell(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_powershell
    let mut h = String::new(); // 变量 h
    for (k, v) in hs { // 循环
        h.push_str(&format!("  \"{}: {}\",\n", esc(k), esc(v))); // 语句
    } // 块结束
    let legacy = body_string_for_legacy(body); // 变量 legacy
    let body_line = match legacy { // 变量 body_line
        Some(b) => format!("-Body '{}' ", b.replace('\'', "''")), // 匹配分支
        None => String::new(), // 匹配分支
    }; // 块结束
    let fallback = body_fallback_lines(body, "# "); // 变量 fallback
    format!( // 语句
        "# PowerShell\n{}$headers = @{{\n{}}}\nInvoke-RestMethod -Uri '{}' -Method {} {} -Headers $headers\n", // 语句
        fallback, h, url, method, body_line // 语句
    ) // 块结束
} // 块结束

fn gen_swift(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_swift
    let mut h = String::new(); // 变量 h
    for (k, v) in hs { // 循环
        h.push_str(&format!( // 语句
            "    request.setValue(\"{}\", forHTTPHeaderField: \"{}\")\n", // 语句
            esc(v), // 调用 esc
            esc(k) // 调用 esc
        )); // 语句
    } // 块结束
    let legacy = body_string_for_legacy(body); // 变量 legacy
    let body_line = match legacy { // 变量 body_line
        Some(b) => format!("    request.httpBody = \"{}\".data(using: .utf8)\n", esc(&b)), // 匹配分支
        None => String::new(), // 匹配分支
    }; // 块结束
    let fallback = body_fallback_lines(body, "// "); // 变量 fallback
    format!( // 语句
        "// Swift URLSession\nimport Foundation\n\n{}var request = URLRequest(url: URL(string: \"{}\")!)\nrequest.httpMethod = \"{}\"\n{}{}let task = URLSession.shared.dataTask(with: request) {{ data, _, _ in\n    print(String(data: data ?? Data(), encoding: .utf8) ?? \"\")\n}}\ntask.resume()\n", // 语句
        fallback, url, method, h, body_line // 语句
    ) // 块结束
} // 块结束

fn gen_http(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String { // 函数 gen_http
    let rest = url.splitn(2, "://").nth(1).unwrap_or(url); // 变量 rest
    let (host, path) = match rest.split_once('/') { // 调用 let
        Some((h, p)) => (h.to_string(), format!("/{}", p)), // 匹配分支
        None => (rest.to_string(), "/".to_string()), // 匹配分支
    }; // 块结束
    let mut s = format!("{} {} HTTP/1.1\nHost: {}\n", method, path, host); // 变量 s
    for (k, v) in hs { // 循环
        s.push_str(&format!("{}: {}\n", k, v)); // 语句
    } // 块结束
    s.push('\n'); // 语句
    match body { // 分支匹配
        BodyData::None => {} // 匹配分支
        BodyData::Raw(b) | BodyData::Graphql(b) => s.push_str(b), // 匹配分支
        BodyData::Urlencoded(pairs) => { // 匹配分支
            let encoded: Vec<String> = pairs.iter().map(|(k, v)| format!("{}={}", k, v)).collect(); // 变量 encoded
            s.push_str(&encoded.join("&")); // 语句
        } // 块结束
        BodyData::Formdata(fields) => { // 匹配分支
            for f in fields { // 循环
                match &f.file { // 分支匹配
                    Some(p) => s.push_str(&format!("[form-data file] {}=@{}\n", f.key, p)), // 匹配分支
                    None => s.push_str(&format!("[form-data] {}={}\n", f.key, f.value)), // 匹配分支
                } // 块结束
            } // 块结束
        } // 块结束
        BodyData::File(p) => s.push_str(&format!("[binary file body] @{}", p)), // 匹配分支
    } // 块结束
    s // 语句
} // 块结束

#[cfg(test)] // 语句
mod tests { // 模块 tests
    use super::*; // 引入依赖
    use crate::model::{FileRef, Header, UrlSpec}; // 引入依赖

    fn sample() -> Request { // 函数 sample
        Request { // 语句
            method: "POST".into(), // 字段 method
            url: UrlSpec::new("https://api.test/users"), // 字段 url
            header: vec![Header { // 语句
                key: "Content-Type".into(), // 字段 key
                value: "application/json".into(), // 字段 value
                ..Default::default() // 语句
            }], // 语句
            body: Some(Body { // 语句
                mode: "raw".into(), // 字段 mode
                raw: Some("{\"name\":\"z\"}".into()), // 字段 raw
                options: Some(serde_json::json!({"raw":{"language":"json"}})), // 字段 options
                ..Default::default() // 语句
            }), // 语句
            ..Default::default() // 语句
        } // 块结束
    } // 块结束

    #[test] // 语句
    fn generates_all() { // 函数 generates_all
        for lang in ["curl", "rust", "python", "js", "go", "nodejs"] { // 循环
            let out = generate(&sample(), lang).unwrap(); // 变量 out
            assert!(!out.is_empty(), "empty for {}", lang); // 语句
            assert!(out.contains("api.test"), "missing url for {}", lang); // 语句
        } // 块结束
    } // 块结束

    #[test] // 语句
    fn unknown_lang_errors() { // 函数 unknown_lang_errors
        assert!(generate(&sample(), "cobol").is_err()); // 语句
    } // 块结束

    fn req_with_body(body: Body) -> Request { // 函数 req_with_body
        Request { // 语句
            method: "POST".into(), // 字段 method
            url: UrlSpec::new("https://api.test/upload"), // 字段 url
            body: Some(body), // 字段 body
            ..Default::default() // 语句
        } // 块结束
    } // 块结束

    #[test] // 语句
    fn urlencoded_body_curl_and_python() { // 函数 urlencoded_body_curl_and_python
        let req = req_with_body(Body { // 变量 req
            mode: "urlencoded".into(), // 字段 mode
            urlencoded: vec![ // 语句
                KV { // 语句
                    key: "a".into(), // 字段 key
                    value: "1".into(), // 字段 value
                    ..Default::default() // 语句
                }, // 块结束
                KV { // 语句
                    key: "skip".into(), // 字段 key
                    value: "x".into(), // 字段 value
                    disabled: true, // 字段 disabled
                    ..Default::default() // 语句
                }, // 块结束
            ], // 语句
            ..Default::default() // 语句
        }); // 语句
        let curl = generate(&req, "curl").unwrap(); // 变量 curl
        assert!(curl.contains("--data-urlencode 'a=1'"), "curl: {curl}"); // 语句
        assert!(!curl.contains("skip"), "disabled field leaked: {curl}"); // 语句
        let python = generate(&req, "python").unwrap(); // 变量 python
        assert!(python.contains("\"a\": \"1\""), "python: {python}"); // 语句
    } // 块结束

    #[test] // 语句
    fn formdata_body_curl_and_python() { // 函数 formdata_body_curl_and_python
        let req = req_with_body(Body { // 变量 req
            mode: "formdata".into(), // 字段 mode
            formdata: vec![ // 语句
                KV { // 语句
                    key: "name".into(), // 字段 key
                    value: "z".into(), // 字段 value
                    ..Default::default() // 语句
                }, // 块结束
                KV { // 语句
                    key: "avatar".into(), // 字段 key
                    value: String::new(), // 字段 value
                    typ: Some("file".into()), // 字段 typ
                    src: Some(serde_json::json!("C:\\tmp\\a.png")), // 字段 src
                    ..Default::default() // 语句
                }, // 块结束
            ], // 语句
            ..Default::default() // 语句
        }); // 语句
        let curl = generate(&req, "curl").unwrap(); // 变量 curl
        assert!(curl.contains("-F 'name=z'"), "curl: {curl}"); // 语句
        assert!( // 语句
            curl.contains("-F 'avatar=@C:\\tmp\\a.png'"), // 语句
            "curl: {curl}" // 语句
        ); // 块结束
        let python = generate(&req, "python").unwrap(); // 变量 python
        assert!(python.contains("files = {"), "python: {python}"); // 语句
        assert!(python.contains("open("), "python: {python}"); // 语句
        assert!(python.contains("a.png"), "python: {python}"); // 语句
    } // 块结束

    #[test] // 语句
    fn file_body_curl_and_python() { // 函数 file_body_curl_and_python
        let req = req_with_body(Body { // 变量 req
            mode: "file".into(), // 字段 mode
            file: Some(FileRef { // 语句
                src: Some("C:\\tmp\\data.bin".into()), // 字段 src
                content: None, // 字段 content
            }), // 语句
            ..Default::default() // 语句
        }); // 语句
        let curl = generate(&req, "curl").unwrap(); // 变量 curl
        assert!( // 语句
            curl.contains("--data-binary '@C:\\tmp\\data.bin'"), // 语句
            "curl: {curl}" // 语句
        ); // 块结束
        let python = generate(&req, "python").unwrap(); // 变量 python
        assert!( // 语句
            python.contains("open(\"C:\\\\tmp\\\\data.bin\", \"rb\")"), // 语句
            "python: {python}" // 语句
        ); // 块结束
    } // 块结束
} // 块结束

