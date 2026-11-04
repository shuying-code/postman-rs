#![windows_subsystem = "windows"] // 语句
//! pmrs-gui — 轻量级 egui 桌面客户端（本地、无账号、中文界面）。
//! 视觉上尽量贴近 Postman 桌面版：橙色品牌色、彩色方法下拉、
//! 请求卡片（参数 / 授权 / 请求头 / 请求体）、响应面板（状态胶囊 + 响应体/响应头）。
use base64::Engine as _; // 引入依赖
use eframe::egui; // 引入依赖
use egui::{Color32, RichText, Rounding, Stroke}; // 引入依赖
use postman_rs::http::{self, ResponseRecord}; // 引入依赖
use postman_rs::import_export; // 引入依赖
use postman_rs::codegen; // 引入依赖
use postman_rs::model::{ // 引入依赖
    Auth, Body, Collection, CollectionInfo, Environment, Event, FileRef, GraphQL, Header, Item, KV, // 语句
    Request, ResponseExample, TestAssertion, UrlSpec, Variable, // 语句
}; // 块结束
use postman_rs::runner::{self, RunOptions}; // 引入依赖
use postman_rs::storage; // 引入依赖
use postman_rs::scripting; // 引入依赖
use postman_rs::tests_engine; // 引入依赖
use postman_rs::variables::VarContext; // 引入依赖
use std::collections::HashMap; // 引入依赖
use std::path::{Path, PathBuf}; // 引入依赖
use std::sync::atomic::{AtomicUsize, Ordering}; // 引入依赖
use std::sync::mpsc; // 引入依赖
// ---------------------------------------------------------------------------
// 调色板（参照 Postman 品牌与界面配色）
// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// 主题系统（多套配色 + 渐变背景）
// ---------------------------------------------------------------------------
struct Theme { // 结构体 Theme
    name: &'static str, // 字段 name
    dark: bool, // 字段 dark
    app: [u8; 3],    // 应用背景（用于渐变基底）
    panel: [u8; 3],  // 卡片/面板
    side: [u8; 3],   // 侧栏
    border: [u8; 3], // 描边
    text: [u8; 3],   // 主文字
    text2: [u8; 3],  // 次文字
    accent: [u8; 3], // 品牌/强调色
} // 块结束
static THEMES: [Theme; 8] = [ // 常量 THEMES
    Theme { // 语句
        name: "Postman 浅色", // 字段 name
        dark: false, // 字段 dark
        app: [249, 250, 251], // 字段 app
        panel: [255, 255, 255], // 字段 panel
        side: [246, 247, 249], // 字段 side
        border: [228, 230, 235], // 字段 border
        text: [33, 37, 41], // 字段 text
        text2: [110, 116, 123], // 字段 text2
        accent: [255, 108, 55], // 字段 accent
    }, // 块结束
    Theme { // 语句
        name: "Postman 深色", // 字段 name
        dark: true, // 字段 dark
        app: [24, 26, 30], // 字段 app
        panel: [32, 34, 39], // 字段 panel
        side: [38, 40, 46], // 字段 side
        border: [60, 63, 72], // 字段 border
        text: [228, 231, 236], // 字段 text
        text2: [150, 156, 166], // 字段 text2
        accent: [255, 108, 55], // 字段 accent
    }, // 块结束
    Theme { // 语句
        name: "午夜蓝", // 字段 name
        dark: true, // 字段 dark
        app: [15, 23, 42], // 字段 app
        panel: [23, 32, 51], // 字段 panel
        side: [30, 41, 59], // 字段 side
        border: [44, 58, 82], // 字段 border
        text: [226, 232, 240], // 字段 text
        text2: [148, 163, 184], // 字段 text2
        accent: [59, 130, 246], // 字段 accent
    }, // 块结束
    Theme { // 语句
        name: "石墨灰", // 字段 name
        dark: true, // 字段 dark
        app: [28, 28, 31], // 字段 app
        panel: [38, 38, 43], // 字段 panel
        side: [46, 46, 52], // 字段 side
        border: [63, 63, 70], // 字段 border
        text: [229, 229, 231], // 字段 text
        text2: [161, 161, 170], // 字段 text2
        accent: [245, 158, 11], // 字段 accent
    }, // 块结束
    Theme { // 语句
        name: "暖沙（浅）", // 字段 name
        dark: false, // 字段 dark
        app: [253, 246, 227], // 字段 app
        panel: [255, 251, 240], // 字段 panel
        side: [245, 239, 217], // 字段 side
        border: [227, 217, 190], // 字段 border
        text: [59, 58, 50], // 字段 text
        text2: [133, 127, 107], // 字段 text2
        accent: [203, 75, 22], // 字段 accent
    }, // 块结束
    Theme { // 语句
        name: "森林绿", // 字段 name
        dark: true, // 字段 dark
        app: [14, 26, 22], // 字段 app
        panel: [20, 36, 30], // 字段 panel
        side: [26, 46, 39], // 字段 side
        border: [42, 68, 58], // 字段 border
        text: [227, 240, 234], // 字段 text
        text2: [147, 179, 166], // 字段 text2
        accent: [34, 197, 94], // 字段 accent
    }, // 块结束
    Theme { // 语句
        name: "樱粉（浅）", // 字段 name
        dark: false, // 字段 dark
        app: [255, 245, 247], // 字段 app
        panel: [255, 255, 255], // 字段 panel
        side: [251, 237, 240], // 字段 side
        border: [240, 217, 223], // 字段 border
        text: [58, 42, 48], // 字段 text
        text2: [138, 107, 116], // 字段 text2
        accent: [244, 63, 94], // 字段 accent
    }, // 块结束
    Theme { // 语句
        name: "高对比", // 字段 name
        dark: true, // 字段 dark
        app: [0, 0, 0], // 字段 app
        panel: [13, 13, 13], // 字段 panel
        side: [20, 20, 20], // 字段 side
        border: [58, 58, 58], // 字段 border
        text: [255, 255, 255], // 字段 text
        text2: [200, 200, 200], // 字段 text2
        accent: [255, 108, 55], // 字段 accent
    }, // 块结束
]; // 语句
static THEME: AtomicUsize = AtomicUsize::new(0); // 常量 THEME
/// 是否跟随系统深浅色（true 时按 ctx.system_theme() 选 0=浅 / 1=深）。
static FOLLOW_SYSTEM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false); // 常量 FOLLOW_SYSTEM
/// 自定义强调色覆盖。0 表示不覆盖；否则低 24 位为 RGB。
static ACCENT_OVERRIDE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0); // 常量 ACCENT_OVERRIDE
fn theme() -> &'static Theme { // 函数 theme
    &THEMES[THEME.load(Ordering::Relaxed).min(THEMES.len() - 1)] // 语句
} // 块结束
fn rgb(a: [u8; 3]) -> Color32 { // 函数 rgb
    Color32::from_rgb(a[0], a[1], a[2]) // 调用 from_rgb
} // 块结束
fn mix(a: Color32, b: Color32, t: f32) -> Color32 { // 函数 mix
    let t = t.clamp(0.0, 1.0); // 变量 t
    let f = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t).round() as u8; // 变量 f
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b())) // 调用 from_rgb
} // 块结束
fn dark() -> bool { // 函数 dark
    theme().dark // 调用 theme
} // 块结束
fn brand() -> Color32 { // 函数 brand
    let v = ACCENT_OVERRIDE.load(Ordering::Relaxed); // 变量 v
    if v != 0 { // 条件分支
        Color32::from_rgb(((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8) // 调用 from_rgb
    } else { // 语句
        rgb(theme().accent) // 调用 rgb
    } // 块结束
} // 块结束
fn brand_soft() -> Color32 { // 函数 brand_soft
    if dark() { // 条件分支
        mix(brand(), Color32::from_rgb(20, 20, 22), 0.72) // 调用 mix
    } else { // 语句
        mix(brand(), Color32::WHITE, 0.86) // 调用 mix
    } // 块结束
} // 块结束
fn send_blue() -> Color32 { // 函数 send_blue
    Color32::from_rgb(9, 123, 237) // Postman 发送按钮蓝
} // 块结束
fn bg_app() -> Color32 { // 函数 bg_app
    rgb(theme().app) // 调用 rgb
} // 块结束
fn bg_panel() -> Color32 { // 函数 bg_panel
    rgb(theme().panel) // 调用 rgb
} // 块结束
fn bg_side() -> Color32 { // 函数 bg_side
    rgb(theme().side) // 调用 rgb
} // 块结束
fn border() -> Color32 { // 函数 border
    rgb(theme().border) // 调用 rgb
} // 块结束
fn text_primary() -> Color32 { // 函数 text_primary
    rgb(theme().text) // 调用 rgb
} // 块结束
fn text_secondary() -> Color32 { // 函数 text_secondary
    rgb(theme().text2) // 调用 rgb
} // 块结束
fn app_top() -> Color32 { // 函数 app_top
    if dark() { // 条件分支
        mix(bg_app(), Color32::WHITE, 0.05) // 调用 mix
    } else { // 语句
        mix(bg_app(), Color32::WHITE, 0.45) // 调用 mix
    } // 块结束
} // 块结束
fn app_bottom() -> Color32 { // 函数 app_bottom
    if dark() { // 条件分支
        mix(bg_app(), Color32::BLACK, 0.22) // 调用 mix
    } else { // 语句
        mix(bg_app(), Color32::BLACK, 0.05) // 调用 mix
    } // 块结束
} // 块结束
/// 在主内容区绘制纵向渐变背景，营造层次感。
fn paint_app_gradient(ui: &egui::Ui) { // 函数 paint_app_gradient
    let rect = ui.max_rect().expand2(egui::vec2(12.0, 10.0)); // 变量 rect
    let mut mesh = egui::Mesh::default(); // 变量 mesh
    mesh.colored_vertex(rect.left_top(), app_top()); // 语句
    mesh.colored_vertex(rect.right_top(), app_top()); // 语句
    mesh.colored_vertex(rect.right_bottom(), app_bottom()); // 语句
    mesh.colored_vertex(rect.left_bottom(), app_bottom()); // 语句
    mesh.add_triangle(0, 1, 2); // 语句
    mesh.add_triangle(0, 2, 3); // 语句
    ui.painter().add(egui::Shape::mesh(mesh)); // 语句
} // 块结束
// ---------------------------------------------------------------------------
// 设计系统：统一圆角 / 间距 / 卡片 / 标签 / 分段控件
// ---------------------------------------------------------------------------
const R_SM: f32 = 6.0; // 常量 R_SM
const R_MD: f32 = 10.0; // 常量 R_MD
const R_LG: f32 = 14.0; // 常量 R_LG

fn card(shadow: bool) -> egui::Frame { // 函数 card
    let mut f = egui::Frame::none() // 变量 f
        .fill(bg_panel()) // 语句
        .stroke(Stroke::new(1.0_f32, border())) // 语句
        .rounding(Rounding::same(R_LG)) // 语句
        .inner_margin(egui::Margin::same(14.0)); // 语句
    if shadow { // 条件分支
        f = f.shadow(egui::epaint::Shadow { // 语句
            offset: egui::vec2(0.0, 4.0), // 字段 offset
            blur: 18.0, // 字段 blur
            spread: 0.0, // 字段 spread
            color: if dark() { // 语句
                Color32::from_black_alpha(90) // 调用 from_black_alpha
            } else { // 语句
                Color32::from_black_alpha(22) // 调用 from_black_alpha
            }, // 块结束
        }); // 语句
    } // 块结束
    f // 语句
} // 块结束

fn section_title(ui: &mut egui::Ui, text: &str) { // 函数 section_title
    ui.horizontal(|ui| { // 语句
        let (r, _) = ui.allocate_exact_size(egui::vec2(3.0, 14.0), egui::Sense::hover()); // 调用 let
        ui.painter().rect_filled(r, Rounding::same(2.0), brand()); // 语句
        ui.add_space(5.0); // 语句
        ui.label(RichText::new(text).size(12.5).strong().color(text_primary())); // 语句
    }); // 语句
} // 块结束

/// 顶部品牌渐变细条
fn top_accent(ui: &mut egui::Ui) { // 函数 top_accent
    let w = ui.available_width(); // 变量 w
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 3.0), egui::Sense::hover()); // 调用 let
    let a = brand(); // 变量 a
    let b = mix(brand(), bg_panel(), 0.7); // 变量 b
    let mut m = egui::Mesh::default(); // 变量 m
    m.colored_vertex(rect.left_top(), a); // 语句
    m.colored_vertex(rect.right_top(), b); // 语句
    m.colored_vertex(rect.right_bottom(), b); // 语句
    m.colored_vertex(rect.left_bottom(), a); // 语句
    m.add_triangle(0, 1, 2); // 语句
    m.add_triangle(0, 2, 3); // 语句
    ui.painter().add(egui::Shape::mesh(m)); // 语句
    ui.add_space(5.0); // 语句
} // 块结束

/// 下划线风格标签页；返回是否切换
fn underline_tabs(ui: &mut egui::Ui, sel: &mut usize, items: &[&str]) { // 函数 underline_tabs
    ui.horizontal(|ui| { // 语句
        ui.spacing_mut().item_spacing.x = 4.0; // 语句
        for (i, name) in items.iter().enumerate() { // 循环
            let active = *sel == i; // 变量 active
            let color = if active { brand() } else { text_secondary() }; // 变量 color
            let resp = ui.add( // 变量 resp
                egui::Button::new(RichText::new(*name).color(color).size(13.5).strong()) // 调用 new
                    .frame(false) // 语句
                    .min_size(egui::vec2(0.0, 30.0)), // 语句
            ); // 块结束
            if active { // 条件分支
                let r = resp.rect; // 变量 r
                let y = r.bottom() + 1.0; // 变量 y
                ui.painter().line_segment( // 语句
                    [egui::pos2(r.left() + 3.0, y), egui::pos2(r.right() - 3.0, y)], // 语句
                    Stroke::new(2.0_f32, brand()), // 字段 Stroke
                ); // 块结束
            } // 块结束
            if resp.clicked() { // 条件分支
                *sel = i; // 语句
            } // 块结束
        } // 块结束
    }); // 语句
} // 块结束

/// 分段控件（侧栏顶部四个入口）
fn segmented(ui: &mut egui::Ui, sel: &mut usize, items: &[&str]) { // 函数 segmented
    let total_w = ui.available_width(); // 变量 total_w
    egui::Frame::none() // 调用 none
        .fill(mix(bg_side(), text_primary(), 0.06)) // 语句
        .rounding(Rounding::same(R_MD)) // 语句
        .inner_margin(egui::Margin::same(3.0)) // 语句
        .show(ui, |ui| { // 语句
            ui.horizontal(|ui| { // 语句
                ui.spacing_mut().item_spacing.x = 2.0; // 语句
                let n = items.len() as f32; // 变量 n
                let w = ((total_w - 6.0) / n - 2.0).max(40.0); // 变量 w
                for (i, name) in items.iter().enumerate() { // 循环
                    let active = *sel == i; // 变量 active
                    let (bg, tc) = if active { // 调用 let
                        (brand(), Color32::WHITE) // 语句
                    } else { // 语句
                        (Color32::TRANSPARENT, text_secondary()) // 语句
                    }; // 块结束
                    if ui // 条件分支
                        .add_sized( // 语句
                            [w, 26.0], // 语句
                            egui::Button::new( // 调用 new
                                RichText::new(*name).color(tc).size(12.5).strong(), // 字段 RichText
                            ) // 块结束
                            .fill(bg) // 语句
                            .rounding(Rounding::same(R_SM)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        *sel = i; // 语句
                    } // 块结束
                } // 块结束
            }); // 语句
        }); // 语句
} // 块结束

/// 方法彩色小药丸（GET/POST…）
fn method_pill(ui: &mut egui::Ui, method: &str) { // 函数 method_pill
    let c = method_color(method); // 变量 c
    egui::Frame::none() // 调用 none
        .fill(mix(c, bg_panel(), 0.82)) // 语句
        .rounding(Rounding::same(R_SM)) // 语句
        .inner_margin(egui::Margin::symmetric(6.0, 1.0)) // 语句
        .show(ui, |ui| { // 语句
            ui.label(RichText::new(method.to_uppercase()).color(c).strong().size(10.5)); // 语句
        }); // 语句
} // 块结束

fn method_color(m: &str) -> Color32 { // 函数 method_color
    match m.to_uppercase().as_str() { // 分支匹配
        "GET" => Color32::from_rgb(73, 162, 91), // 匹配分支
        "POST" => Color32::from_rgb(240, 164, 0), // 匹配分支
        "PUT" => Color32::from_rgb(9, 123, 237), // 匹配分支
        "PATCH" => Color32::from_rgb(155, 89, 182), // 匹配分支
        "DELETE" => Color32::from_rgb(235, 32, 19), // 匹配分支
        "HEAD" => Color32::from_rgb(135, 119, 217), // 匹配分支
        "OPTIONS" => Color32::from_rgb(255, 160, 0), // 匹配分支
        _ => text_secondary(), // 匹配分支
    } // 块结束
} // 块结束
fn status_color(code: u16) -> Color32 { // 函数 status_color
    match code { // 分支匹配
        200..=299 => Color32::from_rgb(73, 162, 91), // 匹配分支
        300..=399 => Color32::from_rgb(9, 123, 237), // 匹配分支
        400..=499 => Color32::from_rgb(240, 164, 0), // 匹配分支
        _ => Color32::from_rgb(235, 32, 19), // 匹配分支
    } // 块结束
} // 块结束
fn human_size(n: usize) -> String { // 函数 human_size
    let f = n as f64; // 变量 f
    if f < 1024.0 { // 条件分支
        format!("{} B", n) // 语句
    } else if f < 1024.0 * 1024.0 { // 条件分支
        format!("{:.1} KB", f / 1024.0) // 语句
    } else { // 语句
        format!("{:.1} MB", f / 1_048_576.0) // 语句
    } // 块结束
} // 块结束
fn pretty_body(s: &str) -> String { // 函数 pretty_body
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) { // 条件分支
        if let Ok(p) = serde_json::to_string_pretty(&v) { // 条件分支
            return p; // 返回
        } // 块结束
    } // 块结束
    s.to_string() // 语句
} // 块结束
fn highlight_json(ui: &egui::Ui, text: &str, wrap_width: f32) -> std::sync::Arc<egui::text::Galley> { // 函数 highlight_json
    use egui::text::{LayoutJob, TextFormat}; // 引入依赖
    let font = egui::FontId::monospace(13.0); // 变量 font
    let base = TextFormat { // 变量 base
        font_id: font.clone(), // 字段 font_id
        color: text_primary(), // 字段 color
        ..Default::default() // 语句
    }; // 块结束
    let str_c = Color32::from_rgb(96, 180, 120); // 变量 str_c
    let num_c = Color32::from_rgb(220, 150, 80); // 变量 num_c
    let key_c = Color32::from_rgb(110, 160, 240); // 变量 key_c
    let kw_c = Color32::from_rgb(170, 140, 220); // 变量 kw_c
    let mut job = LayoutJob::default(); // 变量 job
    job.wrap.max_width = wrap_width; // 语句
    let chars: Vec<char> = text.chars().collect(); // 变量 chars
    let mut i = 0usize; // 变量 i
    while i < chars.len() { // 循环
        let c = chars[i]; // 变量 c
        if c == '"' { // 条件分支
            let start = i; // 变量 start
            i += 1; // 语句
            while i < chars.len() { // 循环
                let d = chars[i]; // 变量 d
                i += 1; // 语句
                if d == '\\' { // 条件分支
                    i += 1; // 语句
                    continue; // 语句
                } // 块结束
                if d == '"' { // 条件分支
                    break; // 语句
                } // 块结束
            } // 块结束
            let s: String = chars[start..i.min(chars.len())].iter().collect(); // 变量 s
            let mut j = i; // 变量 j
            while j < chars.len() && chars[j].is_whitespace() { // 循环
                j += 1; // 语句
            } // 块结束
            let is_key = j < chars.len() && chars[j] == ':'; // 变量 is_key
            job.append( // 语句
                &s, // 语句
                0.0, // 语句
                TextFormat { // 语句
                    font_id: font.clone(), // 字段 font_id
                    color: if is_key { key_c } else { str_c }, // 字段 color
                    ..Default::default() // 语句
                }, // 块结束
            ); // 块结束
        } else if c.is_ascii_digit() // 条件分支
            || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) // 语句
        { // 语句
            let start = i; // 变量 start
            i += 1; // 语句
            while i < chars.len() // 循环
                && (chars[i].is_ascii_digit() // 语句
                    || matches!(chars[i], '.' | 'e' | 'E' | '+' | '-')) // 语句
            { // 语句
                i += 1; // 语句
            } // 块结束
            let s: String = chars[start..i].iter().collect(); // 变量 s
            job.append( // 语句
                &s, // 语句
                0.0, // 语句
                TextFormat { // 语句
                    font_id: font.clone(), // 字段 font_id
                    color: num_c, // 字段 color
                    ..Default::default() // 语句
                }, // 块结束
            ); // 块结束
        } else if c.is_alphabetic() { // 条件分支
            let start = i; // 变量 start
            i += 1; // 语句
            while i < chars.len() && chars[i].is_alphabetic() { // 循环
                i += 1; // 语句
            } // 块结束
            let s: String = chars[start..i].iter().collect(); // 变量 s
            let color = match s.as_str() { // 变量 color
                "true" | "false" | "null" => kw_c, // 匹配分支
                _ => text_primary(), // 匹配分支
            }; // 块结束
            job.append( // 语句
                &s, // 语句
                0.0, // 语句
                TextFormat { // 语句
                    font_id: font.clone(), // 字段 font_id
                    color, // 语句
                    ..Default::default() // 语句
                }, // 块结束
            ); // 块结束
        } else { // 语句
            let s = c.to_string(); // 变量 s
            i += 1; // 语句
            job.append(&s, 0.0, base.clone()); // 语句
        } // 块结束
    } // 块结束
    ui.fonts(|f| f.layout_job(job)) // 语句
} // 块结束
fn parse_query(url: &str) -> Vec<KVRow> { // 函数 parse_query
    let q = url.split_once('?').map(|(_, q)| q).unwrap_or(""); // 变量 q
    q.split('&') // 语句
        .filter(|s| !s.is_empty()) // 语句
        .map(|pair| match pair.split_once('=') { // 语句
            Some((k, v)) => KVRow { // 匹配分支
                key: k.to_string(), // 字段 key
                value: v.to_string(), // 字段 value
                ..Default::default() // 语句
            }, // 块结束
            None => KVRow { // 匹配分支
                key: pair.to_string(), // 字段 key
                ..Default::default() // 语句
            }, // 块结束
        }) // 语句
        .collect() // 语句
} // 块结束

/// 扫描 URL 路径段中的 `:name` 形式的路径变量，返回去重后的变量行。
fn detect_path_vars(url: &str) -> Vec<KVRow> { // 函数 detect_path_vars
    let path = url.split(['?', '#']).next().unwrap_or(""); // 变量 path
    let mut out: Vec<KVRow> = Vec::new(); // 变量 out
    for seg in path.split('/') { // 循环
        if let Some(name) = seg.strip_prefix(':') { // 条件分支
            let name = name.trim(); // 变量 name
            if !name.is_empty() && !out.iter().any(|r| r.key == name) { // 条件分支
                out.push(KVRow { // 语句
                    key: name.to_string(), // 字段 key
                    ..Default::default() // 语句
                }); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    out // 语句
} // 块结束

fn rebuild_url(tab: &mut Tab) { // 函数 rebuild_url
    let base = tab // 变量 base
        .url // 语句
        .split(['?', '#']) // 语句
        .next() // 语句
        .unwrap_or("") // 语句
        .to_string(); // 语句
    let q: Vec<String> = tab // 变量 q
        .params // 语句
        .iter() // 语句
        .filter(|r| !r.disabled && !r.key.trim().is_empty()) // 语句
        .map(|r| { // 语句
            if r.value.is_empty() { // 条件分支
                r.key.clone() // 语句
            } else { // 语句
                format!("{}={}", r.key, r.value) // 语句
            } // 块结束
        }) // 语句
        .collect(); // 语句
    let mut url = if q.is_empty() { // 变量 url
        base // 语句
    } else { // 语句
        format!("{}?{}", base, q.join("&")) // 语句
    }; // 块结束
    if !tab.hash.trim().is_empty() { // 条件分支
        url.push('#'); // 语句
        url.push_str(tab.hash.trim().trim_start_matches('#')); // 语句
    } // 块结束
    tab.url = url; // 语句
} // 块结束
// ---------------------------------------------------------------------------
// 主题 / 字体
// ---------------------------------------------------------------------------
fn install_style(ctx: &egui::Context) { // 函数 install_style
    let mut style = (*ctx.style()).clone(); // 变量 style
    let mut v = egui::Visuals::light(); // 变量 v
    v.dark_mode = dark(); // 语句
    v.panel_fill = bg_panel(); // 语句
    v.window_fill = bg_panel(); // 语句
    v.extreme_bg_color = if dark() { // 语句
        mix(bg_panel(), Color32::BLACK, 0.35) // 调用 mix
    } else { // 语句
        mix(bg_panel(), Color32::WHITE, 0.5) // 调用 mix
    }; // 块结束
    v.faint_bg_color = mix(bg_panel(), text_primary(), 0.07); // 语句
    v.override_text_color = Some(text_primary()); // 语句
    v.hyperlink_color = brand(); // 语句
    let a = brand(); // 变量 a
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(a.r(), a.g(), a.b(), 60); // 语句
    v.selection.stroke = Stroke::new(1.0_f32, brand()); // 语句
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border()); // 语句
    v.widgets.noninteractive.rounding = Rounding::same(6.0); // 语句
    v.widgets.noninteractive.weak_bg_fill = mix(bg_panel(), text_primary(), 0.04); // 语句
    v.widgets.inactive.bg_fill = mix(bg_panel(), text_primary(), 0.06); // 语句
    v.widgets.inactive.weak_bg_fill = mix(bg_panel(), text_primary(), 0.09); // 语句
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border()); // 语句
    v.widgets.inactive.rounding = Rounding::same(6.0); // 语句
    v.widgets.hovered.bg_fill = mix(bg_panel(), text_primary(), 0.11); // 语句
    v.widgets.hovered.weak_bg_fill = mix(bg_panel(), text_primary(), 0.15); // 语句
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, border()); // 语句
    v.widgets.hovered.rounding = Rounding::same(6.0); // 语句
    v.widgets.active.bg_fill = brand_soft(); // 语句
    v.widgets.active.weak_bg_fill = brand_soft(); // 语句
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, brand()); // 语句
    v.widgets.active.rounding = Rounding::same(6.0); // 语句
    v.window_rounding = Rounding::same(8.0); // 语句
    style.visuals = v; // 语句
    style.spacing.item_spacing = egui::vec2(8.0, 7.0); // 语句
    style.spacing.button_padding = egui::vec2(10.0, 5.0); // 语句
    style.spacing.interact_size.y = 26.0; // 语句
    ctx.set_style(style); // 语句
} // 块结束
fn apply_theme(ctx: &egui::Context, idx: usize) { // 函数 apply_theme
    THEME.store(idx.min(THEMES.len() - 1), Ordering::Relaxed); // 语句
    install_style(ctx); // 调用 install_style
} // 块结束
fn theme_pref_path() -> PathBuf { // 函数 theme_pref_path
    storage::home().join("gui_theme.txt") // 调用 home
} // 块结束
fn initial_theme() -> usize { // 函数 initial_theme
    if let Ok(v) = std::env::var("PMRS_THEME") { // 条件分支
        if let Ok(n) = v.trim().parse::<usize>() { // 条件分支
            return n.min(THEMES.len() - 1); // 返回
        } // 块结束
    } // 块结束
    std::fs::read_to_string(theme_pref_path()) // 调用 read_to_string
        .ok() // 语句
        .and_then(|s| s.trim().parse::<usize>().ok()) // 语句
        .unwrap_or(0) // 语句
        .min(THEMES.len() - 1) // 语句
} // 块结束
fn save_theme(idx: usize) { // 函数 save_theme
    let _ = std::fs::create_dir_all(storage::home()); // 变量 _
    let _ = std::fs::write(theme_pref_path(), idx.to_string()); // 变量 _
} // 块结束
/// 解析 "RRGGBB" / "#RRGGBB" 为 24 位 RGB 值。
fn parse_hex_color(s: &str) -> Option<u32> { // 函数 parse_hex_color
    let t = s.trim().trim_start_matches('#'); // 变量 t
    if t.len() != 6 || !t.chars().all(|c| c.is_ascii_hexdigit()) { // 条件分支
        return None; // 返回
    } // 块结束
    u32::from_str_radix(t, 16).ok() // 调用 from_str_radix
} // 块结束
fn install_fonts(ctx: &egui::Context) { // 函数 install_fonts
    let candidates = [ // 变量 candidates
        "C:\\Windows\\Fonts\\simhei.ttf", // 语句
        "C:\\Windows\\Fonts\\msyh.ttc", // 语句
        "C:\\Windows\\Fonts\\simsun.ttc", // 语句
        "/System/Library/Fonts/PingFang.ttc", // 语句
        "/System/Library/Fonts/STHeiti Medium.ttc", // 语句
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", // 语句
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc", // 语句
    ]; // 语句
    for p in candidates { // 循环
        if Path::new(p).exists() { // 条件分支
            if let Ok(bytes) = std::fs::read(p) { // 条件分支
                let mut fonts = egui::FontDefinitions::default(); // 变量 fonts
                fonts // 语句
                    .font_data // 语句
                    .insert("cjk".to_owned(), egui::FontData::from_owned(bytes)); // 语句
                if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Proportional) { // 条件分支
                    fam.insert(0, "cjk".to_owned()); // 语句
                } // 块结束
                if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Monospace) { // 条件分支
                    fam.push("cjk".to_owned()); // 语句
                } // 块结束
                ctx.set_fonts(fonts); // 语句
                return; // 返回
            } // 块结束
        } // 块结束
    } // 块结束
} // 块结束
// ---------------------------------------------------------------------------
// 数据模型
// ---------------------------------------------------------------------------
#[derive(Clone, Default)] // 语句
struct HeaderRow { // 结构体 HeaderRow
    key: String, // 字段 key
    value: String, // 字段 value
    disabled: bool, // 字段 disabled
} // 块结束

/// 表单类请求体的一行（form-data / x-www-form-urlencoded 通用）。
#[derive(Clone, Default)] // 语句
struct KVRow { // 结构体 KVRow
    key: String, // 字段 key
    value: String, // 字段 value
    disabled: bool, // 字段 disabled
    /// form-data 行是否为文件类型（true 时 value 存的是本地文件路径）。
    is_file: bool, // 字段 is_file
} // 块结束

struct Tab { // 结构体 Tab
    title: String, // 字段 title
    method: String, // 字段 method
    url: String, // 字段 url
    headers: Vec<HeaderRow>, // 字段 headers
    body_type: String, // 字段 body_type
    body: String, // 字段 body
    /// raw 体的子类型：Text / JSON / XML / HTML / JavaScript
    raw_lang: String, // 字段 raw_lang
    /// x-www-form-urlencoded 的行
    url_rows: Vec<KVRow>, // 字段 url_rows
    /// form-data 的行（支持文本与文件）
    form_rows: Vec<KVRow>, // 字段 form_rows
    /// binary 体的本地文件路径
    file_path: String, // 字段 file_path
    /// GraphQL 查询与变量
    gql_query: String, // 字段 gql_query
    gql_vars: String, // 字段 gql_vars
    params: Vec<KVRow>, // 字段 params
    path_rows: Vec<KVRow>, // 字段 path_rows
    hash: String, // 字段 hash
    auth_kind: String, // 字段 auth_kind
    auth_token: String, // 字段 auth_token
    auth_user: String, // 字段 auth_user
    auth_pass: String, // 字段 auth_pass
    auth_key: String, // 字段 auth_key
    auth_value: String, // 字段 auth_value
    /// OAuth2 参数（grant_type / accessTokenUrl / client_id / client_secret / scope / ...）
    oauth2: HashMap<String, String>, // 字段 oauth2
    /// 其他鉴权（oauth1 / awsv4 / jwt / digest / ntlm）的键值参数
    auth_extra: Vec<KVRow>, // 字段 auth_extra
    /// 当前标签对应的集合项路径（用于保存/读取 examples）
    item_path: Option<Vec<usize>>, // 字段 item_path
    /// 请求级网络设置
    req_timeout: String, // 字段 req_timeout
    follow_redirects: bool, // 字段 follow_redirects
    proxy: String, // 字段 proxy
    insecure: bool, // 字段 insecure
    client_cert: String, // 字段 client_cert
    client_key: String, // 字段 client_key
    req_subtab: usize, // 字段 req_subtab
    resp_subtab: usize, // 字段 resp_subtab
    pretty: bool, // 字段 pretty
    tests: Vec<TestAssertion>, // 字段 tests
    tests_text: String, // 字段 tests_text
    /// Postman 风格前置脚本（pm.*）
    pre_script: String, // 字段 pre_script
    /// Postman 风格测试脚本（pm.test(...)）
    test_script: String, // 字段 test_script
    /// 请求描述（保存到集合 item.description）
    description: String, // 字段 description
    /// 测试脚本运行后的结果
    script_results: Vec<scripting::TestResult>, // 字段 script_results
    response: Option<ResponseRecord>, // 字段 response
    body_view: String, // 字段 body_view
    status: String, // 字段 status
    error: Option<String>, // 字段 error
    inflight: bool, // 字段 inflight
    rx: Option<mpsc::Receiver<(Result<ResponseRecord, String>, Vec<scripting::TestResult>)>>, // 字段 rx
} // 块结束
impl Tab { // 实现
    fn new(n: usize) -> Self { // 函数 new
        // 截图/验收用种子（无副作用；仅当设置 PMRS_GUI_DEMO 时生效）。
        let demo = std::env::var("PMRS_GUI_DEMO").is_ok(); // 变量 demo
        Tab { // 语句
            title: format!("请求 {}", n), // 字段 title
            method: "GET".into(), // 字段 method
            url: String::new(), // 字段 url
            headers: vec![HeaderRow::default()], // 字段 headers
            body_type: std::env::var("PMRS_GUI_BODYTYPE").unwrap_or_else(|_| "none".into()), // 字段 body_type
            body: String::new(), // 字段 body
            raw_lang: "JSON".into(), // 字段 raw_lang
            url_rows: vec![KVRow::default()], // 字段 url_rows
            form_rows: vec![KVRow::default()], // 字段 form_rows
            file_path: String::new(), // 字段 file_path
            gql_query: String::new(), // 字段 gql_query
            gql_vars: String::new(), // 字段 gql_vars
            params: if demo { vec![KVRow::default()] } else { Vec::new() }, // 字段 params
            path_rows: if demo { vec![KVRow::default()] } else { Vec::new() }, // 字段 path_rows
            hash: String::new(), // 字段 hash
            auth_kind: std::env::var("PMRS_GUI_AUTHKIND").unwrap_or_else(|_| "none".into()), // 字段 auth_kind
            auth_token: String::new(), // 字段 auth_token
            auth_user: String::new(), // 字段 auth_user
            auth_pass: String::new(), // 字段 auth_pass
            auth_key: String::new(), // 字段 auth_key
            auth_value: String::new(), // 字段 auth_value
            oauth2: { // 语句
                let mut m = HashMap::new(); // 变量 m
                m.insert("grant_type".to_string(), "client_credentials".to_string()); // 语句
                m // 语句
            }, // 块结束
            auth_extra: Vec::new(), // 字段 auth_extra
            item_path: None, // 字段 item_path
            req_timeout: String::new(), // 字段 req_timeout
            follow_redirects: true, // 字段 follow_redirects
            proxy: String::new(), // 字段 proxy
            insecure: false, // 字段 insecure
            client_cert: String::new(), // 字段 client_cert
            client_key: String::new(), // 字段 client_key
            req_subtab: std::env::var("PMRS_GUI_REQTAB") // 语句
                .ok() // 语句
                .and_then(|s| s.trim().parse().ok()) // 语句
                .filter(|n| *n < 8) // 语句
                .unwrap_or(2), // 语句
            resp_subtab: 0, // 字段 resp_subtab
            // 响应体默认自动格式化：内容是 JSON 时自动美化输出（可勾选/取消「美化」切换）
            pretty: true, // 字段 pretty
            tests: Vec::new(), // 字段 tests
            tests_text: String::new(), // 字段 tests_text
            pre_script: String::new(), // 字段 pre_script
            test_script: String::new(), // 字段 test_script
            description: String::new(), // 字段 description
            script_results: Vec::new(), // 字段 script_results
            response: None, // 字段 response
            body_view: String::new(), // 字段 body_view
            status: "就绪".into(), // 字段 status
            error: None, // 字段 error
            inflight: false, // 字段 inflight
            rx: None, // 字段 rx
        } // 块结束
    } // 块结束
    fn build_request(&self) -> Request { // 函数 build_request
        let mut headers: Vec<Header> = self // 变量 headers
            .headers // 语句
            .iter() // 语句
            .filter(|h| !h.key.trim().is_empty()) // 语句
            .map(|h| Header { // 语句
                key: h.key.clone(), // 字段 key
                value: h.value.clone(), // 字段 value
                disabled: h.disabled, // 字段 disabled
                ..Default::default() // 语句
            }) // 语句
            .collect(); // 语句
        match self.auth_kind.as_str() { // 分支匹配
            "bearer" => { // 匹配分支
                if !self.auth_token.trim().is_empty() { // 条件分支
                    headers.push(Header { // 语句
                        key: "Authorization".into(), // 字段 key
                        value: format!("Bearer {}", self.auth_token.trim()), // 字段 value
                        ..Default::default() // 语句
                    }); // 语句
                } // 块结束
            } // 块结束
            "basic" => { // 匹配分支
                if !self.auth_user.is_empty() || !self.auth_pass.is_empty() { // 条件分支
                    let raw = format!("{}:{}", self.auth_user, self.auth_pass); // 变量 raw
                    let enc = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes()); // 变量 enc
                    headers.push(Header { // 语句
                        key: "Authorization".into(), // 字段 key
                        value: format!("Basic {}", enc), // 字段 value
                        ..Default::default() // 语句
                    }); // 语句
                } // 块结束
            } // 块结束
            "apikey" => { // 匹配分支
                if !self.auth_key.trim().is_empty() { // 条件分支
                    headers.push(Header { // 语句
                        key: self.auth_key.trim().to_string(), // 字段 key
                        value: self.auth_value.clone(), // 字段 value
                        ..Default::default() // 语句
                    }); // 语句
                } // 块结束
            } // 块结束
            _ => {} // 匹配分支
        } // 块结束
        let body = match self.body_type.as_str() { // 变量 body
            "" | "none" => None, // 匹配分支
            "raw" => { // 匹配分支
                if self.body.trim().is_empty() { // 条件分支
                    None // 语句
                } else { // 语句
                    Some(Body { // 调用 Some
                        mode: "raw".into(), // 字段 mode
                        raw: Some(self.body.clone()), // 字段 raw
                        options: Some(serde_json::json!({ // 语句
                            "raw": { "language": raw_lang_key(&self.raw_lang) } // 语句
                        })), // 语句
                        ..Default::default() // 语句
                    }) // 语句
                } // 块结束
            } // 块结束
            "x-www-form-urlencoded" => { // 匹配分支
                let rows: Vec<KV> = self // 变量 rows
                    .url_rows // 语句
                    .iter() // 语句
                    .filter(|r| !r.key.trim().is_empty()) // 语句
                    .map(|r| KV { // 语句
                        key: r.key.clone(), // 字段 key
                        value: r.value.clone(), // 字段 value
                        disabled: r.disabled, // 字段 disabled
                        ..Default::default() // 语句
                    }) // 语句
                    .collect(); // 语句
                if rows.is_empty() { // 条件分支
                    None // 语句
                } else { // 语句
                    Some(Body { // 调用 Some
                        mode: "urlencoded".into(), // 字段 mode
                        urlencoded: rows, // 字段 urlencoded
                        ..Default::default() // 语句
                    }) // 语句
                } // 块结束
            } // 块结束
            "form-data" => { // 匹配分支
                let rows: Vec<KV> = self // 变量 rows
                    .form_rows // 语句
                    .iter() // 语句
                    .filter(|r| !r.key.trim().is_empty()) // 语句
                    .map(|r| KV { // 语句
                        key: r.key.clone(), // 字段 key
                        value: if r.is_file { // 语句
                            String::new() // 调用 new
                        } else { // 语句
                            r.value.clone() // 语句
                        }, // 块结束
                        typ: Some(if r.is_file { "file".into() } else { "text".into() }), // 字段 typ
                        src: if r.is_file { // 语句
                            Some(serde_json::Value::String(r.value.clone())) // 调用 Some
                        } else { // 语句
                            None // 语句
                        }, // 块结束
                        disabled: r.disabled, // 字段 disabled
                    }) // 语句
                    .collect(); // 语句
                if rows.is_empty() { // 条件分支
                    None // 语句
                } else { // 语句
                    Some(Body { // 调用 Some
                        mode: "formdata".into(), // 字段 mode
                        formdata: rows, // 字段 formdata
                        ..Default::default() // 语句
                    }) // 语句
                } // 块结束
            } // 块结束
            "binary" => { // 匹配分支
                if self.file_path.trim().is_empty() { // 条件分支
                    None // 语句
                } else { // 语句
                    Some(Body { // 调用 Some
                        mode: "file".into(), // 字段 mode
                        file: Some(FileRef { // 语句
                            src: Some(self.file_path.clone()), // 字段 src
                            content: None, // 字段 content
                        }), // 语句
                        ..Default::default() // 语句
                    }) // 语句
                } // 块结束
            } // 块结束
            "graphql" => Some(Body { // 匹配分支
                mode: "graphql".into(), // 字段 mode
                graphql: Some(GraphQL { // 语句
                    query: self.gql_query.clone(), // 字段 query
                    variables: self.gql_vars.clone(), // 字段 variables
                }), // 语句
                ..Default::default() // 语句
            }), // 语句
            _ => None, // 匹配分支
        }; // 块结束
        Request { // 语句
            method: self.method.clone(), // 字段 method
            url: UrlSpec { // 语句
                raw: self.url.clone(), // 字段 raw
                query: self // 语句
                    .params // 语句
                    .iter() // 语句
                    .filter(|r| !r.key.trim().is_empty()) // 语句
                    .map(|r| KV { // 语句
                        key: r.key.clone(), // 字段 key
                        value: r.value.clone(), // 字段 value
                        disabled: r.disabled, // 字段 disabled
                        ..Default::default() // 语句
                    }) // 语句
                    .collect(), // 语句
                variable: self // 语句
                    .path_rows // 语句
                    .iter() // 语句
                    .filter(|r| !r.key.trim().is_empty()) // 语句
                    .map(|r| KV { // 语句
                        key: r.key.clone(), // 字段 key
                        value: r.value.clone(), // 字段 value
                        disabled: r.disabled, // 字段 disabled
                        ..Default::default() // 语句
                    }) // 语句
                    .collect(), // 语句
                hash: if self.hash.trim().is_empty() { // 语句
                    None // 语句
                } else { // 语句
                    Some(self.hash.trim().to_string()) // 调用 Some
                }, // 块结束
                ..Default::default() // 语句
            }, // 块结束
            header: headers, // 字段 header
            body, // 语句
            auth: self.build_auth(), // 字段 auth
            ..Default::default() // 语句
        } // 块结束
    } // 块结束
    /// 构造 Postman 兼容的 Auth 对象（目前仅 oauth2 需要交给引擎处理取 token；
    /// bearer/basic/apikey 已在 header 中直接生成）。
    fn build_auth(&self) -> Option<postman_rs::model::Auth> { // 函数 build_auth
        if self.auth_kind == "oauth2" { // 条件分支
            let mut arr = Vec::new(); // 变量 arr
            for (k, v) in &self.oauth2 { // 循环
                if !v.trim().is_empty() { // 条件分支
                    arr.push(serde_json::json!({ "key": k, "value": v })); // 语句
                } // 块结束
            } // 块结束
            let mut extra = serde_json::Map::new(); // 变量 extra
            extra.insert("oauth2".into(), serde_json::Value::Array(arr)); // 语句
            return Some(postman_rs::model::Auth { // 返回
                kind: "oauth2".into(), // 字段 kind
                extra, // 语句
                ..Default::default() // 语句
            }); // 语句
        } // 块结束
        if matches!( // 条件分支
            self.auth_kind.as_str(), // 语句
            "oauth1" | "awsv4" | "jwt" | "digest" | "ntlm" // 语句
        ) { // 语句
            let arr: Vec<serde_json::Value> = self // 变量 arr
                .auth_extra // 语句
                .iter() // 语句
                .filter(|r| !r.key.trim().is_empty()) // 语句
                .map(|r| serde_json::json!({ "key": r.key, "value": r.value })) // 语句
                .collect(); // 语句
            let mut extra = serde_json::Map::new(); // 变量 extra
            extra.insert(self.auth_kind.clone(), serde_json::Value::Array(arr)); // 语句
            return Some(postman_rs::model::Auth { // 返回
                kind: self.auth_kind.clone(), // 字段 kind
                extra, // 语句
                ..Default::default() // 语句
            }); // 语句
        } // 块结束
        None // 语句
    } // 块结束
    fn poll(&mut self) -> bool { // 函数 poll
        if let Some(rx) = &self.rx { // 条件分支
            match rx.try_recv() { // 分支匹配
                Ok((Ok(resp), results)) => { // 匹配分支
                    self.status = format!("{} {} ({} ms)", resp.code, resp.status, resp.elapsed_ms); // 语句
                    self.body_view = resp.body.clone(); // 语句
                    self.response = Some(resp); // 语句
                    self.script_results = results; // 语句
                    self.inflight = false; // 语句
                    self.rx = None; // 语句
                    return true; // 返回
                } // 块结束
                Ok((Err(e), _)) => { // 匹配分支
                    self.error = Some(e); // 语句
                    self.status = "请求失败".into(); // 语句
                    self.script_results.clear(); // 语句
                    self.inflight = false; // 语句
                    self.rx = None; // 语句
                } // 块结束
                Err(mpsc::TryRecvError::Empty) => {} // 匹配分支
                Err(mpsc::TryRecvError::Disconnected) => { // 匹配分支
                    self.inflight = false; // 语句
                    self.rx = None; // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        false // 语句
    } // 块结束
} // 块结束
struct GuiApp { // 结构体 GuiApp
    tabs: Vec<Tab>, // 字段 tabs
    active: usize, // 字段 active
    coll_files: Vec<PathBuf>, // 字段 coll_files
    coll_sel: usize, // 字段 coll_sel
    requests: Vec<(String, Request, Option<String>)>, // 字段 requests
    tree: Option<Collection>, // 字段 tree
    tree_file: Option<PathBuf>, // 字段 tree_file
    coll_new_name: String, // 字段 coll_new_name
    folder_name: String, // 字段 folder_name
    rename_path: Option<Vec<usize>>, // 字段 rename_path
    rename_text: String, // 字段 rename_text
    env_files: Vec<PathBuf>, // 字段 env_files
    env_sel: usize, // 字段 env_sel
    history: Vec<storage::HistoryEntry>, // 字段 history
    hist_search: String, // 字段 hist_search
    timeout: u64, // 字段 timeout
    top_search: String, // 字段 top_search
    side_search: String, // 字段 side_search
    sidebar_section: usize, // 字段 sidebar_section
    save_coll_name: String, // 字段 save_coll_name
    save_req_name: String, // 字段 save_req_name
    export_path: String, // 字段 export_path
    message: String, // 字段 message
    theme: usize, // 字段 theme
    /// 左侧栏宽度（像素）。用固定宽度绘制，避免 SidePanel 记忆内容溢出导致的无限增长。
    side_w: f32, // 字段 side_w
    follow_system: bool, // 字段 follow_system
    accent_hex: String, // 字段 accent_hex
    env_edit: Vec<(String, String)>, // 字段 env_edit
    env_loaded_for: String, // 字段 env_loaded_for
    import_path: String, // 字段 import_path
    import_format: String, // 字段 import_format
    /// 从 cURL 粘贴导入的输入内容
    curl_input: String, // 字段 curl_input
    run_iterations: u64, // 字段 run_iterations
    run_result: String, // 字段 run_result
    run_bail: bool, // 字段 run_bail
    run_rx: Option<mpsc::Receiver<String>>, // 字段 run_rx
    run_inflight: bool, // 字段 run_inflight
    codegen_lang: String, // 字段 codegen_lang
    codegen_out: String, // 字段 codegen_out
    // WebSocket 协议工作台
    ws_url: String, // 字段 ws_url
    ws_headers: String, // 字段 ws_headers
    ws_subprotocol: String, // 字段 ws_subprotocol
    ws_send: String, // 字段 ws_send
    ws_listen: u64, // 字段 ws_listen
    ws_wait_ms: u64, // 字段 ws_wait_ms
    ws_binary: bool, // 字段 ws_binary
    ws_log: String, // 字段 ws_log
    ws_busy: bool, // 字段 ws_busy
    ws_rx: Option<mpsc::Receiver<Result<Vec<String>, String>>>, // 字段 ws_rx
    /// 响应"保存到文件"的目标路径（留空则用默认 downloads/）
    resp_save_path: String, // 字段 resp_save_path
    /// 全局变量编辑缓冲
    globals_edit: Vec<(String, String)>, // 字段 globals_edit
    globals_loaded: bool, // 字段 globals_loaded
    /// Cookie 管理器新增输入
    cookie_new_host: String, // 字段 cookie_new_host
    cookie_new_kv: String, // 字段 cookie_new_kv
    /// 控制台日志（请求/响应/脚本）
    console: Vec<String>, // 字段 console
    /// 待确认的删除/破坏性操作（确认弹窗）
    pending_delete: Option<PendingDelete>, // 字段 pending_delete
    /// 新建请求对话框状态
    new_req_dialog: bool, // 字段 new_req_dialog
    new_req_name: String, // 字段 new_req_name
    new_req_method: String, // 字段 new_req_method
    /// cURL 导入弹窗是否打开（由 URL 栏 cURL 按钮的菜单触发）
    curl_import_open: bool, // 字段 curl_import_open
    /// 正在重命名的请求标签索引
    renaming_tab: Option<usize>, // 字段 renaming_tab
    /// 标签重命名输入框内容
    rename_tab_text: String, // 字段 rename_tab_text
    /// 标签重命名输入框是否需要自动聚焦（一次性）
    rename_tab_focus: bool, // 字段 rename_tab_focus
} // 块结束

/// 需要二次确认的删除/破坏性操作。
#[derive(Clone)] // 语句
enum PendingDelete { // 枚举 PendingDelete
    Collection { name: String }, // 语句
    TreeItem { path: Vec<usize>, name: String }, // 语句
    Tab { idx: usize, title: String }, // 语句
    EnvVar { idx: usize, key: String }, // 语句
    GlobalVar { idx: usize, key: String }, // 语句
} // 块结束

impl PendingDelete { // 实现
    fn description(&self) -> String { // 函数 description
        match self { // 分支匹配
            PendingDelete::Collection { name } => format!("确定删除集合「{}」？此操作不可恢复。", name), // 匹配分支
            PendingDelete::TreeItem { name, .. } => { // 匹配分支
                format!("确定删除请求/文件夹「{}」？此操作不可恢复。", name) // 语句
            } // 块结束
            PendingDelete::Tab { title, .. } => format!("确定关闭标签「{}」？未保存的改动将丢失。", title), // 匹配分支
            PendingDelete::EnvVar { key, .. } => format!( // 匹配分支
                "确定删除环境变量「{}」？确认后记得点“保存环境”。", // 语句
                if key.trim().is_empty() { "(空键)" } else { key } // 条件分支
            ), // 语句
            PendingDelete::GlobalVar { key, .. } => format!( // 匹配分支
                "确定删除全局变量「{}」？确认后记得点“保存全局变量”。", // 语句
                if key.trim().is_empty() { "(空键)" } else { key } // 条件分支
            ), // 语句
        } // 块结束
    } // 块结束
} // 块结束

impl GuiApp { // 实现
    fn new(cc: &eframe::CreationContext<'_>) -> Self { // 函数 new
        install_fonts(&cc.egui_ctx); // 调用 install_fonts
        let theme_idx = initial_theme(); // 变量 theme_idx
        apply_theme(&cc.egui_ctx, theme_idx); // 调用 apply_theme
        let mut app = GuiApp { // 变量 app
            tabs: vec![Tab::new(1)], // 字段 tabs
            active: 0, // 字段 active
            coll_files: Vec::new(), // 字段 coll_files
            coll_sel: 0, // 字段 coll_sel
            requests: Vec::new(), // 字段 requests
            tree: None, // 字段 tree
            tree_file: None, // 字段 tree_file
            coll_new_name: String::new(), // 字段 coll_new_name
            folder_name: String::new(), // 字段 folder_name
            rename_path: None, // 字段 rename_path
            rename_text: String::new(), // 字段 rename_text
            env_files: Vec::new(), // 字段 env_files
            env_sel: 0, // 字段 env_sel
            history: Vec::new(), // 字段 history
            hist_search: String::new(), // 字段 hist_search
            timeout: 30, // 字段 timeout
            top_search: String::new(), // 字段 top_search
            side_search: String::new(), // 字段 side_search
            sidebar_section: std::env::var("PMRS_GUI_SIDEBAR") // 语句
                .ok() // 语句
                .and_then(|s| s.trim().parse().ok()) // 语句
                .filter(|n| *n <= 5) // 语句
                .unwrap_or(0), // 语句
            save_coll_name: String::new(), // 字段 save_coll_name
            save_req_name: String::new(), // 字段 save_req_name
            export_path: String::new(), // 字段 export_path
            message: String::new(), // 字段 message
            theme: theme_idx, // 字段 theme
            side_w: 320.0, // 字段 side_w
            follow_system: false, // 字段 follow_system
            accent_hex: String::new(), // 字段 accent_hex
            env_edit: Vec::new(), // 字段 env_edit
            env_loaded_for: String::new(), // 字段 env_loaded_for
            import_path: String::new(), // 字段 import_path
            import_format: "postman".into(), // 字段 import_format
            curl_input: String::new(), // 字段 curl_input
            run_iterations: 1, // 字段 run_iterations
            run_result: String::new(), // 字段 run_result
            run_bail: false, // 字段 run_bail
            run_rx: None, // 字段 run_rx
            run_inflight: false, // 字段 run_inflight
            codegen_lang: "curl".into(), // 字段 codegen_lang
            codegen_out: String::new(), // 字段 codegen_out
            ws_url: "wss://echo.websocket.events".into(), // 字段 ws_url
            ws_headers: String::new(), // 字段 ws_headers
            ws_subprotocol: String::new(), // 字段 ws_subprotocol
            ws_send: String::new(), // 字段 ws_send
            ws_listen: 1, // 字段 ws_listen
            ws_wait_ms: 3000, // 字段 ws_wait_ms
            ws_binary: false, // 字段 ws_binary
            ws_log: String::new(), // 字段 ws_log
            ws_busy: false, // 字段 ws_busy
            ws_rx: None, // 字段 ws_rx
            resp_save_path: String::new(), // 字段 resp_save_path
            globals_edit: Vec::new(), // 字段 globals_edit
            globals_loaded: false, // 字段 globals_loaded
            cookie_new_host: String::new(), // 字段 cookie_new_host
            cookie_new_kv: String::new(), // 字段 cookie_new_kv
            console: Vec::new(), // 字段 console
            pending_delete: None, // 字段 pending_delete
            new_req_dialog: false, // 字段 new_req_dialog
            new_req_name: String::new(), // 字段 new_req_name
            new_req_method: "GET".into(), // 字段 new_req_method
            curl_import_open: false, // 字段 curl_import_open
            renaming_tab: None, // 字段 renaming_tab
            rename_tab_text: String::new(), // 字段 rename_tab_text
            rename_tab_focus: false, // 字段 rename_tab_focus
        }; // 块结束
        app.refresh_all(); // 语句
        app // 语句
    } // 块结束
    fn refresh_all(&mut self) { // 函数 refresh_all
        self.coll_files = storage::list_collections().unwrap_or_default(); // 语句
        self.env_files = storage::list_environments().unwrap_or_default(); // 语句
        self.history = storage::read_history().unwrap_or_default(); // 语句
        if self.coll_sel >= self.coll_files.len() { // 条件分支
            self.coll_sel = 0; // 语句
        } // 块结束
        self.load_requests(); // 语句
    } // 块结束
    fn load_requests(&mut self) { // 函数 load_requests
        self.requests.clear(); // 语句
        self.tree = None; // 语句
        self.tree_file = None; // 语句
        if let Some(path) = self.coll_files.get(self.coll_sel) { // 条件分支
            if let Ok(text) = std::fs::read_to_string(path) { // 条件分支
                if let Ok(c) = serde_json::from_str::<Collection>(&text) { // 条件分支
                    flatten(&c.item, "", &mut self.requests); // 调用 flatten
                    self.tree = Some(c); // 语句
                    self.tree_file = Some(path.clone()); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    fn apply_request_to_active(&mut self, idx: usize) { // 函数 apply_request_to_active
        if let Some((name, req, desc)) = self.requests.get(idx).cloned() { // 条件分支
            let tab = &mut self.tabs[self.active]; // 变量 tab
            tab.title = short_title(&name); // 语句
            tab.description = desc.unwrap_or_default(); // 语句
            tab.method = req.method.clone(); // 语句
            tab.url = if req.url.raw.trim().is_empty() { // 语句
                req.url.effective_url() // 语句
            } else { // 语句
                req.url.raw.clone() // 语句
            }; // 块结束
            tab.params = if req.url.query.is_empty() { // 语句
                parse_query(&tab.url) // 调用 parse_query
            } else { // 语句
                req.url // 语句
                    .query // 语句
                    .iter() // 语句
                    .map(|kv| KVRow { // 语句
                        key: kv.key.clone(), // 字段 key
                        value: kv.value.clone(), // 字段 value
                        disabled: kv.disabled, // 字段 disabled
                        is_file: false, // 字段 is_file
                    }) // 语句
                    .collect() // 语句
            }; // 块结束
            tab.hash = req.url.hash.clone().unwrap_or_default(); // 语句
            tab.path_rows = if req.url.variable.is_empty() { // 语句
                detect_path_vars(&tab.url) // 调用 detect_path_vars
            } else { // 语句
                req.url // 语句
                    .variable // 语句
                    .iter() // 语句
                    .map(|kv| KVRow { // 语句
                        key: kv.key.clone(), // 字段 key
                        value: kv.value.clone(), // 字段 value
                        disabled: kv.disabled, // 字段 disabled
                        is_file: false, // 字段 is_file
                    }) // 语句
                    .collect() // 语句
            }; // 块结束
            tab.headers = req // 语句
                .header // 语句
                .iter() // 语句
                .map(|h| HeaderRow { // 语句
                    key: h.key.clone(), // 字段 key
                    value: h.value.clone(), // 字段 value
                    disabled: h.disabled, // 字段 disabled
                }) // 语句
                .collect(); // 语句
            if tab.headers.is_empty() { // 条件分支
                tab.headers.push(HeaderRow::default()); // 语句
            } // 块结束
            // 还原授权信息
            tab.auth_kind = "none".into(); // 语句
            tab.auth_token.clear(); // 语句
            tab.auth_user.clear(); // 语句
            tab.auth_pass.clear(); // 语句
            tab.auth_key.clear(); // 语句
            tab.auth_value.clear(); // 语句
            tab.oauth2.clear(); // 语句
            tab.oauth2 // 语句
                .insert("grant_type".into(), "client_credentials".into()); // 语句
            tab.auth_extra.clear(); // 语句
            if let Some(a) = &req.auth { // 条件分支
                match a.kind.as_str() { // 分支匹配
                    "bearer" => { // 匹配分支
                        tab.auth_kind = "bearer".into(); // 语句
                        tab.auth_token = a.bearer_token().unwrap_or("").to_string(); // 语句
                    } // 块结束
                    "basic" => { // 匹配分支
                        tab.auth_kind = "basic".into(); // 语句
                        tab.auth_user = a.basic_user().unwrap_or("").to_string(); // 语句
                        tab.auth_pass = a.basic_pass().unwrap_or("").to_string(); // 语句
                    } // 块结束
                    "apikey" => { // 匹配分支
                        tab.auth_kind = "apikey".into(); // 语句
                        tab.auth_key = a.apikey_key().unwrap_or("").to_string(); // 语句
                        tab.auth_value = a.apikey_value().unwrap_or("").to_string(); // 语句
                    } // 块结束
                    "oauth2" => { // 匹配分支
                        tab.auth_kind = "oauth2".into(); // 语句
                        for (k, v) in a.oauth2_params() { // 循环
                            tab.oauth2.insert(k, v); // 语句
                        } // 块结束
                    } // 块结束
                    k @ ("oauth1" | "awsv4" | "jwt" | "digest" | "ntlm") => { // 匹配分支
                        tab.auth_kind = k.to_string(); // 语句
                        if let Some(serde_json::Value::Array(arr)) = a.extra.get(k) { // 条件分支
                            for it in arr { // 循环
                                let key = it // 变量 key
                                    .get("key") // 语句
                                    .and_then(|x| x.as_str()) // 语句
                                    .unwrap_or("") // 语句
                                    .to_string(); // 语句
                                let value = it // 变量 value
                                    .get("value") // 语句
                                    .map(|v| { // 语句
                                        v.as_str() // 语句
                                            .map(|s| s.to_string()) // 语句
                                            .unwrap_or_else(|| v.to_string()) // 语句
                                    }) // 语句
                                    .unwrap_or_default(); // 语句
                                tab.auth_extra.push(KVRow { // 语句
                                    key, // 语句
                                    value, // 语句
                                    ..Default::default() // 语句
                                }); // 语句
                            } // 块结束
                        } // 块结束
                    } // 块结束
                    _ => {} // 匹配分支
                } // 块结束
            } // 块结束
            tab.body.clear(); // 语句
            tab.url_rows = vec![KVRow::default()]; // 语句
            tab.form_rows = vec![KVRow::default()]; // 语句
            tab.file_path.clear(); // 语句
            tab.gql_query.clear(); // 语句
            tab.gql_vars.clear(); // 语句
            tab.body_type = "none".into(); // 语句
            if let Some(b) = &req.body { // 条件分支
                match b.mode.as_str() { // 分支匹配
                    "raw" => { // 匹配分支
                        tab.body_type = "raw".into(); // 语句
                        tab.body = b.raw.clone().unwrap_or_default(); // 语句
                        let lang = b // 变量 lang
                            .options // 语句
                            .as_ref() // 语句
                            .and_then(|o| o.get("raw")) // 语句
                            .and_then(|r| r.get("language")) // 语句
                            .and_then(|l| l.as_str()) // 语句
                            .unwrap_or("json"); // 语句
                        tab.raw_lang = raw_lang_label(lang).to_string(); // 语句
                    } // 块结束
                    "urlencoded" => { // 匹配分支
                        tab.body_type = "x-www-form-urlencoded".into(); // 语句
                        tab.url_rows = b // 语句
                            .urlencoded // 语句
                            .iter() // 语句
                            .map(|kv| KVRow { // 语句
                                key: kv.key.clone(), // 字段 key
                                value: kv.value.clone(), // 字段 value
                                disabled: kv.disabled, // 字段 disabled
                                is_file: false, // 字段 is_file
                            }) // 语句
                            .collect(); // 语句
                        if tab.url_rows.is_empty() { // 条件分支
                            tab.url_rows.push(KVRow::default()); // 语句
                        } // 块结束
                    } // 块结束
                    "formdata" => { // 匹配分支
                        tab.body_type = "form-data".into(); // 语句
                        tab.form_rows = b // 语句
                            .formdata // 语句
                            .iter() // 语句
                            .map(|kv| { // 语句
                                let is_file = kv // 变量 is_file
                                    .typ // 语句
                                    .as_deref() // 语句
                                    .map(|t| t.eq_ignore_ascii_case("file")) // 语句
                                    .unwrap_or(false); // 语句
                                let value = if is_file { // 变量 value
                                    kv.src // 语句
                                        .as_ref() // 语句
                                        .and_then(|v| v.as_str()) // 语句
                                        .unwrap_or("") // 语句
                                        .to_string() // 语句
                                } else { // 语句
                                    kv.value.clone() // 语句
                                }; // 块结束
                                KVRow { // 语句
                                    key: kv.key.clone(), // 字段 key
                                    value, // 语句
                                    disabled: kv.disabled, // 字段 disabled
                                    is_file, // 语句
                                } // 块结束
                            }) // 语句
                            .collect(); // 语句
                        if tab.form_rows.is_empty() { // 条件分支
                            tab.form_rows.push(KVRow::default()); // 语句
                        } // 块结束
                    } // 块结束
                    "file" => { // 匹配分支
                        tab.body_type = "binary".into(); // 语句
                        tab.file_path = b // 语句
                            .file // 语句
                            .as_ref() // 语句
                            .and_then(|f| f.src.clone()) // 语句
                            .unwrap_or_default(); // 语句
                    } // 块结束
                    "graphql" => { // 匹配分支
                        tab.body_type = "graphql".into(); // 语句
                        if let Some(g) = &b.graphql { // 条件分支
                            tab.gql_query = g.query.clone(); // 语句
                            tab.gql_vars = g.variables.clone(); // 语句
                        } // 块结束
                    } // 块结束
                    _ => {} // 匹配分支
                } // 块结束
            } // 块结束
            tab.tests = req.tests.clone(); // 语句
            tab.tests.extend(tests_engine::extract_from_events(&req.event)); // 语句
            tab.tests_text = serde_json::to_string_pretty(&tab.tests).unwrap_or_default(); // 语句
            self.message = format!("已载入请求：{}", name); // 语句
        } // 块结束
    } // 块结束
    fn send_active(&mut self, ctx: &egui::Context) { // 函数 send_active
        let env_path = self.env_files.get(self.env_sel).cloned(); // 变量 env_path
        let coll_path = self.coll_files.get(self.coll_sel).cloned(); // 变量 coll_path
        let global_timeout = self.timeout; // 变量 global_timeout
        let tab = &mut self.tabs[self.active]; // 变量 tab
        if tab.inflight { // 条件分支
            return; // 返回
        } // 块结束
        let req = tab.build_request(); // 变量 req
        let cfg = http::NetConfig { // 变量 cfg
            timeout_secs: tab // 语句
                .req_timeout // 语句
                .trim() // 语句
                .parse::<u64>() // 语句
                .unwrap_or(global_timeout), // 语句
            connect_timeout_secs: None, // 字段 connect_timeout_secs
            follow_redirects: Some(tab.follow_redirects), // 字段 follow_redirects
            max_redirects: None, // 字段 max_redirects
            proxy: if tab.proxy.trim().is_empty() { // 语句
                None // 语句
            } else { // 语句
                Some(tab.proxy.trim().to_string()) // 调用 Some
            }, // 块结束
            insecure: tab.insecure, // 字段 insecure
            client_cert: if tab.client_cert.trim().is_empty() { // 语句
                None // 语句
            } else { // 语句
                Some(tab.client_cert.trim().to_string()) // 调用 Some
            }, // 块结束
            client_key: if tab.client_key.trim().is_empty() { // 语句
                None // 语句
            } else { // 语句
                Some(tab.client_key.trim().to_string()) // 调用 Some
            }, // 块结束
        }; // 块结束
        let (tx, rx) = mpsc::channel(); // 调用 let
        tab.rx = Some(rx); // 语句
        tab.inflight = true; // 语句
        tab.error = None; // 语句
        tab.script_results.clear(); // 语句
        tab.status = "发送中…".into(); // 语句
        let pre_script = tab.pre_script.clone(); // 变量 pre_script
        let test_script = tab.test_script.clone(); // 变量 test_script
        let item_path = tab.item_path.clone(); // 变量 item_path
        let auth_kind = tab.auth_kind.clone(); // 变量 auth_kind
        let method_s = tab.method.clone(); // 变量 method_s
        let url_s = tab.url.clone(); // 变量 url_s
        self.console.push(format!( // 语句
            "[{}] → {} {}", // 语句
            chrono::Local::now().format("%H:%M:%S"), // 字段 chrono
            method_s, // 语句
            url_s // 语句
        )); // 语句
        let ctx2 = ctx.clone(); // 变量 ctx2
        std::thread::spawn(move || { // 调用 spawn
            let coll = coll_path // 变量 coll
                .and_then(|p| std::fs::read_to_string(p).ok()) // 语句
                .and_then(|t| serde_json::from_str::<Collection>(&t).ok()); // 语句
            let env = env_path // 变量 env
                .and_then(|p| std::fs::read_to_string(p).ok()) // 语句
                .and_then(|t| serde_json::from_str::<Environment>(&t).ok()); // 语句
            let mut vars = ctx_from(coll.as_ref(), env.as_ref()); // 变量 vars
            // 继承：集合/文件夹级 auth（当请求自身未设时）与文件夹级变量、pre-request/test 脚本。
            let mut req = req; // 变量 req
            let mut pre_script = pre_script; // 变量 pre_script
            let mut test_script = test_script; // 变量 test_script
            if let (Some(c), Some(path)) = (coll.as_ref(), item_path.as_ref()) { // 条件分支
                let (inh_auth, inh_vars, inh_events) = collect_inherited(c, path); // 调用 let
                // 仅当标签自身未设置鉴权（未选择或"继承"）时才套用集合/文件夹级鉴权，
                // 避免与标签内已生成的 Authorization 头重复。
                if req.auth.is_none() && (auth_kind == "none" || auth_kind == "inherit") { // 条件分支
                    req.auth = inh_auth; // 语句
                } // 块结束
                if !inh_vars.is_empty() { // 条件分支
                    let idx = 1.min(vars.scopes.len()); // 变量 idx
                    vars.scopes.insert(idx, inh_vars); // 语句
                } // 块结束
                let inh_pre = scripting::scripts_of(&inh_events, "prerequest"); // 变量 inh_pre
                if !inh_pre.trim().is_empty() { // 条件分支
                    pre_script = format!("{}\n{}", inh_pre, pre_script); // 语句
                } // 块结束
                let inh_test = scripting::scripts_of(&inh_events, "test"); // 变量 inh_test
                if !inh_test.trim().is_empty() { // 条件分支
                    test_script = format!("{}\n{}", test_script, inh_test); // 语句
                } // 块结束
            } // 块结束
            // 前置脚本：pm.<scope>.set(...) 注入为本次请求可用的变量
            if !pre_script.trim().is_empty() { // 条件分支
                let senv = scripting::ScriptEnv { // 变量 senv
                    vars: vars.merged(), // 字段 vars
                    iter: HashMap::new(), // 字段 iter
                    method: req.method.clone(), // 字段 method
                    url: req.url.effective_url(), // 字段 url
                }; // 块结束
                let out = scripting::run_env(&pre_script, None, &senv); // 变量 out
                if !out.sets.is_empty() { // 条件分支
                    let mut map = HashMap::new(); // 变量 map
                    for (_scope, k, v) in out.sets { // 循环
                        map.insert(k, v); // 语句
                    } // 块结束
                    vars.push_high(map); // 语句
                } // 块结束
            } // 块结束
            let res = http::execute_cfg(&req, &vars, &cfg).map_err(|e| e.to_string()); // 变量 res
            // 测试脚本：针对响应执行 pm.test(...) 断言
            let results = match (&res, test_script.trim().is_empty()) { // 变量 results
                (Ok(resp), false) => { // 匹配分支
                    let senv = scripting::ScriptEnv { // 变量 senv
                        vars: vars.merged(), // 字段 vars
                        iter: HashMap::new(), // 字段 iter
                        method: req.method.clone(), // 字段 method
                        url: req.url.effective_url(), // 字段 url
                    }; // 块结束
                    scripting::run_env(&test_script, Some(resp), &senv).tests // 调用 run_env
                } // 块结束
                _ => Vec::new(), // 匹配分支
            }; // 块结束
            let _ = tx.send((res, results)); // 变量 _
            ctx2.request_repaint(); // 语句
        }); // 语句
    } // 块结束
    fn save_active(&mut self) { // 函数 save_active
        let req = self.tabs[self.active].build_request(); // 变量 req
        let tab_title = self.tabs[self.active].title.clone(); // 变量 tab_title
        let tab_desc = self.tabs[self.active].description.clone(); // 变量 tab_desc
        let coll_name = { // 变量 coll_name
            let s = self.save_coll_name.trim(); // 变量 s
            if s.is_empty() { // 条件分支
                self.coll_files // 语句
                    .get(self.coll_sel) // 语句
                    .and_then(|p| p.file_stem().map(|x| x.to_string_lossy().to_string())) // 语句
                    .unwrap_or_else(|| "我的集合".to_string()) // 语句
            } else { // 语句
                s.to_string() // 语句
            } // 块结束
        }; // 块结束
        let req_name = { // 变量 req_name
            let s = self.save_req_name.trim(); // 变量 s
            if s.is_empty() { // 条件分支
                tab_title.clone() // 语句
            } else { // 语句
                s.to_string() // 语句
            } // 块结束
        }; // 块结束
        let existing = self // 变量 existing
            .coll_files // 语句
            .iter() // 语句
            .find(|p| p.file_stem().map(|x| x.to_string_lossy().to_string()) == Some(coll_name.clone())) // 语句
            .cloned(); // 语句
        let mut coll = existing // 变量 coll
            .as_ref() // 语句
            .and_then(|p| std::fs::read_to_string(p).ok()) // 语句
            .and_then(|t| serde_json::from_str::<Collection>(&t).ok()) // 语句
            .unwrap_or_else(|| Collection { // 语句
                info: CollectionInfo { // 语句
                    name: coll_name.clone(), // 字段 name
                    postman_id: Some(uuid::Uuid::new_v4().to_string()), // 字段 postman_id
                    schema: Some( // 语句
                        "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into(), // 语句
                    ), // 语句
                    description: None, // 字段 description
                }, // 块结束
                ..Default::default() // 语句
            }); // 语句
        coll.item.push(Item { // 语句
            name: req_name.clone(), // 字段 name
            request: Some(req), // 字段 request
            description: if tab_desc.trim().is_empty() { // 语句
                None // 语句
            } else { // 语句
                Some(serde_json::Value::String(tab_desc)) // 调用 Some
            }, // 块结束
            ..Default::default() // 语句
        }); // 语句
        match storage::save_collection(&coll) { // 分支匹配
            Ok(p) => { // 匹配分支
                self.message = format!("已保存请求「{}」到集合「{}」→ {}", req_name, coll_name, p.display()); // 语句
                self.refresh_all(); // 语句
            } // 块结束
            Err(e) => self.message = format!("保存失败：{}", e), // 匹配分支
        } // 块结束
    } // 块结束
    /// 把当前响应体写入文件。路径留空时保存到 ~/.postman-rs/downloads/。
    fn save_response_to_file(&mut self, body: &str) { // 函数 save_response_to_file
        let path: PathBuf = if self.resp_save_path.trim().is_empty() { // 变量 path
            let dir = storage::home().join("downloads"); // 变量 dir
            let _ = std::fs::create_dir_all(&dir); // 变量 _
            dir.join(format!( // 语句
                "response_{}.txt", // 语句
                chrono::Local::now().format("%Y%m%d_%H%M%S") // 调用 now
            )) // 语句
        } else { // 语句
            PathBuf::from(self.resp_save_path.trim()) // 调用 from
        }; // 块结束
        match std::fs::write(&path, body) { // 分支匹配
            Ok(_) => self.message = format!("响应已保存 → {}", path.display()), // 匹配分支
            Err(e) => self.message = format!("保存响应失败：{}", e), // 匹配分支
        } // 块结束
    } // 块结束
    fn export_selected(&mut self) { // 函数 export_selected
        let path = match self.coll_files.get(self.coll_sel) { // 变量 path
            Some(p) => p.clone(), // 匹配分支
            None => { // 匹配分支
                self.message = "没有可导出的集合".into(); // 语句
                return; // 返回
            } // 块结束
        }; // 块结束
        let mut coll = match std::fs::read_to_string(&path) // 变量 coll
            .ok() // 语句
            .and_then(|t| serde_json::from_str::<Collection>(&t).ok()) // 语句
        { // 语句
            Some(c) => c, // 匹配分支
            None => { // 匹配分支
                self.message = "读取集合失败".into(); // 语句
                return; // 返回
            } // 块结束
        }; // 块结束
        let out = if self.export_path.trim().is_empty() { // 变量 out
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(); // 变量 stem
            let dir = storage::home().join("exports"); // 变量 dir
            let _ = std::fs::create_dir_all(&dir); // 变量 _
            dir.join(format!("{}.json", stem)) // 语句
        } else { // 语句
            PathBuf::from(self.export_path.trim()) // 调用 from
        }; // 块结束
        match import_export::export_collection(&mut coll) { // 分支匹配
            Ok(text) => match std::fs::write(&out, text) { // 匹配分支
                Ok(_) => self.message = format!("已导出 → {}", out.display()), // 匹配分支
                Err(e) => self.message = format!("写入失败：{}", e), // 匹配分支
            }, // 块结束
            Err(e) => self.message = format!("导出失败：{}", e), // 匹配分支
        } // 块结束
    } // 块结束
    fn import_now(&mut self) { // 函数 import_now
        let path = self.import_path.trim().to_string(); // 变量 path
        if path.is_empty() { // 条件分支
            self.message = "请输入要导入的文件路径".into(); // 语句
            return; // 返回
        } // 块结束
        let fmt = self.import_format.clone(); // 变量 fmt
        match import_export::import_file(&path, &fmt) { // 分支匹配
            Ok(import_export::Imported::Collection(c_box)) => { // 匹配分支
                let mut c = *c_box; // 变量 c
                let name = c.info.name.clone(); // 变量 name
                let existing = self // 变量 existing
                    .coll_files // 语句
                    .iter() // 语句
                    .find(|p| p.file_stem().map(|x| x.to_string_lossy().to_string()) == Some(name.clone())) // 语句
                    .cloned(); // 语句
                if let Some(ep) = existing { // 条件分支
                    if let Ok(t) = std::fs::read_to_string(&ep) { // 条件分支
                        if let Ok(mut base) = serde_json::from_str::<Collection>(&t) { // 条件分支
                            import_export::merge_collections(&mut base, c); // 调用 merge_collections
                            c = base; // 语句
                            self.message = format!("已合并导入集合「{}」", name); // 语句
                        } // 块结束
                    } // 块结束
                } else { // 语句
                    self.message = format!("已导入集合「{}」", name); // 语句
                } // 块结束
                match storage::save_collection(&c) { // 分支匹配
                    Ok(p) => { // 匹配分支
                        self.message.push_str(&format!(" → {}", p.display())); // 语句
                        self.refresh_all(); // 语句
                    } // 块结束
                    Err(e) => self.message = format!("保存失败：{}", e), // 匹配分支
                } // 块结束
            } // 块结束
            Ok(import_export::Imported::Environment(e)) => { // 匹配分支
                let name = e.name.clone(); // 变量 name
                match storage::save_environment(&e) { // 分支匹配
                    Ok(p) => { // 匹配分支
                        self.message = format!("已导入环境「{}」 → {}", name, p.display()); // 语句
                        self.refresh_all(); // 语句
                    } // 块结束
                    Err(err) => self.message = format!("保存环境失败：{}", err), // 匹配分支
                } // 块结束
            } // 块结束
            Err(e) => self.message = format!("导入失败：{}", e), // 匹配分支
        } // 块结束
    } // 块结束
    fn run_current_collection(&mut self) { // 函数 run_current_collection
        if self.run_inflight { // 条件分支
            return; // 返回
        } // 块结束
        let path = match self.coll_files.get(self.coll_sel) { // 变量 path
            Some(p) => p.clone(), // 匹配分支
            None => { // 匹配分支
                self.message = "没有可运行的集合".into(); // 语句
                return; // 返回
            } // 块结束
        }; // 块结束
        let env_path = self.env_files.get(self.env_sel).cloned(); // 变量 env_path
        let iterations = self.run_iterations.max(1) as usize; // 变量 iterations
        let bail = self.run_bail; // 变量 bail
        let timeout = self.timeout; // 变量 timeout
        let (tx, rx) = mpsc::channel(); // 调用 let
        self.run_rx = Some(rx); // 语句
        self.run_inflight = true; // 语句
        self.run_result = "运行中…".into(); // 语句
        std::thread::spawn(move || { // 调用 spawn
            let res = (|| -> Result<String, String> { // 变量 res
                let c: Collection = serde_json::from_str( // 变量 c
                    &std::fs::read_to_string(&path).map_err(|e| e.to_string())?, // 语句
                ) // 块结束
                .map_err(|e| e.to_string())?; // 语句
                let env = env_path // 变量 env
                    .and_then(|p| std::fs::read_to_string(p).ok()) // 语句
                    .and_then(|t| serde_json::from_str::<Environment>(&t).ok()); // 语句
                let summary = runner::run_collection_opts( // 变量 summary
                    &c, // 语句
                    env.as_ref(), // 语句
                    iterations, // 语句
                    Vec::new(), // 字段 Vec
                    0, // 语句
                    timeout, // 语句
                    RunOptions { // 语句
                        folder: None, // 字段 folder
                        bail, // 语句
                        report: None, // 字段 report
                    }, // 块结束
                ); // 块结束
                let mut out = format!( // 变量 out
                    "迭代 {} 次 · 总计 {} / 通过 {} / 失败 {}\n\n", // 语句
                    iterations, summary.total, summary.passed, summary.failed // 语句
                ); // 块结束
                for r in &summary.results { // 循环
                    let code = r.code.map(|c| c.to_string()).unwrap_or_else(|| "-".into()); // 变量 code
                    out.push_str(&format!( // 语句
                        "{} {} {} [{}] {}ms {}\n", // 语句
                        if r.passed { "✓" } else { "✗" }, // 条件分支
                        r.method, // 语句
                        r.url, // 语句
                        code, // 语句
                        r.elapsed_ms, // 语句
                        r.error // 语句
                            .as_ref() // 语句
                            .map(|e| format!("错误：{}", e)) // 语句
                            .unwrap_or_default() // 语句
                    )); // 语句
                    for t in &r.tests { // 循环
                        out.push_str(&format!( // 语句
                            "    {} {}\n", // 语句
                            if t.passed { "· 通过" } else { "· 失败" }, // 条件分支
                            t.name // 语句
                        )); // 语句
                    } // 块结束
                } // 块结束
                Ok(out) // 调用 Ok
            })(); // 语句
            let _ = tx.send(res.unwrap_or_else(|e| format!("运行失败：{}", e))); // 变量 _
        }); // 语句
    } // 块结束
    fn generate_code(&mut self) { // 函数 generate_code
        let req = self.tabs[self.active].build_request(); // 变量 req
        match codegen::generate(&req, &self.codegen_lang) { // 分支匹配
            Ok(code) => self.codegen_out = code, // 匹配分支
            Err(e) => self.codegen_out = format!("生成失败：{}", e), // 匹配分支
        } // 块结束
    } // 块结束
    /// 把当前代码生成结果（如 cURL）保存为文件。
    fn save_codegen_to_file(&mut self) { // 函数 save_codegen_to_file
        let req = self.tabs[self.active].build_request(); // 变量 req
        let code = match codegen::generate(&req, &self.codegen_lang) { // 变量 code
            Ok(c) => c, // 匹配分支
            Err(e) => { // 匹配分支
                self.message = format!("生成失败：{}", e); // 语句
                return; // 返回
            } // 块结束
        }; // 块结束
        self.codegen_out = code.clone(); // 语句
        let dir = storage::home().join("exports"); // 变量 dir
        let _ = std::fs::create_dir_all(&dir); // 变量 _
        let ext = match self.codegen_lang.as_str() { // 变量 ext
            "curl" => "sh", // 匹配分支
            "python" => "py", // 匹配分支
            "js" | "nodejs" => "js", // 匹配分支
            "go" => "go", // 匹配分支
            "rust" => "rs", // 匹配分支
            "java" => "java", // 匹配分支
            "csharp" => "cs", // 匹配分支
            "php" => "php", // 匹配分支
            "ruby" => "rb", // 匹配分支
            "powershell" => "ps1", // 匹配分支
            "swift" => "swift", // 匹配分支
            _ => "txt", // 匹配分支
        }; // 块结束
        let path = dir.join(format!( // 变量 path
            "{}_{}.{}", // 语句
            self.codegen_lang, // 语句
            chrono::Local::now().format("%Y%m%d_%H%M%S"), // 字段 chrono
            ext // 语句
        )); // 语句
        match std::fs::write(&path, code.as_bytes()) { // 分支匹配
            Ok(_) => self.message = format!("已导出 {} → {}", self.codegen_lang, path.display()), // 匹配分支
            Err(e) => self.message = format!("导出失败：{}", e), // 匹配分支
        } // 块结束
    } // 块结束
    // ---- 顶部栏 -----------------------------------------------------------
    /// 按方法给出默认请求头（创建请求时使用）。
    fn default_headers_for(method: &str) -> Vec<HeaderRow> { // 函数 default_headers_for
        let m = method.to_uppercase(); // 变量 m
        let mut rows = Vec::new(); // 变量 rows
        if matches!(m.as_str(), "POST" | "PUT" | "PATCH" | "DELETE") { // 条件分支
            rows.push(HeaderRow { // 语句
                key: "Content-Type".into(), // 字段 key
                value: "application/json".into(), // 字段 value
                disabled: false, // 字段 disabled
            }); // 语句
        } // 块结束
        rows.push(HeaderRow { // 语句
            key: "Accept".into(), // 字段 key
            value: "application/json".into(), // 字段 value
            disabled: false, // 字段 disabled
        }); // 语句
        rows.push(HeaderRow::default()); // 语句
        rows // 语句
    } // 块结束

    /// 打开“新建请求”对话框。
    fn open_new_request_dialog(&mut self) { // 函数 open_new_request_dialog
        let n = self.tabs.len() + 1; // 变量 n
        self.new_req_name = format!("新建请求 {}", n); // 语句
        self.new_req_method = "GET".into(); // 语句
        self.new_req_dialog = true; // 语句
    } // 块结束

    /// 按对话框内容创建新请求标签。
    fn create_request_from_dialog(&mut self) { // 函数 create_request_from_dialog
        let mut tab = Tab::new(self.tabs.len() + 1); // 变量 tab
        let name = self.new_req_name.trim().to_string(); // 变量 name
        if !name.is_empty() { // 条件分支
            tab.title = name; // 语句
        } // 块结束
        let method = self.new_req_method.clone(); // 变量 method
        tab.method = method.clone(); // 语句
        tab.headers = Self::default_headers_for(&method); // 语句
        if matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") { // 条件分支
            tab.body_type = "json".into(); // 语句
        } // 块结束
        self.tabs.push(tab); // 语句
        self.active = self.tabs.len() - 1; // 语句
        self.new_req_dialog = false; // 语句
    } // 块结束

    /// 新建请求对话框。
    fn show_new_request_dialog(&mut self, ctx: &egui::Context) { // 函数 show_new_request_dialog
        if !self.new_req_dialog { // 条件分支
            return; // 返回
        } // 块结束
        let mut create = false; // 变量 create
        let mut cancel = false; // 变量 cancel
        egui::Window::new("新建请求") // 调用 new
            .order(egui::Order::Foreground) // 语句
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // 语句
            .collapsible(false) // 语句
            .resizable(false) // 语句
            .show(ctx, |ui| { // 语句
                ui.set_min_width(360.0); // 语句
                ui.add_space(4.0); // 语句
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("名称").color(text_secondary()).size(12.0)); // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut self.new_req_name) // 调用 singleline
                            .desired_width(250.0) // 语句
                            .hint_text("请求名称（可修改）"), // 语句
                    ); // 块结束
                }); // 语句
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("方法").color(text_secondary()).size(12.0)); // 语句
                    egui::ComboBox::from_id_salt("new_req_method") // 调用 from_id_salt
                        .width(120.0) // 语句
                        .selected_text( // 语句
                            RichText::new(&self.new_req_method) // 调用 new
                                .color(method_color(&self.new_req_method)) // 语句
                                .strong(), // 语句
                        ) // 块结束
                        .show_ui(ui, |ui| { // 语句
                            for m in [ // 循环
                                "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "COPY", // 语句
                                "LINK", "UNLINK", "PURGE", "LOCK", "UNLOCK", "PROPFIND", "VIEW", // 语句
                                "TRACE", // 语句
                            ] { // 语句
                                ui.selectable_value( // 语句
                                    &mut self.new_req_method, // 语句
                                    m.to_string(), // 语句
                                    RichText::new(m).color(method_color(m)).strong(), // 字段 RichText
                                ); // 块结束
                            } // 块结束
                        }); // 语句
                    ui.label( // 语句
                        RichText::new("POST/PUT/PATCH 自动带 Content-Type: application/json") // 调用 new
                            .color(text_secondary()) // 语句
                            .size(10.5), // 语句
                    ); // 块结束
                }); // 语句
                ui.add_space(10.0); // 语句
                ui.horizontal(|ui| { // 语句
                    if ui // 条件分支
                        .add(egui::Button::new(RichText::new("取消").color(text_secondary()))) // 语句
                        .clicked() // 语句
                    { // 语句
                        cancel = true; // 语句
                    } // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("创建").color(Color32::WHITE).strong()) // 调用 new
                                .fill(brand()) // 语句
                                .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        create = true; // 语句
                    } // 块结束
                }); // 语句
            }); // 语句
        if create { // 条件分支
            self.create_request_from_dialog(); // 语句
        } else if cancel { // 条件分支
            self.new_req_dialog = false; // 语句
        } // 块结束
    } // 块结束

    /// 「从 cURL 导入（粘贴）」弹窗：粘贴一条 curl 命令，点“解析为新标签”生成请求。
    fn show_curl_import_dialog(&mut self, ctx: &egui::Context) { // 函数 show_curl_import_dialog
        if !self.curl_import_open { // 条件分支
            return; // 返回
        } // 块结束
        let mut parse = false; // 变量 parse
        let mut cancel = false; // 变量 cancel
        egui::Window::new("从 cURL 导入（粘贴）") // 调用 new
            .order(egui::Order::Foreground) // 语句
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // 语句
            .collapsible(false) // 语句
            .resizable(true) // 语句
            .default_width(560.0) // 语句
            .show(ctx, |ui| { // 语句
                ui.set_min_width(520.0); // 语句
                ui.add_space(2.0); // 语句
                ui.label( // 语句
                    RichText::new("粘贴 curl 命令，例如：curl -X POST 'https://api.test/x' -H 'Content-Type: application/json' -d '{\"a\":1}'") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(11.0), // 语句
                ); // 块结束
                ui.add_space(4.0); // 语句
                ui.add( // 语句
                    egui::TextEdit::multiline(&mut self.curl_input) // 调用 multiline
                        .code_editor() // 语句
                        .desired_rows(6) // 语句
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text("在此粘贴整条 curl 命令"), // 语句
                ); // 块结束
                ui.add_space(8.0); // 语句
                ui.horizontal(|ui| { // 语句
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("取消").color(text_secondary())) // 调用 new
                                .min_size(egui::vec2(84.0, 28.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        cancel = true; // 语句
                    } // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new( // 调用 new
                                RichText::new("解析为新标签").color(Color32::WHITE).strong(), // 字段 RichText
                            ) // 块结束
                            .fill(brand()) // 语句
                            .rounding(Rounding::same(6.0)) // 语句
                            .min_size(egui::vec2(110.0, 28.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        parse = true; // 语句
                    } // 块结束
                }); // 语句
            }); // 语句
        if parse { // 条件分支
            self.import_curl_to_tab(); // 语句
            self.curl_import_open = false; // 语句
        } else if cancel { // 条件分支
            self.curl_import_open = false; // 语句
        } // 块结束
    } // 块结束

    fn apply_pending_delete(&mut self, pd: PendingDelete) { // 函数 apply_pending_delete
        match pd { // 分支匹配
            PendingDelete::Collection { .. } => self.delete_collection(), // 匹配分支
            PendingDelete::TreeItem { path, .. } => { // 匹配分支
                if let Some(c) = self.tree.as_mut() { // 条件分支
                    remove_at(&mut c.item, &path); // 调用 remove_at
                } // 块结束
                self.save_tree(); // 语句
            } // 块结束
            PendingDelete::Tab { idx, .. } => { // 匹配分支
                if self.tabs.len() > 1 && idx < self.tabs.len() { // 条件分支
                    self.tabs.remove(idx); // 语句
                    if self.active >= self.tabs.len() { // 条件分支
                        self.active = self.tabs.len() - 1; // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
            PendingDelete::EnvVar { idx, .. } => { // 匹配分支
                if idx < self.env_edit.len() { // 条件分支
                    self.env_edit.remove(idx); // 语句
                } // 块结束
            } // 块结束
            PendingDelete::GlobalVar { idx, .. } => { // 匹配分支
                if idx < self.globals_edit.len() { // 条件分支
                    self.globals_edit.remove(idx); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束

    /// 删除/关闭确认弹窗（含半透明遮罩，拦截背景点击）。
    fn show_confirm_dialog(&mut self, ctx: &egui::Context) { // 函数 show_confirm_dialog
        let Some(pd) = self.pending_delete.clone() else { // 变量 Some
            return; // 返回
        }; // 块结束
        let mut confirm = false; // 变量 confirm
        let mut cancel = false; // 变量 cancel
        let screen = ctx.screen_rect(); // 变量 screen
        egui::Area::new(egui::Id::new("confirm_overlay")) // 调用 new
            .order(egui::Order::Middle) // 语句
            .fixed_pos(screen.min) // 语句
            .show(ctx, |ui| { // 语句
                let full = egui::Rect::from_min_size(screen.min, screen.size()); // 变量 full
                ui.painter() // 语句
                    .rect_filled(full, 0.0, egui::Color32::from_black_alpha(110)); // 语句
                let resp = ui.allocate_rect(full, egui::Sense::click()); // 变量 resp
                if resp.clicked() { // 条件分支
                    cancel = true; // 语句
                } // 块结束
            }); // 语句
        egui::Window::new("确认操作") // 调用 new
            .order(egui::Order::Foreground) // 语句
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0)) // 语句
            .collapsible(false) // 语句
            .resizable(false) // 语句
            .show(ctx, |ui| { // 语句
                ui.set_min_width(320.0); // 语句
                ui.add_space(4.0); // 语句
                ui.label( // 语句
                    RichText::new(pd.description()) // 调用 new
                        .color(text_primary()) // 语句
                        .size(12.5), // 语句
                ); // 块结束
                ui.add_space(10.0); // 语句
                ui.horizontal(|ui| { // 语句
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("取消").color(text_secondary())) // 调用 new
                                .min_size(egui::vec2(84.0, 28.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        cancel = true; // 语句
                    } // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("确定").color(Color32::WHITE).strong()) // 调用 new
                                .fill(Color32::from_rgb(220, 38, 38)) // 语句
                                .rounding(Rounding::same(6.0)) // 语句
                                .min_size(egui::vec2(84.0, 28.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        confirm = true; // 语句
                    } // 块结束
                }); // 语句
            }); // 语句
        if confirm { // 条件分支
            self.apply_pending_delete(pd); // 语句
            self.pending_delete = None; // 语句
        } else if cancel { // 条件分支
            self.pending_delete = None; // 语句
        } // 块结束
    } // 块结束

    fn ui_topbar(&mut self, ctx: &egui::Context) { // 函数 ui_topbar
        let ctx2 = ctx.clone(); // 变量 ctx2
        egui::TopBottomPanel::top("topbar") // 调用 top
            .frame( // 语句
                egui::Frame::none() // 调用 none
                    .fill(bg_panel()) // 语句
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0)), // 语句
            ) // 块结束
            .show(ctx, |ui| { // 语句
                top_accent(ui); // 调用 top_accent
                ui.horizontal(|ui| { // 语句
                    let (rect, _) = // 调用 let
                        ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover()); // 语句
                    ui.painter().rect_filled(rect, Rounding::same(R_MD), brand()); // 语句
                    ui.painter().text( // 语句
                        rect.center(), // 语句
                        egui::Align2::CENTER_CENTER, // 字段 egui
                        "P", // 语句
                        egui::FontId::proportional(17.0), // 字段 egui
                        Color32::WHITE, // 字段 Color32
                    ); // 块结束
                    ui.add_space(8.0); // 语句
                    ui.vertical(|ui| { // 语句
                        ui.label(RichText::new("pmrs").strong().size(17.0).color(text_primary())); // 语句
                        ui.label( // 语句
                            RichText::new("本地 API 客户端") // 调用 new
                                .color(text_secondary()) // 语句
                                .size(10.5), // 语句
                        ); // 块结束
                    }); // 语句
                    ui.add_space(14.0); // 语句
                    ui.separator(); // 语句
                    ui.add_space(6.0); // 语句
                    egui::Frame::none() // 调用 none
                        .fill(mix(bg_panel(), text_primary(), 0.05)) // 语句
                        .rounding(Rounding::same(R_MD)) // 语句
                        .inner_margin(egui::Margin::symmetric(10.0, 5.0)) // 语句
                        .show(ui, |ui| { // 语句
                            ui.horizontal(|ui| { // 语句
                                ui.label( // 语句
                                    RichText::new("搜索") // 调用 new
                                        .color(text_secondary()) // 语句
                                        .size(11.5), // 语句
                                ); // 块结束
                                ui.add( // 语句
                                    egui::TextEdit::singleline(&mut self.top_search) // 调用 singleline
                                        .frame(false) // 语句
                                        .hint_text("集合 / 请求…") // 语句
                                        .desired_width(250.0), // 语句
                                ); // 块结束
                            }); // 语句
                        }); // 语句
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { // 语句
                        if ui // 条件分支
                            .add( // 语句
                                egui::Button::new(RichText::new("保存").color(Color32::WHITE).strong()) // 调用 new
                                    .fill(brand()) // 语句
                                    .rounding(Rounding::same(R_MD)) // 语句
                                    .min_size(egui::vec2(64.0, 30.0)), // 语句
                            ) // 块结束
                            .clicked() // 语句
                        { // 语句
                            self.save_active(); // 语句
                        } // 块结束
                        ui.add_space(10.0); // 语句
                        let mut sel = self.theme; // 变量 sel
                        egui::ComboBox::from_id_salt("theme_sel") // 调用 from_id_salt
                            .width(120.0) // 语句
                            .selected_text( // 语句
                                RichText::new(THEMES[sel.min(THEMES.len() - 1)].name) // 调用 new
                                    .color(text_primary()) // 语句
                                    .size(12.0), // 语句
                            ) // 块结束
                            .show_ui(ui, |ui| { // 语句
                                for i in 0..THEMES.len() { // 循环
                                    ui.selectable_value(&mut sel, i, THEMES[i].name); // 语句
                                } // 块结束
                            }); // 语句
                        if sel != self.theme { // 条件分支
                            self.theme = sel; // 语句
                            self.follow_system = false; // 语句
                            FOLLOW_SYSTEM.store(false, Ordering::Relaxed); // 语句
                            apply_theme(&ctx2, sel); // 调用 apply_theme
                            save_theme(sel); // 调用 save_theme
                        } // 块结束
                        ui.label(RichText::new("主题").color(text_secondary()).size(11.5)); // 语句
                        ui.add_space(10.0); // 语句
                        let cur = self // 变量 cur
                            .env_files // 语句
                            .get(self.env_sel) // 语句
                            .map(file_name) // 语句
                            .unwrap_or_else(|| "无环境".into()); // 语句
                        egui::ComboBox::from_id_salt("env_top") // 调用 from_id_salt
                            .width(140.0) // 语句
                            .selected_text(RichText::new(cur).color(text_primary()).size(12.0)) // 语句
                            .show_ui(ui, |ui| { // 语句
                                for i in 0..self.env_files.len() { // 循环
                                    ui.selectable_value(&mut self.env_sel, i, file_name(&self.env_files[i])); // 语句
                                } // 块结束
                            }); // 语句
                        ui.label(RichText::new("环境").color(text_secondary()).size(11.5)); // 语句
                        ui.add_space(10.0); // 语句
                        ui.add(egui::DragValue::new(&mut self.timeout).range(1..=600).suffix("s")); // 语句
                        ui.label(RichText::new("超时").color(text_secondary()).size(11.5)); // 语句
                        ui.add_space(8.0); // 语句
                        if ui // 条件分支
                            .add( // 语句
                                egui::Button::new(RichText::new("刷新").color(text_secondary()).size(12.0)) // 调用 new
                                    .frame(false), // 语句
                            ) // 块结束
                            .clicked() // 语句
                        { // 语句
                            self.refresh_all(); // 语句
                        } // 块结束
                    }); // 语句
                }); // 语句
                let r = ui.max_rect(); // 变量 r
                ui.painter() // 语句
                    .hline(r.left()..=r.right(), r.bottom(), Stroke::new(1.0_f32, border())); // 语句
            }); // 语句
    } // 块结束
    // ---- 左侧栏 -----------------------------------------------------------
    fn ui_sidebar(&mut self, ctx: &egui::Context) { // 函数 ui_sidebar
        // 使用固定宽度（而非 resizable 的 SidePanel）：后者会把内容 1~2px 的溢出
        // 记忆为新的面板宽度，导致每次重绘/鼠标移动时左栏不断变宽。这里自行维护宽度，
        // 并在右缘绘制一个可拖拽手柄来保留手动调整能力。
        let w = self.side_w.clamp(240.0, 520.0); // 变量 w
        egui::SidePanel::left("side") // 调用 left
            .resizable(false) // 语句
            .exact_width(w) // 语句
            .frame( // 语句
                egui::Frame::none() // 调用 none
                    .fill(bg_side()) // 语句
                    .inner_margin(egui::Margin::same(12.0)), // 语句
            ) // 块结束
            .show(ctx, |ui| { // 语句
                segmented( // 调用 segmented
                    ui, // 语句
                    &mut self.sidebar_section, // 语句
                    &["集合", "环境", "全局", "历史", "工具", "控制台"], // 语句
                ); // 块结束
                ui.add_space(10.0); // 语句
                match self.sidebar_section { // 分支匹配
                    0 => self.ui_side_collections(ui), // 匹配分支
                    1 => self.ui_side_env(ui), // 匹配分支
                    2 => self.ui_side_globals(ui), // 匹配分支
                    3 => self.ui_side_history(ui), // 匹配分支
                    5 => self.ui_side_console(ui), // 匹配分支
                    _ => self.ui_side_tools(ui), // 匹配分支
                } // 块结束
                // 右侧细边框
                let r = ui.max_rect(); // 变量 r
                ui.painter() // 语句
                    .vline(r.right(), r.top()..=r.bottom(), Stroke::new(1.0_f32, border())); // 语句

                // 自绘拖拽手柄（位于面板外缘的 12px 留白内）
                let edge = r.right() + 11.0; // 变量 edge
                let handle = egui::Rect::from_min_max( // 变量 handle
                    egui::pos2(edge - 4.0, r.top()), // 字段 egui
                    egui::pos2(edge + 4.0, r.bottom()), // 字段 egui
                ); // 块结束
                let hr = ui.interact(handle, ui.id().with("side_resize"), egui::Sense::drag()); // 变量 hr
                if hr.hovered() || hr.dragged() { // 条件分支
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal); // 语句
                } // 块结束
                if hr.dragged() { // 条件分支
                    self.side_w = (self.side_w + hr.drag_delta().x).clamp(240.0, 520.0); // 语句
                } // 块结束
            }); // 语句
    } // 块结束
    fn ui_side_collections(&mut self, ui: &mut egui::Ui) { // 函数 ui_side_collections
        section_title(ui, "集合"); // 调用 section_title
        ui.add_space(6.0); // 语句
        let cur = self // 变量 cur
            .coll_files // 语句
            .get(self.coll_sel) // 语句
            .map(file_name) // 语句
            .unwrap_or_else(|| "（无集合）".into()); // 语句
        let mut changed = false; // 变量 changed
        egui::ComboBox::from_id_salt("coll_sel") // 调用 from_id_salt
            .width(ui.available_width()) // 语句
            .selected_text(RichText::new(cur).color(text_primary()).size(12.5)) // 语句
            .show_ui(ui, |ui| { // 语句
                for i in 0..self.coll_files.len() { // 循环
                    if ui // 条件分支
                        .selectable_value(&mut self.coll_sel, i, file_name(&self.coll_files[i])) // 语句
                        .clicked() // 语句
                    { // 语句
                        changed = true; // 语句
                    } // 块结束
                } // 块结束
            }); // 语句
        if changed { // 条件分支
            self.load_requests(); // 语句
        } // 块结束
        // 新建集合
        ui.add_space(4.0); // 语句
        ui.horizontal_wrapped(|ui| { // 语句
            let w = (ui.available_width() - 76.0).max(60.0); // 变量 w
            ui.add( // 语句
                egui::TextEdit::singleline(&mut self.coll_new_name) // 调用 singleline
                    .hint_text("新集合名称") // 语句
                    .desired_width(w), // 语句
            ); // 块结束
            if ui // 条件分支
                .add( // 语句
                    egui::Button::new(RichText::new("＋集合").color(brand())) // 调用 new
                        .fill(brand_soft()) // 语句
                        .rounding(Rounding::same(6.0)), // 语句
                ) // 块结束
                .clicked() // 语句
            { // 语句
                self.create_collection(); // 语句
            } // 块结束
        }); // 语句
        if ui // 条件分支
            .add( // 语句
                egui::Button::new( // 调用 new
                    RichText::new("删除当前集合").color(Color32::from_rgb(235, 32, 19)), // 字段 RichText
                ) // 块结束
                .frame(false), // 语句
            ) // 块结束
            .clicked() // 语句
        { // 语句
            let name = self // 变量 name
                .coll_files // 语句
                .get(self.coll_sel) // 语句
                .map(|p| file_name(p)) // 语句
                .unwrap_or_else(|| "（当前集合）".into()); // 语句
            self.pending_delete = Some(PendingDelete::Collection { name }); // 语句
        } // 块结束
        ui.add_space(6.0); // 语句
        egui::Frame::none() // 调用 none
            .fill(mix(bg_side(), text_primary(), 0.05)) // 语句
            .rounding(Rounding::same(R_MD)) // 语句
            .inner_margin(egui::Margin::symmetric(10.0, 5.0)) // 语句
            .show(ui, |ui| { // 语句
                ui.add( // 语句
                    egui::TextEdit::singleline(&mut self.side_search) // 调用 singleline
                        .frame(false) // 语句
                        .hint_text("筛选请求…") // 语句
                        .desired_width(f32::INFINITY), // 语句
                ); // 块结束
            }); // 语句
        ui.add_space(6.0); // 语句
        let q = if self.top_search.trim().is_empty() { // 变量 q
            self.side_search.to_lowercase() // 语句
        } else { // 语句
            self.top_search.to_lowercase() // 语句
        }; // 块结束
        // 集合树快照
        let rows = self // 变量 rows
            .tree // 语句
            .as_ref() // 语句
            .map(|c| { // 语句
                let mut r = Vec::new(); // 变量 r
                let mut p = Vec::new(); // 变量 p
                collect_rows(&c.item, 0, &mut p, &mut r); // 调用 collect_rows
                r // 语句
            }) // 语句
            .unwrap_or_default(); // 语句
        let ren = self.rename_path.clone(); // 变量 ren
        let mut clicked: Option<Vec<usize>> = None; // 变量 clicked
        let mut del: Option<(Vec<usize>, String)> = None; // 变量 del
        let mut dup: Option<Vec<usize>> = None; // 变量 dup
        let mut up: Option<Vec<usize>> = None; // 变量 up
        let mut down: Option<Vec<usize>> = None; // 变量 down
        let mut start_rename: Option<(Vec<usize>, String, bool)> = None; // path,name,is_commit
        let mut do_new_folder = false; // 变量 do_new_folder
        egui::ScrollArea::vertical() // 调用 vertical
            .id_salt("reqtree") // 语句
            .auto_shrink([false, false]) // 语句
            .show(ui, |ui| { // 语句
                ui.horizontal_wrapped(|ui| { // 语句
                    let w = (ui.available_width() - 86.0).max(50.0); // 变量 w
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut self.folder_name) // 调用 singleline
                            .hint_text("新文件夹名") // 语句
                            .desired_width(w), // 语句
                    ); // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("＋文件夹").color(brand())) // 调用 new
                                .fill(brand_soft()) // 语句
                                .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        do_new_folder = true; // 语句
                    } // 块结束
                }); // 语句
                if rows.is_empty() { // 条件分支
                    ui.label(RichText::new("（此集合暂无请求）").color(text_secondary()).size(12.0)); // 语句
                } // 块结束
                for row in &rows { // 循环
                    if !q.is_empty() // 条件分支
                        && !row.name.to_lowercase().contains(&q) // 语句
                        && !row // 语句
                            .method // 语句
                            .as_deref() // 语句
                            .unwrap_or("") // 语句
                            .to_lowercase() // 语句
                            .contains(&q) // 语句
                    { // 语句
                        continue; // 语句
                    } // 块结束
                    let ind = row.depth as f32 * 14.0; // 变量 ind
                    ui.horizontal_wrapped(|ui| { // 语句
                        ui.add_space(ind); // 语句
                        if row.is_folder { // 条件分支
                            ui.label(RichText::new("📁").size(13.0)); // 语句
                        } else { // 语句
                            method_pill(ui, row.method.as_deref().unwrap_or("GET")); // 调用 method_pill
                        } // 块结束
                        if ren.as_deref() == Some(row.path.as_slice()) { // 条件分支
                            ui.add( // 语句
                                egui::TextEdit::singleline(&mut self.rename_text) // 调用 singleline
                                    .desired_width(130.0), // 语句
                            ); // 块结束
                            if ui.small_button("确定").clicked() { // 条件分支
                                start_rename = // 语句
                                    Some((row.path.clone(), self.rename_text.clone(), true)); // 调用 Some
                            } // 块结束
                        } else { // 语句
                            let resp = ui.add( // 变量 resp
                                egui::Button::new( // 调用 new
                                    RichText::new(&row.name).color(text_primary()).size(13.0), // 字段 RichText
                                ) // 块结束
                                .frame(false), // 语句
                            ); // 块结束
                            if resp.clicked() && !row.is_folder { // 条件分支
                                clicked = Some(row.path.clone()); // 语句
                            } // 块结束
                            if resp.double_clicked() { // 条件分支
                                start_rename = // 语句
                                    Some((row.path.clone(), row.name.clone(), false)); // 调用 Some
                            } // 块结束
                        } // 块结束
                        if !row.is_folder { // 条件分支
                            if ui.small_button("↑").clicked() { // 条件分支
                                up = Some(row.path.clone()); // 语句
                            } // 块结束
                            if ui.small_button("↓").clicked() { // 条件分支
                                down = Some(row.path.clone()); // 语句
                            } // 块结束
                            if ui.small_button("复制").clicked() { // 条件分支
                                dup = Some(row.path.clone()); // 语句
                            } // 块结束
                        } // 块结束
                        if ui.small_button("重命名").clicked() { // 条件分支
                            start_rename = Some((row.path.clone(), row.name.clone(), false)); // 语句
                        } // 块结束
                        if ui // 条件分支
                            .add( // 语句
                                egui::Button::new( // 调用 new
                                    RichText::new("×").color(text_secondary()).size(12.0), // 字段 RichText
                                ) // 块结束
                                .frame(false), // 语句
                            ) // 块结束
                            .clicked() // 语句
                        { // 语句
                            del = Some((row.path.clone(), row.name.clone())); // 语句
                        } // 块结束
                    }); // 语句
                    ui.add_space(1.0); // 语句
                } // 块结束
            }); // 语句
        // ---- 应用树操作 ----
        if let Some((p, name)) = del { // 条件分支
            self.pending_delete = Some(PendingDelete::TreeItem { path: p, name }); // 语句
        } // 块结束
        if let Some(p) = dup { // 条件分支
            self.duplicate_item(&p); // 语句
        } // 块结束
        if let Some(p) = up { // 条件分支
            self.move_item(&p, true); // 语句
        } // 块结束
        if let Some(p) = down { // 条件分支
            self.move_item(&p, false); // 语句
        } // 块结束
        if let Some((p, name, commit)) = start_rename { // 条件分支
            if commit { // 条件分支
                if let Some(c) = self.tree.as_mut() { // 条件分支
                    if let Some(it) = item_at_mut(&mut c.item, &p) { // 条件分支
                        let nn = self.rename_text.trim().to_string(); // 变量 nn
                        if !nn.is_empty() { // 条件分支
                            it.name = nn; // 语句
                        } // 块结束
                    } // 块结束
                } // 块结束
                self.rename_path = None; // 语句
                self.save_tree(); // 语句
            } else { // 语句
                self.rename_path = Some(p); // 语句
                self.rename_text = name; // 语句
            } // 块结束
        } // 块结束
        if do_new_folder { // 条件分支
            let name = if self.folder_name.trim().is_empty() { // 变量 name
                "新建文件夹".to_string() // 语句
            } else { // 语句
                self.folder_name.trim().to_string() // 语句
            }; // 块结束
            if let Some(c) = self.tree.as_mut() { // 条件分支
                c.item.push(Item { // 语句
                    name, // 语句
                    item: Some(Vec::new()), // 字段 item
                    ..Default::default() // 语句
                }); // 语句
            } // 块结束
            self.folder_name.clear(); // 语句
            self.save_tree(); // 语句
        } // 块结束
        if let Some(p) = clicked { // 条件分支
            if let Some(it) = self.tree.as_ref().and_then(|c| item_at(&c.item, &p)) { // 条件分支
                if let Some(req) = &it.request { // 条件分支
                    let leaf = it.name.rsplit('/').next().unwrap_or(&it.name).to_string(); // 变量 leaf
                    if let Some(idx) = self.requests.iter().position(|(n, r, _)| { // 条件分支
                        n.ends_with(&leaf) // 语句
                            && r.method == req.method // 语句
                            && r.url.raw == req.url.raw // 语句
                    }) { // 语句
                        self.apply_request_to_active(idx); // 语句
                        self.tabs[self.active].item_path = Some(p.clone()); // 语句
                    } // 块结束
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    fn create_collection(&mut self) { // 函数 create_collection
        let name = if self.coll_new_name.trim().is_empty() { // 变量 name
            "新建集合".to_string() // 语句
        } else { // 语句
            self.coll_new_name.trim().to_string() // 语句
        }; // 块结束
        let c = Collection { // 变量 c
            info: CollectionInfo { // 语句
                name: name.clone(), // 字段 name
                postman_id: Some(uuid::Uuid::new_v4().to_string()), // 字段 postman_id
                schema: Some( // 语句
                    "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into(), // 语句
                ), // 语句
                description: None, // 字段 description
            }, // 块结束
            ..Default::default() // 语句
        }; // 块结束
        match storage::save_collection(&c) { // 分支匹配
            Ok(p) => { // 匹配分支
                self.message = format!("已新建集合：{}", name); // 语句
                self.coll_new_name.clear(); // 语句
                self.coll_files = storage::list_collections().unwrap_or_default(); // 语句
                if let Some(i) = self.coll_files.iter().position(|x| x == &p) { // 条件分支
                    self.coll_sel = i; // 语句
                } // 块结束
                self.load_requests(); // 语句
            } // 块结束
            Err(e) => self.message = format!("新建集合失败：{}", e), // 匹配分支
        } // 块结束
    } // 块结束
    fn delete_collection(&mut self) { // 函数 delete_collection
        if let Some(path) = self.coll_files.get(self.coll_sel).cloned() { // 条件分支
            if std::fs::remove_file(&path).is_ok() { // 条件分支
                self.message = "已删除集合".into(); // 语句
                self.coll_sel = 0; // 语句
                self.refresh_all(); // 语句
            } else { // 语句
                self.message = "删除集合失败".into(); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    fn save_tree(&mut self) { // 函数 save_tree
        if let (Some(c), Some(path)) = (self.tree.as_ref(), self.tree_file.clone()) { // 条件分支
            match serde_json::to_string_pretty(c) { // 分支匹配
                Ok(t) => { // 匹配分支
                    if let Err(e) = std::fs::write(&path, t) { // 条件分支
                        self.message = format!("保存集合失败：{}", e); // 语句
                        return; // 返回
                    } // 块结束
                } // 块结束
                Err(e) => { // 匹配分支
                    self.message = format!("序列化失败：{}", e); // 语句
                    return; // 返回
                } // 块结束
            } // 块结束
        } else { // 语句
            return; // 返回
        } // 块结束
        self.message = "已保存集合改动".into(); // 语句
        let sel = self.coll_sel; // 变量 sel
        self.coll_files = storage::list_collections().unwrap_or_default(); // 语句
        self.coll_sel = sel.min(self.coll_files.len().saturating_sub(1)); // 语句
        self.load_requests(); // 语句
    } // 块结束
    fn duplicate_item(&mut self, path: &[usize]) { // 函数 duplicate_item
        let Some(c) = self.tree.as_mut() else { return }; // 变量 Some
        let Some(src) = item_at(&c.item, path).cloned() else { // 变量 Some
            return; // 返回
        }; // 块结束
        let mut clone = src; // 变量 clone
        clone.name = format!("{} 副本", clone.name); // 语句
        let idx = *path.last().unwrap_or(&0); // 变量 idx
        let parent = &path[..path.len().saturating_sub(1)]; // 变量 parent
        if parent.is_empty() { // 条件分支
            c.item.insert((idx + 1).min(c.item.len()), clone); // 语句
        } else if let Some(pi) = item_at_mut(&mut c.item, parent) { // 条件分支
            if let Some(children) = pi.item.as_mut() { // 条件分支
                children.insert((idx + 1).min(children.len()), clone); // 语句
            } // 块结束
        } // 块结束
        self.save_tree(); // 语句
    } // 块结束
    fn move_item(&mut self, path: &[usize], up: bool) { // 函数 move_item
        if path.is_empty() { // 条件分支
            return; // 返回
        } // 块结束
        let idx = path[path.len() - 1]; // 变量 idx
        let parent = &path[..path.len() - 1]; // 变量 parent
        { // 语句
            let Some(c) = self.tree.as_mut() else { return }; // 变量 Some
            let siblings: &mut Vec<Item> = if parent.is_empty() { // 变量 siblings
                &mut c.item // 语句
            } else if let Some(pi) = item_at_mut(&mut c.item, parent) { // 条件分支
                match pi.item.as_mut() { // 分支匹配
                    Some(v) => v, // 匹配分支
                    None => return, // 匹配分支
                } // 块结束
            } else { // 语句
                return; // 返回
            }; // 块结束
            if up { // 条件分支
                if idx > 0 { // 条件分支
                    siblings.swap(idx, idx - 1); // 语句
                } // 块结束
            } else if idx + 1 < siblings.len() { // 条件分支
                siblings.swap(idx, idx + 1); // 语句
            } // 块结束
        } // 块结束
        self.save_tree(); // 语句
    } // 块结束
    /// 当前标签对应集合项的已保存示例响应。
    fn examples_for_active(&self) -> Vec<ResponseExample> { // 函数 examples_for_active
        let Some(p) = self.tabs[self.active].item_path.as_ref() else { // 变量 Some
            return Vec::new(); // 返回
        }; // 块结束
        self.tree // 语句
            .as_ref() // 语句
            .and_then(|c| item_at(&c.item, p)) // 语句
            .map(|it| it.response.clone()) // 语句
            .unwrap_or_default() // 语句
    } // 块结束
    fn save_example(&mut self) { // 函数 save_example
        let active = self.active; // 变量 active
        let (path, resp) = { // 调用 let
            let t = &self.tabs[active]; // 变量 t
            (t.item_path.clone(), t.response.clone()) // 语句
        }; // 块结束
        let Some(path) = path else { // 变量 Some
            self.message = "请先从左侧集合树打开此请求，再保存示例".into(); // 语句
            return; // 返回
        }; // 块结束
        let Some(r) = resp else { // 变量 Some
            self.message = "暂无响应可保存为示例".into(); // 语句
            return; // 返回
        }; // 块结束
        let ex = ResponseExample { // 变量 ex
            name: format!("示例 {}", chrono::Local::now().format("%H:%M:%S")), // 字段 name
            original_request: None, // 字段 original_request
            status: Some(r.status.clone()), // 字段 status
            code: Some(r.code), // 字段 code
            header: r // 语句
                .headers // 语句
                .iter() // 语句
                .map(|(k, v)| Header { // 语句
                    key: k.clone(), // 字段 key
                    value: v.clone(), // 字段 value
                    ..Default::default() // 语句
                }) // 语句
                .collect(), // 语句
            body: Some(r.body.clone()), // 字段 body
            preview_language: None, // 字段 preview_language
        }; // 块结束
        if let Some(c) = self.tree.as_mut() { // 条件分支
            if let Some(it) = item_at_mut(&mut c.item, &path) { // 条件分支
                it.response.push(ex); // 语句
            } // 块结束
        } // 块结束
        self.save_tree(); // 语句
    } // 块结束
    fn ui_side_env(&mut self, ui: &mut egui::Ui) { // 函数 ui_side_env
        section_title(ui, "环境变量"); // 调用 section_title
        ui.add_space(2.0); // 语句
        ui.label( // 语句
            RichText::new("可编辑并保存当前环境") // 调用 new
                .color(text_secondary()) // 语句
                .size(11.0), // 语句
        ); // 块结束
        let cur_stem = self.env_files.get(self.env_sel).map(file_name).unwrap_or_default(); // 变量 cur_stem
        let cur = if cur_stem.is_empty() { // 变量 cur
            "（无环境）".to_string() // 语句
        } else { // 语句
            cur_stem.clone() // 语句
        }; // 块结束
        egui::ComboBox::from_id_salt("env_side") // 调用 from_id_salt
            .width(ui.available_width()) // 语句
            .selected_text(cur) // 语句
            .show_ui(ui, |ui| { // 语句
                for i in 0..self.env_files.len() { // 循环
                    ui.selectable_value(&mut self.env_sel, i, file_name(&self.env_files[i])); // 语句
                } // 块结束
            }); // 语句
        if self.env_loaded_for != cur_stem { // 条件分支
            self.env_edit = self // 语句
                .env_files // 语句
                .get(self.env_sel) // 语句
                .and_then(|p| std::fs::read_to_string(p).ok()) // 语句
                .and_then(|t| serde_json::from_str::<Environment>(&t).ok()) // 语句
                .map(|e| { // 语句
                    e.values // 语句
                        .iter() // 语句
                        .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
                        .collect() // 语句
                }) // 语句
                .unwrap_or_default(); // 语句
            self.env_loaded_for = cur_stem.clone(); // 语句
        } // 块结束
        ui.add_space(6.0); // 语句
        let mut remove: Option<usize> = None; // 变量 remove
        egui::ScrollArea::vertical() // 调用 vertical
            .id_salt("envedit") // 语句
            .auto_shrink([false, false]) // 语句
            .max_height(320.0) // 语句
            .show(ui, |ui| { // 语句
                // 键、值两个输入框各占左侧栏宽度的一半（用 columns(2) 在
                // 「×按钮之前」的剩余区域内强制二等分）。
                const ROW_H: f32 = 22.0; // 常量 ROW_H
                for (i, (k, v)) in self.env_edit.iter_mut().enumerate() { // 循环
                    ui.horizontal(|ui| { // 语句
                        let rem = (ui.available_width() - 26.0).max(120.0); // 变量 rem
                        ui.allocate_ui_with_layout( // 语句
                            egui::vec2(rem, ROW_H), // 字段 egui
                            egui::Layout::left_to_right(egui::Align::Center), // 字段 egui
                            |ui| { // 语句
                                ui.columns(2, |cols| { // 语句
                                    let w0 = cols[0].available_width(); // 变量 w0
                                    cols[0].add_sized( // 语句
                                        [w0, ROW_H], // 语句
                                        egui::TextEdit::singleline(k).hint_text("键"), // 字段 egui
                                    ); // 块结束
                                    let w1 = cols[1].available_width(); // 变量 w1
                                    cols[1].add_sized( // 语句
                                        [w1, ROW_H], // 语句
                                        egui::TextEdit::singleline(v).hint_text("值"), // 字段 egui
                                    ); // 块结束
                                }); // 语句
                            }, // 块结束
                        ); // 块结束
                        if ui // 条件分支
                            .add( // 语句
                                egui::Button::new(RichText::new("×").color(text_secondary())) // 调用 new
                                    .frame(false), // 语句
                            ) // 块结束
                            .clicked() // 语句
                        { // 语句
                            remove = Some(i); // 语句
                        } // 块结束
                    }); // 语句
                } // 块结束
            }); // 语句
        if let Some(i) = remove { // 条件分支
            let key = self.env_edit.get(i).map(|(k, _)| k.clone()).unwrap_or_default(); // 变量 key
            self.pending_delete = Some(PendingDelete::EnvVar { idx: i, key }); // 语句
        } // 块结束
        if ui // 条件分支
            .add( // 语句
                egui::Button::new(RichText::new("＋ 添加变量").color(brand())).fill(brand_soft()), // 字段 egui
            ) // 块结束
            .clicked() // 语句
        { // 语句
            self.env_edit.push((String::new(), String::new())); // 语句
        } // 块结束
        if ui // 条件分支
            .add( // 语句
                egui::Button::new(RichText::new("保存环境").color(Color32::WHITE).strong()) // 调用 new
                    .fill(brand()) // 语句
                    .rounding(Rounding::same(6.0)), // 语句
            ) // 块结束
            .clicked() // 语句
        { // 语句
            if let Some(p) = self.env_files.get(self.env_sel).cloned() { // 条件分支
                let mut e = std::fs::read_to_string(&p) // 变量 e
                    .ok() // 语句
                    .and_then(|t| serde_json::from_str::<Environment>(&t).ok()) // 语句
                    .unwrap_or_default(); // 语句
                e.values = self // 语句
                    .env_edit // 语句
                    .iter() // 语句
                    .filter(|(k, _)| !k.trim().is_empty()) // 语句
                    .map(|(k, v)| Variable { // 语句
                        key: k.clone(), // 字段 key
                        value: serde_json::Value::String(v.clone()), // 字段 value
                        typ: Some("default".into()), // 字段 typ
                        enabled: Some(true), // 字段 enabled
                        description: None, // 字段 description
                    }) // 语句
                    .collect(); // 语句
                match storage::save_environment(&e) { // 分支匹配
                    Ok(_) => self.message = format!("环境已保存：{}", cur_stem), // 匹配分支
                    Err(err) => self.message = format!("保存失败：{}", err), // 匹配分支
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    fn ui_side_globals(&mut self, ui: &mut egui::Ui) { // 函数 ui_side_globals
        section_title(ui, "全局变量"); // 调用 section_title
        ui.add_space(2.0); // 语句
        ui.label( // 语句
            RichText::new("全局变量对所有集合/请求生效（优先级最低，可被环境/集合变量覆盖）") // 调用 new
                .color(text_secondary()) // 语句
                .size(11.0), // 语句
        ); // 块结束
        if !self.globals_loaded { // 条件分支
            self.globals_edit = storage::load_globals() // 语句
                .unwrap_or_default() // 语句
                .values // 语句
                .iter() // 语句
                .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
                .collect(); // 语句
            self.globals_loaded = true; // 语句
        } // 块结束
        ui.add_space(6.0); // 语句
        let mut remove: Option<usize> = None; // 变量 remove
        egui::ScrollArea::vertical() // 调用 vertical
            .id_salt("globalsedit") // 语句
            .auto_shrink([false, false]) // 语句
            .max_height(320.0) // 语句
            .show(ui, |ui| { // 语句
                // 键、值两个输入框各占左侧栏宽度的一半（用 columns(2) 二等分）。
                const ROW_H: f32 = 22.0; // 常量 ROW_H
                for (i, (k, v)) in self.globals_edit.iter_mut().enumerate() { // 循环
                    ui.horizontal(|ui| { // 语句
                        let rem = (ui.available_width() - 26.0).max(120.0); // 变量 rem
                        ui.allocate_ui_with_layout( // 语句
                            egui::vec2(rem, ROW_H), // 字段 egui
                            egui::Layout::left_to_right(egui::Align::Center), // 字段 egui
                            |ui| { // 语句
                                ui.columns(2, |cols| { // 语句
                                    let w0 = cols[0].available_width(); // 变量 w0
                                    cols[0].add_sized( // 语句
                                        [w0, ROW_H], // 语句
                                        egui::TextEdit::singleline(k).hint_text("键"), // 字段 egui
                                    ); // 块结束
                                    let w1 = cols[1].available_width(); // 变量 w1
                                    cols[1].add_sized( // 语句
                                        [w1, ROW_H], // 语句
                                        egui::TextEdit::singleline(v).hint_text("值"), // 字段 egui
                                    ); // 块结束
                                }); // 语句
                            }, // 块结束
                        ); // 块结束
                        if ui // 条件分支
                            .add( // 语句
                                egui::Button::new(RichText::new("×").color(text_secondary())) // 调用 new
                                    .frame(false), // 语句
                            ) // 块结束
                            .clicked() // 语句
                        { // 语句
                            remove = Some(i); // 语句
                        } // 块结束
                    }); // 语句
                } // 块结束
            }); // 语句
        if let Some(i) = remove { // 条件分支
            let key = self.globals_edit.get(i).map(|(k, _)| k.clone()).unwrap_or_default(); // 变量 key
            self.pending_delete = Some(PendingDelete::GlobalVar { idx: i, key }); // 语句
        } // 块结束
        if ui // 条件分支
            .add( // 语句
                egui::Button::new(RichText::new("＋ 添加变量").color(brand())).fill(brand_soft()), // 字段 egui
            ) // 块结束
            .clicked() // 语句
        { // 语句
            self.globals_edit.push((String::new(), String::new())); // 语句
        } // 块结束
        ui.horizontal_wrapped(|ui| { // 语句
            if ui // 条件分支
                .add( // 语句
                    egui::Button::new( // 调用 new
                        RichText::new("保存全局变量").color(Color32::WHITE).strong(), // 字段 RichText
                    ) // 块结束
                    .fill(brand()) // 语句
                    .rounding(Rounding::same(6.0)), // 语句
                ) // 块结束
                .clicked() // 语句
            { // 语句
                let env = Environment { // 变量 env
                    id: None, // 字段 id
                    name: "globals".into(), // 字段 name
                    scope: Some("globals".into()), // 字段 scope
                    values: self // 语句
                        .globals_edit // 语句
                        .iter() // 语句
                        .filter(|(k, _)| !k.trim().is_empty()) // 语句
                        .map(|(k, v)| Variable { // 语句
                            key: k.clone(), // 字段 key
                            value: serde_json::Value::String(v.clone()), // 字段 value
                            typ: Some("default".into()), // 字段 typ
                            enabled: Some(true), // 字段 enabled
                            description: None, // 字段 description
                        }) // 语句
                        .collect(), // 语句
                }; // 块结束
                match storage::save_globals(&env) { // 分支匹配
                    Ok(_) => { // 匹配分支
                        self.message = "全局变量已保存".into(); // 语句
                        self.globals_loaded = false; // 语句
                    } // 块结束
                    Err(e) => self.message = format!("保存失败：{}", e), // 匹配分支
                } // 块结束
            } // 块结束
            if ui // 条件分支
                .add( // 语句
                    egui::Button::new(RichText::new("重新加载").color(text_secondary())) // 调用 new
                        .frame(false), // 语句
                ) // 块结束
                .clicked() // 语句
            { // 语句
                self.globals_loaded = false; // 语句
            } // 块结束
        }); // 语句
    } // 块结束
    fn ui_side_history(&mut self, ui: &mut egui::Ui) { // 函数 ui_side_history
        section_title(ui, "最近请求"); // 调用 section_title
        ui.add_space(4.0); // 语句
        egui::Frame::none() // 调用 none
            .fill(mix(bg_side(), text_primary(), 0.05)) // 语句
            .rounding(Rounding::same(R_MD)) // 语句
            .inner_margin(egui::Margin::symmetric(10.0, 5.0)) // 语句
            .show(ui, |ui| { // 语句
                ui.add( // 语句
                    egui::TextEdit::singleline(&mut self.hist_search) // 调用 singleline
                        .frame(false) // 语句
                        .hint_text("搜索历史（URL / 方法）…") // 语句
                        .desired_width(f32::INFINITY), // 语句
                ); // 块结束
            }); // 语句
        ui.add_space(6.0); // 语句
        let q = self.hist_search.to_lowercase(); // 变量 q
        egui::ScrollArea::vertical() // 调用 vertical
            .id_salt("hist") // 语句
            .auto_shrink([false, false]) // 语句
            .show(ui, |ui| { // 语句
                if self.history.is_empty() { // 条件分支
                    ui.label(RichText::new("（暂无历史）").color(text_secondary()).size(12.0)); // 语句
                } // 块结束
                let mut replay: Option<(String, String)> = None; // 变量 replay
                let mut save_as: Option<(String, String)> = None; // 变量 save_as
                for h in self.history.iter().rev().take(200) { // 循环
                    if !q.is_empty() // 条件分支
                        && !h.url.to_lowercase().contains(&q) // 语句
                        && !h.method.to_lowercase().contains(&q) // 语句
                    { // 语句
                        continue; // 语句
                    } // 块结束
                    let code: u16 = h.status; // 变量 code
                    let (hit, save) = ui // 调用 let
                        .horizontal(|ui| { // 语句
                            ui.label( // 语句
                                RichText::new(&h.method) // 调用 new
                                    .color(method_color(&h.method)) // 语句
                                    .strong() // 语句
                                    .size(11.0), // 语句
                            ); // 块结束
                            let b = ui.add( // 变量 b
                                egui::Button::new( // 调用 new
                                    RichText::new(&h.url).color(text_primary()).size(12.0), // 字段 RichText
                                ) // 块结束
                                .frame(false), // 语句
                            ); // 块结束
                            ui.label( // 语句
                                RichText::new(code.to_string()) // 调用 new
                                    .color(status_color(code)) // 语句
                                    .size(11.0), // 语句
                            ); // 块结束
                            let s = ui.small_button("存为请求").clicked(); // 变量 s
                            (b.clicked(), s) // 语句
                        }) // 语句
                        .inner; // 语句
                    if hit { // 条件分支
                        replay = Some((h.method.clone(), h.url.clone())); // 语句
                    } // 块结束
                    if save { // 条件分支
                        save_as = Some((h.method.clone(), h.url.clone())); // 语句
                    } // 块结束
                } // 块结束
                if let Some((m, u)) = replay { // 条件分支
                    let tab = &mut self.tabs[self.active]; // 变量 tab
                    tab.method = m; // 语句
                    tab.url = u; // 语句
                    tab.params = parse_query(&tab.url); // 语句
                    tab.path_rows = detect_path_vars(&tab.url); // 语句
                    tab.hash.clear(); // 语句
                    self.message = "已从历史载入请求".into(); // 语句
                } // 块结束
                if let Some((m, u)) = save_as { // 条件分支
                    { // 语句
                        let tab = &mut self.tabs[self.active]; // 变量 tab
                        tab.method = m; // 语句
                        tab.url = u; // 语句
                        tab.params = parse_query(&tab.url); // 语句
                        tab.path_rows = detect_path_vars(&tab.url); // 语句
                        tab.hash.clear(); // 语句
                    } // 块结束
                    self.save_active(); // 语句
                } // 块结束
            }); // 语句
    } // 块结束
    fn ui_side_console(&mut self, ui: &mut egui::Ui) { // 函数 ui_side_console
        section_title(ui, "控制台"); // 调用 section_title
        ui.add_space(6.0); // 语句
        ui.horizontal_wrapped(|ui| { // 语句
            if ui.button("清空").clicked() { // 条件分支
                self.console.clear(); // 语句
            } // 块结束
            if ui.button("复制全部").clicked() { // 条件分支
                let text = self.console.join("\n"); // 变量 text
                ui.output_mut(|o| o.copied_text = text); // 语句
            } // 块结束
            ui.label( // 语句
                RichText::new(format!("{} 条", self.console.len())) // 调用 new
                    .color(text_secondary()) // 语句
                    .size(11.0), // 语句
            ); // 块结束
        }); // 语句
        ui.add_space(6.0); // 语句
        egui::ScrollArea::vertical() // 调用 vertical
            .id_salt("consolelog") // 语句
            .auto_shrink([false, false]) // 语句
            .show(ui, |ui| { // 语句
                if self.console.is_empty() { // 条件分支
                    ui.label( // 语句
                        RichText::new("暂无日志。发送请求后这里会显示请求 / 响应 / 脚本测试日志。") // 调用 new
                            .color(text_secondary()) // 语句
                            .size(11.0), // 语句
                    ); // 块结束
                } // 块结束
                for line in &self.console { // 循环
                    ui.label(RichText::new(line).monospace().size(11.0)); // 语句
                } // 块结束
            }); // 语句
    } // 块结束
    fn ui_side_tools(&mut self, ui: &mut egui::Ui) { // 函数 ui_side_tools
        egui::ScrollArea::vertical() // 调用 vertical
            .id_salt("tools") // 语句
            .auto_shrink([false, false]) // 语句
            .show(ui, |ui| { // 语句
                // 外观 / 主题
                section_title(ui, "外观 / 主题"); // 调用 section_title
                ui.horizontal_wrapped(|ui| { // 语句
                    let mut sel = self.theme; // 变量 sel
                    egui::ComboBox::from_id_salt("tools_theme") // 调用 from_id_salt
                        .width(150.0) // 语句
                        .selected_text(THEMES[sel.min(THEMES.len() - 1)].name) // 语句
                        .show_ui(ui, |ui| { // 语句
                            for i in 0..THEMES.len() { // 循环
                                ui.selectable_value(&mut sel, i, THEMES[i].name); // 语句
                            } // 块结束
                        }); // 语句
                    if sel != self.theme { // 条件分支
                        self.theme = sel; // 语句
                        self.follow_system = false; // 语句
                        FOLLOW_SYSTEM.store(false, Ordering::Relaxed); // 语句
                        apply_theme(ui.ctx(), sel); // 调用 apply_theme
                        save_theme(sel); // 调用 save_theme
                    } // 块结束
                    let mut follow = self.follow_system; // 变量 follow
                    if ui // 条件分支
                        .checkbox( // 语句
                            &mut follow, // 语句
                            RichText::new("跟随系统").color(text_secondary()), // 字段 RichText
                        ) // 块结束
                        .changed() // 语句
                    { // 语句
                        self.follow_system = follow; // 语句
                        FOLLOW_SYSTEM.store(follow, Ordering::Relaxed); // 语句
                    } // 块结束
                }); // 语句
                ui.horizontal_wrapped(|ui| { // 语句
                    ui.label(RichText::new("强调色 #").color(text_secondary()).size(12.0)); // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut self.accent_hex) // 调用 singleline
                            .desired_width(90.0) // 语句
                            .hint_text("FF6C37"), // 语句
                    ); // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new( // 调用 new
                                RichText::new("应用强调色").color(Color32::WHITE).strong(), // 字段 RichText
                            ) // 块结束
                            .fill(brand()) // 语句
                            .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        if let Some(rgbv) = parse_hex_color(&self.accent_hex) { // 条件分支
                            ACCENT_OVERRIDE.store(0x01000000 | rgbv, Ordering::Relaxed); // 语句
                            install_style(ui.ctx()); // 调用 install_style
                            self.message = format!("强调色已应用：#{:06X}", rgbv); // 语句
                        } else if self.accent_hex.trim().is_empty() { // 条件分支
                            ACCENT_OVERRIDE.store(0, Ordering::Relaxed); // 语句
                            install_style(ui.ctx()); // 调用 install_style
                            self.message = "已恢复主题默认强调色".into(); // 语句
                        } else { // 语句
                            self.message = "强调色格式无效，请输入 6 位十六进制，如 FF6C37".into(); // 语句
                        } // 块结束
                    } // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("恢复默认").color(text_secondary())) // 调用 new
                                .frame(false), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        ACCENT_OVERRIDE.store(0, Ordering::Relaxed); // 语句
                        self.accent_hex.clear(); // 语句
                        install_style(ui.ctx()); // 调用 install_style
                    } // 块结束
                }); // 语句
                ui.add_space(6.0); // 语句
                ui.separator(); // 语句
                ui.add_space(6.0); // 语句

                // WebSocket 协议工作台
                section_title(ui, "WebSocket 工作台"); // 调用 section_title
                ui.add( // 语句
                    egui::TextEdit::singleline(&mut self.ws_url) // 调用 singleline
                        .hint_text("ws:// 或 wss:// 地址") // 语句
                        .desired_width(f32::INFINITY), // 语句
                ); // 块结束
                ui.horizontal_wrapped(|ui| { // 语句
                    ui.label(RichText::new("子协议").color(text_secondary()).size(12.0)); // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut self.ws_subprotocol) // 调用 singleline
                            .desired_width(110.0) // 语句
                            .hint_text("可选"), // 语句
                    ); // 块结束
                    ui.checkbox(&mut self.ws_binary, RichText::new("二进制发送").color(text_secondary())); // 语句
                }); // 语句
                ui.label(RichText::new("自定义请求头（每行 K: V）").color(text_secondary()).size(11.5)); // 语句
                ui.add( // 语句
                    egui::TextEdit::multiline(&mut self.ws_headers) // 调用 multiline
                        .desired_rows(2) // 语句
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text("Authorization: Bearer xxx"), // 语句
                ); // 块结束
                ui.label(RichText::new("发送消息（每行一条）").color(text_secondary()).size(11.5)); // 语句
                ui.add( // 语句
                    egui::TextEdit::multiline(&mut self.ws_send) // 调用 multiline
                        .desired_rows(3) // 语句
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text("{\"type\":\"ping\"}"), // 语句
                ); // 块结束
                ui.horizontal_wrapped(|ui| { // 语句
                    ui.label(RichText::new("监听条数").color(text_secondary()).size(12.0)); // 语句
                    ui.add(egui::DragValue::new(&mut self.ws_listen).range(0..=1000)); // 语句
                    ui.label(RichText::new("等待(ms)").color(text_secondary()).size(12.0)); // 语句
                    ui.add(egui::DragValue::new(&mut self.ws_wait_ms).range(100..=120000)); // 语句
                    let label = if self.ws_busy { "连接中…" } else { "连接并发送" }; // 变量 label
                    if ui // 条件分支
                        .add_enabled( // 语句
                            !self.ws_busy, // 语句
                            egui::Button::new(RichText::new(label).color(Color32::WHITE).strong()) // 调用 new
                                .fill(send_blue()) // 语句
                                .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        self.send_ws(); // 语句
                    } // 块结束
                }); // 语句
                if !self.ws_log.is_empty() { // 条件分支
                    let mut ro = self.ws_log.clone(); // 变量 ro
                    ui.add( // 语句
                        egui::TextEdit::multiline(&mut ro) // 调用 multiline
                            .code_editor() // 语句
                            .desired_rows(8) // 语句
                            .desired_width(f32::INFINITY) // 语句
                            .interactive(false), // 语句
                    ); // 块结束
                } // 块结束
                ui.add_space(6.0); // 语句
                ui.separator(); // 语句
                ui.add_space(6.0); // 语句

                // 导入
                section_title(ui, "导入集合 / 环境"); // 调用 section_title
                ui.add( // 语句
                    egui::TextEdit::singleline(&mut self.import_path) // 调用 singleline
                        .hint_text("文件路径（.json / .yaml / curl 文本 / .har）") // 语句
                        .desired_width(f32::INFINITY), // 语句
                ); // 块结束
                ui.horizontal_wrapped(|ui| { // 语句
                    egui::ComboBox::from_id_salt("import_fmt") // 调用 from_id_salt
                        .width(150.0) // 语句
                        .selected_text(import_format_label(&self.import_format)) // 语句
                        .show_ui(ui, |ui| { // 语句
                            for f in ["postman", "openapi", "curl", "har"] { // 循环
                                ui.selectable_value( // 语句
                                    &mut self.import_format, // 语句
                                    f.to_string(), // 语句
                                    import_format_label(f), // 调用 import_format_label
                                ); // 块结束
                            } // 块结束
                        }); // 语句
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new( // 调用 new
                                RichText::new("导入并合并").color(Color32::WHITE).strong(), // 字段 RichText
                            ) // 块结束
                            .fill(brand()) // 语句
                            .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        self.import_now(); // 语句
                    } // 块结束
                }); // 语句
                ui.add_space(6.0); // 语句
                ui.separator(); // 语句
                ui.add_space(6.0); // 语句

                // cURL 粘贴导入
                section_title(ui, "从 cURL 导入（粘贴）"); // 调用 section_title
                ui.add( // 语句
                    egui::TextEdit::multiline(&mut self.curl_input) // 调用 multiline
                        .desired_rows(3) // 语句
                        .code_editor() // 语句
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text("粘贴 curl 命令，例如：\ncurl -X POST 'https://api.test/x' -H 'Content-Type: application/json' -d '{\"a\":1}'"), // 语句
                ); // 块结束
                if ui // 条件分支
                    .add( // 语句
                        egui::Button::new( // 调用 new
                            RichText::new("解析为新标签").color(Color32::WHITE).strong(), // 字段 RichText
                        ) // 块结束
                        .fill(brand()) // 语句
                        .rounding(Rounding::same(6.0)), // 语句
                    ) // 块结束
                    .clicked() // 语句
                { // 语句
                    self.import_curl_to_tab(); // 语句
                } // 块结束
                ui.add_space(6.0); // 语句
                ui.separator(); // 语句
                ui.add_space(6.0); // 语句

                // Runner
                section_title(ui, "集合运行器"); // 调用 section_title
                let cur = self // 变量 cur
                    .coll_files // 语句
                    .get(self.coll_sel) // 语句
                    .map(file_name) // 语句
                    .unwrap_or_else(|| "（无集合）".into()); // 语句
                ui.label( // 语句
                    RichText::new(format!("当前集合：{}", cur)) // 调用 new
                        .color(text_secondary()) // 语句
                        .size(12.0), // 语句
                ); // 块结束
                ui.horizontal_wrapped(|ui| { // 语句
                    ui.label(RichText::new("迭代").color(text_secondary()).size(12.0)); // 语句
                    ui.add(egui::DragValue::new(&mut self.run_iterations).range(1..=1000)); // 语句
                    ui.checkbox(&mut self.run_bail, RichText::new("失败即停").color(text_secondary())); // 语句
                }); // 语句
                let label = if self.run_inflight { "运行中…" } else { "运行集合" }; // 变量 label
                if ui // 条件分支
                    .add_enabled( // 语句
                        !self.run_inflight, // 语句
                        egui::Button::new(RichText::new(label).color(Color32::WHITE).strong()) // 调用 new
                            .fill(send_blue()) // 语句
                            .rounding(Rounding::same(6.0)), // 语句
                    ) // 块结束
                    .clicked() // 语句
                { // 语句
                    self.run_current_collection(); // 语句
                } // 块结束
                if !self.run_result.is_empty() { // 条件分支
                    egui::ScrollArea::vertical() // 调用 vertical
                        .id_salt("runres") // 语句
                        .max_height(160.0) // 语句
                        .auto_shrink([false, false]) // 语句
                        .show(ui, |ui| { // 语句
                            let mut ro = self.run_result.clone(); // 变量 ro
                            ui.add( // 语句
                                egui::TextEdit::multiline(&mut ro) // 调用 multiline
                                    .code_editor() // 语句
                                    .desired_rows(6) // 语句
                                    .desired_width(f32::INFINITY) // 语句
                                    .interactive(false), // 语句
                            ); // 块结束
                        }); // 语句
                } // 块结束
                ui.add_space(6.0); // 语句
                ui.separator(); // 语句
                ui.add_space(6.0); // 语句

                // 代码生成
                section_title(ui, "代码生成（当前标签）"); // 调用 section_title
                ui.horizontal_wrapped(|ui| { // 语句
                    egui::ComboBox::from_id_salt("codegen_lang") // 调用 from_id_salt
                        .selected_text(&self.codegen_lang) // 语句
                        .show_ui(ui, |ui| { // 语句
                            for l in [ // 循环
                                "curl", "http", "python", "js", "nodejs", "go", "rust", "java", // 语句
                                "csharp", "php", "ruby", "powershell", "swift", // 语句
                            ] { // 语句
                                ui.selectable_value(&mut self.codegen_lang, l.to_string(), l); // 语句
                            } // 块结束
                        }); // 语句
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("生成").color(Color32::WHITE).strong()) // 调用 new
                                .fill(brand()) // 语句
                                .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        self.generate_code(); // 语句
                    } // 块结束
                    if !self.codegen_out.is_empty() // 条件分支
                        && ui // 语句
                            .add( // 语句
                                egui::Button::new(RichText::new("复制").color(text_secondary())) // 调用 new
                                    .frame(false), // 语句
                            ) // 块结束
                            .clicked() // 语句
                    { // 语句
                        ui.output_mut(|o| o.copied_text = self.codegen_out.clone()); // 语句
                    } // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new( // 调用 new
                                RichText::new("保存为文件").color(text_secondary()), // 字段 RichText
                            ) // 块结束
                            .frame(false), // 语句
                        ) // 块结束
                        .on_hover_text("把生成的代码/命令保存到 ~/.postman-rs/exports/") // 语句
                        .clicked() // 语句
                    { // 语句
                        self.save_codegen_to_file(); // 语句
                    } // 块结束
                }); // 语句
                if !self.codegen_out.is_empty() { // 条件分支
                    let mut ro = self.codegen_out.clone(); // 变量 ro
                    ui.add( // 语句
                        egui::TextEdit::multiline(&mut ro) // 调用 multiline
                            .code_editor() // 语句
                            .desired_rows(10) // 语句
                            .desired_width(f32::INFINITY) // 语句
                            .interactive(false), // 语句
                    ); // 块结束
                } // 块结束
                ui.add_space(6.0); // 语句
                ui.separator(); // 语句
                ui.add_space(6.0); // 语句

                // Cookie 管理器
                section_title(ui, "Cookie 管理器"); // 调用 section_title
                let cookies = http::cookies_dump(); // 变量 cookies
                if cookies.is_empty() { // 条件分支
                    ui.label(RichText::new("暂无已保存的 Cookie。").color(text_secondary()).size(11.0)); // 语句
                } else { // 语句
                    egui::ScrollArea::vertical() // 调用 vertical
                        .id_salt("cookielist") // 语句
                        .max_height(150.0) // 语句
                        .auto_shrink([false, false]) // 语句
                        .show(ui, |ui| { // 语句
                            let mut del: Option<(String, String)> = None; // 变量 del
                            egui::Grid::new("cookiegrid") // 调用 new
                                .num_columns(3) // 语句
                                .striped(true) // 语句
                                .spacing([8.0, 4.0]) // 语句
                                .show(ui, |ui| { // 语句
                                    for (host, name, value) in &cookies { // 循环
                                        ui.label( // 语句
                                            RichText::new(host).color(text_primary()).size(11.0), // 字段 RichText
                                        ); // 块结束
                                        ui.label( // 语句
                                            RichText::new(format!("{}={}", name, value)) // 调用 new
                                                .color(text_secondary()) // 语句
                                                .size(11.0), // 语句
                                        ); // 块结束
                                        if ui // 条件分支
                                            .add( // 语句
                                                egui::Button::new( // 调用 new
                                                    RichText::new("×").color(text_secondary()), // 字段 RichText
                                                ) // 块结束
                                                .frame(false), // 语句
                                            ) // 块结束
                                            .clicked() // 语句
                                        { // 语句
                                            del = Some((host.clone(), name.clone())); // 语句
                                        } // 块结束
                                        ui.end_row(); // 语句
                                    } // 块结束
                                }); // 语句
                            if let Some((h, n)) = del { // 条件分支
                                http::cookie_delete(&h, &n); // 调用 cookie_delete
                                ui.ctx().request_repaint(); // 语句
                            } // 块结束
                        }); // 语句
                } // 块结束
                ui.horizontal_wrapped(|ui| { // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut self.cookie_new_host) // 调用 singleline
                            .desired_width(110.0) // 语句
                            .hint_text("域名 host"), // 语句
                    ); // 块结束
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut self.cookie_new_kv) // 调用 singleline
                            .desired_width(140.0) // 语句
                            .hint_text("name=value"), // 语句
                    ); // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new( // 调用 new
                                RichText::new("添加/覆盖").color(Color32::WHITE).strong(), // 字段 RichText
                            ) // 块结束
                            .fill(brand()) // 语句
                            .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        match self.cookie_new_kv.split_once('=') { // 分支匹配
                            Some((n, v)) // 调用 Some
                                if !self.cookie_new_host.trim().is_empty() // 条件分支
                                    && !n.trim().is_empty() => // 匹配分支
                            { // 语句
                                http::cookie_set(self.cookie_new_host.trim(), n.trim(), v.trim()); // 调用 cookie_set
                                self.cookie_new_kv.clear(); // 语句
                                self.message = "Cookie 已保存".into(); // 语句
                                ui.ctx().request_repaint(); // 语句
                            } // 块结束
                            _ => self.message = "请输入域名与 name=value".into(), // 匹配分支
                        } // 块结束
                    } // 块结束
                }); // 语句
                if ui // 条件分支
                    .add( // 语句
                        egui::Button::new( // 调用 new
                            RichText::new("清空全部 Cookie") // 调用 new
                                .color(Color32::from_rgb(235, 32, 19)), // 语句
                        ) // 块结束
                        .frame(false), // 语句
                    ) // 块结束
                    .clicked() // 语句
                { // 语句
                    http::cookie_clear(); // 调用 cookie_clear
                    self.message = "已清空全部 Cookie".into(); // 语句
                    ui.ctx().request_repaint(); // 语句
                } // 块结束
                ui.add_space(6.0); // 语句
                ui.separator(); // 语句
                ui.add_space(6.0); // 语句
                ui.label( // 语句
                    RichText::new("提示：Mock / 性能测试 / WebSocket / SSE / MQTT / GraphQL 也已提供 CLI 子命令（pmrs mock / perf / ws / sse / mqtt / graphql），详见 README。") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(11.0), // 语句
                ); // 块结束
            }); // 语句
    } // 块结束
    // ---- 主区域 -----------------------------------------------------------
    /// 将「工具 → 从 cURL 导入」文本框里的命令解析为一个新标签。
    fn import_curl_to_tab(&mut self) { // 函数 import_curl_to_tab
        let text = self.curl_input.trim().to_string(); // 变量 text
        if text.is_empty() { // 条件分支
            self.message = "请先粘贴 cURL 命令".into(); // 语句
            return; // 返回
        } // 块结束
        let coll = import_export::import_curl(&text); // 变量 coll
        let Some(item) = coll.item.into_iter().next() else { // 变量 Some
            self.message = "cURL 解析失败".into(); // 语句
            return; // 返回
        }; // 块结束
        let Some(req) = item.request else { // 变量 Some
            self.message = "cURL 中未找到请求".into(); // 语句
            return; // 返回
        }; // 块结束
        let n = self.tabs.len() + 1; // 变量 n
        let mut tab = Tab::new(n); // 变量 tab
        tab.title = "cURL 导入".into(); // 语句
        tab.method = req.method.clone(); // 语句
        tab.url = if req.url.raw.trim().is_empty() { // 语句
            req.url.effective_url() // 语句
        } else { // 语句
            req.url.raw.clone() // 语句
        }; // 块结束
        tab.params = parse_query(&tab.url); // 语句
        tab.path_rows = detect_path_vars(&tab.url); // 语句
        tab.hash = req.url.hash.clone().unwrap_or_default(); // 语句
        tab.headers = req // 语句
            .header // 语句
            .iter() // 语句
            .map(|h| HeaderRow { // 语句
                key: h.key.clone(), // 字段 key
                value: h.value.clone(), // 字段 value
                disabled: h.disabled, // 字段 disabled
            }) // 语句
            .collect(); // 语句
        if tab.headers.is_empty() { // 条件分支
            tab.headers.push(HeaderRow::default()); // 语句
        } // 块结束
        // 请求体
        tab.body_type = "none".into(); // 语句
        if let Some(b) = &req.body { // 条件分支
            match b.mode.as_str() { // 分支匹配
                "raw" => { // 匹配分支
                    tab.body_type = "raw".into(); // 语句
                    tab.body = b.raw.clone().unwrap_or_default(); // 语句
                    let lang = b // 变量 lang
                        .options // 语句
                        .as_ref() // 语句
                        .and_then(|o| o.get("raw")) // 语句
                        .and_then(|r| r.get("language")) // 语句
                        .and_then(|l| l.as_str()) // 语句
                        .unwrap_or("json"); // 语句
                    tab.raw_lang = raw_lang_label(lang).to_string(); // 语句
                } // 块结束
                "urlencoded" => { // 匹配分支
                    tab.body_type = "x-www-form-urlencoded".into(); // 语句
                    tab.url_rows = b // 语句
                        .urlencoded // 语句
                        .iter() // 语句
                        .map(|kv| KVRow { // 语句
                            key: kv.key.clone(), // 字段 key
                            value: kv.value.clone(), // 字段 value
                            disabled: kv.disabled, // 字段 disabled
                            is_file: false, // 字段 is_file
                        }) // 语句
                        .collect(); // 语句
                    if tab.url_rows.is_empty() { // 条件分支
                        tab.url_rows.push(KVRow::default()); // 语句
                    } // 块结束
                } // 块结束
                "formdata" => { // 匹配分支
                    tab.body_type = "form-data".into(); // 语句
                    tab.form_rows = b // 语句
                        .formdata // 语句
                        .iter() // 语句
                        .map(|kv| { // 语句
                            let is_file = kv // 变量 is_file
                                .typ // 语句
                                .as_deref() // 语句
                                .map(|t| t.eq_ignore_ascii_case("file")) // 语句
                                .unwrap_or(false); // 语句
                            let value = if is_file { // 变量 value
                                kv.src // 语句
                                    .as_ref() // 语句
                                    .and_then(|v| v.as_str()) // 语句
                                    .unwrap_or("") // 语句
                                    .to_string() // 语句
                            } else { // 语句
                                kv.value.clone() // 语句
                            }; // 块结束
                            KVRow { key: kv.key.clone(), value, disabled: kv.disabled, is_file } // 语句
                        }) // 语句
                        .collect(); // 语句
                    if tab.form_rows.is_empty() { // 条件分支
                        tab.form_rows.push(KVRow::default()); // 语句
                    } // 块结束
                } // 块结束
                "file" => { // 匹配分支
                    tab.body_type = "binary".into(); // 语句
                    tab.file_path = b.file.as_ref().and_then(|f| f.src.clone()).unwrap_or_default(); // 语句
                } // 块结束
                _ => {} // 匹配分支
            } // 块结束
        } // 块结束
        // 鉴权
        if let Some(a) = &req.auth { // 条件分支
            match a.kind.as_str() { // 分支匹配
                "basic" => { // 匹配分支
                    tab.auth_kind = "basic".into(); // 语句
                    tab.auth_user = a.basic_user().unwrap_or("").to_string(); // 语句
                    tab.auth_pass = a.basic_pass().unwrap_or("").to_string(); // 语句
                } // 块结束
                "bearer" => { // 匹配分支
                    tab.auth_kind = "bearer".into(); // 语句
                    tab.auth_token = a.bearer_token().unwrap_or("").to_string(); // 语句
                } // 块结束
                "apikey" => { // 匹配分支
                    tab.auth_kind = "apikey".into(); // 语句
                    tab.auth_key = a.apikey_key().unwrap_or("").to_string(); // 语句
                    tab.auth_value = a.apikey_value().unwrap_or("").to_string(); // 语句
                } // 块结束
                _ => {} // 匹配分支
            } // 块结束
        } // 块结束
        self.tabs.push(tab); // 语句
        self.active = self.tabs.len() - 1; // 语句
        self.message = "已从 cURL 导入为新标签".into(); // 语句
    } // 块结束
    fn send_ws(&mut self) { // 函数 send_ws
        if self.ws_url.trim().is_empty() { // 条件分支
            self.message = "请输入 WebSocket 地址".into(); // 语句
            return; // 返回
        } // 块结束
        let mut headers = Vec::new(); // 变量 headers
        for line in self.ws_headers.lines() { // 循环
            if let Some((k, v)) = line.split_once(':') { // 条件分支
                let (k, v) = (k.trim(), v.trim()); // 调用 let
                if !k.is_empty() { // 条件分支
                    headers.push((k.to_string(), v.to_string())); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        let cfg = postman_rs::ws::WsConfig { // 变量 cfg
            url: self.ws_url.trim().to_string(), // 字段 url
            messages: self // 语句
                .ws_send // 语句
                .lines() // 语句
                .filter(|l| !l.trim().is_empty()) // 语句
                .map(|s| s.to_string()) // 语句
                .collect(), // 语句
            listen: self.ws_listen as usize, // 字段 listen
            wait_ms: self.ws_wait_ms, // 字段 wait_ms
            headers, // 语句
            subprotocol: if self.ws_subprotocol.trim().is_empty() { // 语句
                None // 语句
            } else { // 语句
                Some(self.ws_subprotocol.trim().to_string()) // 调用 Some
            }, // 块结束
            binary: self.ws_binary, // 字段 binary
        }; // 块结束
        let (tx, rx) = mpsc::channel(); // 调用 let
        self.ws_rx = Some(rx); // 语句
        self.ws_busy = true; // 语句
        self.ws_log = "连接中…".into(); // 语句
        std::thread::spawn(move || { // 调用 spawn
            let r = postman_rs::ws::run_collect(&cfg).map_err(|e| e.to_string()); // 变量 r
            let _ = tx.send(r); // 变量 _
        }); // 语句
    } // 块结束
    fn ui_main(&mut self, ctx: &egui::Context) { // 函数 ui_main
        egui::CentralPanel::default() // 调用 default
            .frame( // 语句
                egui::Frame::none() // 调用 none
                    .fill(app_top()) // 语句
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)), // 语句
            ) // 块结束
            .show(ctx, |ui| { // 语句
                paint_app_gradient(ui); // 调用 paint_app_gradient
                self.ui_request_tabs(ui); // 语句
                ui.add_space(8.0); // 语句
                let mut send = false; // 变量 send
                card(true) // 调用 card
                    .show(ui, |ui| { // 语句
                        send = self.ui_request_card(ui); // 语句
                    }); // 语句
                ui.add_space(12.0); // 语句
                card(true) // 调用 card
                    .show(ui, |ui| { // 语句
                        self.ui_response(ui); // 语句
                    }); // 语句
                ui.add_space(10.0); // 语句
                ui.horizontal(|ui| { // 语句
                    if !self.message.is_empty() { // 条件分支
                        let (r, _) = // 调用 let
                            ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover()); // 语句
                        ui.painter().circle_filled(r.center(), 3.0, brand()); // 语句
                        ui.add_space(4.0); // 语句
                        ui.label(RichText::new(&self.message).color(text_secondary()).size(12.0)); // 语句
                    } // 块结束
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { // 语句
                        if ui // 条件分支
                            .add( // 语句
                                egui::Button::new( // 调用 new
                                    RichText::new("导出集合").color(Color32::WHITE).strong(), // 字段 RichText
                                ) // 块结束
                                    .fill(brand()) // 语句
                                    .rounding(Rounding::same(R_MD)), // 语句
                            ) // 块结束
                            .clicked() // 语句
                        { // 语句
                            self.export_selected(); // 语句
                        } // 块结束
                        egui::Frame::none() // 调用 none
                            .fill(mix(bg_app(), text_primary(), 0.05)) // 语句
                            .rounding(Rounding::same(R_MD)) // 语句
                            .inner_margin(egui::Margin::symmetric(10.0, 5.0)) // 语句
                            .show(ui, |ui| { // 语句
                                ui.add( // 语句
                                    egui::TextEdit::singleline(&mut self.export_path) // 调用 singleline
                                        .frame(false) // 语句
                                        .hint_text("导出路径（留空=默认 exports/）") // 语句
                                        .desired_width(240.0), // 语句
                                ); // 块结束
                            }); // 语句
                    }); // 语句
                }); // 语句
                if send { // 条件分支
                    self.send_active(ctx); // 语句
                } // 块结束
            }); // 语句
    } // 块结束
    fn ui_request_tabs(&mut self, ui: &mut egui::Ui) { // 函数 ui_request_tabs
        let mut select: Option<usize> = None; // 变量 select
        let mut close: Option<usize> = None; // 变量 close
        let mut add = false; // 变量 add
        let mut start_rename: Option<(usize, String)> = None; // 变量 start_rename
        let mut commit_rename: Option<(usize, String)> = None; // 变量 commit_rename
        let mut cancel_rename = false; // 变量 cancel_rename
        ui.horizontal_wrapped(|ui| { // 语句
            for i in 0..self.tabs.len() { // 循环
                let title = self.tabs[i].title.clone(); // 变量 title
                let active = self.active == i; // 变量 active
                let bg = if active { bg_panel() } else { Color32::TRANSPARENT }; // 变量 bg
                let border_c = if active { brand() } else { Color32::TRANSPARENT }; // 变量 border_c
                let tc = if active { brand() } else { text_secondary() }; // 变量 tc
                let renaming = self.renaming_tab == Some(i); // 变量 renaming
                egui::Frame::none() // 调用 none
                    .fill(bg) // 语句
                    .stroke(Stroke::new(1.0_f32, border_c)) // 语句
                    .rounding(Rounding::same(R_MD)) // 语句
                    .inner_margin(egui::Margin::symmetric(10.0, 5.0)) // 语句
                    .show(ui, |ui| { // 语句
                        ui.horizontal(|ui| { // 语句
                            if renaming { // 条件分支
                                let resp = ui.add( // 变量 resp
                                    egui::TextEdit::singleline(&mut self.rename_tab_text) // 调用 singleline
                                        .desired_width(130.0) // 语句
                                        .font(egui::TextStyle::Body), // 语句
                                ); // 块结束
                                if self.rename_tab_focus { // 条件分支
                                    resp.request_focus(); // 语句
                                    self.rename_tab_focus = false; // 语句
                                } // 块结束
                                let enter = ui.input(|x| x.key_pressed(egui::Key::Enter)); // 变量 enter
                                let esc = ui.input(|x| x.key_pressed(egui::Key::Escape)); // 变量 esc
                                if enter || resp.lost_focus() { // 条件分支
                                    commit_rename = Some((i, self.rename_tab_text.clone())); // 语句
                                } // 块结束
                                if esc { // 条件分支
                                    cancel_rename = true; // 语句
                                } // 块结束
                            } else { // 语句
                                let resp = ui.add( // 变量 resp
                                    egui::Button::new( // 调用 new
                                        RichText::new(&title).color(tc).size(12.5).strong(), // 字段 RichText
                                    ) // 块结束
                                    .frame(false), // 语句
                                ); // 块结束
                                if resp.double_clicked() { // 条件分支
                                    select = Some(i); // 语句
                                    start_rename = Some((i, title.clone())); // 语句
                                } else if resp.clicked() { // 条件分支
                                    if active { // 条件分支
                                        // 点击当前标签标题 → 进入重命名
                                        start_rename = Some((i, title.clone())); // 语句
                                    } else { // 语句
                                        select = Some(i); // 语句
                                    } // 块结束
                                } // 块结束
                            } // 块结束
                            if ui // 条件分支
                                .add( // 语句
                                    egui::Button::new( // 调用 new
                                        RichText::new("×") // 调用 new
                                            .color(if active { brand() } else { text_secondary() }) // 语句
                                            .size(13.0), // 语句
                                    ) // 块结束
                                    .frame(false), // 语句
                                ) // 块结束
                                .clicked() // 语句
                            { // 语句
                                close = Some(i); // 语句
                            } // 块结束
                        }); // 语句
                    }); // 语句
            } // 块结束
            if ui // 条件分支
                .add( // 语句
                    egui::Button::new(RichText::new("＋ 新建标签").color(brand())) // 调用 new
                        .fill(brand_soft()) // 语句
                        .rounding(Rounding::same(R_MD)), // 语句
                ) // 块结束
                .clicked() // 语句
            { // 语句
                add = true; // 语句
            } // 块结束
        }); // 语句
        if let Some(i) = select { // 条件分支
            self.active = i; // 语句
        } // 块结束
        if let Some((i, t)) = start_rename { // 条件分支
            self.renaming_tab = Some(i); // 语句
            self.rename_tab_text = t; // 语句
            self.rename_tab_focus = true; // 语句
        } // 块结束
        if let Some((i, t)) = commit_rename { // 条件分支
            let t = t.trim().to_string(); // 变量 t
            if !t.is_empty() && i < self.tabs.len() { // 条件分支
                self.tabs[i].title = t; // 语句
            } // 块结束
            self.renaming_tab = None; // 语句
        } // 块结束
        if cancel_rename { // 条件分支
            self.renaming_tab = None; // 语句
        } // 块结束
        if let Some(i) = close { // 条件分支
            if self.tabs.len() > 1 { // 条件分支
                self.pending_delete = Some(PendingDelete::Tab { // 语句
                    idx: i, // 字段 idx
                    title: self.tabs[i].title.clone(), // 字段 title
                }); // 语句
            } // 块结束
            if self.renaming_tab == Some(i) { // 条件分支
                self.renaming_tab = None; // 语句
            } // 块结束
        } // 块结束
        if add { // 条件分支
            self.open_new_request_dialog(); // 语句
        } // 块结束
    } // 块结束
    fn ui_request_card(&mut self, ui: &mut egui::Ui) -> bool { // 函数 ui_request_card
        let mut send = false; // 变量 send
        let active = self.active; // 变量 active
        let tab = &mut self.tabs[active]; // 变量 tab
        ui.horizontal(|ui| { // 语句
            egui::ComboBox::from_id_salt("method") // 调用 from_id_salt
                .width(92.0) // 语句
                .selected_text( // 语句
                    RichText::new(&tab.method) // 调用 new
                        .color(method_color(&tab.method)) // 语句
                        .size(13.0) // 语句
                        .strong(), // 语句
                ) // 块结束
                .show_ui(ui, |ui| { // 语句
                    for m in [ // 循环
                        "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "COPY", "LINK", // 语句
                        "UNLINK", "PURGE", "LOCK", "UNLOCK", "PROPFIND", "VIEW", "TRACE", // 语句
                    ] { // 语句
                        ui.selectable_value( // 语句
                            &mut tab.method, // 语句
                            m.to_string(), // 语句
                            RichText::new(m).color(method_color(m)).strong(), // 字段 RichText
                        ); // 块结束
                    } // 块结束
                }); // 语句
            let url_w = (ui.available_width() - 262.0).max(160.0); // 变量 url_w
            let resp = egui::Frame::none() // 变量 resp
                .fill(mix(bg_panel(), text_primary(), 0.04)) // 语句
                .rounding(Rounding::same(R_MD)) // 语句
                .inner_margin(egui::Margin::symmetric(10.0, 6.0)) // 语句
                .show(ui, |ui| { // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut tab.url) // 调用 singleline
                            .frame(false) // 语句
                            .desired_width(url_w) // 语句
                            .font(egui::TextStyle::Monospace) // 语句
                            .hint_text("https://api.example.com/path  （支持 {{变量}}）"), // 语句
                    ) // 块结束
                }) // 语句
                .inner; // 语句
            if resp.changed() { // 条件分支
                tab.params = parse_query(&tab.url); // 语句
                tab.path_rows = detect_path_vars(&tab.url); // 语句
            } // 块结束
            // 把「复制 / 导出 / 导入」等 cURL 相关操作收拢到一个下拉菜单，点击 cURL 按钮即可使用。
            let mut curl_copy = false; // 变量 curl_copy
            let mut curl_export = false; // 变量 curl_export
            let mut curl_import = false; // 变量 curl_import
            let curl_resp = ui // 变量 curl_resp
                .menu_button(RichText::new("cURL").color(brand()).strong(), |ui| { // 语句
                    ui.set_min_width(190.0); // 语句
                    if ui.button("复制当前请求为 cURL").clicked() { // 条件分支
                        curl_copy = true; // 语句
                        ui.close_menu(); // 语句
                    } // 块结束
                    if ui.button("导出为 cURL 脚本文件…").clicked() { // 条件分支
                        curl_export = true; // 语句
                        ui.close_menu(); // 语句
                    } // 块结束
                    ui.separator(); // 语句
                    if ui.button("从 cURL 导入（粘贴）…").clicked() { // 条件分支
                        curl_import = true; // 语句
                        ui.close_menu(); // 语句
                    } // 块结束
                }); // 语句
            curl_resp.response.on_hover_text("cURL：复制 / 导出 / 导入"); // 语句
            if curl_copy { // 条件分支
                let code = codegen::generate(&tab.build_request(), "curl").unwrap_or_default(); // 变量 code
                ui.output_mut(|o| o.copied_text = code); // 语句
                tab.status = "已复制 cURL 到剪贴板".into(); // 语句
            } // 块结束
            if curl_export { // 条件分支
                let code = codegen::generate(&tab.build_request(), "curl").unwrap_or_default(); // 变量 code
                let dir = storage::home().join("exports"); // 变量 dir
                let _ = std::fs::create_dir_all(&dir); // 变量 _
                let path = dir.join(format!( // 变量 path
                    "curl_{}.sh", // 语句
                    chrono::Local::now().format("%Y%m%d_%H%M%S") // 调用 now
                )); // 语句
                match std::fs::write(&path, code.as_bytes()) { // 分支匹配
                    Ok(_) => tab.status = format!("已导出 cURL → {}", path.display()), // 匹配分支
                    Err(e) => tab.status = format!("导出失败：{}", e), // 匹配分支
                } // 块结束
            } // 块结束
            if curl_import { // 条件分支
                self.curl_import_open = true; // 语句
            } // 块结束
            let label = if tab.inflight { "发送中…" } else { "发送" }; // 变量 label
            if ui // 条件分支
                .add_enabled( // 语句
                    !tab.inflight, // 语句
                    egui::Button::new(RichText::new(label).color(Color32::WHITE).strong()) // 调用 new
                        .fill(send_blue()) // 语句
                        .rounding(Rounding::same(R_MD)) // 语句
                        .min_size(egui::vec2(96.0, 32.0)), // 语句
                ) // 块结束
                .clicked() // 语句
            { // 语句
                send = true; // 语句
            } // 块结束
            if tab.inflight // 条件分支
                && ui // 语句
                    .add( // 语句
                        egui::Button::new( // 调用 new
                            RichText::new("取消").color(Color32::from_rgb(235, 32, 19)), // 字段 RichText
                        ) // 块结束
                        .fill(brand_soft()) // 语句
                        .rounding(Rounding::same(R_MD)) // 语句
                        .min_size(egui::vec2(64.0, 32.0)), // 语句
                    ) // 块结束
                    .clicked() // 语句
            { // 语句
                tab.rx = None; // 语句
                tab.inflight = false; // 语句
                tab.status = "已取消".into(); // 语句
            } // 块结束
        }); // 语句
        ui.add_space(6.0); // 语句
        underline_tabs( // 调用 underline_tabs
            ui, // 语句
            &mut tab.req_subtab, // 语句
            &["参数", "授权", "请求头", "请求体", "测试", "脚本", "设置", "描述"], // 语句
        ); // 块结束
        ui.add_space(2.0); // 语句
        let rect = ui.available_rect_before_wrap(); // 变量 rect
        ui.painter().hline( // 语句
            rect.left()..=rect.right(), // 语句
            rect.top() + 1.0, // 语句
            Stroke::new(1.0_f32, border()), // 字段 Stroke
        ); // 块结束
        ui.add_space(8.0); // 语句
        // 记录卡片真实内容宽度；ScrollArea 内部的 available_width 会被放大，
        // 这里把子区域宽度钉回真实值，保证"各占一半"等布局按可见宽度计算。
        let content_w = ui.available_width(); // 变量 content_w
        egui::ScrollArea::vertical() // 调用 vertical
            .id_salt("reqbody") // 语句
            .max_height(300.0) // 语句
            .auto_shrink([false, false]) // 语句
            .show(ui, |ui| { // 语句
                ui.set_width(content_w); // 语句
                match tab.req_subtab { // 分支匹配
                    0 => Self::ui_params(tab, ui), // 匹配分支
                    1 => Self::ui_auth(tab, ui), // 匹配分支
                    2 => Self::ui_headers(tab, ui), // 匹配分支
                    3 => Self::ui_body(tab, ui), // 匹配分支
                    4 => Self::ui_tests(tab, ui), // 匹配分支
                    5 => Self::ui_scripts(tab, ui), // 匹配分支
                    6 => Self::ui_settings(tab, ui), // 匹配分支
                    7 => Self::ui_desc(tab, ui), // 匹配分支
                    _ => Self::ui_settings(tab, ui), // 匹配分支
                } // 块结束
            }); // 语句
        send // 语句
    } // 块结束
    fn ui_params(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_params
        ui.label( // 语句
            RichText::new("查询参数（URL ? 之后的部分；取消勾选=禁用，发送时不带上但仍保留在集合中）") // 调用 new
                .color(text_secondary()) // 语句
                .size(11.0), // 语句
        ); // 块结束
        ui.add_space(4.0); // 语句
        let mut changed = false; // 变量 changed
        const ROW_H: f32 = 22.0; // 常量 ROW_H
        const CHECK_W: f32 = 22.0; // 常量 CHECK_W
        const DEL_W: f32 = 26.0; // 常量 DEL_W
        // 键、值各占一半：以「启用」勾选框之后的可用宽度二等分，右侧预留「×」按钮。
        ui.horizontal(|ui| { // 语句
            ui.add_space(CHECK_W); // 语句
            let rem = (ui.available_width() - DEL_W).max(120.0); // 变量 rem
            let half = ((rem - 8.0) / 2.0).max(60.0); // 变量 half
            ui.add_sized( // 语句
                [half, 16.0], // 语句
                egui::Label::new(RichText::new("键").color(text_secondary()).size(11.0)), // 字段 egui
            ); // 块结束
            ui.add_sized( // 语句
                [half, 16.0], // 语句
                egui::Label::new(RichText::new("值").color(text_secondary()).size(11.0)), // 字段 egui
            ); // 块结束
        }); // 语句
        ui.add_space(2.0); // 语句
        let mut remove: Option<usize> = None; // 变量 remove
        for i in 0..tab.params.len() { // 循环
            let mut remove_this = false; // 变量 remove_this
            let mut row_changed = false; // 变量 row_changed
            { // 语句
                let r = &mut tab.params[i]; // 变量 r
                ui.horizontal(|ui| { // 语句
                    let mut enabled = !r.disabled; // 变量 enabled
                    if ui.checkbox(&mut enabled, "").changed() { // 条件分支
                        r.disabled = !enabled; // 语句
                        row_changed = true; // 语句
                    } // 块结束
                    let rem = (ui.available_width() - DEL_W).max(120.0); // 变量 rem
                    let half = ((rem - 8.0) / 2.0).max(60.0); // 变量 half
                    if ui // 条件分支
                        .add_sized( // 语句
                            [half, ROW_H], // 语句
                            egui::TextEdit::singleline(&mut r.key).hint_text("键"), // 字段 egui
                        ) // 块结束
                        .changed() // 语句
                    { // 语句
                        row_changed = true; // 语句
                    } // 块结束
                    if ui // 条件分支
                        .add_sized( // 语句
                            [half, ROW_H], // 语句
                            egui::TextEdit::singleline(&mut r.value).hint_text("值"), // 字段 egui
                        ) // 块结束
                        .changed() // 语句
                    { // 语句
                        row_changed = true; // 语句
                    } // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("×").color(text_secondary())) // 调用 new
                                .frame(false), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        remove_this = true; // 语句
                    } // 块结束
                }); // 语句
            } // 块结束
            if row_changed { // 条件分支
                changed = true; // 语句
            } // 块结束
            if remove_this { // 条件分支
                remove = Some(i); // 语句
            } // 块结束
        } // 块结束
        if let Some(i) = remove { // 条件分支
            tab.params.remove(i); // 语句
            changed = true; // 语句
        } // 块结束
        if ui // 条件分支
            .add( // 语句
                egui::Button::new(RichText::new("＋ 添加参数").color(brand())) // 调用 new
                    .fill(brand_soft()) // 语句
                    .rounding(Rounding::same(6.0)), // 语句
            ) // 块结束
            .clicked() // 语句
        { // 语句
            tab.params.push(KVRow::default()); // 语句
        } // 块结束
        // 路径变量（:name）
        if !tab.path_rows.is_empty() { // 条件分支
            ui.add_space(10.0); // 语句
            ui.label( // 语句
                RichText::new("路径变量（URL 中的 :名称；发送时用这里的值替换）") // 调用 new
                    .color(text_secondary()) // 语句
                    .size(11.0), // 语句
            ); // 块结束
            ui.add_space(4.0); // 语句
            // 名称、值各占一半：占满可用宽度，右侧预留「×」按钮。
            ui.horizontal(|ui| { // 语句
                let rem = (ui.available_width() - DEL_W).max(120.0); // 变量 rem
                let half = ((rem - 8.0) / 2.0).max(60.0); // 变量 half
                ui.add_sized( // 语句
                    [half, 16.0], // 语句
                    egui::Label::new(RichText::new("名称").color(text_secondary()).size(11.0)), // 字段 egui
                ); // 块结束
                ui.add_sized( // 语句
                    [half, 16.0], // 语句
                    egui::Label::new(RichText::new("值").color(text_secondary()).size(11.0)), // 字段 egui
                ); // 块结束
            }); // 语句
            ui.add_space(2.0); // 语句
            let mut rmv: Option<usize> = None; // 变量 rmv
            for (i, r) in tab.path_rows.iter_mut().enumerate() { // 循环
                ui.horizontal(|ui| { // 语句
                    let rem = (ui.available_width() - DEL_W).max(120.0); // 变量 rem
                    let half = ((rem - 8.0) / 2.0).max(60.0); // 变量 half
                    ui.add_sized( // 语句
                        [half, ROW_H], // 语句
                        egui::TextEdit::singleline(&mut r.key).hint_text("名称"), // 字段 egui
                    ); // 块结束
                    ui.add_sized( // 语句
                        [half, ROW_H], // 语句
                        egui::TextEdit::singleline(&mut r.value).hint_text("值"), // 字段 egui
                    ); // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("×").color(text_secondary())) // 调用 new
                                .frame(false), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        rmv = Some(i); // 语句
                    } // 块结束
                }); // 语句
            } // 块结束
            if let Some(i) = rmv { // 条件分支
                tab.path_rows.remove(i); // 语句
            } // 块结束
        } // 块结束
        // URL 片段 hash
        ui.add_space(10.0); // 语句
        ui.horizontal(|ui| { // 语句
            ui.label(RichText::new("URL 片段（#hash）").color(text_secondary()).size(12.0)); // 语句
            if ui // 条件分支
                .add( // 语句
                    egui::TextEdit::singleline(&mut tab.hash) // 调用 singleline
                        .desired_width(240.0) // 语句
                        .hint_text("锚点/片段，可留空"), // 语句
                ) // 块结束
                .changed() // 语句
            { // 语句
                changed = true; // 语句
            } // 块结束
        }); // 语句
        if changed { // 条件分支
            rebuild_url(tab); // 调用 rebuild_url
        } // 块结束
    } // 块结束
    fn ui_auth(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_auth
        ui.horizontal(|ui| { // 语句
            ui.label(RichText::new("授权类型").color(text_secondary()).size(12.0)); // 语句
            let cur = match tab.auth_kind.as_str() { // 变量 cur
                "bearer" => "Bearer 令牌", // 匹配分支
                "basic" => "基本认证", // 匹配分支
                "apikey" => "API 密钥", // 匹配分支
                "oauth2" => "OAuth 2.0", // 匹配分支
                "oauth1" => "OAuth 1.0", // 匹配分支
                "awsv4" => "AWS 签名 (SigV4)", // 匹配分支
                "jwt" => "JWT Bearer", // 匹配分支
                "digest" => "Digest 摘要认证", // 匹配分支
                "ntlm" => "NTLM（不支持）", // 匹配分支
                _ => "无", // 匹配分支
            }; // 块结束
            egui::ComboBox::from_id_salt("auth_kind") // 调用 from_id_salt
                .selected_text(cur) // 语句
                .show_ui(ui, |ui| { // 语句
                    ui.selectable_value(&mut tab.auth_kind, "none".into(), "无"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "bearer".into(), "Bearer 令牌"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "basic".into(), "基本认证"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "apikey".into(), "API 密钥"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "oauth2".into(), "OAuth 2.0"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "oauth1".into(), "OAuth 1.0"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "awsv4".into(), "AWS 签名 (SigV4)"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "jwt".into(), "JWT Bearer"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "digest".into(), "Digest 摘要认证"); // 语句
                    ui.selectable_value(&mut tab.auth_kind, "ntlm".into(), "NTLM（不支持）"); // 语句
                }); // 语句
        }); // 语句
        ui.add_space(6.0); // 语句
        match tab.auth_kind.as_str() { // 分支匹配
            "bearer" => { // 匹配分支
                ui.label(RichText::new("令牌").color(text_secondary()).size(12.0)); // 语句
                ui.add( // 语句
                    egui::TextEdit::singleline(&mut tab.auth_token) // 调用 singleline
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text("粘贴 Bearer 令牌"), // 语句
                ); // 块结束
                ui.label( // 语句
                    RichText::new("发送时会添加请求头：Authorization: Bearer <令牌>") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(11.0), // 语句
                ); // 块结束
            } // 块结束
            "basic" => { // 匹配分支
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("用户名").color(text_secondary()).size(12.0)); // 语句
                    ui.add(egui::TextEdit::singleline(&mut tab.auth_user).desired_width(200.0)); // 语句
                }); // 语句
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("密码").color(text_secondary()).size(12.0)); // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut tab.auth_pass) // 调用 singleline
                            .desired_width(200.0) // 语句
                            .password(true), // 语句
                    ); // 块结束
                }); // 语句
            } // 块结束
            "apikey" => { // 匹配分支
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("请求头名称").color(text_secondary()).size(12.0)); // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut tab.auth_key) // 调用 singleline
                            .desired_width(200.0) // 语句
                            .hint_text("X-API-Key"), // 语句
                    ); // 块结束
                }); // 语句
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("值").color(text_secondary()).size(12.0)); // 语句
                    ui.add(egui::TextEdit::singleline(&mut tab.auth_value).desired_width(260.0)); // 语句
                }); // 语句
            } // 块结束
            "oauth2" => { // 匹配分支
                let grant = tab // 变量 grant
                    .oauth2 // 语句
                    .get("grant_type") // 语句
                    .cloned() // 语句
                    .unwrap_or_else(|| "client_credentials".into()); // 语句
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("授权流程").color(text_secondary()).size(12.0)); // 语句
                    egui::ComboBox::from_id_salt("oauth2_grant") // 调用 from_id_salt
                        .selected_text(grant_label(&grant)) // 语句
                        .show_ui(ui, |ui| { // 语句
                            for g in ["client_credentials", "password", "refresh_token"] { // 循环
                                let slot = tab // 变量 slot
                                    .oauth2 // 语句
                                    .entry("grant_type".to_string()) // 语句
                                    .or_insert_with(|| g.to_string()); // 语句
                                ui.selectable_value(slot, g.to_string(), grant_label(g)); // 语句
                            } // 块结束
                        }); // 语句
                }); // 语句
                ui.add_space(4.0); // 语句
                oauth2_field(ui, &mut tab.oauth2, "令牌地址 (accessTokenUrl)", "accessTokenUrl", false); // 调用 oauth2_field
                oauth2_field(ui, &mut tab.oauth2, "client_id", "client_id", false); // 调用 oauth2_field
                oauth2_field(ui, &mut tab.oauth2, "client_secret", "client_secret", true); // 调用 oauth2_field
                oauth2_field(ui, &mut tab.oauth2, "scope", "scope", false); // 调用 oauth2_field
                if grant == "password" { // 条件分支
                    oauth2_field(ui, &mut tab.oauth2, "用户名", "username", false); // 调用 oauth2_field
                    oauth2_field(ui, &mut tab.oauth2, "密码", "password", true); // 调用 oauth2_field
                } // 块结束
                if grant == "refresh_token" { // 条件分支
                    oauth2_field(ui, &mut tab.oauth2, "refresh_token", "refresh_token", true); // 调用 oauth2_field
                } // 块结束
                oauth2_field(ui, &mut tab.oauth2, "访问令牌 (可选，填了直接使用)", "accessToken", true); // 调用 oauth2_field
                ui.label( // 语句
                    RichText::new("发送时会向令牌地址请求访问令牌，并添加响应头 Authorization: Bearer <token>") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(11.0), // 语句
                ); // 块结束
            } // 块结束
            "oauth1" | "awsv4" | "jwt" | "digest" | "ntlm" => { // 匹配分支
                if tab.auth_extra.is_empty() { // 条件分支
                    for k in default_auth_keys(&tab.auth_kind) { // 循环
                        tab.auth_extra.push(KVRow { // 语句
                            key: k.to_string(), // 字段 key
                            ..Default::default() // 语句
                        }); // 语句
                    } // 块结束
                } // 块结束
                if tab.auth_kind == "ntlm" { // 条件分支
                    ui.label( // 语句
                        RichText::new("NTLM 握手协议较复杂，暂未实现；请改用 Basic / Bearer。") // 调用 new
                            .color(Color32::from_rgb(235, 32, 19)) // 语句
                            .size(11.0), // 语句
                    ); // 块结束
                } // 块结束
                // 键、值各占一半，填满可用宽度。
                ui.horizontal(|ui| { // 语句
                    let half = ((ui.available_width() - 8.0) / 2.0).max(60.0); // 变量 half
                    ui.add_sized( // 语句
                        [half, 16.0], // 语句
                        egui::Label::new(RichText::new("键").color(text_secondary()).size(11.0)), // 字段 egui
                    ); // 块结束
                    ui.add_sized( // 语句
                        [half, 16.0], // 语句
                        egui::Label::new(RichText::new("值").color(text_secondary()).size(11.0)), // 字段 egui
                    ); // 块结束
                }); // 语句
                ui.add_space(2.0); // 语句
                for r in tab.auth_extra.iter_mut() { // 循环
                    ui.horizontal(|ui| { // 语句
                        let half = ((ui.available_width() - 8.0) / 2.0).max(60.0); // 变量 half
                        ui.add_sized( // 语句
                            [half, 22.0], // 语句
                            egui::TextEdit::singleline(&mut r.key).hint_text("键"), // 字段 egui
                        ); // 块结束
                        ui.add_sized( // 语句
                            [half, 22.0], // 语句
                            egui::TextEdit::singleline(&mut r.value).hint_text("值"), // 字段 egui
                        ); // 块结束
                    }); // 语句
                } // 块结束
            } // 块结束
            _ => { // 匹配分支
                ui.label(RichText::new("此请求不使用授权。").color(text_secondary())); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    fn ui_headers(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_headers
        // 请求头以「键/值输入框」呈现（含默认请求头），不使用“启用”勾选框。
        // 键、值各占一半：用 ui.columns(2) 在「×按钮之前」的剩余区域内强制二等分。
        const ROW_H: f32 = 22.0; // 常量 ROW_H
        const RESERVE_DEL: f32 = 26.0; // 预留给右侧「×」按钮
        // 表头
        ui.horizontal(|ui| { // 语句
            let rem = (ui.available_width() - RESERVE_DEL).max(120.0); // 变量 rem
            ui.allocate_ui_with_layout( // 语句
                egui::vec2(rem, 16.0), // 字段 egui
                egui::Layout::left_to_right(egui::Align::Center), // 字段 egui
                |ui| { // 语句
                    ui.columns(2, |cols| { // 语句
                        let w0 = cols[0].available_width(); // 变量 w0
                        cols[0].add_sized( // 语句
                            [w0, 16.0], // 语句
                            egui::Label::new( // 调用 new
                                RichText::new("键").color(text_secondary()).size(11.0), // 字段 RichText
                            ), // 语句
                        ); // 块结束
                        let w1 = cols[1].available_width(); // 变量 w1
                        cols[1].add_sized( // 语句
                            [w1, 16.0], // 语句
                            egui::Label::new( // 调用 new
                                RichText::new("值").color(text_secondary()).size(11.0), // 字段 RichText
                            ), // 语句
                        ); // 块结束
                    }); // 语句
                }, // 块结束
            ); // 块结束
        }); // 语句
        ui.add_space(2.0); // 语句
        let mut remove: Option<usize> = None; // 变量 remove
        for i in 0..tab.headers.len() { // 循环
            { // 语句
                let h = &mut tab.headers[i]; // 变量 h
                let key = &mut h.key; // 变量 key
                let value = &mut h.value; // 变量 value
                ui.horizontal(|ui| { // 语句
                    let rem = (ui.available_width() - RESERVE_DEL).max(120.0); // 变量 rem
                    ui.allocate_ui_with_layout( // 语句
                        egui::vec2(rem, ROW_H), // 字段 egui
                        egui::Layout::left_to_right(egui::Align::Center), // 字段 egui
                        |ui| { // 语句
                            ui.columns(2, |cols| { // 语句
                                let w0 = cols[0].available_width(); // 变量 w0
                                cols[0].add_sized( // 语句
                                    [w0, ROW_H], // 语句
                                    egui::TextEdit::singleline(key).hint_text("请求头名"), // 字段 egui
                                ); // 块结束
                                let w1 = cols[1].available_width(); // 变量 w1
                                cols[1].add_sized( // 语句
                                    [w1, ROW_H], // 语句
                                    egui::TextEdit::singleline(value).hint_text("值"), // 字段 egui
                                ); // 块结束
                            }); // 语句
                        }, // 块结束
                    ); // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("×").color(text_secondary())) // 调用 new
                                .frame(false), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        remove = Some(i); // 语句
                    } // 块结束
                }); // 语句
            } // 块结束
        } // 块结束
        if let Some(i) = remove { // 条件分支
            tab.headers.remove(i); // 语句
        } // 块结束
        if ui // 条件分支
            .add( // 语句
                egui::Button::new(RichText::new("＋ 添加请求头").color(brand())) // 调用 new
                    .fill(brand_soft()) // 语句
                    .rounding(Rounding::same(6.0)), // 语句
            ) // 块结束
            .clicked() // 语句
        { // 语句
            tab.headers.push(HeaderRow::default()); // 语句
        } // 块结束
    } // 块结束
    fn ui_body(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_body
        // 类型选择行（对齐 Postman：none / form-data / x-www-form-urlencoded / raw / binary / GraphQL）
        ui.horizontal(|ui| { // 语句
            ui.label(RichText::new("类型").color(text_secondary()).size(12.0)); // 语句
            egui::ComboBox::from_id_salt("bodytype") // 调用 from_id_salt
                .width(210.0) // 语句
                .selected_text(body_type_label(&tab.body_type)) // 语句
                .show_ui(ui, |ui| { // 语句
                    for t in [ // 循环
                        "none", // 语句
                        "form-data", // 语句
                        "x-www-form-urlencoded", // 语句
                        "raw", // 语句
                        "binary", // 语句
                        "graphql", // 语句
                    ] { // 语句
                        ui.selectable_value(&mut tab.body_type, t.to_string(), body_type_label(t)); // 语句
                    } // 块结束
                }); // 语句
            if tab.body_type == "raw" { // 条件分支
                ui.add_space(10.0); // 语句
                ui.label(RichText::new("语言").color(text_secondary()).size(12.0)); // 语句
                egui::ComboBox::from_id_salt("rawlang") // 调用 from_id_salt
                    .width(120.0) // 语句
                    .selected_text(&tab.raw_lang) // 语句
                    .show_ui(ui, |ui| { // 语句
                        for l in ["Text", "JSON", "XML", "HTML", "JavaScript"] { // 循环
                            ui.selectable_value(&mut tab.raw_lang, l.to_string(), l); // 语句
                        } // 块结束
                    }); // 语句
                if tab.raw_lang == "JSON" && !tab.body.trim().is_empty() { // 条件分支
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("格式化").color(brand())) // 调用 new
                                .fill(brand_soft()) // 语句
                                .rounding(Rounding::same(6.0)), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&tab.body) { // 条件分支
                            if let Ok(p) = serde_json::to_string_pretty(&v) { // 条件分支
                                tab.body = p; // 语句
                            } // 块结束
                        } // 块结束
                    } // 块结束
                } // 块结束
            } // 块结束
        }); // 语句
        ui.add_space(8.0); // 语句
        match tab.body_type.as_str() { // 分支匹配
            "none" => { // 匹配分支
                ui.label( // 语句
                    RichText::new( // 调用 new
                        "该请求没有请求体。可在上方「类型」中选择 raw / 表单 / 二进制 / GraphQL。", // 语句
                    ) // 块结束
                    .color(text_secondary()) // 语句
                    .size(12.0), // 语句
                ); // 块结束
            } // 块结束
            "raw" => { // 匹配分支
                ui.add( // 语句
                    egui::TextEdit::multiline(&mut tab.body) // 调用 multiline
                        .code_editor() // 语句
                        .desired_rows(12) // 语句
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text(if tab.raw_lang == "JSON" { // 语句
                            "{\n  \"键\": \"值\"\n}" // 语句
                        } else { // 语句
                            "请求体内容…" // 语句
                        }), // 语句
                ); // 块结束
            } // 块结束
            "x-www-form-urlencoded" => { // 匹配分支
                Self::ui_kv_editor(ui, &mut tab.url_rows, false, "urlenc"); // 调用 ui_kv_editor
            } // 块结束
            "form-data" => { // 匹配分支
                Self::ui_kv_editor(ui, &mut tab.form_rows, true, "formdata"); // 调用 ui_kv_editor
            } // 块结束
            "binary" => { // 匹配分支
                ui.label( // 语句
                    RichText::new("选择一个本地文件作为原始请求体（binary）。") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(12.0), // 语句
                ); // 块结束
                ui.add_space(4.0); // 语句
                ui.horizontal(|ui| { // 语句
                    ui.label(RichText::new("文件路径").color(text_secondary()).size(12.0)); // 语句
                    ui.add( // 语句
                        egui::TextEdit::singleline(&mut tab.file_path) // 调用 singleline
                            .desired_width(f32::INFINITY) // 语句
                            .hint_text("C:\\path\\to\\file.bin（留空则不发送请求体）"), // 语句
                    ); // 块结束
                }); // 语句
            } // 块结束
            "graphql" => { // 匹配分支
                ui.label(RichText::new("查询（Query）").color(text_secondary()).size(12.0)); // 语句
                ui.add( // 语句
                    egui::TextEdit::multiline(&mut tab.gql_query) // 调用 multiline
                        .code_editor() // 语句
                        .desired_rows(8) // 语句
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text("query {\n  ...\n}"), // 语句
                ); // 块结束
                ui.add_space(6.0); // 语句
                ui.label( // 语句
                    RichText::new("变量（Variables，JSON）") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(12.0), // 语句
                ); // 块结束
                ui.add( // 语句
                    egui::TextEdit::multiline(&mut tab.gql_vars) // 调用 multiline
                        .code_editor() // 语句
                        .desired_rows(5) // 语句
                        .desired_width(f32::INFINITY) // 语句
                        .hint_text("{\n  \"变量\": \"值\"\n}"), // 语句
                ); // 块结束
            } // 块结束
            _ => {} // 匹配分支
        } // 块结束
    } // 块结束
    /// 表单类请求体编辑器（urlencoded / form-data 通用）。
    fn ui_kv_editor( // 函数 ui_kv_editor
        ui: &mut egui::Ui, // 字段 ui
        rows: &mut Vec<KVRow>, // 字段 rows
        allow_file: bool, // 字段 allow_file
        id: &str, // 字段 id
    ) { // 语句
        const ROW_H: f32 = 22.0; // 常量 ROW_H
        const CHECK_W: f32 = 22.0; // 常量 CHECK_W
        const DEL_W: f32 = 26.0; // 常量 DEL_W
        let reserved = DEL_W + if allow_file { 84.0 } else { 0.0 }; // 变量 reserved
        // 键、值各占一半：勾选框之后、类型/删除之前，剩余宽度二等分并填满。
        ui.horizontal(|ui| { // 语句
            ui.add_space(CHECK_W); // 语句
            let rem = (ui.available_width() - reserved).max(160.0); // 变量 rem
            let half = ((rem - 8.0) / 2.0).max(60.0); // 变量 half
            ui.add_sized( // 语句
                [half, 16.0], // 语句
                egui::Label::new(RichText::new("键").color(text_secondary()).size(11.0)), // 字段 egui
            ); // 块结束
            ui.add_sized( // 语句
                [half, 16.0], // 语句
                egui::Label::new(RichText::new("值").color(text_secondary()).size(11.0)), // 字段 egui
            ); // 块结束
        }); // 语句
        ui.add_space(2.0); // 语句
        let mut remove: Option<usize> = None; // 变量 remove
        for (i, r) in rows.iter_mut().enumerate() { // 循环
            ui.horizontal(|ui| { // 语句
                let mut enabled = !r.disabled; // 变量 enabled
                if ui.checkbox(&mut enabled, "").changed() { // 条件分支
                    r.disabled = !enabled; // 语句
                } // 块结束
                let rem = (ui.available_width() - reserved).max(160.0); // 变量 rem
                let half = ((rem - 8.0) / 2.0).max(60.0); // 变量 half
                ui.add_sized( // 语句
                    [half, ROW_H], // 语句
                    egui::TextEdit::singleline(&mut r.key).hint_text("键"), // 字段 egui
                ); // 块结束
                let hint = if allow_file && r.is_file { // 变量 hint
                    "文件路径" // 语句
                } else { // 语句
                    "值" // 语句
                }; // 块结束
                ui.add_sized( // 语句
                    [half, ROW_H], // 语句
                    egui::TextEdit::singleline(&mut r.value).hint_text(hint), // 字段 egui
                ); // 块结束
                if allow_file { // 条件分支
                    let mut is_file = r.is_file; // 变量 is_file
                    egui::ComboBox::from_id_salt(format!("kvtype_{}_{}", id, i)) // 调用 from_id_salt
                        .width(72.0) // 语句
                        .selected_text(if is_file { "文件" } else { "文本" }) // 语句
                        .show_ui(ui, |ui| { // 语句
                            ui.selectable_value(&mut is_file, false, "文本"); // 语句
                            ui.selectable_value(&mut is_file, true, "文件"); // 语句
                        }); // 语句
                    r.is_file = is_file; // 语句
                } // 块结束
                if ui // 条件分支
                    .add( // 语句
                        egui::Button::new(RichText::new("×").color(text_secondary())) // 调用 new
                            .frame(false), // 语句
                    ) // 块结束
                    .clicked() // 语句
                { // 语句
                    remove = Some(i); // 语句
                } // 块结束
            }); // 语句
        } // 块结束
        if let Some(i) = remove { // 条件分支
            rows.remove(i); // 语句
        } // 块结束
        if ui // 条件分支
            .add( // 语句
                egui::Button::new(RichText::new("＋ 添加一行").color(brand())) // 调用 new
                    .fill(brand_soft()) // 语句
                    .rounding(Rounding::same(6.0)), // 语句
            ) // 块结束
            .clicked() // 语句
        { // 语句
            rows.push(KVRow::default()); // 语句
        } // 块结束
    } // 块结束
    fn ui_tests(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_tests
        ui.label( // 语句
            RichText::new( // 调用 new
                "测试断言（JSON 数组）。支持 type：status_equals / body_contains / json_path_exists / body_equals / response_time / regex 等。发送后可在响应区「测试」查看结果。", // 语句
            ) // 块结束
            .color(text_secondary()) // 语句
            .size(11.0), // 语句
        ); // 块结束
        ui.add_space(4.0); // 语句
        ui.add( // 语句
            egui::TextEdit::multiline(&mut tab.tests_text) // 调用 multiline
                .code_editor() // 语句
                .desired_rows(10) // 语句
                .desired_width(f32::INFINITY) // 语句
                .hint_text("[\n  { \"type\": \"status_equals\", \"value\": 200 }\n]"), // 语句
        ); // 块结束
        if tab.tests_text.trim().is_empty() { // 条件分支
            tab.tests.clear(); // 语句
            ui.label(RichText::new("当前无测试断言").color(text_secondary()).size(11.0)); // 语句
        } else { // 语句
            match serde_json::from_str::<Vec<TestAssertion>>(&tab.tests_text) { // 分支匹配
                Ok(v) => { // 匹配分支
                    let n = v.len(); // 变量 n
                    tab.tests = v; // 语句
                    ui.label( // 语句
                        RichText::new(format!("已解析 {} 条断言", n)) // 调用 new
                            .color(Color32::from_rgb(73, 162, 91)) // 语句
                            .size(11.0), // 语句
                    ); // 块结束
                } // 块结束
                Err(e) => { // 匹配分支
                    ui.label( // 语句
                        RichText::new(format!("JSON 解析错误：{}", e)) // 调用 new
                            .color(Color32::from_rgb(235, 32, 19)) // 语句
                            .size(11.0), // 语句
                    ); // 块结束
                } // 块结束
            } // 块结束
        } // 块结束
    } // 块结束
    fn ui_scripts(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_scripts
        ui.label( // 语句
            RichText::new( // 调用 new
                "前置脚本：请求发送前执行，可用 pm.environment.set / pm.globals.set / pm.collectionVariables.set / pm.variables.set 写入变量。", // 语句
            ) // 块结束
            .color(text_secondary()) // 语句
            .size(11.0), // 语句
        ); // 块结束
        ui.add( // 语句
            egui::TextEdit::multiline(&mut tab.pre_script) // 调用 multiline
                .code_editor() // 语句
                .desired_rows(7) // 语句
                .desired_width(f32::INFINITY) // 语句
                .hint_text("pm.environment.set(\"token\", \"abc123\");"), // 语句
        ); // 块结束
        ui.add_space(8.0); // 语句
        ui.label( // 语句
            RichText::new( // 调用 new
                "测试脚本：响应返回后执行。支持 pm.test(name, function(){ ... })、pm.response.to.have.status(N)、pm.expect(pm.response.code).to.eql(N)、pm.expect(pm.response.json().a.b).to.eql(\"x\")。结果会显示在响应区「测试」。", // 语句
            ) // 块结束
            .color(text_secondary()) // 语句
            .size(11.0), // 语句
        ); // 块结束
        ui.add( // 语句
            egui::TextEdit::multiline(&mut tab.test_script) // 调用 multiline
                .code_editor() // 语句
                .desired_rows(8) // 语句
                .desired_width(f32::INFINITY) // 语句
                .hint_text( // 语句
                    "pm.test(\"状态为 200\", function () {\n    pm.response.to.have.status(200);\n});", // 语句
                ), // 语句
        ); // 块结束
    } // 块结束
    fn ui_settings(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_settings
        ui.label( // 语句
            RichText::new( // 调用 new
                "请求级网络设置：留空的项使用全局配置（全局可用环境变量 PMRS_PROXY / PMRS_INSECURE / PMRS_CONNECT_TIMEOUT / PMRS_MAX_REDIRECTS / PMRS_CA_BUNDLE / PMRS_CLIENT_CERT / PMRS_CLIENT_KEY 设置）。", // 语句
            ) // 块结束
            .color(text_secondary()) // 语句
            .size(11.0), // 语句
        ); // 块结束
        ui.add_space(6.0); // 语句
        ui.horizontal(|ui| { // 语句
            ui.label(RichText::new("超时（秒）").color(text_secondary()).size(12.0)); // 语句
            ui.add( // 语句
                egui::TextEdit::singleline(&mut tab.req_timeout) // 调用 singleline
                    .desired_width(80.0) // 语句
                    .hint_text("全局"), // 语句
            ); // 块结束
            ui.add_space(12.0); // 语句
            ui.checkbox( // 语句
                &mut tab.follow_redirects, // 语句
                RichText::new("跟随重定向").color(text_secondary()), // 字段 RichText
            ); // 块结束
            ui.add_space(12.0); // 语句
            ui.checkbox( // 语句
                &mut tab.insecure, // 语句
                RichText::new("跳过 TLS 证书校验（不安全）").color(text_secondary()), // 字段 RichText
            ); // 块结束
        }); // 语句
        ui.add_space(6.0); // 语句
        ui.horizontal(|ui| { // 语句
            ui.add_sized( // 语句
                [90.0, 20.0], // 语句
                egui::Label::new(RichText::new("代理").color(text_secondary()).size(12.0)), // 字段 egui
            ); // 块结束
            ui.add( // 语句
                egui::TextEdit::singleline(&mut tab.proxy) // 调用 singleline
                    .desired_width(f32::INFINITY) // 语句
                    .hint_text("http://127.0.0.1:8080（SOCKS 需启用 reqwest 的 socks 特性）"), // 语句
            ); // 块结束
        }); // 语句
        ui.horizontal(|ui| { // 语句
            ui.add_sized( // 语句
                [90.0, 20.0], // 语句
                egui::Label::new(RichText::new("客户端证书").color(text_secondary()).size(12.0)), // 字段 egui
            ); // 块结束
            ui.add( // 语句
                egui::TextEdit::singleline(&mut tab.client_cert) // 调用 singleline
                    .desired_width(f32::INFINITY) // 语句
                    .hint_text("client.crt / PEM 路径"), // 语句
            ); // 块结束
        }); // 语句
        ui.horizontal(|ui| { // 语句
            ui.add_sized( // 语句
                [90.0, 20.0], // 语句
                egui::Label::new(RichText::new("客户端私钥").color(text_secondary()).size(12.0)), // 字段 egui
            ); // 块结束
            ui.add( // 语句
                egui::TextEdit::singleline(&mut tab.client_key) // 调用 singleline
                    .desired_width(f32::INFINITY) // 语句
                    .hint_text("client.key / PEM 路径"), // 语句
            ); // 块结束
        }); // 语句
    } // 块结束
    fn ui_desc(tab: &mut Tab, ui: &mut egui::Ui) { // 函数 ui_desc
        ui.label( // 语句
            RichText::new("请求描述（保存到集合后写入 item.description，供文档/团队查看；支持 Markdown 文本）") // 调用 new
                .color(text_secondary()) // 语句
                .size(11.0), // 语句
        ); // 块结束
        ui.add_space(6.0); // 语句
        ui.add( // 语句
            egui::TextEdit::multiline(&mut tab.description) // 调用 multiline
                .desired_width(f32::INFINITY) // 语句
                .desired_rows(8) // 语句
                .hint_text("例如：创建用户，需 Bearer 令牌；返回 201 与用户 ID。"), // 语句
        ); // 块结束
        if ui.button("从响应体填入").clicked() { // 条件分支
            if let Some(r) = &tab.response { // 条件分支
                tab.description = r.body.clone(); // 语句
            } // 块结束
        } // 块结束
    } // 块结束
    fn ui_response(&mut self, ui: &mut egui::Ui) { // 函数 ui_response
        let active = self.active; // 变量 active
        let examples = self.examples_for_active(); // 变量 examples
        let mut save_ex = false; // 变量 save_ex
        let mut load_ex: Option<usize> = None; // 变量 load_ex
        let tab = &mut self.tabs[active]; // 变量 tab
        let has_resp = tab.response.is_some(); // 变量 has_resp
        let mut pretty = tab.pretty; // 变量 pretty
        let mut copy_clicked = false; // 变量 copy_clicked
        let mut save_file = false; // 变量 save_file
        let body_clone = tab.response.as_ref().map(|r| r.body.clone()).unwrap_or_default(); // 变量 body_clone
        ui.horizontal(|ui| { // 语句
            match (&tab.response, &tab.error) { // 分支匹配
                (Some(r), _) => { // 匹配分支
                    let c = status_color(r.code); // 变量 c
                    egui::Frame::none() // 调用 none
                        .fill(Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 34)) // 语句
                        .rounding(Rounding::same(R_SM)) // 语句
                        .inner_margin(egui::Margin::symmetric(10.0, 3.0)) // 语句
                        .show(ui, |ui| { // 语句
                            ui.label( // 语句
                                RichText::new(format!("{} {}", r.code, r.status)) // 调用 new
                                    .color(c) // 语句
                                    .size(12.0) // 语句
                                    .strong(), // 语句
                            ); // 块结束
                        }); // 语句
                    ui.add_space(8.0); // 语句
                    ui.label( // 语句
                        RichText::new(format!("耗时 {} ms", r.elapsed_ms)) // 调用 new
                            .color(text_secondary()) // 语句
                            .size(11.5), // 语句
                    ); // 块结束
                    ui.add_space(7.0); // 语句
                    ui.label( // 语句
                        RichText::new(format!("大小 {}", human_size(r.body.len()))) // 调用 new
                            .color(text_secondary()) // 语句
                            .size(11.5), // 语句
                    ); // 块结束
                } // 块结束
                (None, Some(e)) => { // 匹配分支
                    ui.label( // 语句
                        RichText::new(format!("错误：{}", e)) // 调用 new
                            .color(Color32::from_rgb(235, 32, 19)), // 语句
                    ); // 块结束
                } // 块结束
                (None, None) => { // 匹配分支
                    ui.label(RichText::new(&tab.status).color(text_secondary())); // 语句
                } // 块结束
            } // 块结束
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { // 语句
                if has_resp { // 条件分支
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new( // 调用 new
                                RichText::new("保存为示例").color(text_secondary()).size(12.0), // 字段 RichText
                            ) // 块结束
                            .frame(false), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        save_ex = true; // 语句
                    } // 块结束
                    if ui // 条件分支
                        .add( // 语句
                            egui::Button::new(RichText::new("复制").color(text_secondary())) // 调用 new
                                .frame(false), // 语句
                        ) // 块结束
                        .clicked() // 语句
                    { // 语句
                        copy_clicked = true; // 语句
                    } // 块结束
                    ui.checkbox(&mut pretty, RichText::new("美化").color(text_secondary())); // 语句
                } // 块结束
            }); // 语句
        }); // 语句
        if has_resp { // 条件分支
            ui.add_space(4.0); // 语句
            ui.horizontal(|ui| { // 语句
                ui.label( // 语句
                    RichText::new("保存响应到文件：") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(11.5), // 语句
                ); // 块结束
                ui.add( // 语句
                    egui::TextEdit::singleline(&mut self.resp_save_path) // 调用 singleline
                        .desired_width(300.0) // 语句
                        .hint_text("留空 = ~/.postman-rs/downloads/response_<时间>.txt"), // 语句
                ); // 块结束
                if ui // 条件分支
                    .add( // 语句
                        egui::Button::new(RichText::new("保存").color(Color32::WHITE).strong()) // 调用 new
                            .fill(brand()) // 语句
                            .rounding(Rounding::same(6.0)), // 语句
                    ) // 块结束
                    .clicked() // 语句
                { // 语句
                    save_file = true; // 语句
                } // 块结束
            }); // 语句
        } // 块结束
        if !examples.is_empty() { // 条件分支
            ui.add_space(4.0); // 语句
            ui.horizontal(|ui| { // 语句
                ui.label(RichText::new("示例：").color(text_secondary()).size(11.5)); // 语句
                for (i, ex) in examples.iter().enumerate() { // 循环
                    if ui.small_button(&ex.name).clicked() { // 条件分支
                        load_ex = Some(i); // 语句
                    } // 块结束
                } // 块结束
            }); // 语句
        } // 块结束
        tab.pretty = pretty; // 语句
        if copy_clicked { // 条件分支
            ui.output_mut(|o| o.copied_text = body_clone.clone()); // 语句
        } // 块结束
        ui.add_space(6.0); // 语句
        underline_tabs( // 调用 underline_tabs
            ui, // 语句
            &mut tab.resp_subtab, // 语句
            &["响应体", "响应头", "Cookies", "测试", "计时"], // 语句
        ); // 块结束
        ui.add_space(2.0); // 语句
        let rect = ui.available_rect_before_wrap(); // 变量 rect
        ui.painter().hline( // 语句
            rect.left()..=rect.right(), // 语句
            rect.top() + 1.0, // 语句
            Stroke::new(1.0_f32, border()), // 字段 Stroke
        ); // 块结束
        ui.add_space(8.0); // 语句
        if let Some(r) = &tab.response { // 条件分支
            match tab.resp_subtab { // 分支匹配
                0 => { // 匹配分支
                    let text = if tab.pretty { // 变量 text
                        pretty_body(&r.body) // 调用 pretty_body
                    } else { // 语句
                        r.body.clone() // 语句
                    }; // 块结束
                    let is_json = serde_json::from_str::<serde_json::Value>(&text).is_ok(); // 变量 is_json
                    egui::ScrollArea::both() // 调用 both
                        .id_salt("rbody") // 语句
                        .auto_shrink([false, false]) // 语句
                        .show(ui, |ui| { // 语句
                            if is_json { // 条件分支
                                let galley = highlight_json(ui, &text, f32::INFINITY); // 变量 galley
                                ui.add(egui::Label::new(galley)); // 语句
                            } else { // 语句
                                let mut ro = text; // 变量 ro
                                ui.add( // 语句
                                    egui::TextEdit::multiline(&mut ro) // 调用 multiline
                                        .code_editor() // 语句
                                        .desired_rows(14) // 语句
                                        .desired_width(f32::INFINITY) // 语句
                                        .interactive(false), // 语句
                                ); // 块结束
                            } // 块结束
                        }); // 语句
                } // 块结束
                1 => { // 匹配分支
                    egui::ScrollArea::vertical() // 调用 vertical
                        .id_salt("rh") // 语句
                        .auto_shrink([false, false]) // 语句
                        .show(ui, |ui| { // 语句
                            for (k, v) in &r.headers { // 循环
                                ui.label( // 语句
                                    RichText::new(format!("{}: {}", k, v)) // 调用 new
                                        .color(text_primary()) // 语句
                                        .size(12.0), // 语句
                                ); // 块结束
                            } // 块结束
                        }); // 语句
                } // 块结束
                2 => { // 匹配分支
                    let cookies: Vec<&(String, String)> = r // 变量 cookies
                        .headers // 语句
                        .iter() // 语句
                        .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie")) // 语句
                        .collect(); // 语句
                    egui::ScrollArea::vertical() // 调用 vertical
                        .id_salt("rcookies") // 语句
                        .auto_shrink([false, false]) // 语句
                        .show(ui, |ui| { // 语句
                            if cookies.is_empty() { // 条件分支
                                ui.label( // 语句
                                    RichText::new("此响应未设置 Cookie。").color(text_secondary()), // 字段 RichText
                                ); // 块结束
                            } // 块结束
                            for (_, v) in cookies { // 循环
                                ui.label( // 语句
                                    RichText::new(v).color(text_primary()).size(12.0), // 字段 RichText
                                ); // 块结束
                            } // 块结束
                            ui.add_space(6.0); // 语句
                            ui.label( // 语句
                                RichText::new( // 调用 new
                                    "Cookie 已持久化到 ~/.postman-rs/cookies.json（跨请求自动携带）。", // 语句
                                ) // 块结束
                                .color(text_secondary()) // 语句
                                .size(11.0), // 语句
                            ); // 块结束
                        }); // 语句
                } // 块结束
                3 => { // 匹配分支
                    let results = tests_engine::evaluate(&tab.tests, r); // 变量 results
                    egui::ScrollArea::vertical() // 调用 vertical
                        .id_salt("rtests") // 语句
                        .auto_shrink([false, false]) // 语句
                        .show(ui, |ui| { // 语句
                            if results.is_empty() { // 条件分支
                                ui.label( // 语句
                                    RichText::new( // 调用 new
                                        "此请求没有测试断言。可在请求区「测试」标签用 JSON 添加，或在「脚本」标签用 pm.test 编写。", // 语句
                                    ) // 块结束
                                    .color(text_secondary()), // 语句
                                ); // 块结束
                            } // 块结束
                            for t in &results { // 循环
                                let c = if t.passed { // 变量 c
                                    Color32::from_rgb(73, 162, 91) // 调用 from_rgb
                                } else { // 语句
                                    Color32::from_rgb(235, 32, 19) // 调用 from_rgb
                                }; // 块结束
                                ui.horizontal(|ui| { // 语句
                                    ui.label( // 语句
                                        RichText::new(if t.passed { "✓" } else { "✗" }) // 调用 new
                                            .color(c) // 语句
                                            .strong(), // 语句
                                    ); // 块结束
                                    ui.label(RichText::new(&t.name).color(text_primary())); // 语句
                                    if !t.detail.is_empty() { // 条件分支
                                        ui.label( // 语句
                                            RichText::new(&t.detail) // 调用 new
                                                .color(text_secondary()) // 语句
                                                .size(11.0), // 语句
                                        ); // 块结束
                                    } // 块结束
                                }); // 语句
                            } // 块结束
                            for t in &tab.script_results { // 循环
                                let c = if t.passed { // 变量 c
                                    Color32::from_rgb(73, 162, 91) // 调用 from_rgb
                                } else { // 语句
                                    Color32::from_rgb(235, 32, 19) // 调用 from_rgb
                                }; // 块结束
                                ui.horizontal(|ui| { // 语句
                                    ui.label( // 语句
                                        RichText::new(if t.passed { "✓" } else { "✗" }) // 调用 new
                                            .color(c) // 语句
                                            .strong(), // 语句
                                    ); // 块结束
                                    ui.label(RichText::new(&t.name).color(text_primary())); // 语句
                                    if let Some(e) = &t.error { // 条件分支
                                        if !e.is_empty() { // 条件分支
                                            ui.label( // 语句
                                                RichText::new(e) // 调用 new
                                                    .color(text_secondary()) // 语句
                                                    .size(11.0), // 语句
                                            ); // 块结束
                                        } // 块结束
                                    } // 块结束
                                }); // 语句
                            } // 块结束
                        }); // 语句
                } // 块结束
                _ => { // 匹配分支
                    egui::ScrollArea::vertical() // 调用 vertical
                        .id_salt("rtiming") // 语句
                        .auto_shrink([false, false]) // 语句
                        .show(ui, |ui| { // 语句
                            ui.label( // 语句
                                RichText::new(format!("总耗时：{} ms", r.elapsed_ms)) // 调用 new
                                    .color(text_primary()) // 语句
                                    .strong(), // 语句
                            ); // 块结束
                            ui.add_space(4.0); // 语句
                            let mut line = |label: &str, v: Option<u128>| { // 变量 line
                                if let Some(ms) = v { // 条件分支
                                    ui.label( // 语句
                                        RichText::new(format!("{}：{} ms", label, ms)) // 调用 new
                                            .color(text_secondary()) // 语句
                                            .size(11.5), // 语句
                                    ); // 块结束
                                } // 块结束
                            }; // 块结束
                            line("DNS 解析", r.dns_ms); // 调用 line
                            line("TCP 连接", r.tcp_ms); // 调用 line
                            line("首字节 (TTFB，含 TLS/服务器)", r.ttfb_ms); // 调用 line
                            line("内容下载", r.download_ms); // 调用 line
                            ui.label( // 语句
                                RichText::new("说明：DNS/TCP 为独立探测阶段（近似值）；TLS 握手包含在首字节耗时内。") // 调用 new
                                    .color(text_secondary()) // 语句
                                    .size(11.0), // 语句
                            ); // 块结束
                            if r.binary { // 条件分支
                                ui.label( // 语句
                                    RichText::new("响应体为二进制，已以 Base64 显示。") // 调用 new
                                        .color(text_secondary()) // 语句
                                        .size(11.0), // 语句
                                ); // 块结束
                            } // 块结束
                        }); // 语句
                } // 块结束
            } // 块结束
        } else { // 语句
            ui.add_space(22.0); // 语句
            ui.vertical_centered(|ui| { // 语句
                ui.label( // 语句
                    RichText::new("尚无响应") // 调用 new
                        .color(text_primary()) // 语句
                        .size(14.0) // 语句
                        .strong(), // 语句
                ); // 块结束
                ui.add_space(4.0); // 语句
                ui.label( // 语句
                    RichText::new("填好请求后点击「发送」，响应会显示在这里") // 调用 new
                        .color(text_secondary()) // 语句
                        .size(12.0), // 语句
                ); // 块结束
                if !tab.status.is_empty() { // 条件分支
                    ui.add_space(4.0); // 语句
                    ui.label(RichText::new(&tab.status).color(text_secondary()).size(11.5)); // 语句
                } // 块结束
            }); // 语句
        } // 块结束
        if let Some(i) = load_ex { // 条件分支
            if let Some(ex) = examples.get(i) { // 条件分支
                tab.response = Some(example_to_record(ex)); // 语句
                tab.status = format!( // 语句
                    "示例 {} {}", // 语句
                    ex.code.unwrap_or(0), // 语句
                    ex.status.clone().unwrap_or_default() // 语句
                ); // 块结束
                tab.resp_subtab = 0; // 语句
            } // 块结束
        } // 块结束
        if save_ex { // 条件分支
            self.save_example(); // 语句
        } // 块结束
        if save_file { // 条件分支
            self.save_response_to_file(&body_clone); // 语句
        } // 块结束
    } // 块结束
} // 块结束
impl eframe::App for GuiApp { // 实现
    /// 用主题背景色清屏：即使某个面板因内容溢出而没有铺满，也不会露出纯黑底（黑边）。
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] { // 函数 clear_color
        bg_side().to_normalized_gamma_f32() // 调用 bg_side
    } // 块结束

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) { // 函数 update
        // 跟随系统：按操作系统深浅色偏好选择内置的浅色/深色主题。
        if self.follow_system { // 条件分支
            let idx = match ctx.system_theme() { // 变量 idx
                Some(egui::Theme::Dark) => 1, // 匹配分支
                Some(egui::Theme::Light) => 0, // 匹配分支
                None => { // 匹配分支
                    if ctx.style().visuals.dark_mode { 1 } else { 0 } // 条件分支
                } // 块结束
            }; // 块结束
            if THEME.load(Ordering::Relaxed) != idx { // 条件分支
                THEME.store(idx, Ordering::Relaxed); // 语句
            } // 块结束
        } // 块结束
        // eframe 可能按系统主题覆盖 egui 视觉；每帧重申一次我们的主题样式，
        // 保证下拉框/复选框等原生控件与自定义配色一致。
        install_style(ctx); // 调用 install_style
        // 键盘快捷键（Ctrl/Cmd 组合）
        let (k_send, k_save, k_new, k_close) = ctx.input(|i| { // 调用 let
            let cmd = i.modifiers.command; // 变量 cmd
            ( // 语句
                cmd && i.key_pressed(egui::Key::Enter), // 语句
                cmd && i.key_pressed(egui::Key::S), // 语句
                cmd && i.key_pressed(egui::Key::N), // 语句
                cmd && i.key_pressed(egui::Key::W), // 语句
            ) // 块结束
        }); // 语句
        if k_new { // 条件分支
            self.open_new_request_dialog(); // 语句
        } // 块结束
        if k_close && self.tabs.len() > 1 { // 条件分支
            let idx = self.active; // 变量 idx
            self.pending_delete = Some(PendingDelete::Tab { // 语句
                idx, // 语句
                title: self.tabs[idx].title.clone(), // 字段 title
            }); // 语句
        } // 块结束
        if k_save { // 条件分支
            self.save_active(); // 语句
        } // 块结束
        if k_send { // 条件分支
            self.send_active(ctx); // 语句
        } // 块结束
        let mut got_new = false; // 变量 got_new
        for t in &mut self.tabs { // 循环
            if t.poll() { // 条件分支
                got_new = true; // 语句
                let ts = chrono::Local::now().format("%H:%M:%S").to_string(); // 变量 ts
                let line = match (&t.error, &t.response) { // 变量 line
                    (_, Some(resp)) => format!( // 匹配分支
                        "[{}] {} {} → {} {} ({} ms)", // 语句
                        ts, t.method, t.url, resp.code, resp.status, resp.elapsed_ms // 语句
                    ), // 语句
                    (Some(e), None) => format!("[{}] {} {} → 错误：{}", ts, t.method, t.url, e), // 匹配分支
                    _ => String::new(), // 匹配分支
                }; // 块结束
                if !line.is_empty() { // 条件分支
                    self.console.push(line); // 语句
                } // 块结束
                for tr in &t.script_results { // 循环
                    self.console.push(format!( // 语句
                        "[{}] 测试{}：{}", // 语句
                        ts, // 语句
                        if tr.passed { "通过" } else { "失败" }, // 条件分支
                        tr.name // 语句
                    )); // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        if got_new { // 条件分支
            self.history = storage::read_history().unwrap_or_default(); // 语句
        } // 块结束
        if let Some(rx) = &self.run_rx { // 条件分支
            match rx.try_recv() { // 分支匹配
                Ok(msg) => { // 匹配分支
                    self.run_result = msg; // 语句
                    self.run_inflight = false; // 语句
                    self.run_rx = None; // 语句
                } // 块结束
                Err(mpsc::TryRecvError::Empty) => {} // 匹配分支
                Err(mpsc::TryRecvError::Disconnected) => { // 匹配分支
                    self.run_inflight = false; // 语句
                    self.run_rx = None; // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        if let Some(rx) = &self.ws_rx { // 条件分支
            match rx.try_recv() { // 分支匹配
                Ok(Ok(lines)) => { // 匹配分支
                    self.ws_log = lines.join("\n"); // 语句
                    self.ws_busy = false; // 语句
                    self.ws_rx = None; // 语句
                } // 块结束
                Ok(Err(e)) => { // 匹配分支
                    self.ws_log = format!("错误：{}", e); // 语句
                    self.ws_busy = false; // 语句
                    self.ws_rx = None; // 语句
                } // 块结束
                Err(mpsc::TryRecvError::Empty) => {} // 匹配分支
                Err(mpsc::TryRecvError::Disconnected) => { // 匹配分支
                    self.ws_busy = false; // 语句
                    self.ws_rx = None; // 语句
                } // 块结束
            } // 块结束
        } // 块结束
        if self.tabs.iter().any(|t| t.inflight) { // 条件分支
            ctx.request_repaint_after(std::time::Duration::from_millis(120)); // 语句
        } // 块结束
        if self.run_inflight { // 条件分支
            ctx.request_repaint_after(std::time::Duration::from_millis(200)); // 语句
        } // 块结束
        if self.ws_busy { // 条件分支
            ctx.request_repaint_after(std::time::Duration::from_millis(200)); // 语句
        } // 块结束
        self.ui_topbar(ctx); // 语句
        self.ui_sidebar(ctx); // 语句
        self.ui_main(ctx); // 语句
        self.show_new_request_dialog(ctx); // 语句
        self.show_confirm_dialog(ctx); // 语句
        self.show_curl_import_dialog(ctx); // 语句
    } // 块结束
} // 块结束
// ---------------------------------------------------------------------------
// 工具函数
// ---------------------------------------------------------------------------
fn flatten(items: &[Item], prefix: &str, out: &mut Vec<(String, Request, Option<String>)>) { // 函数 flatten
    for it in items { // 循环
        let name = if prefix.is_empty() { // 变量 name
            it.name.clone() // 语句
        } else { // 语句
            format!("{} / {}", prefix, it.name) // 语句
        }; // 块结束
        if let Some(children) = &it.item { // 条件分支
            flatten(children, &name, out); // 调用 flatten
        } // 块结束
        if let Some(req) = &it.request { // 条件分支
            let desc = it.description.as_ref().map(|d| match d { // 变量 desc
                serde_json::Value::String(s) => s.clone(), // 匹配分支
                other => other.to_string(), // 匹配分支
            }); // 语句
            out.push((name, req.clone(), desc)); // 语句
        } // 块结束
    } // 块结束
} // 块结束

/// 集合树的一行（用于 GUI 树形展示与 CRUD）
struct TreeRow { // 结构体 TreeRow
    path: Vec<usize>, // 字段 path
    depth: usize, // 字段 depth
    is_folder: bool, // 字段 is_folder
    name: String, // 字段 name
    method: Option<String>, // 字段 method
} // 块结束

fn collect_rows(items: &[Item], depth: usize, prefix: &mut Vec<usize>, out: &mut Vec<TreeRow>) { // 函数 collect_rows
    for (i, it) in items.iter().enumerate() { // 循环
        prefix.push(i); // 语句
        let is_folder = it.request.is_none() && it.item.is_some(); // 变量 is_folder
        out.push(TreeRow { // 语句
            path: prefix.clone(), // 字段 path
            depth, // 语句
            is_folder, // 语句
            name: it.name.clone(), // 字段 name
            method: it.request.as_ref().map(|r| r.method.clone()), // 字段 method
        }); // 语句
        if let Some(children) = &it.item { // 条件分支
            collect_rows(children, depth + 1, prefix, out); // 调用 collect_rows
        } // 块结束
        prefix.pop(); // 语句
    } // 块结束
} // 块结束

fn item_at_mut<'a>(items: &'a mut Vec<Item>, path: &[usize]) -> Option<&'a mut Item> { // 函数 item_at_mut
    if path.is_empty() { // 条件分支
        return None; // 返回
    } // 块结束
    if path.len() == 1 { // 条件分支
        return items.get_mut(path[0]); // 返回
    } // 块结束
    let head = items.get_mut(path[0])?; // 变量 head
    let children = head.item.as_mut()?; // 变量 children
    item_at_mut(children, &path[1..]) // 调用 item_at_mut
} // 块结束

fn item_at<'a>(items: &'a [Item], path: &[usize]) -> Option<&'a Item> { // 函数 item_at
    if path.is_empty() { // 条件分支
        return None; // 返回
    } // 块结束
    if path.len() == 1 { // 条件分支
        return items.get(path[0]); // 返回
    } // 块结束
    let head = items.get(path[0])?; // 变量 head
    item_at(head.item.as_ref()?, &path[1..]) // 调用 item_at
} // 块结束

fn remove_at(items: &mut Vec<Item>, path: &[usize]) -> Option<Item> { // 函数 remove_at
    if path.is_empty() { // 条件分支
        return None; // 返回
    } // 块结束
    if path.len() == 1 { // 条件分支
        if path[0] < items.len() { // 条件分支
            return Some(items.remove(path[0])); // 返回
        } // 块结束
        return None; // 返回
    } // 块结束
    let head = items.get_mut(path[0])?; // 变量 head
    remove_at(head.item.as_mut()?, &path[1..]) // 调用 remove_at
} // 块结束

fn short_title(name: &str) -> String { // 函数 short_title
    name.rsplit('/').next().unwrap_or(name).trim().to_string() // 语句
} // 块结束
fn ctx_from(coll: Option<&Collection>, env: Option<&Environment>) -> VarContext { // 函数 ctx_from
    let mut ctx = VarContext::new(); // 变量 ctx
    if let Some(e) = env { // 条件分支
        let m: HashMap<String, String> = e // 变量 m
            .values // 语句
            .iter() // 语句
            .filter(|v| v.enabled.unwrap_or(true)) // 语句
            .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
            .collect(); // 语句
        if !m.is_empty() { // 条件分支
            ctx.scopes.push(m); // 语句
        } // 块结束
    } // 块结束
    if let Some(c) = coll { // 条件分支
        let m: HashMap<String, String> = c // 变量 m
            .variable // 语句
            .iter() // 语句
            .filter(|v| v.enabled.unwrap_or(true)) // 语句
            .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
            .collect(); // 语句
        if !m.is_empty() { // 条件分支
            ctx.scopes.push(m); // 语句
        } // 块结束
    } // 块结束
    // 全局变量（最低优先级）——修复：此前 GUI 发送时未纳入 globals，导致全局变量不生效。
    if let Ok(g) = storage::load_globals() { // 条件分支
        let m: HashMap<String, String> = g // 变量 m
            .values // 语句
            .iter() // 语句
            .filter(|v| v.enabled.unwrap_or(true)) // 语句
            .map(|v| (v.key.clone(), value_str(&v.value))) // 语句
            .collect(); // 语句
        if !m.is_empty() { // 条件分支
            ctx.scopes.push(m); // 语句
        } // 块结束
    } // 块结束
    ctx // 语句
} // 块结束

/// 沿集合项路径收集继承自集合/文件夹的 auth、变量与事件（后者含 pre-request/test）。
fn collect_inherited(coll: &Collection, path: &[usize]) -> (Option<Auth>, HashMap<String, String>, Vec<Event>) { // 函数 collect_inherited
    let mut auth = coll.auth.clone(); // 变量 auth
    let mut vars: HashMap<String, String> = HashMap::new(); // 变量 vars
    let mut events: Vec<Event> = coll.event.clone(); // 变量 events
    let mut items: &[Item] = &coll.item; // 变量 items
    for &idx in path { // 循环
        match items.get(idx) { // 分支匹配
            Some(it) => { // 匹配分支
                if let Some(a) = &it.auth { // 条件分支
                    auth = Some(a.clone()); // 语句
                } // 块结束
                for v in &it.variable { // 循环
                    if v.enabled.unwrap_or(true) { // 条件分支
                        vars.insert(v.key.clone(), value_str(&v.value)); // 语句
                    } // 块结束
                } // 块结束
                events.extend(it.event.iter().cloned()); // 语句
                items = it.item.as_deref().unwrap_or(&[]); // 语句
            } // 块结束
            None => break, // 匹配分支
        } // 块结束
    } // 块结束
    (auth, vars, events) // 语句
} // 块结束
fn value_str(v: &serde_json::Value) -> String { // 函数 value_str
    match v { // 分支匹配
        serde_json::Value::String(s) => s.clone(), // 匹配分支
        serde_json::Value::Null => String::new(), // 匹配分支
        other => other.to_string(), // 匹配分支
    } // 块结束
} // 块结束

/// 请求体类型（内部键）→ 中文标签
fn body_type_label(t: &str) -> &'static str { // 函数 body_type_label
    match t { // 分支匹配
        "form-data" => "form-data 表单", // 匹配分支
        "x-www-form-urlencoded" => "x-www-form-urlencoded 表单", // 匹配分支
        "raw" => "raw 原始", // 匹配分支
        "binary" => "binary 二进制", // 匹配分支
        "graphql" => "GraphQL 查询", // 匹配分支
        "none" => "无", // 匹配分支
        _ => "无", // 匹配分支
    } // 块结束
} // 块结束

/// 导入格式（内部键）→ 中文标签
fn import_format_label(f: &str) -> &'static str { // 函数 import_format_label
    match f { // 分支匹配
        "postman" => "Postman 集合", // 匹配分支
        "openapi" => "OpenAPI / Swagger", // 匹配分支
        "curl" => "cURL 命令", // 匹配分支
        "har" => "HAR 抓包", // 匹配分支
        _ => "Postman 集合", // 匹配分支
    } // 块结束
} // 块结束

fn grant_label(g: &str) -> &'static str { // 函数 grant_label
    match g { // 分支匹配
        "password" => "密码模式 (password)", // 匹配分支
        "refresh_token" => "刷新令牌 (refresh_token)", // 匹配分支
        _ => "客户端凭证 (client_credentials)", // 匹配分支
    } // 块结束
} // 块结束

fn default_auth_keys(kind: &str) -> &'static [&'static str] { // 函数 default_auth_keys
    match kind { // 分支匹配
        "oauth1" => &["consumerKey", "consumerSecret", "token", "tokenSecret"], // 匹配分支
        "awsv4" => &["accessKey", "secretKey", "region", "service", "sessionToken"], // 匹配分支
        "jwt" => &["algorithm", "secret", "payload", "token"], // 匹配分支
        "digest" => &["username", "password"], // 匹配分支
        "ntlm" => &["username", "password"], // 匹配分支
        _ => &[], // 匹配分支
    } // 块结束
} // 块结束

fn example_to_record(ex: &ResponseExample) -> ResponseRecord { // 函数 example_to_record
    ResponseRecord { // 语句
        status: ex.status.clone().unwrap_or_default(), // 字段 status
        code: ex.code.unwrap_or(0), // 字段 code
        headers: ex // 语句
            .header // 语句
            .iter() // 语句
            .map(|h| (h.key.clone(), h.value.clone())) // 语句
            .collect(), // 语句
        body: ex.body.clone().unwrap_or_default(), // 字段 body
        binary: false, // 字段 binary
        elapsed_ms: 0, // 字段 elapsed_ms
        final_url: String::new(), // 字段 final_url
        dns_ms: None, // 字段 dns_ms
        tcp_ms: None, // 字段 tcp_ms
        ttfb_ms: None, // 字段 ttfb_ms
        download_ms: None, // 字段 download_ms
    } // 块结束
} // 块结束

fn oauth2_field( // 函数 oauth2_field
    ui: &mut egui::Ui, // 字段 ui
    map: &mut HashMap<String, String>, // 字段 map
    label: &str, // 字段 label
    key: &str, // 字段 key
    secret: bool, // 字段 secret
) { // 语句
    ui.horizontal(|ui| { // 语句
        ui.add_sized( // 语句
            [150.0, 20.0], // 语句
            egui::Label::new(RichText::new(label).color(text_secondary()).size(12.0)), // 字段 egui
        ); // 块结束
        let mut val = map.get(key).cloned().unwrap_or_default(); // 变量 val
        if ui // 条件分支
            .add( // 语句
                egui::TextEdit::singleline(&mut val) // 调用 singleline
                    .password(secret) // 语句
                    .desired_width(300.0), // 语句
            ) // 块结束
            .changed() // 语句
        { // 语句
            map.insert(key.to_string(), val); // 语句
        } // 块结束
    }); // 语句
} // 块结束

/// raw 子语言标签 → Postman 存储用小写键
fn raw_lang_key(l: &str) -> &'static str { // 函数 raw_lang_key
    match l { // 分支匹配
        "JSON" => "json", // 匹配分支
        "XML" => "xml", // 匹配分支
        "HTML" => "html", // 匹配分支
        "JavaScript" => "javascript", // 匹配分支
        _ => "text", // 匹配分支
    } // 块结束
} // 块结束

/// Postman 存储用小写语言键 → 标签
fn raw_lang_label(l: &str) -> &'static str { // 函数 raw_lang_label
    match l.to_ascii_lowercase().as_str() { // 分支匹配
        "json" => "JSON", // 匹配分支
        "xml" => "XML", // 匹配分支
        "html" => "HTML", // 匹配分支
        "javascript" => "JavaScript", // 匹配分支
        _ => "Text", // 匹配分支
    } // 块结束
} // 块结束

fn file_name(p: &PathBuf) -> String { // 函数 file_name
    p.file_stem() // 语句
        .map(|s| s.to_string_lossy().to_string()) // 语句
        .unwrap_or_else(|| p.display().to_string()) // 语句
} // 块结束
fn main() -> eframe::Result<()> { // 函数 main
    let options = eframe::NativeOptions { // 变量 options
        viewport: egui::ViewportBuilder::default() // 语句
            .with_inner_size([1320.0, 880.0]) // 语句
            .with_min_inner_size([960.0, 620.0]) // 语句
            .with_title("pmrs — 本地 API 客户端"), // 语句
        ..Default::default() // 语句
    }; // 块结束
    eframe::run_native("pmrs", options, Box::new(|cc| Ok(Box::new(GuiApp::new(cc))))) // 调用 run_native
} // 块结束

#[cfg(test)] // 语句
mod tests { // 模块 tests
    use super::*; // 引入依赖

    #[test] // 语句
    fn body_types_build_correctly() { // 函数 body_types_build_correctly
        let mut t = Tab::new(1); // 变量 t

        // 无请求体
        t.body_type = "none".into(); // 语句
        assert!(t.build_request().body.is_none()); // 语句

        // raw / JSON
        t.body_type = "raw".into(); // 语句
        t.raw_lang = "JSON".into(); // 语句
        t.body = "{\"a\":1}".into(); // 语句
        let b = t.build_request().body.unwrap(); // 变量 b
        assert_eq!(b.mode, "raw"); // 语句
        assert_eq!(b.raw.as_deref(), Some("{\"a\":1}")); // 语句
        assert_eq!(b.raw_content_type().as_deref(), Some("application/json")); // 语句

        // x-www-form-urlencoded
        t.body_type = "x-www-form-urlencoded".into(); // 语句
        t.url_rows = vec![KVRow { // 语句
            key: "a".into(), // 字段 key
            value: "1".into(), // 字段 value
            disabled: false, // 字段 disabled
            is_file: false, // 字段 is_file
        }]; // 语句
        let b = t.build_request().body.unwrap(); // 变量 b
        assert_eq!(b.mode, "urlencoded"); // 语句
        assert_eq!(b.urlencoded[0].key, "a"); // 语句
        assert_eq!(b.urlencoded[0].value, "1"); // 语句

        // form-data（含文件行）
        t.body_type = "form-data".into(); // 语句
        t.form_rows = vec![KVRow { // 语句
            key: "f".into(), // 字段 key
            value: "C:/x.bin".into(), // 字段 value
            disabled: false, // 字段 disabled
            is_file: true, // 字段 is_file
        }]; // 语句
        let b = t.build_request().body.unwrap(); // 变量 b
        assert_eq!(b.mode, "formdata"); // 语句
        assert_eq!(b.formdata[0].typ.as_deref(), Some("file")); // 语句
        assert_eq!( // 语句
            b.formdata[0].src.as_ref().and_then(|v| v.as_str()), // 语句
            Some("C:/x.bin") // 调用 Some
        ); // 块结束

        // binary（本地文件）
        t.body_type = "binary".into(); // 语句
        t.file_path = "C:/x.bin".into(); // 语句
        let b = t.build_request().body.unwrap(); // 变量 b
        assert_eq!(b.mode, "file"); // 语句
        assert_eq!(b.file.unwrap().src.as_deref(), Some("C:/x.bin")); // 语句

        // GraphQL
        t.body_type = "graphql".into(); // 语句
        t.gql_query = "query{a}".into(); // 语句
        t.gql_vars = "{}".into(); // 语句
        let b = t.build_request().body.unwrap(); // 变量 b
        assert_eq!(b.mode, "graphql"); // 语句
        assert_eq!(b.graphql.unwrap().query, "query{a}"); // 语句
    } // 块结束

    #[test] // 语句
    fn label_mappings() { // 函数 label_mappings
        assert_eq!(raw_lang_key("JSON"), "json"); // 语句
        assert_eq!(raw_lang_key("JavaScript"), "javascript"); // 语句
        assert_eq!(raw_lang_label("xml"), "XML"); // 语句
        assert_eq!(body_type_label("form-data"), "form-data 表单"); // 语句
        assert_eq!(import_format_label("openapi"), "OpenAPI / Swagger"); // 语句
    } // 块结束
} // 块结束


