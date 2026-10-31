#![windows_subsystem = "windows"]
//! pmrs-gui — 轻量级 egui 桌面客户端（本地、无账号、中文界面）。
//! 视觉上尽量贴近 Postman 桌面版：橙色品牌色、彩色方法下拉、
//! 请求卡片（参数 / 授权 / 请求头 / 请求体）、响应面板（状态胶囊 + 响应体/响应头）。
use base64::Engine as _;
use eframe::egui;
use egui::{Color32, RichText, Rounding, Stroke};
use postman_rs::http::{self, ResponseRecord};
use postman_rs::import_export;
use postman_rs::codegen;
use postman_rs::model::{
    Auth, Body, Collection, CollectionInfo, Environment, Event, FileRef, GraphQL, Header, Item, KV,
    Request, ResponseExample, TestAssertion, UrlSpec, Variable,
};
use postman_rs::runner::{self, RunOptions};
use postman_rs::storage;
use postman_rs::scripting;
use postman_rs::tests_engine;
use postman_rs::variables::VarContext;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
// ---------------------------------------------------------------------------
// 调色板（参照 Postman 品牌与界面配色）
// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// 主题系统（多套配色 + 渐变背景）
// ---------------------------------------------------------------------------
struct Theme {
    name: &'static str,
    dark: bool,
    app: [u8; 3],    // 应用背景（用于渐变基底）
    panel: [u8; 3],  // 卡片/面板
    side: [u8; 3],   // 侧栏
    border: [u8; 3], // 描边
    text: [u8; 3],   // 主文字
    text2: [u8; 3],  // 次文字
    accent: [u8; 3], // 品牌/强调色
}
static THEMES: [Theme; 8] = [
    Theme {
        name: "Postman 浅色",
        dark: false,
        app: [249, 250, 251],
        panel: [255, 255, 255],
        side: [246, 247, 249],
        border: [228, 230, 235],
        text: [33, 37, 41],
        text2: [110, 116, 123],
        accent: [255, 108, 55],
    },
    Theme {
        name: "Postman 深色",
        dark: true,
        app: [24, 26, 30],
        panel: [32, 34, 39],
        side: [38, 40, 46],
        border: [60, 63, 72],
        text: [228, 231, 236],
        text2: [150, 156, 166],
        accent: [255, 108, 55],
    },
    Theme {
        name: "午夜蓝",
        dark: true,
        app: [15, 23, 42],
        panel: [23, 32, 51],
        side: [30, 41, 59],
        border: [44, 58, 82],
        text: [226, 232, 240],
        text2: [148, 163, 184],
        accent: [59, 130, 246],
    },
    Theme {
        name: "石墨灰",
        dark: true,
        app: [28, 28, 31],
        panel: [38, 38, 43],
        side: [46, 46, 52],
        border: [63, 63, 70],
        text: [229, 229, 231],
        text2: [161, 161, 170],
        accent: [245, 158, 11],
    },
    Theme {
        name: "暖沙（浅）",
        dark: false,
        app: [253, 246, 227],
        panel: [255, 251, 240],
        side: [245, 239, 217],
        border: [227, 217, 190],
        text: [59, 58, 50],
        text2: [133, 127, 107],
        accent: [203, 75, 22],
    },
    Theme {
        name: "森林绿",
        dark: true,
        app: [14, 26, 22],
        panel: [20, 36, 30],
        side: [26, 46, 39],
        border: [42, 68, 58],
        text: [227, 240, 234],
        text2: [147, 179, 166],
        accent: [34, 197, 94],
    },
    Theme {
        name: "樱粉（浅）",
        dark: false,
        app: [255, 245, 247],
        panel: [255, 255, 255],
        side: [251, 237, 240],
        border: [240, 217, 223],
        text: [58, 42, 48],
        text2: [138, 107, 116],
        accent: [244, 63, 94],
    },
    Theme {
        name: "高对比",
        dark: true,
        app: [0, 0, 0],
        panel: [13, 13, 13],
        side: [20, 20, 20],
        border: [58, 58, 58],
        text: [255, 255, 255],
        text2: [200, 200, 200],
        accent: [255, 108, 55],
    },
];
static THEME: AtomicUsize = AtomicUsize::new(0);
/// 是否跟随系统深浅色（true 时按 ctx.system_theme() 选 0=浅 / 1=深）。
static FOLLOW_SYSTEM: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// 自定义强调色覆盖。0 表示不覆盖；否则低 24 位为 RGB。
static ACCENT_OVERRIDE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
fn theme() -> &'static Theme {
    &THEMES[THEME.load(Ordering::Relaxed).min(THEMES.len() - 1)]
}
fn rgb(a: [u8; 3]) -> Color32 {
    Color32::from_rgb(a[0], a[1], a[2])
}
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let f = |x: u8, y: u8| (x as f32 * (1.0 - t) + y as f32 * t).round() as u8;
    Color32::from_rgb(f(a.r(), b.r()), f(a.g(), b.g()), f(a.b(), b.b()))
}
fn dark() -> bool {
    theme().dark
}
fn brand() -> Color32 {
    let v = ACCENT_OVERRIDE.load(Ordering::Relaxed);
    if v != 0 {
        Color32::from_rgb(((v >> 16) & 0xff) as u8, ((v >> 8) & 0xff) as u8, (v & 0xff) as u8)
    } else {
        rgb(theme().accent)
    }
}
fn brand_soft() -> Color32 {
    if dark() {
        mix(brand(), Color32::from_rgb(20, 20, 22), 0.72)
    } else {
        mix(brand(), Color32::WHITE, 0.86)
    }
}
fn send_blue() -> Color32 {
    Color32::from_rgb(9, 123, 237) // Postman 发送按钮蓝
}
fn bg_app() -> Color32 {
    rgb(theme().app)
}
fn bg_panel() -> Color32 {
    rgb(theme().panel)
}
fn bg_side() -> Color32 {
    rgb(theme().side)
}
fn border() -> Color32 {
    rgb(theme().border)
}
fn text_primary() -> Color32 {
    rgb(theme().text)
}
fn text_secondary() -> Color32 {
    rgb(theme().text2)
}
fn app_top() -> Color32 {
    if dark() {
        mix(bg_app(), Color32::WHITE, 0.05)
    } else {
        mix(bg_app(), Color32::WHITE, 0.45)
    }
}
fn app_bottom() -> Color32 {
    if dark() {
        mix(bg_app(), Color32::BLACK, 0.22)
    } else {
        mix(bg_app(), Color32::BLACK, 0.05)
    }
}
/// 在主内容区绘制纵向渐变背景，营造层次感。
fn paint_app_gradient(ui: &egui::Ui) {
    let rect = ui.max_rect().expand2(egui::vec2(12.0, 10.0));
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), app_top());
    mesh.colored_vertex(rect.right_top(), app_top());
    mesh.colored_vertex(rect.right_bottom(), app_bottom());
    mesh.colored_vertex(rect.left_bottom(), app_bottom());
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(mesh));
}
// ---------------------------------------------------------------------------
// 设计系统：统一圆角 / 间距 / 卡片 / 标签 / 分段控件
// ---------------------------------------------------------------------------
const R_SM: f32 = 6.0;
const R_MD: f32 = 10.0;
const R_LG: f32 = 14.0;

fn card(shadow: bool) -> egui::Frame {
    let mut f = egui::Frame::none()
        .fill(bg_panel())
        .stroke(Stroke::new(1.0_f32, border()))
        .rounding(Rounding::same(R_LG))
        .inner_margin(egui::Margin::same(14.0));
    if shadow {
        f = f.shadow(egui::epaint::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 18.0,
            spread: 0.0,
            color: if dark() {
                Color32::from_black_alpha(90)
            } else {
                Color32::from_black_alpha(22)
            },
        });
    }
    f
}

fn section_title(ui: &mut egui::Ui, text: &str) {
    ui.horizontal(|ui| {
        let (r, _) = ui.allocate_exact_size(egui::vec2(3.0, 14.0), egui::Sense::hover());
        ui.painter().rect_filled(r, Rounding::same(2.0), brand());
        ui.add_space(5.0);
        ui.label(RichText::new(text).size(12.5).strong().color(text_primary()));
    });
}

/// 顶部品牌渐变细条
fn top_accent(ui: &mut egui::Ui) {
    let w = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(w, 3.0), egui::Sense::hover());
    let a = brand();
    let b = mix(brand(), bg_panel(), 0.7);
    let mut m = egui::Mesh::default();
    m.colored_vertex(rect.left_top(), a);
    m.colored_vertex(rect.right_top(), b);
    m.colored_vertex(rect.right_bottom(), b);
    m.colored_vertex(rect.left_bottom(), a);
    m.add_triangle(0, 1, 2);
    m.add_triangle(0, 2, 3);
    ui.painter().add(egui::Shape::mesh(m));
    ui.add_space(5.0);
}

/// 下划线风格标签页；返回是否切换
fn underline_tabs(ui: &mut egui::Ui, sel: &mut usize, items: &[&str]) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (i, name) in items.iter().enumerate() {
            let active = *sel == i;
            let color = if active { brand() } else { text_secondary() };
            let resp = ui.add(
                egui::Button::new(RichText::new(*name).color(color).size(13.5).strong())
                    .frame(false)
                    .min_size(egui::vec2(0.0, 30.0)),
            );
            if active {
                let r = resp.rect;
                let y = r.bottom() + 1.0;
                ui.painter().line_segment(
                    [egui::pos2(r.left() + 3.0, y), egui::pos2(r.right() - 3.0, y)],
                    Stroke::new(2.0_f32, brand()),
                );
            }
            if resp.clicked() {
                *sel = i;
            }
        }
    });
}

/// 分段控件（侧栏顶部四个入口）
fn segmented(ui: &mut egui::Ui, sel: &mut usize, items: &[&str]) {
    let total_w = ui.available_width();
    egui::Frame::none()
        .fill(mix(bg_side(), text_primary(), 0.06))
        .rounding(Rounding::same(R_MD))
        .inner_margin(egui::Margin::same(3.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                let n = items.len() as f32;
                let w = ((total_w - 6.0) / n - 2.0).max(40.0);
                for (i, name) in items.iter().enumerate() {
                    let active = *sel == i;
                    let (bg, tc) = if active {
                        (brand(), Color32::WHITE)
                    } else {
                        (Color32::TRANSPARENT, text_secondary())
                    };
                    if ui
                        .add_sized(
                            [w, 26.0],
                            egui::Button::new(
                                RichText::new(*name).color(tc).size(12.5).strong(),
                            )
                            .fill(bg)
                            .rounding(Rounding::same(R_SM)),
                        )
                        .clicked()
                    {
                        *sel = i;
                    }
                }
            });
        });
}

/// 方法彩色小药丸（GET/POST…）
fn method_pill(ui: &mut egui::Ui, method: &str) {
    let c = method_color(method);
    egui::Frame::none()
        .fill(mix(c, bg_panel(), 0.82))
        .rounding(Rounding::same(R_SM))
        .inner_margin(egui::Margin::symmetric(6.0, 1.0))
        .show(ui, |ui| {
            ui.label(RichText::new(method.to_uppercase()).color(c).strong().size(10.5));
        });
}

fn method_color(m: &str) -> Color32 {
    match m.to_uppercase().as_str() {
        "GET" => Color32::from_rgb(73, 162, 91),
        "POST" => Color32::from_rgb(240, 164, 0),
        "PUT" => Color32::from_rgb(9, 123, 237),
        "PATCH" => Color32::from_rgb(155, 89, 182),
        "DELETE" => Color32::from_rgb(235, 32, 19),
        "HEAD" => Color32::from_rgb(135, 119, 217),
        "OPTIONS" => Color32::from_rgb(255, 160, 0),
        _ => text_secondary(),
    }
}
fn status_color(code: u16) -> Color32 {
    match code {
        200..=299 => Color32::from_rgb(73, 162, 91),
        300..=399 => Color32::from_rgb(9, 123, 237),
        400..=499 => Color32::from_rgb(240, 164, 0),
        _ => Color32::from_rgb(235, 32, 19),
    }
}
fn human_size(n: usize) -> String {
    let f = n as f64;
    if f < 1024.0 {
        format!("{} B", n)
    } else if f < 1024.0 * 1024.0 {
        format!("{:.1} KB", f / 1024.0)
    } else {
        format!("{:.1} MB", f / 1_048_576.0)
    }
}
fn pretty_body(s: &str) -> String {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
        if let Ok(p) = serde_json::to_string_pretty(&v) {
            return p;
        }
    }
    s.to_string()
}
fn highlight_json(ui: &egui::Ui, text: &str, wrap_width: f32) -> std::sync::Arc<egui::text::Galley> {
    use egui::text::{LayoutJob, TextFormat};
    let font = egui::FontId::monospace(13.0);
    let base = TextFormat {
        font_id: font.clone(),
        color: text_primary(),
        ..Default::default()
    };
    let str_c = Color32::from_rgb(96, 180, 120);
    let num_c = Color32::from_rgb(220, 150, 80);
    let key_c = Color32::from_rgb(110, 160, 240);
    let kw_c = Color32::from_rgb(170, 140, 220);
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap_width;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        let c = chars[i];
        if c == '"' {
            let start = i;
            i += 1;
            while i < chars.len() {
                let d = chars[i];
                i += 1;
                if d == '\\' {
                    i += 1;
                    continue;
                }
                if d == '"' {
                    break;
                }
            }
            let s: String = chars[start..i.min(chars.len())].iter().collect();
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            let is_key = j < chars.len() && chars[j] == ':';
            job.append(
                &s,
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color: if is_key { key_c } else { str_c },
                    ..Default::default()
                },
            );
        } else if c.is_ascii_digit()
            || (c == '-' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            i += 1;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || matches!(chars[i], '.' | 'e' | 'E' | '+' | '-'))
            {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            job.append(
                &s,
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color: num_c,
                    ..Default::default()
                },
            );
        } else if c.is_alphabetic() {
            let start = i;
            i += 1;
            while i < chars.len() && chars[i].is_alphabetic() {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            let color = match s.as_str() {
                "true" | "false" | "null" => kw_c,
                _ => text_primary(),
            };
            job.append(
                &s,
                0.0,
                TextFormat {
                    font_id: font.clone(),
                    color,
                    ..Default::default()
                },
            );
        } else {
            let s = c.to_string();
            i += 1;
            job.append(&s, 0.0, base.clone());
        }
    }
    ui.fonts(|f| f.layout_job(job))
}
fn parse_query(url: &str) -> Vec<KVRow> {
    let q = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    q.split('&')
        .filter(|s| !s.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((k, v)) => KVRow {
                key: k.to_string(),
                value: v.to_string(),
                ..Default::default()
            },
            None => KVRow {
                key: pair.to_string(),
                ..Default::default()
            },
        })
        .collect()
}

/// 扫描 URL 路径段中的 `:name` 形式的路径变量，返回去重后的变量行。
fn detect_path_vars(url: &str) -> Vec<KVRow> {
    let path = url.split(['?', '#']).next().unwrap_or("");
    let mut out: Vec<KVRow> = Vec::new();
    for seg in path.split('/') {
        if let Some(name) = seg.strip_prefix(':') {
            let name = name.trim();
            if !name.is_empty() && !out.iter().any(|r| r.key == name) {
                out.push(KVRow {
                    key: name.to_string(),
                    ..Default::default()
                });
            }
        }
    }
    out
}

fn rebuild_url(tab: &mut Tab) {
    let base = tab
        .url
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .to_string();
    let q: Vec<String> = tab
        .params
        .iter()
        .filter(|r| !r.disabled && !r.key.trim().is_empty())
        .map(|r| {
            if r.value.is_empty() {
                r.key.clone()
            } else {
                format!("{}={}", r.key, r.value)
            }
        })
        .collect();
    let mut url = if q.is_empty() {
        base
    } else {
        format!("{}?{}", base, q.join("&"))
    };
    if !tab.hash.trim().is_empty() {
        url.push('#');
        url.push_str(tab.hash.trim().trim_start_matches('#'));
    }
    tab.url = url;
}
// ---------------------------------------------------------------------------
// 主题 / 字体
// ---------------------------------------------------------------------------
fn install_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut v = egui::Visuals::light();
    v.dark_mode = dark();
    v.panel_fill = bg_panel();
    v.window_fill = bg_panel();
    v.extreme_bg_color = if dark() {
        mix(bg_panel(), Color32::BLACK, 0.35)
    } else {
        mix(bg_panel(), Color32::WHITE, 0.5)
    };
    v.faint_bg_color = mix(bg_panel(), text_primary(), 0.07);
    v.override_text_color = Some(text_primary());
    v.hyperlink_color = brand();
    let a = brand();
    v.selection.bg_fill = Color32::from_rgba_unmultiplied(a.r(), a.g(), a.b(), 60);
    v.selection.stroke = Stroke::new(1.0_f32, brand());
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border());
    v.widgets.noninteractive.rounding = Rounding::same(6.0);
    v.widgets.noninteractive.weak_bg_fill = mix(bg_panel(), text_primary(), 0.04);
    v.widgets.inactive.bg_fill = mix(bg_panel(), text_primary(), 0.06);
    v.widgets.inactive.weak_bg_fill = mix(bg_panel(), text_primary(), 0.09);
    v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, border());
    v.widgets.inactive.rounding = Rounding::same(6.0);
    v.widgets.hovered.bg_fill = mix(bg_panel(), text_primary(), 0.11);
    v.widgets.hovered.weak_bg_fill = mix(bg_panel(), text_primary(), 0.15);
    v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, border());
    v.widgets.hovered.rounding = Rounding::same(6.0);
    v.widgets.active.bg_fill = brand_soft();
    v.widgets.active.weak_bg_fill = brand_soft();
    v.widgets.active.bg_stroke = Stroke::new(1.0_f32, brand());
    v.widgets.active.rounding = Rounding::same(6.0);
    v.window_rounding = Rounding::same(8.0);
    style.visuals = v;
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.spacing.button_padding = egui::vec2(10.0, 5.0);
    style.spacing.interact_size.y = 26.0;
    ctx.set_style(style);
}
fn apply_theme(ctx: &egui::Context, idx: usize) {
    THEME.store(idx.min(THEMES.len() - 1), Ordering::Relaxed);
    install_style(ctx);
}
fn theme_pref_path() -> PathBuf {
    storage::home().join("gui_theme.txt")
}
fn initial_theme() -> usize {
    if let Ok(v) = std::env::var("PMRS_THEME") {
        if let Ok(n) = v.trim().parse::<usize>() {
            return n.min(THEMES.len() - 1);
        }
    }
    std::fs::read_to_string(theme_pref_path())
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(0)
        .min(THEMES.len() - 1)
}
fn save_theme(idx: usize) {
    let _ = std::fs::create_dir_all(storage::home());
    let _ = std::fs::write(theme_pref_path(), idx.to_string());
}
/// 解析 "RRGGBB" / "#RRGGBB" 为 24 位 RGB 值。
fn parse_hex_color(s: &str) -> Option<u32> {
    let t = s.trim().trim_start_matches('#');
    if t.len() != 6 || !t.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(t, 16).ok()
}
fn install_fonts(ctx: &egui::Context) {
    let candidates = [
        "C:\\Windows\\Fonts\\simhei.ttf",
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
    ];
    for p in candidates {
        if Path::new(p).exists() {
            if let Ok(bytes) = std::fs::read(p) {
                let mut fonts = egui::FontDefinitions::default();
                fonts
                    .font_data
                    .insert("cjk".to_owned(), egui::FontData::from_owned(bytes));
                if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
                    fam.insert(0, "cjk".to_owned());
                }
                if let Some(fam) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
                    fam.push("cjk".to_owned());
                }
                ctx.set_fonts(fonts);
                return;
            }
        }
    }
}
// ---------------------------------------------------------------------------
// 数据模型
// ---------------------------------------------------------------------------
#[derive(Clone, Default)]
struct HeaderRow {
    key: String,
    value: String,
    disabled: bool,
}

/// 表单类请求体的一行（form-data / x-www-form-urlencoded 通用）。
#[derive(Clone, Default)]
struct KVRow {
    key: String,
    value: String,
    disabled: bool,
    /// form-data 行是否为文件类型（true 时 value 存的是本地文件路径）。
    is_file: bool,
}

struct Tab {
    title: String,
    method: String,
    url: String,
    headers: Vec<HeaderRow>,
    body_type: String,
    body: String,
    /// raw 体的子类型：Text / JSON / XML / HTML / JavaScript
    raw_lang: String,
    /// x-www-form-urlencoded 的行
    url_rows: Vec<KVRow>,
    /// form-data 的行（支持文本与文件）
    form_rows: Vec<KVRow>,
    /// binary 体的本地文件路径
    file_path: String,
    /// GraphQL 查询与变量
    gql_query: String,
    gql_vars: String,
    params: Vec<KVRow>,
    path_rows: Vec<KVRow>,
    hash: String,
    auth_kind: String,
    auth_token: String,
    auth_user: String,
    auth_pass: String,
    auth_key: String,
    auth_value: String,
    /// OAuth2 参数（grant_type / accessTokenUrl / client_id / client_secret / scope / ...）
    oauth2: HashMap<String, String>,
    /// 其他鉴权（oauth1 / awsv4 / jwt / digest / ntlm）的键值参数
    auth_extra: Vec<KVRow>,
    /// 当前标签对应的集合项路径（用于保存/读取 examples）
    item_path: Option<Vec<usize>>,
    /// 请求级网络设置
    req_timeout: String,
    follow_redirects: bool,
    proxy: String,
    insecure: bool,
    client_cert: String,
    client_key: String,
    req_subtab: usize,
    resp_subtab: usize,
    pretty: bool,
    tests: Vec<TestAssertion>,
    tests_text: String,
    /// Postman 风格前置脚本（pm.*）
    pre_script: String,
    /// Postman 风格测试脚本（pm.test(...)）
    test_script: String,
    /// 请求描述（保存到集合 item.description）
    description: String,
    /// 测试脚本运行后的结果
    script_results: Vec<scripting::TestResult>,
    response: Option<ResponseRecord>,
    body_view: String,
    status: String,
    error: Option<String>,
    inflight: bool,
    rx: Option<mpsc::Receiver<(Result<ResponseRecord, String>, Vec<scripting::TestResult>)>>,
}
impl Tab {
    fn new(n: usize) -> Self {
        Tab {
            title: format!("请求 {}", n),
            method: "GET".into(),
            url: String::new(),
            headers: vec![HeaderRow::default()],
            body_type: "none".into(),
            body: String::new(),
            raw_lang: "JSON".into(),
            url_rows: vec![KVRow::default()],
            form_rows: vec![KVRow::default()],
            file_path: String::new(),
            gql_query: String::new(),
            gql_vars: String::new(),
            params: Vec::new(),
            path_rows: Vec::new(),
            hash: String::new(),
            auth_kind: "none".into(),
            auth_token: String::new(),
            auth_user: String::new(),
            auth_pass: String::new(),
            auth_key: String::new(),
            auth_value: String::new(),
            oauth2: {
                let mut m = HashMap::new();
                m.insert("grant_type".to_string(), "client_credentials".to_string());
                m
            },
            auth_extra: Vec::new(),
            item_path: None,
            req_timeout: String::new(),
            follow_redirects: true,
            proxy: String::new(),
            insecure: false,
            client_cert: String::new(),
            client_key: String::new(),
            req_subtab: 2,
            resp_subtab: 0,
            pretty: false,
            tests: Vec::new(),
            tests_text: String::new(),
            pre_script: String::new(),
            test_script: String::new(),
            description: String::new(),
            script_results: Vec::new(),
            response: None,
            body_view: String::new(),
            status: "就绪".into(),
            error: None,
            inflight: false,
            rx: None,
        }
    }
    fn build_request(&self) -> Request {
        let mut headers: Vec<Header> = self
            .headers
            .iter()
            .filter(|h| !h.key.trim().is_empty())
            .map(|h| Header {
                key: h.key.clone(),
                value: h.value.clone(),
                disabled: h.disabled,
                ..Default::default()
            })
            .collect();
        match self.auth_kind.as_str() {
            "bearer" => {
                if !self.auth_token.trim().is_empty() {
                    headers.push(Header {
                        key: "Authorization".into(),
                        value: format!("Bearer {}", self.auth_token.trim()),
                        ..Default::default()
                    });
                }
            }
            "basic" => {
                if !self.auth_user.is_empty() || !self.auth_pass.is_empty() {
                    let raw = format!("{}:{}", self.auth_user, self.auth_pass);
                    let enc = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
                    headers.push(Header {
                        key: "Authorization".into(),
                        value: format!("Basic {}", enc),
                        ..Default::default()
                    });
                }
            }
            "apikey" => {
                if !self.auth_key.trim().is_empty() {
                    headers.push(Header {
                        key: self.auth_key.trim().to_string(),
                        value: self.auth_value.clone(),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }
        let body = match self.body_type.as_str() {
            "" | "none" => None,
            "raw" => {
                if self.body.trim().is_empty() {
                    None
                } else {
                    Some(Body {
                        mode: "raw".into(),
                        raw: Some(self.body.clone()),
                        options: Some(serde_json::json!({
                            "raw": { "language": raw_lang_key(&self.raw_lang) }
                        })),
                        ..Default::default()
                    })
                }
            }
            "x-www-form-urlencoded" => {
                let rows: Vec<KV> = self
                    .url_rows
                    .iter()
                    .filter(|r| !r.key.trim().is_empty())
                    .map(|r| KV {
                        key: r.key.clone(),
                        value: r.value.clone(),
                        disabled: r.disabled,
                        ..Default::default()
                    })
                    .collect();
                if rows.is_empty() {
                    None
                } else {
                    Some(Body {
                        mode: "urlencoded".into(),
                        urlencoded: rows,
                        ..Default::default()
                    })
                }
            }
            "form-data" => {
                let rows: Vec<KV> = self
                    .form_rows
                    .iter()
                    .filter(|r| !r.key.trim().is_empty())
                    .map(|r| KV {
                        key: r.key.clone(),
                        value: if r.is_file {
                            String::new()
                        } else {
                            r.value.clone()
                        },
                        typ: Some(if r.is_file { "file".into() } else { "text".into() }),
                        src: if r.is_file {
                            Some(serde_json::Value::String(r.value.clone()))
                        } else {
                            None
                        },
                        disabled: r.disabled,
                    })
                    .collect();
                if rows.is_empty() {
                    None
                } else {
                    Some(Body {
                        mode: "formdata".into(),
                        formdata: rows,
                        ..Default::default()
                    })
                }
            }
            "binary" => {
                if self.file_path.trim().is_empty() {
                    None
                } else {
                    Some(Body {
                        mode: "file".into(),
                        file: Some(FileRef {
                            src: Some(self.file_path.clone()),
                            content: None,
                        }),
                        ..Default::default()
                    })
                }
            }
            "graphql" => Some(Body {
                mode: "graphql".into(),
                graphql: Some(GraphQL {
                    query: self.gql_query.clone(),
                    variables: self.gql_vars.clone(),
                }),
                ..Default::default()
            }),
            _ => None,
        };
        Request {
            method: self.method.clone(),
            url: UrlSpec {
                raw: self.url.clone(),
                query: self
                    .params
                    .iter()
                    .filter(|r| !r.key.trim().is_empty())
                    .map(|r| KV {
                        key: r.key.clone(),
                        value: r.value.clone(),
                        disabled: r.disabled,
                        ..Default::default()
                    })
                    .collect(),
                variable: self
                    .path_rows
                    .iter()
                    .filter(|r| !r.key.trim().is_empty())
                    .map(|r| KV {
                        key: r.key.clone(),
                        value: r.value.clone(),
                        disabled: r.disabled,
                        ..Default::default()
                    })
                    .collect(),
                hash: if self.hash.trim().is_empty() {
                    None
                } else {
                    Some(self.hash.trim().to_string())
                },
                ..Default::default()
            },
            header: headers,
            body,
            auth: self.build_auth(),
            ..Default::default()
        }
    }
    /// 构造 Postman 兼容的 Auth 对象（目前仅 oauth2 需要交给引擎处理取 token；
    /// bearer/basic/apikey 已在 header 中直接生成）。
    fn build_auth(&self) -> Option<postman_rs::model::Auth> {
        if self.auth_kind == "oauth2" {
            let mut arr = Vec::new();
            for (k, v) in &self.oauth2 {
                if !v.trim().is_empty() {
                    arr.push(serde_json::json!({ "key": k, "value": v }));
                }
            }
            let mut extra = serde_json::Map::new();
            extra.insert("oauth2".into(), serde_json::Value::Array(arr));
            return Some(postman_rs::model::Auth {
                kind: "oauth2".into(),
                extra,
                ..Default::default()
            });
        }
        if matches!(
            self.auth_kind.as_str(),
            "oauth1" | "awsv4" | "jwt" | "digest" | "ntlm"
        ) {
            let arr: Vec<serde_json::Value> = self
                .auth_extra
                .iter()
                .filter(|r| !r.key.trim().is_empty())
                .map(|r| serde_json::json!({ "key": r.key, "value": r.value }))
                .collect();
            let mut extra = serde_json::Map::new();
            extra.insert(self.auth_kind.clone(), serde_json::Value::Array(arr));
            return Some(postman_rs::model::Auth {
                kind: self.auth_kind.clone(),
                extra,
                ..Default::default()
            });
        }
        None
    }
    fn poll(&mut self) -> bool {
        if let Some(rx) = &self.rx {
            match rx.try_recv() {
                Ok((Ok(resp), results)) => {
                    self.status = format!("{} {} ({} ms)", resp.code, resp.status, resp.elapsed_ms);
                    self.body_view = resp.body.clone();
                    self.response = Some(resp);
                    self.script_results = results;
                    self.inflight = false;
                    self.rx = None;
                    return true;
                }
                Ok((Err(e), _)) => {
                    self.error = Some(e);
                    self.status = "请求失败".into();
                    self.script_results.clear();
                    self.inflight = false;
                    self.rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.inflight = false;
                    self.rx = None;
                }
            }
        }
        false
    }
}
struct GuiApp {
    tabs: Vec<Tab>,
    active: usize,
    coll_files: Vec<PathBuf>,
    coll_sel: usize,
    requests: Vec<(String, Request, Option<String>)>,
    tree: Option<Collection>,
    tree_file: Option<PathBuf>,
    coll_new_name: String,
    folder_name: String,
    rename_path: Option<Vec<usize>>,
    rename_text: String,
    env_files: Vec<PathBuf>,
    env_sel: usize,
    history: Vec<storage::HistoryEntry>,
    hist_search: String,
    timeout: u64,
    top_search: String,
    side_search: String,
    sidebar_section: usize,
    save_coll_name: String,
    save_req_name: String,
    export_path: String,
    message: String,
    theme: usize,
    /// 左侧栏宽度（像素）。用固定宽度绘制，避免 SidePanel 记忆内容溢出导致的无限增长。
    side_w: f32,
    follow_system: bool,
    accent_hex: String,
    env_edit: Vec<(String, String)>,
    env_loaded_for: String,
    import_path: String,
    import_format: String,
    /// 从 cURL 粘贴导入的输入内容
    curl_input: String,
    run_iterations: u64,
    run_result: String,
    run_bail: bool,
    run_rx: Option<mpsc::Receiver<String>>,
    run_inflight: bool,
    codegen_lang: String,
    codegen_out: String,
    // WebSocket 协议工作台
    ws_url: String,
    ws_headers: String,
    ws_subprotocol: String,
    ws_send: String,
    ws_listen: u64,
    ws_wait_ms: u64,
    ws_binary: bool,
    ws_log: String,
    ws_busy: bool,
    ws_rx: Option<mpsc::Receiver<Result<Vec<String>, String>>>,
    /// 响应"保存到文件"的目标路径（留空则用默认 downloads/）
    resp_save_path: String,
    /// 全局变量编辑缓冲
    globals_edit: Vec<(String, String)>,
    globals_loaded: bool,
    /// Cookie 管理器新增输入
    cookie_new_host: String,
    cookie_new_kv: String,
    /// 控制台日志（请求/响应/脚本）
    console: Vec<String>,
}
impl GuiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_fonts(&cc.egui_ctx);
        let theme_idx = initial_theme();
        apply_theme(&cc.egui_ctx, theme_idx);
        let mut app = GuiApp {
            tabs: vec![Tab::new(1)],
            active: 0,
            coll_files: Vec::new(),
            coll_sel: 0,
            requests: Vec::new(),
            tree: None,
            tree_file: None,
            coll_new_name: String::new(),
            folder_name: String::new(),
            rename_path: None,
            rename_text: String::new(),
            env_files: Vec::new(),
            env_sel: 0,
            history: Vec::new(),
            hist_search: String::new(),
            timeout: 30,
            top_search: String::new(),
            side_search: String::new(),
            sidebar_section: std::env::var("PMRS_GUI_SIDEBAR")
                .ok()
                .and_then(|s| s.trim().parse().ok())
                .filter(|n| *n <= 5)
                .unwrap_or(0),
            save_coll_name: String::new(),
            save_req_name: String::new(),
            export_path: String::new(),
            message: String::new(),
            theme: theme_idx,
            side_w: 320.0,
            follow_system: false,
            accent_hex: String::new(),
            env_edit: Vec::new(),
            env_loaded_for: String::new(),
            import_path: String::new(),
            import_format: "postman".into(),
            curl_input: String::new(),
            run_iterations: 1,
            run_result: String::new(),
            run_bail: false,
            run_rx: None,
            run_inflight: false,
            codegen_lang: "curl".into(),
            codegen_out: String::new(),
            ws_url: "wss://echo.websocket.events".into(),
            ws_headers: String::new(),
            ws_subprotocol: String::new(),
            ws_send: String::new(),
            ws_listen: 1,
            ws_wait_ms: 3000,
            ws_binary: false,
            ws_log: String::new(),
            ws_busy: false,
            ws_rx: None,
            resp_save_path: String::new(),
            globals_edit: Vec::new(),
            globals_loaded: false,
            cookie_new_host: String::new(),
            cookie_new_kv: String::new(),
            console: Vec::new(),
        };
        app.refresh_all();
        app
    }
    fn refresh_all(&mut self) {
        self.coll_files = storage::list_collections().unwrap_or_default();
        self.env_files = storage::list_environments().unwrap_or_default();
        self.history = storage::read_history().unwrap_or_default();
        if self.coll_sel >= self.coll_files.len() {
            self.coll_sel = 0;
        }
        self.load_requests();
    }
    fn load_requests(&mut self) {
        self.requests.clear();
        self.tree = None;
        self.tree_file = None;
        if let Some(path) = self.coll_files.get(self.coll_sel) {
            if let Ok(text) = std::fs::read_to_string(path) {
                if let Ok(c) = serde_json::from_str::<Collection>(&text) {
                    flatten(&c.item, "", &mut self.requests);
                    self.tree = Some(c);
                    self.tree_file = Some(path.clone());
                }
            }
        }
    }
    fn apply_request_to_active(&mut self, idx: usize) {
        if let Some((name, req, desc)) = self.requests.get(idx).cloned() {
            let tab = &mut self.tabs[self.active];
            tab.title = short_title(&name);
            tab.description = desc.unwrap_or_default();
            tab.method = req.method.clone();
            tab.url = if req.url.raw.trim().is_empty() {
                req.url.effective_url()
            } else {
                req.url.raw.clone()
            };
            tab.params = if req.url.query.is_empty() {
                parse_query(&tab.url)
            } else {
                req.url
                    .query
                    .iter()
                    .map(|kv| KVRow {
                        key: kv.key.clone(),
                        value: kv.value.clone(),
                        disabled: kv.disabled,
                        is_file: false,
                    })
                    .collect()
            };
            tab.hash = req.url.hash.clone().unwrap_or_default();
            tab.path_rows = if req.url.variable.is_empty() {
                detect_path_vars(&tab.url)
            } else {
                req.url
                    .variable
                    .iter()
                    .map(|kv| KVRow {
                        key: kv.key.clone(),
                        value: kv.value.clone(),
                        disabled: kv.disabled,
                        is_file: false,
                    })
                    .collect()
            };
            tab.headers = req
                .header
                .iter()
                .map(|h| HeaderRow {
                    key: h.key.clone(),
                    value: h.value.clone(),
                    disabled: h.disabled,
                })
                .collect();
            if tab.headers.is_empty() {
                tab.headers.push(HeaderRow::default());
            }
            // 还原授权信息
            tab.auth_kind = "none".into();
            tab.auth_token.clear();
            tab.auth_user.clear();
            tab.auth_pass.clear();
            tab.auth_key.clear();
            tab.auth_value.clear();
            tab.oauth2.clear();
            tab.oauth2
                .insert("grant_type".into(), "client_credentials".into());
            tab.auth_extra.clear();
            if let Some(a) = &req.auth {
                match a.kind.as_str() {
                    "bearer" => {
                        tab.auth_kind = "bearer".into();
                        tab.auth_token = a.bearer_token().unwrap_or("").to_string();
                    }
                    "basic" => {
                        tab.auth_kind = "basic".into();
                        tab.auth_user = a.basic_user().unwrap_or("").to_string();
                        tab.auth_pass = a.basic_pass().unwrap_or("").to_string();
                    }
                    "apikey" => {
                        tab.auth_kind = "apikey".into();
                        tab.auth_key = a.apikey_key().unwrap_or("").to_string();
                        tab.auth_value = a.apikey_value().unwrap_or("").to_string();
                    }
                    "oauth2" => {
                        tab.auth_kind = "oauth2".into();
                        for (k, v) in a.oauth2_params() {
                            tab.oauth2.insert(k, v);
                        }
                    }
                    k @ ("oauth1" | "awsv4" | "jwt" | "digest" | "ntlm") => {
                        tab.auth_kind = k.to_string();
                        if let Some(serde_json::Value::Array(arr)) = a.extra.get(k) {
                            for it in arr {
                                let key = it
                                    .get("key")
                                    .and_then(|x| x.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let value = it
                                    .get("value")
                                    .map(|v| {
                                        v.as_str()
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| v.to_string())
                                    })
                                    .unwrap_or_default();
                                tab.auth_extra.push(KVRow {
                                    key,
                                    value,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                    _ => {}
                }
            }
            tab.body.clear();
            tab.url_rows = vec![KVRow::default()];
            tab.form_rows = vec![KVRow::default()];
            tab.file_path.clear();
            tab.gql_query.clear();
            tab.gql_vars.clear();
            tab.body_type = "none".into();
            if let Some(b) = &req.body {
                match b.mode.as_str() {
                    "raw" => {
                        tab.body_type = "raw".into();
                        tab.body = b.raw.clone().unwrap_or_default();
                        let lang = b
                            .options
                            .as_ref()
                            .and_then(|o| o.get("raw"))
                            .and_then(|r| r.get("language"))
                            .and_then(|l| l.as_str())
                            .unwrap_or("json");
                        tab.raw_lang = raw_lang_label(lang).to_string();
                    }
                    "urlencoded" => {
                        tab.body_type = "x-www-form-urlencoded".into();
                        tab.url_rows = b
                            .urlencoded
                            .iter()
                            .map(|kv| KVRow {
                                key: kv.key.clone(),
                                value: kv.value.clone(),
                                disabled: kv.disabled,
                                is_file: false,
                            })
                            .collect();
                        if tab.url_rows.is_empty() {
                            tab.url_rows.push(KVRow::default());
                        }
                    }
                    "formdata" => {
                        tab.body_type = "form-data".into();
                        tab.form_rows = b
                            .formdata
                            .iter()
                            .map(|kv| {
                                let is_file = kv
                                    .typ
                                    .as_deref()
                                    .map(|t| t.eq_ignore_ascii_case("file"))
                                    .unwrap_or(false);
                                let value = if is_file {
                                    kv.src
                                        .as_ref()
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("")
                                        .to_string()
                                } else {
                                    kv.value.clone()
                                };
                                KVRow {
                                    key: kv.key.clone(),
                                    value,
                                    disabled: kv.disabled,
                                    is_file,
                                }
                            })
                            .collect();
                        if tab.form_rows.is_empty() {
                            tab.form_rows.push(KVRow::default());
                        }
                    }
                    "file" => {
                        tab.body_type = "binary".into();
                        tab.file_path = b
                            .file
                            .as_ref()
                            .and_then(|f| f.src.clone())
                            .unwrap_or_default();
                    }
                    "graphql" => {
                        tab.body_type = "graphql".into();
                        if let Some(g) = &b.graphql {
                            tab.gql_query = g.query.clone();
                            tab.gql_vars = g.variables.clone();
                        }
                    }
                    _ => {}
                }
            }
            tab.tests = req.tests.clone();
            tab.tests.extend(tests_engine::extract_from_events(&req.event));
            tab.tests_text = serde_json::to_string_pretty(&tab.tests).unwrap_or_default();
            self.message = format!("已载入请求：{}", name);
        }
    }
    fn send_active(&mut self, ctx: &egui::Context) {
        let env_path = self.env_files.get(self.env_sel).cloned();
        let coll_path = self.coll_files.get(self.coll_sel).cloned();
        let global_timeout = self.timeout;
        let tab = &mut self.tabs[self.active];
        if tab.inflight {
            return;
        }
        let req = tab.build_request();
        let cfg = http::NetConfig {
            timeout_secs: tab
                .req_timeout
                .trim()
                .parse::<u64>()
                .unwrap_or(global_timeout),
            connect_timeout_secs: None,
            follow_redirects: Some(tab.follow_redirects),
            max_redirects: None,
            proxy: if tab.proxy.trim().is_empty() {
                None
            } else {
                Some(tab.proxy.trim().to_string())
            },
            insecure: tab.insecure,
            client_cert: if tab.client_cert.trim().is_empty() {
                None
            } else {
                Some(tab.client_cert.trim().to_string())
            },
            client_key: if tab.client_key.trim().is_empty() {
                None
            } else {
                Some(tab.client_key.trim().to_string())
            },
        };
        let (tx, rx) = mpsc::channel();
        tab.rx = Some(rx);
        tab.inflight = true;
        tab.error = None;
        tab.script_results.clear();
        tab.status = "发送中…".into();
        let pre_script = tab.pre_script.clone();
        let test_script = tab.test_script.clone();
        let item_path = tab.item_path.clone();
        let auth_kind = tab.auth_kind.clone();
        let method_s = tab.method.clone();
        let url_s = tab.url.clone();
        self.console.push(format!(
            "[{}] → {} {}",
            chrono::Local::now().format("%H:%M:%S"),
            method_s,
            url_s
        ));
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            let coll = coll_path
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|t| serde_json::from_str::<Collection>(&t).ok());
            let env = env_path
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|t| serde_json::from_str::<Environment>(&t).ok());
            let mut vars = ctx_from(coll.as_ref(), env.as_ref());
            // 继承：集合/文件夹级 auth（当请求自身未设时）与文件夹级变量、pre-request/test 脚本。
            let mut req = req;
            let mut pre_script = pre_script;
            let mut test_script = test_script;
            if let (Some(c), Some(path)) = (coll.as_ref(), item_path.as_ref()) {
                let (inh_auth, inh_vars, inh_events) = collect_inherited(c, path);
                // 仅当标签自身未设置鉴权（未选择或"继承"）时才套用集合/文件夹级鉴权，
                // 避免与标签内已生成的 Authorization 头重复。
                if req.auth.is_none() && (auth_kind == "none" || auth_kind == "inherit") {
                    req.auth = inh_auth;
                }
                if !inh_vars.is_empty() {
                    let idx = 1.min(vars.scopes.len());
                    vars.scopes.insert(idx, inh_vars);
                }
                let inh_pre = scripting::scripts_of(&inh_events, "prerequest");
                if !inh_pre.trim().is_empty() {
                    pre_script = format!("{}\n{}", inh_pre, pre_script);
                }
                let inh_test = scripting::scripts_of(&inh_events, "test");
                if !inh_test.trim().is_empty() {
                    test_script = format!("{}\n{}", test_script, inh_test);
                }
            }
            // 前置脚本：pm.<scope>.set(...) 注入为本次请求可用的变量
            if !pre_script.trim().is_empty() {
                let senv = scripting::ScriptEnv {
                    vars: vars.merged(),
                    iter: HashMap::new(),
                    method: req.method.clone(),
                    url: req.url.effective_url(),
                };
                let out = scripting::run_env(&pre_script, None, &senv);
                if !out.sets.is_empty() {
                    let mut map = HashMap::new();
                    for (_scope, k, v) in out.sets {
                        map.insert(k, v);
                    }
                    vars.push_high(map);
                }
            }
            let res = http::execute_cfg(&req, &vars, &cfg).map_err(|e| e.to_string());
            // 测试脚本：针对响应执行 pm.test(...) 断言
            let results = match (&res, test_script.trim().is_empty()) {
                (Ok(resp), false) => {
                    let senv = scripting::ScriptEnv {
                        vars: vars.merged(),
                        iter: HashMap::new(),
                        method: req.method.clone(),
                        url: req.url.effective_url(),
                    };
                    scripting::run_env(&test_script, Some(resp), &senv).tests
                }
                _ => Vec::new(),
            };
            let _ = tx.send((res, results));
            ctx2.request_repaint();
        });
    }
    fn save_active(&mut self) {
        let req = self.tabs[self.active].build_request();
        let tab_title = self.tabs[self.active].title.clone();
        let tab_desc = self.tabs[self.active].description.clone();
        let coll_name = {
            let s = self.save_coll_name.trim();
            if s.is_empty() {
                self.coll_files
                    .get(self.coll_sel)
                    .and_then(|p| p.file_stem().map(|x| x.to_string_lossy().to_string()))
                    .unwrap_or_else(|| "我的集合".to_string())
            } else {
                s.to_string()
            }
        };
        let req_name = {
            let s = self.save_req_name.trim();
            if s.is_empty() {
                tab_title.clone()
            } else {
                s.to_string()
            }
        };
        let existing = self
            .coll_files
            .iter()
            .find(|p| p.file_stem().map(|x| x.to_string_lossy().to_string()) == Some(coll_name.clone()))
            .cloned();
        let mut coll = existing
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str::<Collection>(&t).ok())
            .unwrap_or_else(|| Collection {
                info: CollectionInfo {
                    name: coll_name.clone(),
                    postman_id: Some(uuid::Uuid::new_v4().to_string()),
                    schema: Some(
                        "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into(),
                    ),
                    description: None,
                },
                ..Default::default()
            });
        coll.item.push(Item {
            name: req_name.clone(),
            request: Some(req),
            description: if tab_desc.trim().is_empty() {
                None
            } else {
                Some(serde_json::Value::String(tab_desc))
            },
            ..Default::default()
        });
        match storage::save_collection(&coll) {
            Ok(p) => {
                self.message = format!("已保存请求「{}」到集合「{}」→ {}", req_name, coll_name, p.display());
                self.refresh_all();
            }
            Err(e) => self.message = format!("保存失败：{}", e),
        }
    }
    /// 把当前响应体写入文件。路径留空时保存到 ~/.postman-rs/downloads/。
    fn save_response_to_file(&mut self, body: &str) {
        let path: PathBuf = if self.resp_save_path.trim().is_empty() {
            let dir = storage::home().join("downloads");
            let _ = std::fs::create_dir_all(&dir);
            dir.join(format!(
                "response_{}.txt",
                chrono::Local::now().format("%Y%m%d_%H%M%S")
            ))
        } else {
            PathBuf::from(self.resp_save_path.trim())
        };
        match std::fs::write(&path, body) {
            Ok(_) => self.message = format!("响应已保存 → {}", path.display()),
            Err(e) => self.message = format!("保存响应失败：{}", e),
        }
    }
    fn export_selected(&mut self) {
        let path = match self.coll_files.get(self.coll_sel) {
            Some(p) => p.clone(),
            None => {
                self.message = "没有可导出的集合".into();
                return;
            }
        };
        let mut coll = match std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Collection>(&t).ok())
        {
            Some(c) => c,
            None => {
                self.message = "读取集合失败".into();
                return;
            }
        };
        let out = if self.export_path.trim().is_empty() {
            let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            let dir = storage::home().join("exports");
            let _ = std::fs::create_dir_all(&dir);
            dir.join(format!("{}.json", stem))
        } else {
            PathBuf::from(self.export_path.trim())
        };
        match import_export::export_collection(&mut coll) {
            Ok(text) => match std::fs::write(&out, text) {
                Ok(_) => self.message = format!("已导出 → {}", out.display()),
                Err(e) => self.message = format!("写入失败：{}", e),
            },
            Err(e) => self.message = format!("导出失败：{}", e),
        }
    }
    fn import_now(&mut self) {
        let path = self.import_path.trim().to_string();
        if path.is_empty() {
            self.message = "请输入要导入的文件路径".into();
            return;
        }
        let fmt = self.import_format.clone();
        match import_export::import_file(&path, &fmt) {
            Ok(import_export::Imported::Collection(c_box)) => {
                let mut c = *c_box;
                let name = c.info.name.clone();
                let existing = self
                    .coll_files
                    .iter()
                    .find(|p| p.file_stem().map(|x| x.to_string_lossy().to_string()) == Some(name.clone()))
                    .cloned();
                if let Some(ep) = existing {
                    if let Ok(t) = std::fs::read_to_string(&ep) {
                        if let Ok(mut base) = serde_json::from_str::<Collection>(&t) {
                            import_export::merge_collections(&mut base, c);
                            c = base;
                            self.message = format!("已合并导入集合「{}」", name);
                        }
                    }
                } else {
                    self.message = format!("已导入集合「{}」", name);
                }
                match storage::save_collection(&c) {
                    Ok(p) => {
                        self.message.push_str(&format!(" → {}", p.display()));
                        self.refresh_all();
                    }
                    Err(e) => self.message = format!("保存失败：{}", e),
                }
            }
            Ok(import_export::Imported::Environment(e)) => {
                let name = e.name.clone();
                match storage::save_environment(&e) {
                    Ok(p) => {
                        self.message = format!("已导入环境「{}」 → {}", name, p.display());
                        self.refresh_all();
                    }
                    Err(err) => self.message = format!("保存环境失败：{}", err),
                }
            }
            Err(e) => self.message = format!("导入失败：{}", e),
        }
    }
    fn run_current_collection(&mut self) {
        if self.run_inflight {
            return;
        }
        let path = match self.coll_files.get(self.coll_sel) {
            Some(p) => p.clone(),
            None => {
                self.message = "没有可运行的集合".into();
                return;
            }
        };
        let env_path = self.env_files.get(self.env_sel).cloned();
        let iterations = self.run_iterations.max(1) as usize;
        let bail = self.run_bail;
        let timeout = self.timeout;
        let (tx, rx) = mpsc::channel();
        self.run_rx = Some(rx);
        self.run_inflight = true;
        self.run_result = "运行中…".into();
        std::thread::spawn(move || {
            let res = (|| -> Result<String, String> {
                let c: Collection = serde_json::from_str(
                    &std::fs::read_to_string(&path).map_err(|e| e.to_string())?,
                )
                .map_err(|e| e.to_string())?;
                let env = env_path
                    .and_then(|p| std::fs::read_to_string(p).ok())
                    .and_then(|t| serde_json::from_str::<Environment>(&t).ok());
                let summary = runner::run_collection_opts(
                    &c,
                    env.as_ref(),
                    iterations,
                    Vec::new(),
                    0,
                    timeout,
                    RunOptions {
                        folder: None,
                        bail,
                        report: None,
                    },
                );
                let mut out = format!(
                    "迭代 {} 次 · 总计 {} / 通过 {} / 失败 {}\n\n",
                    iterations, summary.total, summary.passed, summary.failed
                );
                for r in &summary.results {
                    let code = r.code.map(|c| c.to_string()).unwrap_or_else(|| "-".into());
                    out.push_str(&format!(
                        "{} {} {} [{}] {}ms {}\n",
                        if r.passed { "✓" } else { "✗" },
                        r.method,
                        r.url,
                        code,
                        r.elapsed_ms,
                        r.error
                            .as_ref()
                            .map(|e| format!("错误：{}", e))
                            .unwrap_or_default()
                    ));
                    for t in &r.tests {
                        out.push_str(&format!(
                            "    {} {}\n",
                            if t.passed { "· 通过" } else { "· 失败" },
                            t.name
                        ));
                    }
                }
                Ok(out)
            })();
            let _ = tx.send(res.unwrap_or_else(|e| format!("运行失败：{}", e)));
        });
    }
    fn generate_code(&mut self) {
        let req = self.tabs[self.active].build_request();
        match codegen::generate(&req, &self.codegen_lang) {
            Ok(code) => self.codegen_out = code,
            Err(e) => self.codegen_out = format!("生成失败：{}", e),
        }
    }
    // ---- 顶部栏 -----------------------------------------------------------
    fn ui_topbar(&mut self, ctx: &egui::Context) {
        let ctx2 = ctx.clone();
        egui::TopBottomPanel::top("topbar")
            .frame(
                egui::Frame::none()
                    .fill(bg_panel())
                    .inner_margin(egui::Margin::symmetric(14.0, 10.0)),
            )
            .show(ctx, |ui| {
                top_accent(ui);
                ui.horizontal(|ui| {
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, Rounding::same(R_MD), brand());
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "P",
                        egui::FontId::proportional(17.0),
                        Color32::WHITE,
                    );
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("pmrs").strong().size(17.0).color(text_primary()));
                        ui.label(
                            RichText::new("本地 API 客户端")
                                .color(text_secondary())
                                .size(10.5),
                        );
                    });
                    ui.add_space(14.0);
                    ui.separator();
                    ui.add_space(6.0);
                    egui::Frame::none()
                        .fill(mix(bg_panel(), text_primary(), 0.05))
                        .rounding(Rounding::same(R_MD))
                        .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new("搜索")
                                        .color(text_secondary())
                                        .size(11.5),
                                );
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.top_search)
                                        .frame(false)
                                        .hint_text("集合 / 请求…")
                                        .desired_width(250.0),
                                );
                            });
                        });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(RichText::new("保存").color(Color32::WHITE).strong())
                                    .fill(brand())
                                    .rounding(Rounding::same(R_MD))
                                    .min_size(egui::vec2(64.0, 30.0)),
                            )
                            .clicked()
                        {
                            self.save_active();
                        }
                        ui.add_space(10.0);
                        let mut sel = self.theme;
                        egui::ComboBox::from_id_salt("theme_sel")
                            .width(120.0)
                            .selected_text(
                                RichText::new(THEMES[sel.min(THEMES.len() - 1)].name)
                                    .color(text_primary())
                                    .size(12.0),
                            )
                            .show_ui(ui, |ui| {
                                for i in 0..THEMES.len() {
                                    ui.selectable_value(&mut sel, i, THEMES[i].name);
                                }
                            });
                        if sel != self.theme {
                            self.theme = sel;
                            self.follow_system = false;
                            FOLLOW_SYSTEM.store(false, Ordering::Relaxed);
                            apply_theme(&ctx2, sel);
                            save_theme(sel);
                        }
                        ui.label(RichText::new("主题").color(text_secondary()).size(11.5));
                        ui.add_space(10.0);
                        let cur = self
                            .env_files
                            .get(self.env_sel)
                            .map(file_name)
                            .unwrap_or_else(|| "无环境".into());
                        egui::ComboBox::from_id_salt("env_top")
                            .width(140.0)
                            .selected_text(RichText::new(cur).color(text_primary()).size(12.0))
                            .show_ui(ui, |ui| {
                                for i in 0..self.env_files.len() {
                                    ui.selectable_value(&mut self.env_sel, i, file_name(&self.env_files[i]));
                                }
                            });
                        ui.label(RichText::new("环境").color(text_secondary()).size(11.5));
                        ui.add_space(10.0);
                        ui.add(egui::DragValue::new(&mut self.timeout).range(1..=600).suffix("s"));
                        ui.label(RichText::new("超时").color(text_secondary()).size(11.5));
                        ui.add_space(8.0);
                        if ui
                            .add(
                                egui::Button::new(RichText::new("刷新").color(text_secondary()).size(12.0))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            self.refresh_all();
                        }
                    });
                });
                let r = ui.max_rect();
                ui.painter()
                    .hline(r.left()..=r.right(), r.bottom(), Stroke::new(1.0_f32, border()));
            });
    }
    // ---- 左侧栏 -----------------------------------------------------------
    fn ui_sidebar(&mut self, ctx: &egui::Context) {
        // 使用固定宽度（而非 resizable 的 SidePanel）：后者会把内容 1~2px 的溢出
        // 记忆为新的面板宽度，导致每次重绘/鼠标移动时左栏不断变宽。这里自行维护宽度，
        // 并在右缘绘制一个可拖拽手柄来保留手动调整能力。
        let w = self.side_w.clamp(240.0, 520.0);
        egui::SidePanel::left("side")
            .resizable(false)
            .exact_width(w)
            .frame(
                egui::Frame::none()
                    .fill(bg_side())
                    .inner_margin(egui::Margin::same(12.0)),
            )
            .show(ctx, |ui| {
                segmented(
                    ui,
                    &mut self.sidebar_section,
                    &["集合", "环境", "全局", "历史", "工具", "控制台"],
                );
                ui.add_space(10.0);
                match self.sidebar_section {
                    0 => self.ui_side_collections(ui),
                    1 => self.ui_side_env(ui),
                    2 => self.ui_side_globals(ui),
                    3 => self.ui_side_history(ui),
                    5 => self.ui_side_console(ui),
                    _ => self.ui_side_tools(ui),
                }
                // 右侧细边框
                let r = ui.max_rect();
                ui.painter()
                    .vline(r.right(), r.top()..=r.bottom(), Stroke::new(1.0_f32, border()));

                // 自绘拖拽手柄（位于面板外缘的 12px 留白内）
                let edge = r.right() + 11.0;
                let handle = egui::Rect::from_min_max(
                    egui::pos2(edge - 4.0, r.top()),
                    egui::pos2(edge + 4.0, r.bottom()),
                );
                let hr = ui.interact(handle, ui.id().with("side_resize"), egui::Sense::drag());
                if hr.hovered() || hr.dragged() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
                if hr.dragged() {
                    self.side_w = (self.side_w + hr.drag_delta().x).clamp(240.0, 520.0);
                }
            });
    }
    fn ui_side_collections(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "集合");
        ui.add_space(6.0);
        let cur = self
            .coll_files
            .get(self.coll_sel)
            .map(file_name)
            .unwrap_or_else(|| "（无集合）".into());
        let mut changed = false;
        egui::ComboBox::from_id_salt("coll_sel")
            .width(ui.available_width())
            .selected_text(RichText::new(cur).color(text_primary()).size(12.5))
            .show_ui(ui, |ui| {
                for i in 0..self.coll_files.len() {
                    if ui
                        .selectable_value(&mut self.coll_sel, i, file_name(&self.coll_files[i]))
                        .clicked()
                    {
                        changed = true;
                    }
                }
            });
        if changed {
            self.load_requests();
        }
        // 新建集合
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            let w = (ui.available_width() - 76.0).max(60.0);
            ui.add(
                egui::TextEdit::singleline(&mut self.coll_new_name)
                    .hint_text("新集合名称")
                    .desired_width(w),
            );
            if ui
                .add(
                    egui::Button::new(RichText::new("＋集合").color(brand()))
                        .fill(brand_soft())
                        .rounding(Rounding::same(6.0)),
                )
                .clicked()
            {
                self.create_collection();
            }
        });
        if ui
            .add(
                egui::Button::new(
                    RichText::new("删除当前集合").color(Color32::from_rgb(235, 32, 19)),
                )
                .frame(false),
            )
            .clicked()
        {
            self.delete_collection();
        }
        ui.add_space(6.0);
        egui::Frame::none()
            .fill(mix(bg_side(), text_primary(), 0.05))
            .rounding(Rounding::same(R_MD))
            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.side_search)
                        .frame(false)
                        .hint_text("筛选请求…")
                        .desired_width(f32::INFINITY),
                );
            });
        ui.add_space(6.0);
        let q = if self.top_search.trim().is_empty() {
            self.side_search.to_lowercase()
        } else {
            self.top_search.to_lowercase()
        };
        // 集合树快照
        let rows = self
            .tree
            .as_ref()
            .map(|c| {
                let mut r = Vec::new();
                let mut p = Vec::new();
                collect_rows(&c.item, 0, &mut p, &mut r);
                r
            })
            .unwrap_or_default();
        let ren = self.rename_path.clone();
        let mut clicked: Option<Vec<usize>> = None;
        let mut del: Option<Vec<usize>> = None;
        let mut dup: Option<Vec<usize>> = None;
        let mut up: Option<Vec<usize>> = None;
        let mut down: Option<Vec<usize>> = None;
        let mut start_rename: Option<(Vec<usize>, String, bool)> = None; // path,name,is_commit
        let mut do_new_folder = false;
        egui::ScrollArea::vertical()
            .id_salt("reqtree")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let w = (ui.available_width() - 86.0).max(50.0);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.folder_name)
                            .hint_text("新文件夹名")
                            .desired_width(w),
                    );
                    if ui
                        .add(
                            egui::Button::new(RichText::new("＋文件夹").color(brand()))
                                .fill(brand_soft())
                                .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        do_new_folder = true;
                    }
                });
                if rows.is_empty() {
                    ui.label(RichText::new("（此集合暂无请求）").color(text_secondary()).size(12.0));
                }
                for row in &rows {
                    if !q.is_empty()
                        && !row.name.to_lowercase().contains(&q)
                        && !row
                            .method
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .contains(&q)
                    {
                        continue;
                    }
                    let ind = row.depth as f32 * 14.0;
                    ui.horizontal_wrapped(|ui| {
                        ui.add_space(ind);
                        if row.is_folder {
                            ui.label(RichText::new("📁").size(13.0));
                        } else {
                            method_pill(ui, row.method.as_deref().unwrap_or("GET"));
                        }
                        if ren.as_deref() == Some(row.path.as_slice()) {
                            ui.add(
                                egui::TextEdit::singleline(&mut self.rename_text)
                                    .desired_width(130.0),
                            );
                            if ui.small_button("确定").clicked() {
                                start_rename =
                                    Some((row.path.clone(), self.rename_text.clone(), true));
                            }
                        } else {
                            let resp = ui.add(
                                egui::Button::new(
                                    RichText::new(&row.name).color(text_primary()).size(13.0),
                                )
                                .frame(false),
                            );
                            if resp.clicked() && !row.is_folder {
                                clicked = Some(row.path.clone());
                            }
                            if resp.double_clicked() {
                                start_rename =
                                    Some((row.path.clone(), row.name.clone(), false));
                            }
                        }
                        if !row.is_folder {
                            if ui.small_button("↑").clicked() {
                                up = Some(row.path.clone());
                            }
                            if ui.small_button("↓").clicked() {
                                down = Some(row.path.clone());
                            }
                            if ui.small_button("复制").clicked() {
                                dup = Some(row.path.clone());
                            }
                        }
                        if ui.small_button("重命名").clicked() {
                            start_rename = Some((row.path.clone(), row.name.clone(), false));
                        }
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("×").color(text_secondary()).size(12.0),
                                )
                                .frame(false),
                            )
                            .clicked()
                        {
                            del = Some(row.path.clone());
                        }
                    });
                    ui.add_space(1.0);
                }
            });
        // ---- 应用树操作 ----
        if let Some(p) = del {
            if let Some(c) = self.tree.as_mut() {
                remove_at(&mut c.item, &p);
            }
            self.save_tree();
        }
        if let Some(p) = dup {
            self.duplicate_item(&p);
        }
        if let Some(p) = up {
            self.move_item(&p, true);
        }
        if let Some(p) = down {
            self.move_item(&p, false);
        }
        if let Some((p, name, commit)) = start_rename {
            if commit {
                if let Some(c) = self.tree.as_mut() {
                    if let Some(it) = item_at_mut(&mut c.item, &p) {
                        let nn = self.rename_text.trim().to_string();
                        if !nn.is_empty() {
                            it.name = nn;
                        }
                    }
                }
                self.rename_path = None;
                self.save_tree();
            } else {
                self.rename_path = Some(p);
                self.rename_text = name;
            }
        }
        if do_new_folder {
            let name = if self.folder_name.trim().is_empty() {
                "新建文件夹".to_string()
            } else {
                self.folder_name.trim().to_string()
            };
            if let Some(c) = self.tree.as_mut() {
                c.item.push(Item {
                    name,
                    item: Some(Vec::new()),
                    ..Default::default()
                });
            }
            self.folder_name.clear();
            self.save_tree();
        }
        if let Some(p) = clicked {
            if let Some(it) = self.tree.as_ref().and_then(|c| item_at(&c.item, &p)) {
                if let Some(req) = &it.request {
                    let leaf = it.name.rsplit('/').next().unwrap_or(&it.name).to_string();
                    if let Some(idx) = self.requests.iter().position(|(n, r, _)| {
                        n.ends_with(&leaf)
                            && r.method == req.method
                            && r.url.raw == req.url.raw
                    }) {
                        self.apply_request_to_active(idx);
                        self.tabs[self.active].item_path = Some(p.clone());
                    }
                }
            }
        }
    }
    fn create_collection(&mut self) {
        let name = if self.coll_new_name.trim().is_empty() {
            "新建集合".to_string()
        } else {
            self.coll_new_name.trim().to_string()
        };
        let c = Collection {
            info: CollectionInfo {
                name: name.clone(),
                postman_id: Some(uuid::Uuid::new_v4().to_string()),
                schema: Some(
                    "https://schema.getpostman.com/json/collection/v2.1.0/collection.json".into(),
                ),
                description: None,
            },
            ..Default::default()
        };
        match storage::save_collection(&c) {
            Ok(p) => {
                self.message = format!("已新建集合：{}", name);
                self.coll_new_name.clear();
                self.coll_files = storage::list_collections().unwrap_or_default();
                if let Some(i) = self.coll_files.iter().position(|x| x == &p) {
                    self.coll_sel = i;
                }
                self.load_requests();
            }
            Err(e) => self.message = format!("新建集合失败：{}", e),
        }
    }
    fn delete_collection(&mut self) {
        if let Some(path) = self.coll_files.get(self.coll_sel).cloned() {
            if std::fs::remove_file(&path).is_ok() {
                self.message = "已删除集合".into();
                self.coll_sel = 0;
                self.refresh_all();
            } else {
                self.message = "删除集合失败".into();
            }
        }
    }
    fn save_tree(&mut self) {
        if let (Some(c), Some(path)) = (self.tree.as_ref(), self.tree_file.clone()) {
            match serde_json::to_string_pretty(c) {
                Ok(t) => {
                    if let Err(e) = std::fs::write(&path, t) {
                        self.message = format!("保存集合失败：{}", e);
                        return;
                    }
                }
                Err(e) => {
                    self.message = format!("序列化失败：{}", e);
                    return;
                }
            }
        } else {
            return;
        }
        self.message = "已保存集合改动".into();
        let sel = self.coll_sel;
        self.coll_files = storage::list_collections().unwrap_or_default();
        self.coll_sel = sel.min(self.coll_files.len().saturating_sub(1));
        self.load_requests();
    }
    fn duplicate_item(&mut self, path: &[usize]) {
        let Some(c) = self.tree.as_mut() else { return };
        let Some(src) = item_at(&c.item, path).cloned() else {
            return;
        };
        let mut clone = src;
        clone.name = format!("{} 副本", clone.name);
        let idx = *path.last().unwrap_or(&0);
        let parent = &path[..path.len().saturating_sub(1)];
        if parent.is_empty() {
            c.item.insert((idx + 1).min(c.item.len()), clone);
        } else if let Some(pi) = item_at_mut(&mut c.item, parent) {
            if let Some(children) = pi.item.as_mut() {
                children.insert((idx + 1).min(children.len()), clone);
            }
        }
        self.save_tree();
    }
    fn move_item(&mut self, path: &[usize], up: bool) {
        if path.is_empty() {
            return;
        }
        let idx = path[path.len() - 1];
        let parent = &path[..path.len() - 1];
        {
            let Some(c) = self.tree.as_mut() else { return };
            let siblings: &mut Vec<Item> = if parent.is_empty() {
                &mut c.item
            } else if let Some(pi) = item_at_mut(&mut c.item, parent) {
                match pi.item.as_mut() {
                    Some(v) => v,
                    None => return,
                }
            } else {
                return;
            };
            if up {
                if idx > 0 {
                    siblings.swap(idx, idx - 1);
                }
            } else if idx + 1 < siblings.len() {
                siblings.swap(idx, idx + 1);
            }
        }
        self.save_tree();
    }
    /// 当前标签对应集合项的已保存示例响应。
    fn examples_for_active(&self) -> Vec<ResponseExample> {
        let Some(p) = self.tabs[self.active].item_path.as_ref() else {
            return Vec::new();
        };
        self.tree
            .as_ref()
            .and_then(|c| item_at(&c.item, p))
            .map(|it| it.response.clone())
            .unwrap_or_default()
    }
    fn save_example(&mut self) {
        let active = self.active;
        let (path, resp) = {
            let t = &self.tabs[active];
            (t.item_path.clone(), t.response.clone())
        };
        let Some(path) = path else {
            self.message = "请先从左侧集合树打开此请求，再保存示例".into();
            return;
        };
        let Some(r) = resp else {
            self.message = "暂无响应可保存为示例".into();
            return;
        };
        let ex = ResponseExample {
            name: format!("示例 {}", chrono::Local::now().format("%H:%M:%S")),
            original_request: None,
            status: Some(r.status.clone()),
            code: Some(r.code),
            header: r
                .headers
                .iter()
                .map(|(k, v)| Header {
                    key: k.clone(),
                    value: v.clone(),
                    ..Default::default()
                })
                .collect(),
            body: Some(r.body.clone()),
            preview_language: None,
        };
        if let Some(c) = self.tree.as_mut() {
            if let Some(it) = item_at_mut(&mut c.item, &path) {
                it.response.push(ex);
            }
        }
        self.save_tree();
    }
    fn ui_side_env(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "环境变量");
        ui.add_space(2.0);
        ui.label(
            RichText::new("可编辑并保存当前环境")
                .color(text_secondary())
                .size(11.0),
        );
        let cur_stem = self.env_files.get(self.env_sel).map(file_name).unwrap_or_default();
        let cur = if cur_stem.is_empty() {
            "（无环境）".to_string()
        } else {
            cur_stem.clone()
        };
        egui::ComboBox::from_id_salt("env_side")
            .width(ui.available_width())
            .selected_text(cur)
            .show_ui(ui, |ui| {
                for i in 0..self.env_files.len() {
                    ui.selectable_value(&mut self.env_sel, i, file_name(&self.env_files[i]));
                }
            });
        if self.env_loaded_for != cur_stem {
            self.env_edit = self
                .env_files
                .get(self.env_sel)
                .and_then(|p| std::fs::read_to_string(p).ok())
                .and_then(|t| serde_json::from_str::<Environment>(&t).ok())
                .map(|e| {
                    e.values
                        .iter()
                        .map(|v| (v.key.clone(), value_str(&v.value)))
                        .collect()
                })
                .unwrap_or_default();
            self.env_loaded_for = cur_stem.clone();
        }
        ui.add_space(6.0);
        let mut remove: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("envedit")
            .auto_shrink([false, false])
            .max_height(320.0)
            .show(ui, |ui| {
                let avail = ui.available_width();
                let key_w = (avail * 0.42).clamp(80.0, 160.0);
                let val_w = (avail - key_w - 30.0).clamp(90.0, 400.0);
                egui::Grid::new("enveditgrid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        for (i, (k, v)) in self.env_edit.iter_mut().enumerate() {
                            ui.add(
                                egui::TextEdit::singleline(k).desired_width(key_w).hint_text("键"),
                            );
                            ui.add(
                                egui::TextEdit::singleline(v)
                                    .desired_width(val_w)
                                    .hint_text("值"),
                            );
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("×").color(text_secondary()))
                                        .frame(false),
                                )
                                .clicked()
                            {
                                remove = Some(i);
                            }
                            ui.end_row();
                        }
                    });
            });
        if let Some(i) = remove {
            self.env_edit.remove(i);
        }
        if ui
            .add(
                egui::Button::new(RichText::new("＋ 添加变量").color(brand())).fill(brand_soft()),
            )
            .clicked()
        {
            self.env_edit.push((String::new(), String::new()));
        }
        if ui
            .add(
                egui::Button::new(RichText::new("保存环境").color(Color32::WHITE).strong())
                    .fill(brand())
                    .rounding(Rounding::same(6.0)),
            )
            .clicked()
        {
            if let Some(p) = self.env_files.get(self.env_sel).cloned() {
                let mut e = std::fs::read_to_string(&p)
                    .ok()
                    .and_then(|t| serde_json::from_str::<Environment>(&t).ok())
                    .unwrap_or_default();
                e.values = self
                    .env_edit
                    .iter()
                    .filter(|(k, _)| !k.trim().is_empty())
                    .map(|(k, v)| Variable {
                        key: k.clone(),
                        value: serde_json::Value::String(v.clone()),
                        typ: Some("default".into()),
                        enabled: Some(true),
                        description: None,
                    })
                    .collect();
                match storage::save_environment(&e) {
                    Ok(_) => self.message = format!("环境已保存：{}", cur_stem),
                    Err(err) => self.message = format!("保存失败：{}", err),
                }
            }
        }
    }
    fn ui_side_globals(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "全局变量");
        ui.add_space(2.0);
        ui.label(
            RichText::new("全局变量对所有集合/请求生效（优先级最低，可被环境/集合变量覆盖）")
                .color(text_secondary())
                .size(11.0),
        );
        if !self.globals_loaded {
            self.globals_edit = storage::load_globals()
                .unwrap_or_default()
                .values
                .iter()
                .map(|v| (v.key.clone(), value_str(&v.value)))
                .collect();
            self.globals_loaded = true;
        }
        ui.add_space(6.0);
        let mut remove: Option<usize> = None;
        egui::ScrollArea::vertical()
            .id_salt("globalsedit")
            .auto_shrink([false, false])
            .max_height(320.0)
            .show(ui, |ui| {
                let avail = ui.available_width();
                let key_w = (avail * 0.42).clamp(80.0, 160.0);
                let val_w = (avail - key_w - 30.0).clamp(90.0, 400.0);
                egui::Grid::new("globalseditgrid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        for (i, (k, v)) in self.globals_edit.iter_mut().enumerate() {
                            ui.add(
                                egui::TextEdit::singleline(k).desired_width(key_w).hint_text("键"),
                            );
                            ui.add(
                                egui::TextEdit::singleline(v)
                                    .desired_width(val_w)
                                    .hint_text("值"),
                            );
                            if ui
                                .add(
                                    egui::Button::new(RichText::new("×").color(text_secondary()))
                                        .frame(false),
                                )
                                .clicked()
                            {
                                remove = Some(i);
                            }
                            ui.end_row();
                        }
                    });
            });
        if let Some(i) = remove {
            self.globals_edit.remove(i);
        }
        if ui
            .add(
                egui::Button::new(RichText::new("＋ 添加变量").color(brand())).fill(brand_soft()),
            )
            .clicked()
        {
            self.globals_edit.push((String::new(), String::new()));
        }
        ui.horizontal_wrapped(|ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("保存全局变量").color(Color32::WHITE).strong(),
                    )
                    .fill(brand())
                    .rounding(Rounding::same(6.0)),
                )
                .clicked()
            {
                let env = Environment {
                    id: None,
                    name: "globals".into(),
                    scope: Some("globals".into()),
                    values: self
                        .globals_edit
                        .iter()
                        .filter(|(k, _)| !k.trim().is_empty())
                        .map(|(k, v)| Variable {
                            key: k.clone(),
                            value: serde_json::Value::String(v.clone()),
                            typ: Some("default".into()),
                            enabled: Some(true),
                            description: None,
                        })
                        .collect(),
                };
                match storage::save_globals(&env) {
                    Ok(_) => {
                        self.message = "全局变量已保存".into();
                        self.globals_loaded = false;
                    }
                    Err(e) => self.message = format!("保存失败：{}", e),
                }
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("重新加载").color(text_secondary()))
                        .frame(false),
                )
                .clicked()
            {
                self.globals_loaded = false;
            }
        });
    }
    fn ui_side_history(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "最近请求");
        ui.add_space(4.0);
        egui::Frame::none()
            .fill(mix(bg_side(), text_primary(), 0.05))
            .rounding(Rounding::same(R_MD))
            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.hist_search)
                        .frame(false)
                        .hint_text("搜索历史（URL / 方法）…")
                        .desired_width(f32::INFINITY),
                );
            });
        ui.add_space(6.0);
        let q = self.hist_search.to_lowercase();
        egui::ScrollArea::vertical()
            .id_salt("hist")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.history.is_empty() {
                    ui.label(RichText::new("（暂无历史）").color(text_secondary()).size(12.0));
                }
                let mut replay: Option<(String, String)> = None;
                let mut save_as: Option<(String, String)> = None;
                for h in self.history.iter().rev().take(200) {
                    if !q.is_empty()
                        && !h.url.to_lowercase().contains(&q)
                        && !h.method.to_lowercase().contains(&q)
                    {
                        continue;
                    }
                    let code: u16 = h.status;
                    let (hit, save) = ui
                        .horizontal(|ui| {
                            ui.label(
                                RichText::new(&h.method)
                                    .color(method_color(&h.method))
                                    .strong()
                                    .size(11.0),
                            );
                            let b = ui.add(
                                egui::Button::new(
                                    RichText::new(&h.url).color(text_primary()).size(12.0),
                                )
                                .frame(false),
                            );
                            ui.label(
                                RichText::new(code.to_string())
                                    .color(status_color(code))
                                    .size(11.0),
                            );
                            let s = ui.small_button("存为请求").clicked();
                            (b.clicked(), s)
                        })
                        .inner;
                    if hit {
                        replay = Some((h.method.clone(), h.url.clone()));
                    }
                    if save {
                        save_as = Some((h.method.clone(), h.url.clone()));
                    }
                }
                if let Some((m, u)) = replay {
                    let tab = &mut self.tabs[self.active];
                    tab.method = m;
                    tab.url = u;
                    tab.params = parse_query(&tab.url);
                    tab.path_rows = detect_path_vars(&tab.url);
                    tab.hash.clear();
                    self.message = "已从历史载入请求".into();
                }
                if let Some((m, u)) = save_as {
                    {
                        let tab = &mut self.tabs[self.active];
                        tab.method = m;
                        tab.url = u;
                        tab.params = parse_query(&tab.url);
                        tab.path_rows = detect_path_vars(&tab.url);
                        tab.hash.clear();
                    }
                    self.save_active();
                }
            });
    }
    fn ui_side_console(&mut self, ui: &mut egui::Ui) {
        section_title(ui, "控制台");
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.button("清空").clicked() {
                self.console.clear();
            }
            if ui.button("复制全部").clicked() {
                let text = self.console.join("\n");
                ui.output_mut(|o| o.copied_text = text);
            }
            ui.label(
                RichText::new(format!("{} 条", self.console.len()))
                    .color(text_secondary())
                    .size(11.0),
            );
        });
        ui.add_space(6.0);
        egui::ScrollArea::vertical()
            .id_salt("consolelog")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                if self.console.is_empty() {
                    ui.label(
                        RichText::new("暂无日志。发送请求后这里会显示请求 / 响应 / 脚本测试日志。")
                            .color(text_secondary())
                            .size(11.0),
                    );
                }
                for line in &self.console {
                    ui.label(RichText::new(line).monospace().size(11.0));
                }
            });
    }
    fn ui_side_tools(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_salt("tools")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // 外观 / 主题
                section_title(ui, "外观 / 主题");
                ui.horizontal_wrapped(|ui| {
                    let mut sel = self.theme;
                    egui::ComboBox::from_id_salt("tools_theme")
                        .width(150.0)
                        .selected_text(THEMES[sel.min(THEMES.len() - 1)].name)
                        .show_ui(ui, |ui| {
                            for i in 0..THEMES.len() {
                                ui.selectable_value(&mut sel, i, THEMES[i].name);
                            }
                        });
                    if sel != self.theme {
                        self.theme = sel;
                        self.follow_system = false;
                        FOLLOW_SYSTEM.store(false, Ordering::Relaxed);
                        apply_theme(ui.ctx(), sel);
                        save_theme(sel);
                    }
                    let mut follow = self.follow_system;
                    if ui
                        .checkbox(
                            &mut follow,
                            RichText::new("跟随系统").color(text_secondary()),
                        )
                        .changed()
                    {
                        self.follow_system = follow;
                        FOLLOW_SYSTEM.store(follow, Ordering::Relaxed);
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("强调色 #").color(text_secondary()).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.accent_hex)
                            .desired_width(90.0)
                            .hint_text("FF6C37"),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("应用强调色").color(Color32::WHITE).strong(),
                            )
                            .fill(brand())
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        if let Some(rgbv) = parse_hex_color(&self.accent_hex) {
                            ACCENT_OVERRIDE.store(0x01000000 | rgbv, Ordering::Relaxed);
                            install_style(ui.ctx());
                            self.message = format!("强调色已应用：#{:06X}", rgbv);
                        } else if self.accent_hex.trim().is_empty() {
                            ACCENT_OVERRIDE.store(0, Ordering::Relaxed);
                            install_style(ui.ctx());
                            self.message = "已恢复主题默认强调色".into();
                        } else {
                            self.message = "强调色格式无效，请输入 6 位十六进制，如 FF6C37".into();
                        }
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("恢复默认").color(text_secondary()))
                                .frame(false),
                        )
                        .clicked()
                    {
                        ACCENT_OVERRIDE.store(0, Ordering::Relaxed);
                        self.accent_hex.clear();
                        install_style(ui.ctx());
                    }
                });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // WebSocket 协议工作台
                section_title(ui, "WebSocket 工作台");
                ui.add(
                    egui::TextEdit::singleline(&mut self.ws_url)
                        .hint_text("ws:// 或 wss:// 地址")
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("子协议").color(text_secondary()).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut self.ws_subprotocol)
                            .desired_width(110.0)
                            .hint_text("可选"),
                    );
                    ui.checkbox(&mut self.ws_binary, RichText::new("二进制发送").color(text_secondary()));
                });
                ui.label(RichText::new("自定义请求头（每行 K: V）").color(text_secondary()).size(11.5));
                ui.add(
                    egui::TextEdit::multiline(&mut self.ws_headers)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY)
                        .hint_text("Authorization: Bearer xxx"),
                );
                ui.label(RichText::new("发送消息（每行一条）").color(text_secondary()).size(11.5));
                ui.add(
                    egui::TextEdit::multiline(&mut self.ws_send)
                        .desired_rows(3)
                        .desired_width(f32::INFINITY)
                        .hint_text("{\"type\":\"ping\"}"),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("监听条数").color(text_secondary()).size(12.0));
                    ui.add(egui::DragValue::new(&mut self.ws_listen).range(0..=1000));
                    ui.label(RichText::new("等待(ms)").color(text_secondary()).size(12.0));
                    ui.add(egui::DragValue::new(&mut self.ws_wait_ms).range(100..=120000));
                    let label = if self.ws_busy { "连接中…" } else { "连接并发送" };
                    if ui
                        .add_enabled(
                            !self.ws_busy,
                            egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
                                .fill(send_blue())
                                .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        self.send_ws();
                    }
                });
                if !self.ws_log.is_empty() {
                    let mut ro = self.ws_log.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut ro)
                            .code_editor()
                            .desired_rows(8)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 导入
                section_title(ui, "导入集合 / 环境");
                ui.add(
                    egui::TextEdit::singleline(&mut self.import_path)
                        .hint_text("文件路径（.json / .yaml / curl 文本 / .har）")
                        .desired_width(f32::INFINITY),
                );
                ui.horizontal_wrapped(|ui| {
                    egui::ComboBox::from_id_salt("import_fmt")
                        .width(150.0)
                        .selected_text(import_format_label(&self.import_format))
                        .show_ui(ui, |ui| {
                            for f in ["postman", "openapi", "curl", "har"] {
                                ui.selectable_value(
                                    &mut self.import_format,
                                    f.to_string(),
                                    import_format_label(f),
                                );
                            }
                        });
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("导入并合并").color(Color32::WHITE).strong(),
                            )
                            .fill(brand())
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        self.import_now();
                    }
                });
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // cURL 粘贴导入
                section_title(ui, "从 cURL 导入（粘贴）");
                ui.add(
                    egui::TextEdit::multiline(&mut self.curl_input)
                        .desired_rows(3)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                        .hint_text("粘贴 curl 命令，例如：\ncurl -X POST 'https://api.test/x' -H 'Content-Type: application/json' -d '{\"a\":1}'"),
                );
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("解析为新标签").color(Color32::WHITE).strong(),
                        )
                        .fill(brand())
                        .rounding(Rounding::same(6.0)),
                    )
                    .clicked()
                {
                    self.import_curl_to_tab();
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Runner
                section_title(ui, "集合运行器");
                let cur = self
                    .coll_files
                    .get(self.coll_sel)
                    .map(file_name)
                    .unwrap_or_else(|| "（无集合）".into());
                ui.label(
                    RichText::new(format!("当前集合：{}", cur))
                        .color(text_secondary())
                        .size(12.0),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new("迭代").color(text_secondary()).size(12.0));
                    ui.add(egui::DragValue::new(&mut self.run_iterations).range(1..=1000));
                    ui.checkbox(&mut self.run_bail, RichText::new("失败即停").color(text_secondary()));
                });
                let label = if self.run_inflight { "运行中…" } else { "运行集合" };
                if ui
                    .add_enabled(
                        !self.run_inflight,
                        egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
                            .fill(send_blue())
                            .rounding(Rounding::same(6.0)),
                    )
                    .clicked()
                {
                    self.run_current_collection();
                }
                if !self.run_result.is_empty() {
                    egui::ScrollArea::vertical()
                        .id_salt("runres")
                        .max_height(160.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut ro = self.run_result.clone();
                            ui.add(
                                egui::TextEdit::multiline(&mut ro)
                                    .code_editor()
                                    .desired_rows(6)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false),
                            );
                        });
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 代码生成
                section_title(ui, "代码生成（当前标签）");
                ui.horizontal_wrapped(|ui| {
                    egui::ComboBox::from_id_salt("codegen_lang")
                        .selected_text(&self.codegen_lang)
                        .show_ui(ui, |ui| {
                            for l in [
                                "curl", "http", "python", "js", "nodejs", "go", "rust", "java",
                                "csharp", "php", "ruby", "powershell", "swift",
                            ] {
                                ui.selectable_value(&mut self.codegen_lang, l.to_string(), l);
                            }
                        });
                    if ui
                        .add(
                            egui::Button::new(RichText::new("生成").color(Color32::WHITE).strong())
                                .fill(brand())
                                .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        self.generate_code();
                    }
                    if !self.codegen_out.is_empty()
                        && ui
                            .add(
                                egui::Button::new(RichText::new("复制").color(text_secondary()))
                                    .frame(false),
                            )
                            .clicked()
                    {
                        ui.output_mut(|o| o.copied_text = self.codegen_out.clone());
                    }
                });
                if !self.codegen_out.is_empty() {
                    let mut ro = self.codegen_out.clone();
                    ui.add(
                        egui::TextEdit::multiline(&mut ro)
                            .code_editor()
                            .desired_rows(10)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // Cookie 管理器
                section_title(ui, "Cookie 管理器");
                let cookies = http::cookies_dump();
                if cookies.is_empty() {
                    ui.label(RichText::new("暂无已保存的 Cookie。").color(text_secondary()).size(11.0));
                } else {
                    egui::ScrollArea::vertical()
                        .id_salt("cookielist")
                        .max_height(150.0)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut del: Option<(String, String)> = None;
                            egui::Grid::new("cookiegrid")
                                .num_columns(3)
                                .striped(true)
                                .spacing([8.0, 4.0])
                                .show(ui, |ui| {
                                    for (host, name, value) in &cookies {
                                        ui.label(
                                            RichText::new(host).color(text_primary()).size(11.0),
                                        );
                                        ui.label(
                                            RichText::new(format!("{}={}", name, value))
                                                .color(text_secondary())
                                                .size(11.0),
                                        );
                                        if ui
                                            .add(
                                                egui::Button::new(
                                                    RichText::new("×").color(text_secondary()),
                                                )
                                                .frame(false),
                                            )
                                            .clicked()
                                        {
                                            del = Some((host.clone(), name.clone()));
                                        }
                                        ui.end_row();
                                    }
                                });
                            if let Some((h, n)) = del {
                                http::cookie_delete(&h, &n);
                                ui.ctx().request_repaint();
                            }
                        });
                }
                ui.horizontal_wrapped(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.cookie_new_host)
                            .desired_width(110.0)
                            .hint_text("域名 host"),
                    );
                    ui.add(
                        egui::TextEdit::singleline(&mut self.cookie_new_kv)
                            .desired_width(140.0)
                            .hint_text("name=value"),
                    );
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("添加/覆盖").color(Color32::WHITE).strong(),
                            )
                            .fill(brand())
                            .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        match self.cookie_new_kv.split_once('=') {
                            Some((n, v))
                                if !self.cookie_new_host.trim().is_empty()
                                    && !n.trim().is_empty() =>
                            {
                                http::cookie_set(self.cookie_new_host.trim(), n.trim(), v.trim());
                                self.cookie_new_kv.clear();
                                self.message = "Cookie 已保存".into();
                                ui.ctx().request_repaint();
                            }
                            _ => self.message = "请输入域名与 name=value".into(),
                        }
                    }
                });
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("清空全部 Cookie")
                                .color(Color32::from_rgb(235, 32, 19)),
                        )
                        .frame(false),
                    )
                    .clicked()
                {
                    http::cookie_clear();
                    self.message = "已清空全部 Cookie".into();
                    ui.ctx().request_repaint();
                }
                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);
                ui.label(
                    RichText::new("提示：Mock / 性能测试 / WebSocket / SSE / MQTT / GraphQL 也已提供 CLI 子命令（pmrs mock / perf / ws / sse / mqtt / graphql），详见 README。")
                        .color(text_secondary())
                        .size(11.0),
                );
            });
    }
    // ---- 主区域 -----------------------------------------------------------
    /// 将「工具 → 从 cURL 导入」文本框里的命令解析为一个新标签。
    fn import_curl_to_tab(&mut self) {
        let text = self.curl_input.trim().to_string();
        if text.is_empty() {
            self.message = "请先粘贴 cURL 命令".into();
            return;
        }
        let coll = import_export::import_curl(&text);
        let Some(item) = coll.item.into_iter().next() else {
            self.message = "cURL 解析失败".into();
            return;
        };
        let Some(req) = item.request else {
            self.message = "cURL 中未找到请求".into();
            return;
        };
        let n = self.tabs.len() + 1;
        let mut tab = Tab::new(n);
        tab.title = "cURL 导入".into();
        tab.method = req.method.clone();
        tab.url = if req.url.raw.trim().is_empty() {
            req.url.effective_url()
        } else {
            req.url.raw.clone()
        };
        tab.params = parse_query(&tab.url);
        tab.path_rows = detect_path_vars(&tab.url);
        tab.hash = req.url.hash.clone().unwrap_or_default();
        tab.headers = req
            .header
            .iter()
            .map(|h| HeaderRow {
                key: h.key.clone(),
                value: h.value.clone(),
                disabled: h.disabled,
            })
            .collect();
        if tab.headers.is_empty() {
            tab.headers.push(HeaderRow::default());
        }
        // 请求体
        tab.body_type = "none".into();
        if let Some(b) = &req.body {
            match b.mode.as_str() {
                "raw" => {
                    tab.body_type = "raw".into();
                    tab.body = b.raw.clone().unwrap_or_default();
                    let lang = b
                        .options
                        .as_ref()
                        .and_then(|o| o.get("raw"))
                        .and_then(|r| r.get("language"))
                        .and_then(|l| l.as_str())
                        .unwrap_or("json");
                    tab.raw_lang = raw_lang_label(lang).to_string();
                }
                "urlencoded" => {
                    tab.body_type = "x-www-form-urlencoded".into();
                    tab.url_rows = b
                        .urlencoded
                        .iter()
                        .map(|kv| KVRow {
                            key: kv.key.clone(),
                            value: kv.value.clone(),
                            disabled: kv.disabled,
                            is_file: false,
                        })
                        .collect();
                    if tab.url_rows.is_empty() {
                        tab.url_rows.push(KVRow::default());
                    }
                }
                "formdata" => {
                    tab.body_type = "form-data".into();
                    tab.form_rows = b
                        .formdata
                        .iter()
                        .map(|kv| {
                            let is_file = kv
                                .typ
                                .as_deref()
                                .map(|t| t.eq_ignore_ascii_case("file"))
                                .unwrap_or(false);
                            let value = if is_file {
                                kv.src
                                    .as_ref()
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("")
                                    .to_string()
                            } else {
                                kv.value.clone()
                            };
                            KVRow { key: kv.key.clone(), value, disabled: kv.disabled, is_file }
                        })
                        .collect();
                    if tab.form_rows.is_empty() {
                        tab.form_rows.push(KVRow::default());
                    }
                }
                "file" => {
                    tab.body_type = "binary".into();
                    tab.file_path = b.file.as_ref().and_then(|f| f.src.clone()).unwrap_or_default();
                }
                _ => {}
            }
        }
        // 鉴权
        if let Some(a) = &req.auth {
            match a.kind.as_str() {
                "basic" => {
                    tab.auth_kind = "basic".into();
                    tab.auth_user = a.basic_user().unwrap_or("").to_string();
                    tab.auth_pass = a.basic_pass().unwrap_or("").to_string();
                }
                "bearer" => {
                    tab.auth_kind = "bearer".into();
                    tab.auth_token = a.bearer_token().unwrap_or("").to_string();
                }
                "apikey" => {
                    tab.auth_kind = "apikey".into();
                    tab.auth_key = a.apikey_key().unwrap_or("").to_string();
                    tab.auth_value = a.apikey_value().unwrap_or("").to_string();
                }
                _ => {}
            }
        }
        self.tabs.push(tab);
        self.active = self.tabs.len() - 1;
        self.message = "已从 cURL 导入为新标签".into();
    }
    fn send_ws(&mut self) {
        if self.ws_url.trim().is_empty() {
            self.message = "请输入 WebSocket 地址".into();
            return;
        }
        let mut headers = Vec::new();
        for line in self.ws_headers.lines() {
            if let Some((k, v)) = line.split_once(':') {
                let (k, v) = (k.trim(), v.trim());
                if !k.is_empty() {
                    headers.push((k.to_string(), v.to_string()));
                }
            }
        }
        let cfg = postman_rs::ws::WsConfig {
            url: self.ws_url.trim().to_string(),
            messages: self
                .ws_send
                .lines()
                .filter(|l| !l.trim().is_empty())
                .map(|s| s.to_string())
                .collect(),
            listen: self.ws_listen as usize,
            wait_ms: self.ws_wait_ms,
            headers,
            subprotocol: if self.ws_subprotocol.trim().is_empty() {
                None
            } else {
                Some(self.ws_subprotocol.trim().to_string())
            },
            binary: self.ws_binary,
        };
        let (tx, rx) = mpsc::channel();
        self.ws_rx = Some(rx);
        self.ws_busy = true;
        self.ws_log = "连接中…".into();
        std::thread::spawn(move || {
            let r = postman_rs::ws::run_collect(&cfg).map_err(|e| e.to_string());
            let _ = tx.send(r);
        });
    }
    fn ui_main(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::none()
                    .fill(app_top())
                    .inner_margin(egui::Margin::symmetric(16.0, 12.0)),
            )
            .show(ctx, |ui| {
                paint_app_gradient(ui);
                self.ui_request_tabs(ui);
                ui.add_space(8.0);
                let mut send = false;
                card(true)
                    .show(ui, |ui| {
                        send = self.ui_request_card(ui);
                    });
                ui.add_space(12.0);
                card(true)
                    .show(ui, |ui| {
                        self.ui_response(ui);
                    });
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if !self.message.is_empty() {
                        let (r, _) =
                            ui.allocate_exact_size(egui::vec2(7.0, 7.0), egui::Sense::hover());
                        ui.painter().circle_filled(r.center(), 3.0, brand());
                        ui.add_space(4.0);
                        ui.label(RichText::new(&self.message).color(text_secondary()).size(12.0));
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new("导出集合").color(Color32::WHITE).strong(),
                                )
                                    .fill(brand())
                                    .rounding(Rounding::same(R_MD)),
                            )
                            .clicked()
                        {
                            self.export_selected();
                        }
                        egui::Frame::none()
                            .fill(mix(bg_app(), text_primary(), 0.05))
                            .rounding(Rounding::same(R_MD))
                            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                            .show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::singleline(&mut self.export_path)
                                        .frame(false)
                                        .hint_text("导出路径（留空=默认 exports/）")
                                        .desired_width(240.0),
                                );
                            });
                    });
                });
                if send {
                    self.send_active(ctx);
                }
            });
    }
    fn ui_request_tabs(&mut self, ui: &mut egui::Ui) {
        let mut select: Option<usize> = None;
        let mut close: Option<usize> = None;
        let mut add = false;
        ui.horizontal_wrapped(|ui| {
            for i in 0..self.tabs.len() {
                let title = self.tabs[i].title.clone();
                let active = self.active == i;
                let bg = if active { bg_panel() } else { Color32::TRANSPARENT };
                let border_c = if active { brand() } else { Color32::TRANSPARENT };
                let tc = if active { brand() } else { text_secondary() };
                egui::Frame::none()
                    .fill(bg)
                    .stroke(Stroke::new(1.0_f32, border_c))
                    .rounding(Rounding::same(R_MD))
                    .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new(&title).color(tc).size(12.5).strong(),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                select = Some(i);
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("×")
                                            .color(if active { brand() } else { text_secondary() })
                                            .size(13.0),
                                    )
                                    .frame(false),
                                )
                                .clicked()
                            {
                                close = Some(i);
                            }
                        });
                    });
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("＋ 新建标签").color(brand()))
                        .fill(brand_soft())
                        .rounding(Rounding::same(R_MD)),
                )
                .clicked()
            {
                add = true;
            }
        });
        if let Some(i) = select {
            self.active = i;
        }
        if let Some(i) = close {
            if self.tabs.len() > 1 {
                self.tabs.remove(i);
                if self.active >= self.tabs.len() {
                    self.active = self.tabs.len() - 1;
                }
            }
        }
        if add {
            let n = self.tabs.len() + 1;
            self.tabs.push(Tab::new(n));
            self.active = self.tabs.len() - 1;
        }
    }
    fn ui_request_card(&mut self, ui: &mut egui::Ui) -> bool {
        let mut send = false;
        let active = self.active;
        let tab = &mut self.tabs[active];
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("method")
                .width(92.0)
                .selected_text(
                    RichText::new(&tab.method)
                        .color(method_color(&tab.method))
                        .size(13.0)
                        .strong(),
                )
                .show_ui(ui, |ui| {
                    for m in [
                        "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS", "COPY", "LINK",
                        "UNLINK", "PURGE", "LOCK", "UNLOCK", "PROPFIND", "VIEW", "TRACE",
                    ] {
                        ui.selectable_value(
                            &mut tab.method,
                            m.to_string(),
                            RichText::new(m).color(method_color(m)).strong(),
                        );
                    }
                });
            let url_w = (ui.available_width() - 186.0).max(160.0);
            let resp = egui::Frame::none()
                .fill(mix(bg_panel(), text_primary(), 0.04))
                .rounding(Rounding::same(R_MD))
                .inner_margin(egui::Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut tab.url)
                            .frame(false)
                            .desired_width(url_w)
                            .font(egui::TextStyle::Monospace)
                            .hint_text("https://api.example.com/path  （支持 {{变量}}）"),
                    )
                })
                .inner;
            if resp.changed() {
                tab.params = parse_query(&tab.url);
                tab.path_rows = detect_path_vars(&tab.url);
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("cURL").color(brand()).strong())
                        .fill(brand_soft())
                        .rounding(Rounding::same(R_MD))
                        .min_size(egui::vec2(56.0, 32.0)),
                )
                .on_hover_text("复制当前请求为 cURL 命令")
                .clicked()
            {
                let code = codegen::generate(&tab.build_request(), "curl").unwrap_or_default();
                ui.output_mut(|o| o.copied_text = code);
                tab.status = "已复制 cURL 到剪贴板".into();
            }
            let label = if tab.inflight { "发送中…" } else { "发送" };
            if ui
                .add_enabled(
                    !tab.inflight,
                    egui::Button::new(RichText::new(label).color(Color32::WHITE).strong())
                        .fill(send_blue())
                        .rounding(Rounding::same(R_MD))
                        .min_size(egui::vec2(96.0, 32.0)),
                )
                .clicked()
            {
                send = true;
            }
            if tab.inflight
                && ui
                    .add(
                        egui::Button::new(
                            RichText::new("取消").color(Color32::from_rgb(235, 32, 19)),
                        )
                        .fill(brand_soft())
                        .rounding(Rounding::same(R_MD))
                        .min_size(egui::vec2(64.0, 32.0)),
                    )
                    .clicked()
            {
                tab.rx = None;
                tab.inflight = false;
                tab.status = "已取消".into();
            }
        });
        ui.add_space(6.0);
        underline_tabs(
            ui,
            &mut tab.req_subtab,
            &["参数", "授权", "请求头", "请求体", "测试", "脚本", "设置", "描述"],
        );
        ui.add_space(2.0);
        let rect = ui.available_rect_before_wrap();
        ui.painter().hline(
            rect.left()..=rect.right(),
            rect.top() + 1.0,
            Stroke::new(1.0_f32, border()),
        );
        ui.add_space(8.0);
        // 记录卡片真实内容宽度；ScrollArea 内部的 available_width 会被放大，
        // 这里把子区域宽度钉回真实值，保证"各占一半"等布局按可见宽度计算。
        let content_w = ui.available_width();
        egui::ScrollArea::vertical()
            .id_salt("reqbody")
            .max_height(300.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(content_w);
                match tab.req_subtab {
                    0 => Self::ui_params(tab, ui),
                    1 => Self::ui_auth(tab, ui),
                    2 => Self::ui_headers(tab, ui),
                    3 => Self::ui_body(tab, ui),
                    4 => Self::ui_tests(tab, ui),
                    5 => Self::ui_scripts(tab, ui),
                    6 => Self::ui_settings(tab, ui),
                    7 => Self::ui_desc(tab, ui),
                    _ => Self::ui_settings(tab, ui),
                }
            });
        send
    }
    fn ui_params(tab: &mut Tab, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("查询参数（URL ? 之后的部分；取消勾选=禁用，发送时不带上但仍保留在集合中）")
                .color(text_secondary())
                .size(11.0),
        );
        ui.add_space(4.0);
        let mut changed = false;
        let avail = ui.available_width();
        let key_w = (avail * 0.28).clamp(110.0, 260.0);
        let val_w = (avail - key_w - 64.0).clamp(150.0, 1400.0);
        egui::Grid::new("params_grid")
            .num_columns(4)
            .striped(true)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new("启用").color(text_secondary()).size(11.0));
                ui.label(RichText::new("键").color(text_secondary()).size(11.0));
                ui.label(RichText::new("值").color(text_secondary()).size(11.0));
                ui.label("");
                ui.end_row();
                let mut remove: Option<usize> = None;
                for (i, r) in tab.params.iter_mut().enumerate() {
                    let mut enabled = !r.disabled;
                    if ui.checkbox(&mut enabled, "").changed() {
                        r.disabled = !enabled;
                        changed = true;
                    }
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut r.key)
                                .desired_width(key_w)
                                .hint_text("键"),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut r.value)
                                .desired_width(val_w)
                                .hint_text("值"),
                        )
                        .changed()
                    {
                        changed = true;
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("×").color(text_secondary()))
                                .frame(false),
                        )
                        .clicked()
                    {
                        remove = Some(i);
                    }
                    ui.end_row();
                }
                if let Some(i) = remove {
                    tab.params.remove(i);
                    changed = true;
                }
            });
        if ui
            .add(
                egui::Button::new(RichText::new("＋ 添加参数").color(brand()))
                    .fill(brand_soft())
                    .rounding(Rounding::same(6.0)),
            )
            .clicked()
        {
            tab.params.push(KVRow::default());
        }
        // 路径变量（:name）
        if !tab.path_rows.is_empty() {
            ui.add_space(10.0);
            ui.label(
                RichText::new("路径变量（URL 中的 :名称；发送时用这里的值替换）")
                    .color(text_secondary())
                    .size(11.0),
            );
            ui.add_space(4.0);
            egui::Grid::new("pathvars_grid")
                .num_columns(3)
                .striped(true)
                .spacing([8.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("名称").color(text_secondary()).size(11.0));
                    ui.label(RichText::new("值").color(text_secondary()).size(11.0));
                    ui.label("");
                    ui.end_row();
                    let mut rmv: Option<usize> = None;
                    for (i, r) in tab.path_rows.iter_mut().enumerate() {
                        ui.add(
                            egui::TextEdit::singleline(&mut r.key)
                                .desired_width(key_w)
                                .hint_text("名称"),
                        );
                        ui.add(
                            egui::TextEdit::singleline(&mut r.value)
                                .desired_width(val_w)
                                .hint_text("值"),
                        );
                        if ui
                            .add(
                                egui::Button::new(RichText::new("×").color(text_secondary()))
                                    .frame(false),
                            )
                            .clicked()
                        {
                            rmv = Some(i);
                        }
                        ui.end_row();
                    }
                    if let Some(i) = rmv {
                        tab.path_rows.remove(i);
                    }
                });
        }
        // URL 片段 hash
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("URL 片段（#hash）").color(text_secondary()).size(12.0));
            if ui
                .add(
                    egui::TextEdit::singleline(&mut tab.hash)
                        .desired_width(240.0)
                        .hint_text("锚点/片段，可留空"),
                )
                .changed()
            {
                changed = true;
            }
        });
        if changed {
            rebuild_url(tab);
        }
    }
    fn ui_auth(tab: &mut Tab, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new("授权类型").color(text_secondary()).size(12.0));
            let cur = match tab.auth_kind.as_str() {
                "bearer" => "Bearer 令牌",
                "basic" => "基本认证",
                "apikey" => "API 密钥",
                "oauth2" => "OAuth 2.0",
                "oauth1" => "OAuth 1.0",
                "awsv4" => "AWS 签名 (SigV4)",
                "jwt" => "JWT Bearer",
                "digest" => "Digest 摘要认证",
                "ntlm" => "NTLM（不支持）",
                _ => "无",
            };
            egui::ComboBox::from_id_salt("auth_kind")
                .selected_text(cur)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut tab.auth_kind, "none".into(), "无");
                    ui.selectable_value(&mut tab.auth_kind, "bearer".into(), "Bearer 令牌");
                    ui.selectable_value(&mut tab.auth_kind, "basic".into(), "基本认证");
                    ui.selectable_value(&mut tab.auth_kind, "apikey".into(), "API 密钥");
                    ui.selectable_value(&mut tab.auth_kind, "oauth2".into(), "OAuth 2.0");
                    ui.selectable_value(&mut tab.auth_kind, "oauth1".into(), "OAuth 1.0");
                    ui.selectable_value(&mut tab.auth_kind, "awsv4".into(), "AWS 签名 (SigV4)");
                    ui.selectable_value(&mut tab.auth_kind, "jwt".into(), "JWT Bearer");
                    ui.selectable_value(&mut tab.auth_kind, "digest".into(), "Digest 摘要认证");
                    ui.selectable_value(&mut tab.auth_kind, "ntlm".into(), "NTLM（不支持）");
                });
        });
        ui.add_space(6.0);
        match tab.auth_kind.as_str() {
            "bearer" => {
                ui.label(RichText::new("令牌").color(text_secondary()).size(12.0));
                ui.add(
                    egui::TextEdit::singleline(&mut tab.auth_token)
                        .desired_width(f32::INFINITY)
                        .hint_text("粘贴 Bearer 令牌"),
                );
                ui.label(
                    RichText::new("发送时会添加请求头：Authorization: Bearer <令牌>")
                        .color(text_secondary())
                        .size(11.0),
                );
            }
            "basic" => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("用户名").color(text_secondary()).size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut tab.auth_user).desired_width(200.0));
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("密码").color(text_secondary()).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut tab.auth_pass)
                            .desired_width(200.0)
                            .password(true),
                    );
                });
            }
            "apikey" => {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("请求头名称").color(text_secondary()).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut tab.auth_key)
                            .desired_width(200.0)
                            .hint_text("X-API-Key"),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label(RichText::new("值").color(text_secondary()).size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut tab.auth_value).desired_width(260.0));
                });
            }
            "oauth2" => {
                let grant = tab
                    .oauth2
                    .get("grant_type")
                    .cloned()
                    .unwrap_or_else(|| "client_credentials".into());
                ui.horizontal(|ui| {
                    ui.label(RichText::new("授权流程").color(text_secondary()).size(12.0));
                    egui::ComboBox::from_id_salt("oauth2_grant")
                        .selected_text(grant_label(&grant))
                        .show_ui(ui, |ui| {
                            for g in ["client_credentials", "password", "refresh_token"] {
                                let slot = tab
                                    .oauth2
                                    .entry("grant_type".to_string())
                                    .or_insert_with(|| g.to_string());
                                ui.selectable_value(slot, g.to_string(), grant_label(g));
                            }
                        });
                });
                ui.add_space(4.0);
                oauth2_field(ui, &mut tab.oauth2, "令牌地址 (accessTokenUrl)", "accessTokenUrl", false);
                oauth2_field(ui, &mut tab.oauth2, "client_id", "client_id", false);
                oauth2_field(ui, &mut tab.oauth2, "client_secret", "client_secret", true);
                oauth2_field(ui, &mut tab.oauth2, "scope", "scope", false);
                if grant == "password" {
                    oauth2_field(ui, &mut tab.oauth2, "用户名", "username", false);
                    oauth2_field(ui, &mut tab.oauth2, "密码", "password", true);
                }
                if grant == "refresh_token" {
                    oauth2_field(ui, &mut tab.oauth2, "refresh_token", "refresh_token", true);
                }
                oauth2_field(ui, &mut tab.oauth2, "访问令牌 (可选，填了直接使用)", "accessToken", true);
                ui.label(
                    RichText::new("发送时会向令牌地址请求访问令牌，并添加响应头 Authorization: Bearer <token>")
                        .color(text_secondary())
                        .size(11.0),
                );
            }
            "oauth1" | "awsv4" | "jwt" | "digest" | "ntlm" => {
                if tab.auth_extra.is_empty() {
                    for k in default_auth_keys(&tab.auth_kind) {
                        tab.auth_extra.push(KVRow {
                            key: k.to_string(),
                            ..Default::default()
                        });
                    }
                }
                if tab.auth_kind == "ntlm" {
                    ui.label(
                        RichText::new("NTLM 握手协议较复杂，暂未实现；请改用 Basic / Bearer。")
                            .color(Color32::from_rgb(235, 32, 19))
                            .size(11.0),
                    );
                }
                egui::Grid::new("auth_extra_grid")
                    .num_columns(2)
                    .striped(true)
                    .spacing([8.0, 6.0])
                    .show(ui, |ui| {
                        for r in tab.auth_extra.iter_mut() {
                            ui.add(
                                egui::TextEdit::singleline(&mut r.key)
                                    .desired_width(150.0)
                                    .hint_text("键"),
                            );
                            ui.add(
                                egui::TextEdit::singleline(&mut r.value)
                                    .desired_width(300.0)
                                    .hint_text("值"),
                            );
                            ui.end_row();
                        }
                    });
            }
            _ => {
                ui.label(RichText::new("此请求不使用授权。").color(text_secondary()));
            }
        }
    }
    fn ui_headers(tab: &mut Tab, ui: &mut egui::Ui) {
        // 键、值各占一半：用 ui.columns(2) 在「勾选框之后、×按钮之前」的
        // 剩余区域内强制二等分，保证两者等宽且不溢出。
        const ROW_H: f32 = 22.0;
        const RESERVE_DEL: f32 = 26.0; // 预留给右侧「×」按钮
        // 表头
        ui.horizontal(|ui| {
            ui.add_sized(
                [24.0, 16.0],
                egui::Label::new(RichText::new("启用").color(text_secondary()).size(11.0)),
            );
            let rem = (ui.available_width() - RESERVE_DEL).max(120.0);
            ui.allocate_ui_with_layout(
                egui::vec2(rem, 16.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.columns(2, |cols| {
                        let w0 = cols[0].available_width();
                        cols[0].add_sized(
                            [w0, 16.0],
                            egui::Label::new(
                                RichText::new("键").color(text_secondary()).size(11.0),
                            ),
                        );
                        let w1 = cols[1].available_width();
                        cols[1].add_sized(
                            [w1, 16.0],
                            egui::Label::new(
                                RichText::new("值").color(text_secondary()).size(11.0),
                            ),
                        );
                    });
                },
            );
        });
        ui.add_space(2.0);
        let mut remove: Option<usize> = None;
        for i in 0..tab.headers.len() {
            let mut enabled = !tab.headers[i].disabled;
            let mut chg = false;
            {
                let h = &mut tab.headers[i];
                let key = &mut h.key;
                let value = &mut h.value;
                ui.horizontal(|ui| {
                    if ui
                        .add_sized([24.0, ROW_H], egui::Checkbox::new(&mut enabled, ""))
                        .changed()
                    {
                        chg = true;
                    }
                    let rem = (ui.available_width() - RESERVE_DEL).max(120.0);
                    ui.allocate_ui_with_layout(
                        egui::vec2(rem, ROW_H),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.columns(2, |cols| {
                                let w0 = cols[0].available_width();
                                cols[0].add_sized(
                                    [w0, ROW_H],
                                    egui::TextEdit::singleline(key).hint_text("请求头名"),
                                );
                                let w1 = cols[1].available_width();
                                cols[1].add_sized(
                                    [w1, ROW_H],
                                    egui::TextEdit::singleline(value).hint_text("值"),
                                );
                            });
                        },
                    );
                    if ui
                        .add(
                            egui::Button::new(RichText::new("×").color(text_secondary()))
                                .frame(false),
                        )
                        .clicked()
                    {
                        remove = Some(i);
                    }
                });
            }
            if chg {
                tab.headers[i].disabled = !enabled;
            }
        }
        if let Some(i) = remove {
            tab.headers.remove(i);
        }
        if ui
            .add(
                egui::Button::new(RichText::new("＋ 添加请求头").color(brand()))
                    .fill(brand_soft())
                    .rounding(Rounding::same(6.0)),
            )
            .clicked()
        {
            tab.headers.push(HeaderRow::default());
        }
    }
    fn ui_body(tab: &mut Tab, ui: &mut egui::Ui) {
        // 类型选择行（对齐 Postman：none / form-data / x-www-form-urlencoded / raw / binary / GraphQL）
        ui.horizontal(|ui| {
            ui.label(RichText::new("类型").color(text_secondary()).size(12.0));
            egui::ComboBox::from_id_salt("bodytype")
                .width(210.0)
                .selected_text(body_type_label(&tab.body_type))
                .show_ui(ui, |ui| {
                    for t in [
                        "none",
                        "form-data",
                        "x-www-form-urlencoded",
                        "raw",
                        "binary",
                        "graphql",
                    ] {
                        ui.selectable_value(&mut tab.body_type, t.to_string(), body_type_label(t));
                    }
                });
            if tab.body_type == "raw" {
                ui.add_space(10.0);
                ui.label(RichText::new("语言").color(text_secondary()).size(12.0));
                egui::ComboBox::from_id_salt("rawlang")
                    .width(120.0)
                    .selected_text(&tab.raw_lang)
                    .show_ui(ui, |ui| {
                        for l in ["Text", "JSON", "XML", "HTML", "JavaScript"] {
                            ui.selectable_value(&mut tab.raw_lang, l.to_string(), l);
                        }
                    });
                if tab.raw_lang == "JSON" && !tab.body.trim().is_empty() {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("格式化").color(brand()))
                                .fill(brand_soft())
                                .rounding(Rounding::same(6.0)),
                        )
                        .clicked()
                    {
                        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&tab.body) {
                            if let Ok(p) = serde_json::to_string_pretty(&v) {
                                tab.body = p;
                            }
                        }
                    }
                }
            }
        });
        ui.add_space(8.0);
        let avail = ui.available_width();
        let key_w = (avail * 0.30).clamp(120.0, 280.0);
        let val_w = (avail - key_w - 88.0).clamp(150.0, 1400.0);
        match tab.body_type.as_str() {
            "none" => {
                ui.label(
                    RichText::new(
                        "该请求没有请求体。可在上方「类型」中选择 raw / 表单 / 二进制 / GraphQL。",
                    )
                    .color(text_secondary())
                    .size(12.0),
                );
            }
            "raw" => {
                ui.add(
                    egui::TextEdit::multiline(&mut tab.body)
                        .code_editor()
                        .desired_rows(12)
                        .desired_width(f32::INFINITY)
                        .hint_text(if tab.raw_lang == "JSON" {
                            "{\n  \"键\": \"值\"\n}"
                        } else {
                            "请求体内容…"
                        }),
                );
            }
            "x-www-form-urlencoded" => {
                Self::ui_kv_editor(ui, &mut tab.url_rows, key_w, val_w, false, "urlenc");
            }
            "form-data" => {
                Self::ui_kv_editor(ui, &mut tab.form_rows, key_w, val_w, true, "formdata");
            }
            "binary" => {
                ui.label(
                    RichText::new("选择一个本地文件作为原始请求体（binary）。")
                        .color(text_secondary())
                        .size(12.0),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("文件路径").color(text_secondary()).size(12.0));
                    ui.add(
                        egui::TextEdit::singleline(&mut tab.file_path)
                            .desired_width(f32::INFINITY)
                            .hint_text("C:\\path\\to\\file.bin（留空则不发送请求体）"),
                    );
                });
            }
            "graphql" => {
                ui.label(RichText::new("查询（Query）").color(text_secondary()).size(12.0));
                ui.add(
                    egui::TextEdit::multiline(&mut tab.gql_query)
                        .code_editor()
                        .desired_rows(8)
                        .desired_width(f32::INFINITY)
                        .hint_text("query {\n  ...\n}"),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("变量（Variables，JSON）")
                        .color(text_secondary())
                        .size(12.0),
                );
                ui.add(
                    egui::TextEdit::multiline(&mut tab.gql_vars)
                        .code_editor()
                        .desired_rows(5)
                        .desired_width(f32::INFINITY)
                        .hint_text("{\n  \"变量\": \"值\"\n}"),
                );
            }
            _ => {}
        }
    }
    /// 表单类请求体编辑器（urlencoded / form-data 通用）。
    fn ui_kv_editor(
        ui: &mut egui::Ui,
        rows: &mut Vec<KVRow>,
        key_w: f32,
        val_w: f32,
        allow_file: bool,
        id: &str,
    ) {
        let mut remove: Option<usize> = None;
        egui::Grid::new(format!("kv_{}", id))
            .num_columns(if allow_file { 5 } else { 4 })
            .striped(true)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label(RichText::new("启用").color(text_secondary()).size(11.0));
                ui.label(RichText::new("键").color(text_secondary()).size(11.0));
                ui.label(RichText::new("值").color(text_secondary()).size(11.0));
                if allow_file {
                    ui.label(RichText::new("类型").color(text_secondary()).size(11.0));
                }
                ui.label("");
                ui.end_row();
                for (i, r) in rows.iter_mut().enumerate() {
                    let mut enabled = !r.disabled;
                    if ui.checkbox(&mut enabled, "").changed() {
                        r.disabled = !enabled;
                    }
                    ui.add(
                        egui::TextEdit::singleline(&mut r.key)
                            .desired_width(key_w)
                            .hint_text("键"),
                    );
                    let hint = if allow_file && r.is_file {
                        "文件路径"
                    } else {
                        "值"
                    };
                    ui.add(
                        egui::TextEdit::singleline(&mut r.value)
                            .desired_width(val_w)
                            .hint_text(hint),
                    );
                    if allow_file {
                        let mut is_file = r.is_file;
                        egui::ComboBox::from_id_salt(format!("kvtype_{}_{}", id, i))
                            .width(72.0)
                            .selected_text(if is_file { "文件" } else { "文本" })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut is_file, false, "文本");
                                ui.selectable_value(&mut is_file, true, "文件");
                            });
                        r.is_file = is_file;
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("×").color(text_secondary()))
                                .frame(false),
                        )
                        .clicked()
                    {
                        remove = Some(i);
                    }
                    ui.end_row();
                }
                if let Some(i) = remove {
                    rows.remove(i);
                }
            });
        if ui
            .add(
                egui::Button::new(RichText::new("＋ 添加一行").color(brand()))
                    .fill(brand_soft())
                    .rounding(Rounding::same(6.0)),
            )
            .clicked()
        {
            rows.push(KVRow::default());
        }
    }
    fn ui_tests(tab: &mut Tab, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(
                "测试断言（JSON 数组）。支持 type：status_equals / body_contains / json_path_exists / body_equals / response_time / regex 等。发送后可在响应区「测试」查看结果。",
            )
            .color(text_secondary())
            .size(11.0),
        );
        ui.add_space(4.0);
        ui.add(
            egui::TextEdit::multiline(&mut tab.tests_text)
                .code_editor()
                .desired_rows(10)
                .desired_width(f32::INFINITY)
                .hint_text("[\n  { \"type\": \"status_equals\", \"value\": 200 }\n]"),
        );
        if tab.tests_text.trim().is_empty() {
            tab.tests.clear();
            ui.label(RichText::new("当前无测试断言").color(text_secondary()).size(11.0));
        } else {
            match serde_json::from_str::<Vec<TestAssertion>>(&tab.tests_text) {
                Ok(v) => {
                    let n = v.len();
                    tab.tests = v;
                    ui.label(
                        RichText::new(format!("已解析 {} 条断言", n))
                            .color(Color32::from_rgb(73, 162, 91))
                            .size(11.0),
                    );
                }
                Err(e) => {
                    ui.label(
                        RichText::new(format!("JSON 解析错误：{}", e))
                            .color(Color32::from_rgb(235, 32, 19))
                            .size(11.0),
                    );
                }
            }
        }
    }
    fn ui_scripts(tab: &mut Tab, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(
                "前置脚本：请求发送前执行，可用 pm.environment.set / pm.globals.set / pm.collectionVariables.set / pm.variables.set 写入变量。",
            )
            .color(text_secondary())
            .size(11.0),
        );
        ui.add(
            egui::TextEdit::multiline(&mut tab.pre_script)
                .code_editor()
                .desired_rows(7)
                .desired_width(f32::INFINITY)
                .hint_text("pm.environment.set(\"token\", \"abc123\");"),
        );
        ui.add_space(8.0);
        ui.label(
            RichText::new(
                "测试脚本：响应返回后执行。支持 pm.test(name, function(){ ... })、pm.response.to.have.status(N)、pm.expect(pm.response.code).to.eql(N)、pm.expect(pm.response.json().a.b).to.eql(\"x\")。结果会显示在响应区「测试」。",
            )
            .color(text_secondary())
            .size(11.0),
        );
        ui.add(
            egui::TextEdit::multiline(&mut tab.test_script)
                .code_editor()
                .desired_rows(8)
                .desired_width(f32::INFINITY)
                .hint_text(
                    "pm.test(\"状态为 200\", function () {\n    pm.response.to.have.status(200);\n});",
                ),
        );
    }
    fn ui_settings(tab: &mut Tab, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(
                "请求级网络设置：留空的项使用全局配置（全局可用环境变量 PMRS_PROXY / PMRS_INSECURE / PMRS_CONNECT_TIMEOUT / PMRS_MAX_REDIRECTS / PMRS_CA_BUNDLE / PMRS_CLIENT_CERT / PMRS_CLIENT_KEY 设置）。",
            )
            .color(text_secondary())
            .size(11.0),
        );
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("超时（秒）").color(text_secondary()).size(12.0));
            ui.add(
                egui::TextEdit::singleline(&mut tab.req_timeout)
                    .desired_width(80.0)
                    .hint_text("全局"),
            );
            ui.add_space(12.0);
            ui.checkbox(
                &mut tab.follow_redirects,
                RichText::new("跟随重定向").color(text_secondary()),
            );
            ui.add_space(12.0);
            ui.checkbox(
                &mut tab.insecure,
                RichText::new("跳过 TLS 证书校验（不安全）").color(text_secondary()),
            );
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [90.0, 20.0],
                egui::Label::new(RichText::new("代理").color(text_secondary()).size(12.0)),
            );
            ui.add(
                egui::TextEdit::singleline(&mut tab.proxy)
                    .desired_width(f32::INFINITY)
                    .hint_text("http://127.0.0.1:8080（SOCKS 需启用 reqwest 的 socks 特性）"),
            );
        });
        ui.horizontal(|ui| {
            ui.add_sized(
                [90.0, 20.0],
                egui::Label::new(RichText::new("客户端证书").color(text_secondary()).size(12.0)),
            );
            ui.add(
                egui::TextEdit::singleline(&mut tab.client_cert)
                    .desired_width(f32::INFINITY)
                    .hint_text("client.crt / PEM 路径"),
            );
        });
        ui.horizontal(|ui| {
            ui.add_sized(
                [90.0, 20.0],
                egui::Label::new(RichText::new("客户端私钥").color(text_secondary()).size(12.0)),
            );
            ui.add(
                egui::TextEdit::singleline(&mut tab.client_key)
                    .desired_width(f32::INFINITY)
                    .hint_text("client.key / PEM 路径"),
            );
        });
    }
    fn ui_desc(tab: &mut Tab, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("请求描述（保存到集合后写入 item.description，供文档/团队查看；支持 Markdown 文本）")
                .color(text_secondary())
                .size(11.0),
        );
        ui.add_space(6.0);
        ui.add(
            egui::TextEdit::multiline(&mut tab.description)
                .desired_width(f32::INFINITY)
                .desired_rows(8)
                .hint_text("例如：创建用户，需 Bearer 令牌；返回 201 与用户 ID。"),
        );
        if ui.button("从响应体填入").clicked() {
            if let Some(r) = &tab.response {
                tab.description = r.body.clone();
            }
        }
    }
    fn ui_response(&mut self, ui: &mut egui::Ui) {
        let active = self.active;
        let examples = self.examples_for_active();
        let mut save_ex = false;
        let mut load_ex: Option<usize> = None;
        let tab = &mut self.tabs[active];
        let has_resp = tab.response.is_some();
        let mut pretty = tab.pretty;
        let mut copy_clicked = false;
        let mut save_file = false;
        let body_clone = tab.response.as_ref().map(|r| r.body.clone()).unwrap_or_default();
        ui.horizontal(|ui| {
            match (&tab.response, &tab.error) {
                (Some(r), _) => {
                    let c = status_color(r.code);
                    egui::Frame::none()
                        .fill(Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 34))
                        .rounding(Rounding::same(R_SM))
                        .inner_margin(egui::Margin::symmetric(10.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("{} {}", r.code, r.status))
                                    .color(c)
                                    .size(12.0)
                                    .strong(),
                            );
                        });
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("耗时 {} ms", r.elapsed_ms))
                            .color(text_secondary())
                            .size(11.5),
                    );
                    ui.add_space(7.0);
                    ui.label(
                        RichText::new(format!("大小 {}", human_size(r.body.len())))
                            .color(text_secondary())
                            .size(11.5),
                    );
                }
                (None, Some(e)) => {
                    ui.label(
                        RichText::new(format!("错误：{}", e))
                            .color(Color32::from_rgb(235, 32, 19)),
                    );
                }
                (None, None) => {
                    ui.label(RichText::new(&tab.status).color(text_secondary()));
                }
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if has_resp {
                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("保存为示例").color(text_secondary()).size(12.0),
                            )
                            .frame(false),
                        )
                        .clicked()
                    {
                        save_ex = true;
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("复制").color(text_secondary()))
                                .frame(false),
                        )
                        .clicked()
                    {
                        copy_clicked = true;
                    }
                    ui.checkbox(&mut pretty, RichText::new("美化").color(text_secondary()));
                }
            });
        });
        if has_resp {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("保存响应到文件：")
                        .color(text_secondary())
                        .size(11.5),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.resp_save_path)
                        .desired_width(300.0)
                        .hint_text("留空 = ~/.postman-rs/downloads/response_<时间>.txt"),
                );
                if ui
                    .add(
                        egui::Button::new(RichText::new("保存").color(Color32::WHITE).strong())
                            .fill(brand())
                            .rounding(Rounding::same(6.0)),
                    )
                    .clicked()
                {
                    save_file = true;
                }
            });
        }
        if !examples.is_empty() {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new("示例：").color(text_secondary()).size(11.5));
                for (i, ex) in examples.iter().enumerate() {
                    if ui.small_button(&ex.name).clicked() {
                        load_ex = Some(i);
                    }
                }
            });
        }
        tab.pretty = pretty;
        if copy_clicked {
            ui.output_mut(|o| o.copied_text = body_clone.clone());
        }
        ui.add_space(6.0);
        underline_tabs(
            ui,
            &mut tab.resp_subtab,
            &["响应体", "响应头", "Cookies", "测试", "计时"],
        );
        ui.add_space(2.0);
        let rect = ui.available_rect_before_wrap();
        ui.painter().hline(
            rect.left()..=rect.right(),
            rect.top() + 1.0,
            Stroke::new(1.0_f32, border()),
        );
        ui.add_space(8.0);
        if let Some(r) = &tab.response {
            match tab.resp_subtab {
                0 => {
                    let text = if tab.pretty {
                        pretty_body(&r.body)
                    } else {
                        r.body.clone()
                    };
                    let is_json = serde_json::from_str::<serde_json::Value>(&text).is_ok();
                    egui::ScrollArea::both()
                        .id_salt("rbody")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if is_json {
                                let galley = highlight_json(ui, &text, f32::INFINITY);
                                ui.add(egui::Label::new(galley));
                            } else {
                                let mut ro = text;
                                ui.add(
                                    egui::TextEdit::multiline(&mut ro)
                                        .code_editor()
                                        .desired_rows(14)
                                        .desired_width(f32::INFINITY)
                                        .interactive(false),
                                );
                            }
                        });
                }
                1 => {
                    egui::ScrollArea::vertical()
                        .id_salt("rh")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (k, v) in &r.headers {
                                ui.label(
                                    RichText::new(format!("{}: {}", k, v))
                                        .color(text_primary())
                                        .size(12.0),
                                );
                            }
                        });
                }
                2 => {
                    let cookies: Vec<&(String, String)> = r
                        .headers
                        .iter()
                        .filter(|(k, _)| k.eq_ignore_ascii_case("set-cookie"))
                        .collect();
                    egui::ScrollArea::vertical()
                        .id_salt("rcookies")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if cookies.is_empty() {
                                ui.label(
                                    RichText::new("此响应未设置 Cookie。").color(text_secondary()),
                                );
                            }
                            for (_, v) in cookies {
                                ui.label(
                                    RichText::new(v).color(text_primary()).size(12.0),
                                );
                            }
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(
                                    "Cookie 已持久化到 ~/.postman-rs/cookies.json（跨请求自动携带）。",
                                )
                                .color(text_secondary())
                                .size(11.0),
                            );
                        });
                }
                3 => {
                    let results = tests_engine::evaluate(&tab.tests, r);
                    egui::ScrollArea::vertical()
                        .id_salt("rtests")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if results.is_empty() {
                                ui.label(
                                    RichText::new(
                                        "此请求没有测试断言。可在请求区「测试」标签用 JSON 添加，或在「脚本」标签用 pm.test 编写。",
                                    )
                                    .color(text_secondary()),
                                );
                            }
                            for t in &results {
                                let c = if t.passed {
                                    Color32::from_rgb(73, 162, 91)
                                } else {
                                    Color32::from_rgb(235, 32, 19)
                                };
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(if t.passed { "✓" } else { "✗" })
                                            .color(c)
                                            .strong(),
                                    );
                                    ui.label(RichText::new(&t.name).color(text_primary()));
                                    if !t.detail.is_empty() {
                                        ui.label(
                                            RichText::new(&t.detail)
                                                .color(text_secondary())
                                                .size(11.0),
                                        );
                                    }
                                });
                            }
                            for t in &tab.script_results {
                                let c = if t.passed {
                                    Color32::from_rgb(73, 162, 91)
                                } else {
                                    Color32::from_rgb(235, 32, 19)
                                };
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(if t.passed { "✓" } else { "✗" })
                                            .color(c)
                                            .strong(),
                                    );
                                    ui.label(RichText::new(&t.name).color(text_primary()));
                                    if let Some(e) = &t.error {
                                        if !e.is_empty() {
                                            ui.label(
                                                RichText::new(e)
                                                    .color(text_secondary())
                                                    .size(11.0),
                                            );
                                        }
                                    }
                                });
                            }
                        });
                }
                _ => {
                    egui::ScrollArea::vertical()
                        .id_salt("rtiming")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(format!("总耗时：{} ms", r.elapsed_ms))
                                    .color(text_primary())
                                    .strong(),
                            );
                            ui.add_space(4.0);
                            let mut line = |label: &str, v: Option<u128>| {
                                if let Some(ms) = v {
                                    ui.label(
                                        RichText::new(format!("{}：{} ms", label, ms))
                                            .color(text_secondary())
                                            .size(11.5),
                                    );
                                }
                            };
                            line("DNS 解析", r.dns_ms);
                            line("TCP 连接", r.tcp_ms);
                            line("首字节 (TTFB，含 TLS/服务器)", r.ttfb_ms);
                            line("内容下载", r.download_ms);
                            ui.label(
                                RichText::new("说明：DNS/TCP 为独立探测阶段（近似值）；TLS 握手包含在首字节耗时内。")
                                    .color(text_secondary())
                                    .size(11.0),
                            );
                            if r.binary {
                                ui.label(
                                    RichText::new("响应体为二进制，已以 Base64 显示。")
                                        .color(text_secondary())
                                        .size(11.0),
                                );
                            }
                        });
                }
            }
        } else {
            ui.add_space(22.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("尚无响应")
                        .color(text_primary())
                        .size(14.0)
                        .strong(),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("填好请求后点击「发送」，响应会显示在这里")
                        .color(text_secondary())
                        .size(12.0),
                );
                if !tab.status.is_empty() {
                    ui.add_space(4.0);
                    ui.label(RichText::new(&tab.status).color(text_secondary()).size(11.5));
                }
            });
        }
        if let Some(i) = load_ex {
            if let Some(ex) = examples.get(i) {
                tab.response = Some(example_to_record(ex));
                tab.status = format!(
                    "示例 {} {}",
                    ex.code.unwrap_or(0),
                    ex.status.clone().unwrap_or_default()
                );
                tab.resp_subtab = 0;
            }
        }
        if save_ex {
            self.save_example();
        }
        if save_file {
            self.save_response_to_file(&body_clone);
        }
    }
}
impl eframe::App for GuiApp {
    /// 用主题背景色清屏：即使某个面板因内容溢出而没有铺满，也不会露出纯黑底（黑边）。
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        bg_side().to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 跟随系统：按操作系统深浅色偏好选择内置的浅色/深色主题。
        if self.follow_system {
            let idx = match ctx.system_theme() {
                Some(egui::Theme::Dark) => 1,
                Some(egui::Theme::Light) => 0,
                None => {
                    if ctx.style().visuals.dark_mode { 1 } else { 0 }
                }
            };
            if THEME.load(Ordering::Relaxed) != idx {
                THEME.store(idx, Ordering::Relaxed);
            }
        }
        // eframe 可能按系统主题覆盖 egui 视觉；每帧重申一次我们的主题样式，
        // 保证下拉框/复选框等原生控件与自定义配色一致。
        install_style(ctx);
        // 键盘快捷键（Ctrl/Cmd 组合）
        let (k_send, k_save, k_new, k_close) = ctx.input(|i| {
            let cmd = i.modifiers.command;
            (
                cmd && i.key_pressed(egui::Key::Enter),
                cmd && i.key_pressed(egui::Key::S),
                cmd && i.key_pressed(egui::Key::N),
                cmd && i.key_pressed(egui::Key::W),
            )
        });
        if k_new {
            let n = self.tabs.len() + 1;
            self.tabs.push(Tab::new(n));
            self.active = self.tabs.len() - 1;
        }
        if k_close && self.tabs.len() > 1 {
            self.tabs.remove(self.active);
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len() - 1;
            }
        }
        if k_save {
            self.save_active();
        }
        if k_send {
            self.send_active(ctx);
        }
        let mut got_new = false;
        for t in &mut self.tabs {
            if t.poll() {
                got_new = true;
                let ts = chrono::Local::now().format("%H:%M:%S").to_string();
                let line = match (&t.error, &t.response) {
                    (_, Some(resp)) => format!(
                        "[{}] {} {} → {} {} ({} ms)",
                        ts, t.method, t.url, resp.code, resp.status, resp.elapsed_ms
                    ),
                    (Some(e), None) => format!("[{}] {} {} → 错误：{}", ts, t.method, t.url, e),
                    _ => String::new(),
                };
                if !line.is_empty() {
                    self.console.push(line);
                }
                for tr in &t.script_results {
                    self.console.push(format!(
                        "[{}] 测试{}：{}",
                        ts,
                        if tr.passed { "通过" } else { "失败" },
                        tr.name
                    ));
                }
            }
        }
        if got_new {
            self.history = storage::read_history().unwrap_or_default();
        }
        if let Some(rx) = &self.run_rx {
            match rx.try_recv() {
                Ok(msg) => {
                    self.run_result = msg;
                    self.run_inflight = false;
                    self.run_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.run_inflight = false;
                    self.run_rx = None;
                }
            }
        }
        if let Some(rx) = &self.ws_rx {
            match rx.try_recv() {
                Ok(Ok(lines)) => {
                    self.ws_log = lines.join("\n");
                    self.ws_busy = false;
                    self.ws_rx = None;
                }
                Ok(Err(e)) => {
                    self.ws_log = format!("错误：{}", e);
                    self.ws_busy = false;
                    self.ws_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.ws_busy = false;
                    self.ws_rx = None;
                }
            }
        }
        if self.tabs.iter().any(|t| t.inflight) {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        }
        if self.run_inflight {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
        if self.ws_busy {
            ctx.request_repaint_after(std::time::Duration::from_millis(200));
        }
        self.ui_topbar(ctx);
        self.ui_sidebar(ctx);
        self.ui_main(ctx);
    }
}
// ---------------------------------------------------------------------------
// 工具函数
// ---------------------------------------------------------------------------
fn flatten(items: &[Item], prefix: &str, out: &mut Vec<(String, Request, Option<String>)>) {
    for it in items {
        let name = if prefix.is_empty() {
            it.name.clone()
        } else {
            format!("{} / {}", prefix, it.name)
        };
        if let Some(children) = &it.item {
            flatten(children, &name, out);
        }
        if let Some(req) = &it.request {
            let desc = it.description.as_ref().map(|d| match d {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            });
            out.push((name, req.clone(), desc));
        }
    }
}

/// 集合树的一行（用于 GUI 树形展示与 CRUD）
struct TreeRow {
    path: Vec<usize>,
    depth: usize,
    is_folder: bool,
    name: String,
    method: Option<String>,
}

fn collect_rows(items: &[Item], depth: usize, prefix: &mut Vec<usize>, out: &mut Vec<TreeRow>) {
    for (i, it) in items.iter().enumerate() {
        prefix.push(i);
        let is_folder = it.request.is_none() && it.item.is_some();
        out.push(TreeRow {
            path: prefix.clone(),
            depth,
            is_folder,
            name: it.name.clone(),
            method: it.request.as_ref().map(|r| r.method.clone()),
        });
        if let Some(children) = &it.item {
            collect_rows(children, depth + 1, prefix, out);
        }
        prefix.pop();
    }
}

fn item_at_mut<'a>(items: &'a mut Vec<Item>, path: &[usize]) -> Option<&'a mut Item> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        return items.get_mut(path[0]);
    }
    let head = items.get_mut(path[0])?;
    let children = head.item.as_mut()?;
    item_at_mut(children, &path[1..])
}

fn item_at<'a>(items: &'a [Item], path: &[usize]) -> Option<&'a Item> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        return items.get(path[0]);
    }
    let head = items.get(path[0])?;
    item_at(head.item.as_ref()?, &path[1..])
}

fn remove_at(items: &mut Vec<Item>, path: &[usize]) -> Option<Item> {
    if path.is_empty() {
        return None;
    }
    if path.len() == 1 {
        if path[0] < items.len() {
            return Some(items.remove(path[0]));
        }
        return None;
    }
    let head = items.get_mut(path[0])?;
    remove_at(head.item.as_mut()?, &path[1..])
}

fn short_title(name: &str) -> String {
    name.rsplit('/').next().unwrap_or(name).trim().to_string()
}
fn ctx_from(coll: Option<&Collection>, env: Option<&Environment>) -> VarContext {
    let mut ctx = VarContext::new();
    if let Some(e) = env {
        let m: HashMap<String, String> = e
            .values
            .iter()
            .filter(|v| v.enabled.unwrap_or(true))
            .map(|v| (v.key.clone(), value_str(&v.value)))
            .collect();
        if !m.is_empty() {
            ctx.scopes.push(m);
        }
    }
    if let Some(c) = coll {
        let m: HashMap<String, String> = c
            .variable
            .iter()
            .filter(|v| v.enabled.unwrap_or(true))
            .map(|v| (v.key.clone(), value_str(&v.value)))
            .collect();
        if !m.is_empty() {
            ctx.scopes.push(m);
        }
    }
    // 全局变量（最低优先级）——修复：此前 GUI 发送时未纳入 globals，导致全局变量不生效。
    if let Ok(g) = storage::load_globals() {
        let m: HashMap<String, String> = g
            .values
            .iter()
            .filter(|v| v.enabled.unwrap_or(true))
            .map(|v| (v.key.clone(), value_str(&v.value)))
            .collect();
        if !m.is_empty() {
            ctx.scopes.push(m);
        }
    }
    ctx
}

/// 沿集合项路径收集继承自集合/文件夹的 auth、变量与事件（后者含 pre-request/test）。
fn collect_inherited(coll: &Collection, path: &[usize]) -> (Option<Auth>, HashMap<String, String>, Vec<Event>) {
    let mut auth = coll.auth.clone();
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut events: Vec<Event> = coll.event.clone();
    let mut items: &[Item] = &coll.item;
    for &idx in path {
        match items.get(idx) {
            Some(it) => {
                if let Some(a) = &it.auth {
                    auth = Some(a.clone());
                }
                for v in &it.variable {
                    if v.enabled.unwrap_or(true) {
                        vars.insert(v.key.clone(), value_str(&v.value));
                    }
                }
                events.extend(it.event.iter().cloned());
                items = it.item.as_deref().unwrap_or(&[]);
            }
            None => break,
        }
    }
    (auth, vars, events)
}
fn value_str(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// 请求体类型（内部键）→ 中文标签
fn body_type_label(t: &str) -> &'static str {
    match t {
        "form-data" => "form-data 表单",
        "x-www-form-urlencoded" => "x-www-form-urlencoded 表单",
        "raw" => "raw 原始",
        "binary" => "binary 二进制",
        "graphql" => "GraphQL 查询",
        "none" => "无",
        _ => "无",
    }
}

/// 导入格式（内部键）→ 中文标签
fn import_format_label(f: &str) -> &'static str {
    match f {
        "postman" => "Postman 集合",
        "openapi" => "OpenAPI / Swagger",
        "curl" => "cURL 命令",
        "har" => "HAR 抓包",
        _ => "Postman 集合",
    }
}

fn grant_label(g: &str) -> &'static str {
    match g {
        "password" => "密码模式 (password)",
        "refresh_token" => "刷新令牌 (refresh_token)",
        _ => "客户端凭证 (client_credentials)",
    }
}

fn default_auth_keys(kind: &str) -> &'static [&'static str] {
    match kind {
        "oauth1" => &["consumerKey", "consumerSecret", "token", "tokenSecret"],
        "awsv4" => &["accessKey", "secretKey", "region", "service", "sessionToken"],
        "jwt" => &["algorithm", "secret", "payload", "token"],
        "digest" => &["username", "password"],
        "ntlm" => &["username", "password"],
        _ => &[],
    }
}

fn example_to_record(ex: &ResponseExample) -> ResponseRecord {
    ResponseRecord {
        status: ex.status.clone().unwrap_or_default(),
        code: ex.code.unwrap_or(0),
        headers: ex
            .header
            .iter()
            .map(|h| (h.key.clone(), h.value.clone()))
            .collect(),
        body: ex.body.clone().unwrap_or_default(),
        binary: false,
        elapsed_ms: 0,
        final_url: String::new(),
        dns_ms: None,
        tcp_ms: None,
        ttfb_ms: None,
        download_ms: None,
    }
}

fn oauth2_field(
    ui: &mut egui::Ui,
    map: &mut HashMap<String, String>,
    label: &str,
    key: &str,
    secret: bool,
) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [150.0, 20.0],
            egui::Label::new(RichText::new(label).color(text_secondary()).size(12.0)),
        );
        let mut val = map.get(key).cloned().unwrap_or_default();
        if ui
            .add(
                egui::TextEdit::singleline(&mut val)
                    .password(secret)
                    .desired_width(300.0),
            )
            .changed()
        {
            map.insert(key.to_string(), val);
        }
    });
}

/// raw 子语言标签 → Postman 存储用小写键
fn raw_lang_key(l: &str) -> &'static str {
    match l {
        "JSON" => "json",
        "XML" => "xml",
        "HTML" => "html",
        "JavaScript" => "javascript",
        _ => "text",
    }
}

/// Postman 存储用小写语言键 → 标签
fn raw_lang_label(l: &str) -> &'static str {
    match l.to_ascii_lowercase().as_str() {
        "json" => "JSON",
        "xml" => "XML",
        "html" => "HTML",
        "javascript" => "JavaScript",
        _ => "Text",
    }
}

fn file_name(p: &PathBuf) -> String {
    p.file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| p.display().to_string())
}
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1320.0, 880.0])
            .with_min_inner_size([960.0, 620.0])
            .with_title("pmrs — 本地 API 客户端"),
        ..Default::default()
    };
    eframe::run_native("pmrs", options, Box::new(|cc| Ok(Box::new(GuiApp::new(cc)))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_types_build_correctly() {
        let mut t = Tab::new(1);

        // 无请求体
        t.body_type = "none".into();
        assert!(t.build_request().body.is_none());

        // raw / JSON
        t.body_type = "raw".into();
        t.raw_lang = "JSON".into();
        t.body = "{\"a\":1}".into();
        let b = t.build_request().body.unwrap();
        assert_eq!(b.mode, "raw");
        assert_eq!(b.raw.as_deref(), Some("{\"a\":1}"));
        assert_eq!(b.raw_content_type().as_deref(), Some("application/json"));

        // x-www-form-urlencoded
        t.body_type = "x-www-form-urlencoded".into();
        t.url_rows = vec![KVRow {
            key: "a".into(),
            value: "1".into(),
            disabled: false,
            is_file: false,
        }];
        let b = t.build_request().body.unwrap();
        assert_eq!(b.mode, "urlencoded");
        assert_eq!(b.urlencoded[0].key, "a");
        assert_eq!(b.urlencoded[0].value, "1");

        // form-data（含文件行）
        t.body_type = "form-data".into();
        t.form_rows = vec![KVRow {
            key: "f".into(),
            value: "C:/x.bin".into(),
            disabled: false,
            is_file: true,
        }];
        let b = t.build_request().body.unwrap();
        assert_eq!(b.mode, "formdata");
        assert_eq!(b.formdata[0].typ.as_deref(), Some("file"));
        assert_eq!(
            b.formdata[0].src.as_ref().and_then(|v| v.as_str()),
            Some("C:/x.bin")
        );

        // binary（本地文件）
        t.body_type = "binary".into();
        t.file_path = "C:/x.bin".into();
        let b = t.build_request().body.unwrap();
        assert_eq!(b.mode, "file");
        assert_eq!(b.file.unwrap().src.as_deref(), Some("C:/x.bin"));

        // GraphQL
        t.body_type = "graphql".into();
        t.gql_query = "query{a}".into();
        t.gql_vars = "{}".into();
        let b = t.build_request().body.unwrap();
        assert_eq!(b.mode, "graphql");
        assert_eq!(b.graphql.unwrap().query, "query{a}");
    }

    #[test]
    fn label_mappings() {
        assert_eq!(raw_lang_key("JSON"), "json");
        assert_eq!(raw_lang_key("JavaScript"), "javascript");
        assert_eq!(raw_lang_label("xml"), "XML");
        assert_eq!(body_type_label("form-data"), "form-data 表单");
        assert_eq!(import_format_label("openapi"), "OpenAPI / Swagger");
    }
}


