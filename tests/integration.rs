use postman_rs::http;
use postman_rs::model::*;
use postman_rs::runner;
use postman_rs::variables::VarContext;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

/// Spin up a tiny HTTP/1.1 server that answers a few requests, then returns its port.
fn start_server(conns: usize) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    thread::spawn(move || {
        for _ in 0..conns {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let first = req.lines().next().unwrap_or("");
                let resp: String = if first.contains("/hello") {
                    let body = "hello world";
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                } else if first.contains("/users") {
                    let body = r#"{"id":"abc","ok":true}"#;
                    format!(
                        "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                } else {
                    let body = "not found";
                    format!(
                        "HTTP/1.1 404 Not Found\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    )
                };
                let _ = stream.write_all(resp.as_bytes());
            }
        }
    });
    port
}

fn demo_collection(base: &str) -> Collection {
    Collection {
        info: CollectionInfo {
            name: "it".into(),
            ..Default::default()
        },
        variable: vec![Variable {
            key: "baseUrl".into(),
            value: serde_json::json!(base),
            enabled: Some(true),
            ..Default::default()
        }],
        item: vec![
            Item {
                name: "hello".into(),
                request: Some(Request {
                    method: "GET".into(),
                    url: UrlSpec::new("{{baseUrl}}/hello"),
                    ..Default::default()
                }),
                ..Default::default()
            },
            Item {
                name: "create".into(),
                request: Some(Request {
                    method: "POST".into(),
                    url: UrlSpec::new("{{baseUrl}}/users"),
                    body: Some(Body {
                        mode: "raw".into(),
                        raw: Some("{\"name\":\"z\"}".into()),
                        ..Default::default()
                    }),
                    tests: vec![
                        TestAssertion {
                            kind: "status".into(),
                            equals: Some(serde_json::json!(201)),
                            ..Default::default()
                        },
                        TestAssertion {
                            kind: "json_path".into(),
                            path: Some("ok".into()),
                            equals: Some(serde_json::json!(true)),
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

#[test]
fn send_single_request() {
    let port = start_server(1);
    let mut ctx = VarContext::new();
    let mut m = std::collections::HashMap::new();
    m.insert("baseUrl".to_string(), format!("http://127.0.0.1:{}", port));
    ctx.push_high(m);
    let req = Request {
        method: "GET".into(),
        url: UrlSpec::new("{{baseUrl}}/hello"),
        ..Default::default()
    };
    let rec = http::execute(&req, &ctx, 5).unwrap();
    assert_eq!(rec.code, 200);
    assert!(rec.body.contains("hello world"));
}

#[test]
fn run_collection_end_to_end() {
    let port = start_server(2);
    let base = format!("http://127.0.0.1:{}", port);
    let c = demo_collection(&base);
    let summary = runner::run_collection(&c, None, 1, vec![], 0, 5);
    assert_eq!(summary.total, 2);
    assert_eq!(summary.failed, 0, "results: {:?}", summary.results.iter().map(|r| (&r.name, r.code, r.passed)).collect::<Vec<_>>());
}

/// 验证 P3-1 脚本引擎在集合运行器中的端到端集成：
/// 前置脚本写入变量并用于 URL 解析；测试脚本执行 pm.test 断言。
#[test]
fn run_collection_with_pm_scripts() {
    let port = start_server(1);
    let base = format!("http://127.0.0.1:{}", port);
    let c = Collection {
        info: CollectionInfo { name: "scripts".into(), ..Default::default() },
        variable: vec![Variable {
            key: "baseUrl".into(),
            value: serde_json::json!(base),
            enabled: Some(true),
            ..Default::default()
        }],
        item: vec![Item {
            name: "scripted".into(),
            request: Some(Request {
                method: "GET".into(),
                // 路径变量 path 由前置脚本写入
                url: UrlSpec::new("{{baseUrl}}/{{path}}"),
                ..Default::default()
            }),
            event: vec![
                Event {
                    listen: "prerequest".into(),
                    script: Some(Script {
                        exec: vec!["pm.environment.set(\"path\", \"users\");".into()],
                        kind: None,
                    }),
                },
                Event {
                    listen: "test".into(),
                    script: Some(Script {
                        exec: vec![
                            "pm.test(\"status 201\", function () { pm.response.to.have.status(201); });".into(),
                            "pm.test(\"ok true\", function () { pm.expect(pm.response.json().ok).to.eql(true); });".into(),
                        ],
                        kind: None,
                    }),
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    };
    let summary = runner::run_collection(&c, None, 1, vec![], 0, 5);
    assert_eq!(summary.total, 1);
    assert_eq!(summary.results[0].code, Some(201), "前置脚本应把 {{path}} 解析为 users");
    let names: Vec<(String, bool)> = summary.results[0]
        .tests
        .iter()
        .map(|t| (t.name.clone(), t.passed))
        .collect();
    assert!(
        names.iter().any(|(n, p)| n == "status 201" && *p),
        "tests: {:?}",
        names
    );
    assert!(
        names.iter().any(|(n, p)| n == "ok true" && *p),
        "tests: {:?}",
        names
    );
    assert_eq!(summary.failed, 0, "tests: {:?}", names);
}
