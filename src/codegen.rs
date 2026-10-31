//! Generate client code for a request in several languages (Postman-style snippets).
use crate::model::{Body, KV, Request};
use base64::Engine as _;

fn headers(req: &Request) -> Vec<(String, String)> {
    let mut hs: Vec<(String, String)> = req
        .header
        .iter()
        .filter(|h| !h.disabled)
        .map(|h| (h.key.clone(), h.value.clone()))
        .collect();
    if let Some(auth) = &req.auth {
        match auth.kind.to_ascii_lowercase().as_str() {
            "bearer" => {
                if let Some(t) = auth.bearer_token() {
                    hs.push(("Authorization".to_string(), format!("Bearer {}", t)));
                }
            }
            "basic" => {
                let u = auth.basic_user().unwrap_or("");
                let p = auth.basic_pass().unwrap_or("");
                let enc = base64::engine::general_purpose::STANDARD.encode(format!("{}:{}", u, p));
                hs.push(("Authorization".to_string(), format!("Basic {}", enc)));
            }
            "apikey" => {
                if !auth.apikey_in().eq_ignore_ascii_case("query") {
                    hs.push((
                        auth.apikey_key().unwrap_or("").to_string(),
                        auth.apikey_value().unwrap_or("").to_string(),
                    ));
                }
            }
            _ => {}
        }
    }
    hs
}

/// A field of a multipart/form-data body, normalized for code generation.
struct FormField {
    key: String,
    value: String,
    /// `Some(path)` when the row is a file upload (`kv.typ == "file"`).
    file: Option<String>,
}

/// Normalized view of a request body so every language generator can see the
/// full `Body` mode (raw, graphql, urlencoded, formdata, file).
enum BodyData {
    None,
    Raw(String),
    Graphql(String),
    Urlencoded(Vec<(String, String)>),
    Formdata(Vec<FormField>),
    File(String),
}

/// Resolve the local path for a form-data file row: prefer `src` (string or
/// array), otherwise fall back to the row's `value`.
fn file_path(kv: &KV) -> String {
    if let Some(src) = &kv.src {
        match src {
            serde_json::Value::String(s) => return s.clone(),
            serde_json::Value::Array(a) => {
                if let Some(first) = a.first() {
                    if let Some(s) = first.as_str() {
                        return s.to_string();
                    }
                    return first.to_string();
                }
            }
            other => return other.to_string(),
        }
    }
    kv.value.clone()
}

fn body_data(body: &Option<Body>) -> BodyData {
    let b = match body.as_ref() {
        Some(b) => b,
        None => return BodyData::None,
    };
    match b.mode.as_str() {
        "raw" => match &b.raw {
            Some(r) => BodyData::Raw(r.clone()),
            None => BodyData::None,
        },
        "graphql" => match &b.graphql {
            Some(g) => BodyData::Graphql(format!(
                "{{\"query\":{},\"variables\":{}}}",
                serde_json::to_string(&g.query).unwrap_or_default(),
                if g.variables.trim().is_empty() {
                    "{}".to_string()
                } else {
                    g.variables.clone()
                }
            )),
            None => BodyData::None,
        },
        "urlencoded" => BodyData::Urlencoded(
            b.urlencoded
                .iter()
                .filter(|kv| !kv.disabled)
                .map(|kv| (kv.key.clone(), kv.value.clone()))
                .collect(),
        ),
        "formdata" => BodyData::Formdata(
            b.formdata
                .iter()
                .filter(|kv| !kv.disabled)
                .map(|kv| {
                    let is_file = kv.typ.as_deref() == Some("file");
                    FormField {
                        key: kv.key.clone(),
                        value: kv.value.clone(),
                        file: if is_file { Some(file_path(kv)) } else { None },
                    }
                })
                .collect(),
        ),
        "file" => match &b.file {
            Some(f) => BodyData::File(f.src.clone().unwrap_or_default()),
            None => BodyData::None,
        },
        _ => BodyData::None,
    }
}

fn has_content_type(hs: &[(String, String)]) -> bool {
    hs.iter().any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
}

fn curl_escape(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Legacy content used by fallback languages: only raw/graphql have a plain
/// string payload.
fn body_string_for_legacy(body: &BodyData) -> Option<String> {
    match body {
        BodyData::Raw(s) | BodyData::Graphql(s) => Some(s.clone()),
        _ => None,
    }
}

/// For languages without full body support, include the structured field data
/// as comments (or an equivalent representation) so nothing is silently lost.
fn body_fallback_lines(body: &BodyData, comment: &str) -> String {
    match body {
        BodyData::None | BodyData::Raw(_) | BodyData::Graphql(_) => String::new(),
        BodyData::Urlencoded(pairs) => {
            let mut s = format!("{} urlencoded body:\n", comment);
            for (k, v) in pairs {
                s.push_str(&format!("{}   {}={}\n", comment, k, v));
            }
            s
        }
        BodyData::Formdata(fields) => {
            let mut s = format!("{} multipart/form-data body:\n", comment);
            for f in fields {
                match &f.file {
                    Some(p) => s.push_str(&format!("{}   {}=@{} (file)\n", comment, f.key, p)),
                    None => s.push_str(&format!("{}   {}={}\n", comment, f.key, f.value)),
                }
            }
            s
        }
        BodyData::File(p) => format!("{} binary file body: @{}\n", comment, p),
    }
}

pub fn generate(req: &Request, lang: &str) -> anyhow::Result<String> {
    let method = req.method.to_uppercase();
    let url = req.url.effective_url();
    let hs = headers(req);
    let body = body_data(&req.body);
    let out = match lang.to_ascii_lowercase().as_str() {
        "curl" => gen_curl(&method, &url, &hs, &body, req),
        "rust" | "reqwest" => gen_rust(&method, &url, &hs, &body),
        "python" | "requests" => gen_python(&method, &url, &hs, &body),
        "js" | "javascript" | "fetch" => gen_js(&method, &url, &hs, &body),
        "go" | "golang" => gen_go(&method, &url, &hs, &body),
        "nodejs" | "node" | "axios" => gen_node(&method, &url, &hs, &body),
        "java" | "okhttp" => gen_java(&method, &url, &hs, &body),
        "csharp" | "c#" | "dotnet" => gen_csharp(&method, &url, &hs, &body),
        "php" => gen_php(&method, &url, &hs, &body),
        "ruby" => gen_ruby(&method, &url, &hs, &body),
        "powershell" | "ps" => gen_powershell(&method, &url, &hs, &body),
        "swift" => gen_swift(&method, &url, &hs, &body),
        "http" => gen_http(&method, &url, &hs, &body),
        other => anyhow::bail!("unsupported codegen language: {}", other),
    };
    Ok(out)
}

fn gen_curl(
    method: &str,
    url: &str,
    hs: &[(String, String)],
    body: &BodyData,
    req: &Request,
) -> String {
    let mut s = format!("curl --location --request {} '{}'", method, url);
    for (k, v) in hs {
        s.push_str(&format!(" \\\n  --header '{}: {}'", k, v));
    }
    if let Some(auth) = &req.auth {
        match auth.kind.to_ascii_lowercase().as_str() {
            "bearer" => {
                if let Some(t) = auth.bearer_token() {
                    s.push_str(&format!(" \\\n  --header 'Authorization: Bearer {}'", t));
                }
            }
            "basic" => {
                let u = auth.basic_user().unwrap_or("");
                let p = auth.basic_pass().unwrap_or("");
                s.push_str(&format!(" \\\n  --user '{}:{}'", u, p));
            }
            _ => {}
        }
    }
    match body {
        BodyData::None => {}
        BodyData::Raw(b) => s.push_str(&format!(" \\\n  --data '{}'", curl_escape(b))),
        BodyData::Graphql(j) => {
            if !has_content_type(hs) {
                s.push_str(" \\\n  --header 'Content-Type: application/json'");
            }
            s.push_str(&format!(" \\\n  --data '{}'", curl_escape(j)));
        }
        BodyData::Urlencoded(pairs) => {
            for (k, v) in pairs {
                s.push_str(&format!(
                    " \\\n  --data-urlencode '{}={}'",
                    curl_escape(k),
                    curl_escape(v)
                ));
            }
        }
        BodyData::Formdata(fields) => {
            for f in fields {
                match &f.file {
                    Some(p) => s.push_str(&format!(
                        " \\\n  -F '{}=@{}'",
                        curl_escape(&f.key),
                        curl_escape(p)
                    )),
                    None => s.push_str(&format!(
                        " \\\n  -F '{}={}'",
                        curl_escape(&f.key),
                        curl_escape(&f.value)
                    )),
                }
            }
        }
        BodyData::File(p) => s.push_str(&format!(" \\\n  --data-binary '@{}'", curl_escape(p))),
    }
    s
}

fn gen_rust(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut header_lines = String::new();
    for (k, v) in hs {
        header_lines.push_str(&format!("        .header({:?}, {:?})\n", k, v));
    }
    let mut body_lines = String::new();
    match body {
        BodyData::None => {}
        BodyData::Raw(b) | BodyData::Graphql(b) => {
            body_lines.push_str(&format!("        .body({:?})\n", b));
        }
        BodyData::Urlencoded(pairs) => {
            let items: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("({:?}, {:?})", k, v))
                .collect();
            body_lines.push_str(&format!("        .form(&[{}])\n", items.join(", ")));
        }
        BodyData::Formdata(fields) => {
            let mut form = String::from("reqwest::blocking::multipart::Form::new()");
            for f in fields {
                match &f.file {
                    Some(p) => form.push_str(&format!(
                        "\n            .part({:?}, reqwest::blocking::multipart::Part::bytes(std::fs::read({:?})?).file_name({:?}))",
                        f.key, p, f.key
                    )),
                    None => form.push_str(&format!("\n            .text({:?}, {:?})", f.key, f.value)),
                }
            }
            body_lines.push_str(&format!("        .multipart({})\n", form));
        }
        BodyData::File(p) => {
            body_lines.push_str(&format!("        .body(std::fs::read({:?})?)\n", p));
        }
    }
    format!(
        "// Cargo.toml: reqwest = {{ version = \"0.12\", features = [\"blocking\", \"rustls-tls\", \"multipart\"] }}\n\
use reqwest::blocking::Client;\n\n\
fn main() -> Result<(), Box<dyn std::error::Error>> {{\n    \
let client = Client::new();\n    \
let resp = client\n        \
.request(reqwest::Method::from_bytes(b\"{}\")?, \"{}\")\n{}{}        \
.send()?;\n    \
println!(\"{{}}\", resp.status());\n    \
println!(\"{{}}\", resp.text()?);\n    \
Ok(())\n}}\n",
        method, url, header_lines, body_lines
    )
}

fn gen_python(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::new();
    if !hs.is_empty() {
        h.push_str("headers = {\n");
        for (k, v) in hs {
            h.push_str(&format!("    {:?}: {:?},\n", k, v));
        }
        h.push_str("}\n");
    }
    let mut setup = String::new();
    let mut call_args = String::new();
    match body {
        BodyData::None => {}
        BodyData::Raw(b) | BodyData::Graphql(b) => {
            setup.push_str(&format!("data = {:?}\n", b));
            call_args.push_str(", data=data");
        }
        BodyData::Urlencoded(pairs) => {
            setup.push_str("data = {\n");
            for (k, v) in pairs {
                setup.push_str(&format!("    {:?}: {:?},\n", k, v));
            }
            setup.push_str("}\n");
            call_args.push_str(", data=data");
        }
        BodyData::Formdata(fields) => {
            let mut data_fields = String::new();
            let mut file_fields = String::new();
            let mut has_data = false;
            let mut has_files = false;
            for f in fields {
                match &f.file {
                    Some(p) => {
                        file_fields.push_str(&format!("    {:?}: open({:?}, \"rb\"),\n", f.key, p));
                        has_files = true;
                    }
                    None => {
                        data_fields.push_str(&format!("    {:?}: {:?},\n", f.key, f.value));
                        has_data = true;
                    }
                }
            }
            if has_files {
                setup.push_str(&format!("files = {{\n{}}}\n", file_fields));
            }
            if has_data {
                setup.push_str(&format!("data = {{\n{}}}\n", data_fields));
            }
            if has_files {
                call_args.push_str(", files=files");
            }
            if has_data {
                call_args.push_str(", data=data");
            }
        }
        BodyData::File(p) => {
            setup.push_str(&format!("data = open({:?}, \"rb\")\n", p));
            call_args.push_str(", data=data");
        }
    }
    format!(
        "import requests\n\n{}{}\nresponse = requests.request({:?}, {:?}, headers=headers{})\nprint(response.status_code)\nprint(response.text)\n",
        h, setup, method, url, call_args
    )
}

fn js_str_comment(s: &str) -> String {
    s.replace("*/", "*\\/").replace('\n', " ")
}

fn gen_js(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::from("{\n");
    for (k, v) in hs {
        h.push_str(&format!("  {:?}: {:?},\n", k, v));
    }
    h.push('}');
    let mut prelude = String::new();
    let body_part = match body {
        BodyData::None => String::new(),
        BodyData::Raw(b) | BodyData::Graphql(b) => format!(",\n  body: {:?}", b),
        BodyData::Urlencoded(pairs) => {
            let items: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{:?}: {:?}", k, v))
                .collect();
            format!(",\n  body: new URLSearchParams({{{}}})", items.join(", "))
        }
        BodyData::Formdata(fields) => {
            prelude.push_str("const formData = new FormData();\n");
            for f in fields {
                match &f.file {
                    Some(p) => prelude.push_str(&format!(
                        "formData.append({:?}, /* file: {} */ fileFromPath);\n",
                        f.key,
                        js_str_comment(p)
                    )),
                    None => prelude.push_str(&format!(
                        "formData.append({:?}, {:?});\n",
                        f.key, f.value
                    )),
                }
            }
            ",\n  body: formData".to_string()
        }
        BodyData::File(p) => format!(",\n  body: new Blob([/* contents of {} */])", js_str_comment(p)),
    };
    format!(
        "{}const options = {{\n  method: {:?},\n  headers: {}{}\n}};\n\nfetch({:?}, options)\n  .then(res => res.text())\n  .then(console.log);\n",
        prelude, method, h, body_part, url
    )
}

fn gen_go(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut imports: Vec<&str> = vec!["fmt", "io", "net/http"];
    let mut prelude = String::new();
    let mut content_type: Option<&str> = None;
    match body {
        BodyData::None => {
            prelude.push_str("\tvar payload io.Reader\n");
        }
        BodyData::Raw(b) | BodyData::Graphql(b) => {
            imports.push("strings");
            prelude.push_str(&format!("\tpayload := strings.NewReader({:?})\n", b));
            content_type = Some("application/json");
        }
        BodyData::Urlencoded(pairs) => {
            imports.push("net/url");
            imports.push("strings");
            prelude.push_str("\tdata := url.Values{}\n");
            for (k, v) in pairs {
                prelude.push_str(&format!("\tdata.Set({:?}, {:?})\n", k, v));
            }
            prelude.push_str("\tpayload := strings.NewReader(data.Encode())\n");
            content_type = Some("application/x-www-form-urlencoded");
        }
        BodyData::Formdata(fields) => {
            imports.push("bytes");
            imports.push("mime/multipart");
            prelude.push_str("\tvar buf bytes.Buffer\n\tw := multipart.NewWriter(&buf)\n");
            let mut need_os = false;
            for f in fields {
                match &f.file {
                    Some(p) => {
                        need_os = true;
                        prelude.push_str(&format!(
                            "\tfw, _ := w.CreateFormFile({:?}, {:?})\n\tfr, _ := os.Open({:?})\n\tio.Copy(fw, fr)\n\tfr.Close()\n",
                            f.key, p, p
                        ));
                    }
                    None => prelude.push_str(&format!("\tw.WriteField({:?}, {:?})\n", f.key, f.value)),
                }
            }
            if need_os {
                imports.push("os");
            }
            prelude.push_str("\tw.Close()\n\tpayload := io.Reader(&buf)\n");
            content_type = Some("multipart/form-data");
        }
        BodyData::File(p) => {
            imports.push("os");
            prelude.push_str(&format!(
                "\tfr, _ := os.Open({:?})\n\tpayload := io.Reader(fr)\n",
                p
            ));
        }
    }
    let mut uniq: Vec<&str> = Vec::new();
    for i in imports {
        if !uniq.contains(&i) {
            uniq.push(i);
        }
    }
    let mut import_block = String::new();
    for i in &uniq {
        import_block.push_str(&format!("\t{:?}\n", i));
    }
    let mut header_lines = String::new();
    if !has_content_type(hs) {
        if let Some(ct) = content_type {
            if ct != "multipart/form-data" {
                header_lines.push_str(&format!("\treq.Header.Set(\"Content-Type\", {:?})\n", ct));
            }
        }
    }
    for (k, v) in hs {
        header_lines.push_str(&format!("\treq.Header.Set({:?}, {:?})\n", k, v));
    }
    let mut post = String::new();
    if matches!(body, BodyData::Formdata(_)) && !has_content_type(hs) {
        post.push_str("\treq.Header.Set(\"Content-Type\", w.FormDataContentType())\n");
    }
    format!(
        "package main\n\nimport (\n{}\n)\n\nfunc main() {{\n{}{}\treq, _ := http.NewRequest({:?}, {:?}, payload)\n{}{}\tresp, err := http.DefaultClient.Do(req)\n\tif err != nil {{ panic(err) }}\n\tdefer resp.Body.Close()\n\tb, _ := io.ReadAll(resp.Body)\n\tfmt.Println(resp.StatusCode)\n\tfmt.Println(string(b))\n}}\n",
        import_block, prelude, "", method, url, header_lines, post
    )
}

fn gen_node(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::from("{\n");
    for (k, v) in hs {
        h.push_str(&format!("    {:?}: {:?},\n", k, v));
    }
    h.push_str("  }");
    let mut prelude = String::new();
    let (headers_expr, data_expr) = match body {
        BodyData::None => (h.clone(), None),
        BodyData::Raw(b) | BodyData::Graphql(b) => (h.clone(), Some(format!("{:?}", b))),
        BodyData::Urlencoded(pairs) => {
            let items: Vec<String> = pairs
                .iter()
                .map(|(k, v)| format!("{:?}: {:?}", k, v))
                .collect();
            (h.clone(), Some(format!("{{{}}}", items.join(", "))))
        }
        BodyData::Formdata(fields) => {
            prelude.push_str(
                "const FormData = require('form-data');\nconst fs = require('fs');\nconst form = new FormData();\n",
            );
            for f in fields {
                match &f.file {
                    Some(p) => prelude.push_str(&format!(
                        "form.append({:?}, fs.createReadStream({:?}));\n",
                        f.key, p
                    )),
                    None => prelude.push_str(&format!(
                        "form.append({:?}, {:?});\n",
                        f.key, f.value
                    )),
                }
            }
            (
                format!("Object.assign({{}}, {}, form.getHeaders())", h),
                Some("form".to_string()),
            )
        }
        BodyData::File(p) => {
            prelude.push_str("const fs = require('fs');\n");
            (h.clone(), Some(format!("fs.createReadStream({:?})", p)))
        }
    };
    let data_line = data_expr
        .map(|d| format!(",\n  data: {}", d))
        .unwrap_or_default();
    format!(
        "const axios = require('axios');\n{}\naxios({{\n  method: {:?},\n  url: {:?},\n  headers: {}{}\n}}).then(r => console.log(r.status, r.data)).catch(console.error);\n",
        prelude,
        method.to_lowercase(),
        url,
        headers_expr,
        data_line
    )
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn gen_java(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::new();
    for (k, v) in hs {
        h.push_str(&format!("        .header(\"{}\", \"{}\")\n", esc(k), esc(v)));
    }
    let legacy = body_string_for_legacy(body);
    let method_line = match legacy {
        Some(b) => format!(
            "        .method(\"{}\", RequestBody.create(\"{}\", MediaType.parse(\"application/json\")))\n",
            method,
            esc(&b)
        ),
        None => format!("        .method(\"{}\", null)\n", method),
    };
    let fallback = body_fallback_lines(body, "// ");
    format!(
        "// OkHttp (Java)\nimport okhttp3.*;\n\n{}OkHttpClient client = new OkHttpClient();\nRequest request = new Request.Builder()\n        .url(\"{}\")\n{}{}        .build();\nResponse response = client.newCall(request).execute();\nSystem.out.println(response.body().string());\n",
        fallback, url, h, method_line
    )
}

fn gen_csharp(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::new();
    for (k, v) in hs {
        h.push_str(&format!(
            "request.Headers.TryAddWithoutValidation(\"{}\", \"{}\");\n",
            esc(k),
            esc(v)
        ));
    }
    let legacy = body_string_for_legacy(body);
    let body_line = match legacy {
        Some(b) => format!(
            "request.Content = new StringContent(\"{}\", Encoding.UTF8, \"application/json\");\n",
            esc(&b)
        ),
        None => String::new(),
    };
    let fallback = body_fallback_lines(body, "// ");
    format!(
        "// C# HttpClient\nusing System.Net.Http;\nusing System.Text;\n\n{}var client = new HttpClient();\nvar request = new HttpRequestMessage(HttpMethod.{}, \"{}\");\n{}{}var response = await client.SendAsync(request);\nConsole.WriteLine(await response.Content.ReadAsStringAsync());\n",
        fallback, method, url, h, body_line
    )
}

fn gen_php(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let hdrs: Vec<String> = hs
        .iter()
        .map(|(k, v)| format!("\"{}\"", esc(&format!("{}: {}", k, v))))
        .collect();
    let legacy = body_string_for_legacy(body);
    let body_line = match legacy {
        Some(b) => format!("curl_setopt($ch, CURLOPT_POSTFIELDS, \"{}\");\n", esc(&b)),
        None => String::new(),
    };
    let fallback = body_fallback_lines(body, "// ");
    format!(
        "<?php\n{}$ch = curl_init();\ncurl_setopt($ch, CURLOPT_URL, \"{}\");\ncurl_setopt($ch, CURLOPT_RETURNTRANSFER, true);\ncurl_setopt($ch, CURLOPT_CUSTOMREQUEST, \"{}\");\ncurl_setopt($ch, CURLOPT_HTTPHEADER, array({}));\n{}$response = curl_exec($ch);\ncurl_close($ch);\necho $response;\n",
        fallback, url, method, hdrs.join(", "), body_line
    )
}

fn gen_ruby(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::new();
    for (k, v) in hs {
        h.push_str(&format!("  request[\"{}\"] = \"{}\"\n", esc(k), esc(v)));
    }
    let legacy = body_string_for_legacy(body);
    let body_line = match legacy {
        Some(b) => format!("  request.body = \"{}\"\n", esc(&b)),
        None => String::new(),
    };
    let fallback = body_fallback_lines(body, "# ");
    format!(
        "# Ruby net/http\n{}require 'net/http'\nrequire 'uri'\n\nuri = URI('{}')\nrequest = Net::HTTP::{} .new(uri)\n{}{}response = Net::HTTP.start(uri.host, uri.port, use_ssl: uri.scheme == 'https') do |http|\n  http.request(request)\nend\nputs response.body\n",
        fallback, url, method, h, body_line
    )
}

fn gen_powershell(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::new();
    for (k, v) in hs {
        h.push_str(&format!("  \"{}: {}\",\n", esc(k), esc(v)));
    }
    let legacy = body_string_for_legacy(body);
    let body_line = match legacy {
        Some(b) => format!("-Body '{}' ", b.replace('\'', "''")),
        None => String::new(),
    };
    let fallback = body_fallback_lines(body, "# ");
    format!(
        "# PowerShell\n{}$headers = @{{\n{}}}\nInvoke-RestMethod -Uri '{}' -Method {} {} -Headers $headers\n",
        fallback, h, url, method, body_line
    )
}

fn gen_swift(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let mut h = String::new();
    for (k, v) in hs {
        h.push_str(&format!(
            "    request.setValue(\"{}\", forHTTPHeaderField: \"{}\")\n",
            esc(v),
            esc(k)
        ));
    }
    let legacy = body_string_for_legacy(body);
    let body_line = match legacy {
        Some(b) => format!("    request.httpBody = \"{}\".data(using: .utf8)\n", esc(&b)),
        None => String::new(),
    };
    let fallback = body_fallback_lines(body, "// ");
    format!(
        "// Swift URLSession\nimport Foundation\n\n{}var request = URLRequest(url: URL(string: \"{}\")!)\nrequest.httpMethod = \"{}\"\n{}{}let task = URLSession.shared.dataTask(with: request) {{ data, _, _ in\n    print(String(data: data ?? Data(), encoding: .utf8) ?? \"\")\n}}\ntask.resume()\n",
        fallback, url, method, h, body_line
    )
}

fn gen_http(method: &str, url: &str, hs: &[(String, String)], body: &BodyData) -> String {
    let rest = url.splitn(2, "://").nth(1).unwrap_or(url);
    let (host, path) = match rest.split_once('/') {
        Some((h, p)) => (h.to_string(), format!("/{}", p)),
        None => (rest.to_string(), "/".to_string()),
    };
    let mut s = format!("{} {} HTTP/1.1\nHost: {}\n", method, path, host);
    for (k, v) in hs {
        s.push_str(&format!("{}: {}\n", k, v));
    }
    s.push('\n');
    match body {
        BodyData::None => {}
        BodyData::Raw(b) | BodyData::Graphql(b) => s.push_str(b),
        BodyData::Urlencoded(pairs) => {
            let encoded: Vec<String> = pairs.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
            s.push_str(&encoded.join("&"));
        }
        BodyData::Formdata(fields) => {
            for f in fields {
                match &f.file {
                    Some(p) => s.push_str(&format!("[form-data file] {}=@{}\n", f.key, p)),
                    None => s.push_str(&format!("[form-data] {}={}\n", f.key, f.value)),
                }
            }
        }
        BodyData::File(p) => s.push_str(&format!("[binary file body] @{}", p)),
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FileRef, Header, UrlSpec};

    fn sample() -> Request {
        Request {
            method: "POST".into(),
            url: UrlSpec::new("https://api.test/users"),
            header: vec![Header {
                key: "Content-Type".into(),
                value: "application/json".into(),
                ..Default::default()
            }],
            body: Some(Body {
                mode: "raw".into(),
                raw: Some("{\"name\":\"z\"}".into()),
                options: Some(serde_json::json!({"raw":{"language":"json"}})),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn generates_all() {
        for lang in ["curl", "rust", "python", "js", "go", "nodejs"] {
            let out = generate(&sample(), lang).unwrap();
            assert!(!out.is_empty(), "empty for {}", lang);
            assert!(out.contains("api.test"), "missing url for {}", lang);
        }
    }

    #[test]
    fn unknown_lang_errors() {
        assert!(generate(&sample(), "cobol").is_err());
    }

    fn req_with_body(body: Body) -> Request {
        Request {
            method: "POST".into(),
            url: UrlSpec::new("https://api.test/upload"),
            body: Some(body),
            ..Default::default()
        }
    }

    #[test]
    fn urlencoded_body_curl_and_python() {
        let req = req_with_body(Body {
            mode: "urlencoded".into(),
            urlencoded: vec![
                KV {
                    key: "a".into(),
                    value: "1".into(),
                    ..Default::default()
                },
                KV {
                    key: "skip".into(),
                    value: "x".into(),
                    disabled: true,
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        let curl = generate(&req, "curl").unwrap();
        assert!(curl.contains("--data-urlencode 'a=1'"), "curl: {curl}");
        assert!(!curl.contains("skip"), "disabled field leaked: {curl}");
        let python = generate(&req, "python").unwrap();
        assert!(python.contains("\"a\": \"1\""), "python: {python}");
    }

    #[test]
    fn formdata_body_curl_and_python() {
        let req = req_with_body(Body {
            mode: "formdata".into(),
            formdata: vec![
                KV {
                    key: "name".into(),
                    value: "z".into(),
                    ..Default::default()
                },
                KV {
                    key: "avatar".into(),
                    value: String::new(),
                    typ: Some("file".into()),
                    src: Some(serde_json::json!("C:\\tmp\\a.png")),
                    ..Default::default()
                },
            ],
            ..Default::default()
        });
        let curl = generate(&req, "curl").unwrap();
        assert!(curl.contains("-F 'name=z'"), "curl: {curl}");
        assert!(
            curl.contains("-F 'avatar=@C:\\tmp\\a.png'"),
            "curl: {curl}"
        );
        let python = generate(&req, "python").unwrap();
        assert!(python.contains("files = {"), "python: {python}");
        assert!(python.contains("open("), "python: {python}");
        assert!(python.contains("a.png"), "python: {python}");
    }

    #[test]
    fn file_body_curl_and_python() {
        let req = req_with_body(Body {
            mode: "file".into(),
            file: Some(FileRef {
                src: Some("C:\\tmp\\data.bin".into()),
                content: None,
            }),
            ..Default::default()
        });
        let curl = generate(&req, "curl").unwrap();
        assert!(
            curl.contains("--data-binary '@C:\\tmp\\data.bin'"),
            "curl: {curl}"
        );
        let python = generate(&req, "python").unwrap();
        assert!(
            python.contains("open(\"C:\\\\tmp\\\\data.bin\", \"rb\")"),
            "python: {python}"
        );
    }
}

