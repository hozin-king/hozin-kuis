// Hozin Kuis v2.5 — full Rust (eframe/egui + android-activity).
// Offline penuh: bank soal Bahasa Indonesia + data/sprite 151 Pokemon Gen 1
// dibundel di binary. Tanpa WebView, tanpa iklan, tanpa IAP, tanpa login,
// tanpa network saat runtime.
mod pokemon_data;
mod questions_data;

use egui::epaint::Mesh;
use egui::{
    Align, Button, Color32, ColorImage, CornerRadius, FontFamily, FontId, Frame, LayerId,
    Layout, Margin, ProgressBar, RichText, Shape, Stroke, TextStyle, TextureHandle,
    TextureOptions, Vec2,
};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

include!(concat!(env!("OUT_DIR"), "/sprites.rs"));

const QUESTION_SECS: u64 = 30;
const ROUND_LEN: usize = 10;
const CATEGORIES: [&str; 6] = ["Umum", "Sains", "Sejarah", "Olahraga", "Film", "Komputer"];

// Palet v2.2: gradien latar biru gelap -> ungu + aksen biru terang;
// hijau/merah hanya untuk benar/salah (di bawah).
const ACCENT: Color32 = Color32::from_rgb(0, 170, 210);
const MUTED: Color32 = Color32::from_rgb(165, 175, 185);
const GRAD_TOP: Color32 = Color32::from_rgb(10, 18, 48);
const GRAD_MID: Color32 = Color32::from_rgb(26, 22, 70);
const GRAD_BOT: Color32 = Color32::from_rgb(56, 20, 84);
const CARD: Color32 = Color32::from_rgb(18, 26, 54);
const CHIP_DARK: Color32 = Color32::from_rgb(45, 52, 64);
const POKEMON_GOLD: Color32 = Color32::from_rgb(230, 160, 30);
// Warna tiap kategori / tingkat kesulitan (hanya tampilan;
// logika seleksi tidak berubah).
const CAT_COLORS: [Color32; 6] = [
    Color32::from_rgb(0, 170, 210),   // Umum — biru
    Color32::from_rgb(46, 160, 67),   // Sains — hijau
    Color32::from_rgb(230, 160, 30),  // Sejarah — emas
    Color32::from_rgb(220, 90, 60),   // Olahraga — merah-oranye
    Color32::from_rgb(150, 90, 220),  // Film — ungu
    Color32::from_rgb(30, 180, 170),  // Komputer — teal
];
const DIFF_COLORS: [Color32; 3] = [
    Color32::from_rgb(46, 160, 67),  // Mudah — hijau
    Color32::from_rgb(230, 160, 30), // Sedang — emas
    Color32::from_rgb(200, 60, 60),  // Sulit — merah
];
// Tint gelap per huruf jawaban (A biru, B hijau, C emas, D ungu) —
// cukup gelap untuk teks putih, cukup beda untuk dikenali sekilas.
const LETTER_TINTS: [Color32; 4] = [
    Color32::from_rgb(14, 58, 78),  // A — biru
    Color32::from_rgb(18, 66, 36),  // B — hijau
    Color32::from_rgb(74, 56, 18),  // C — emas
    Color32::from_rgb(50, 32, 80),  // D — ungu
];
// Maskot menu: starter + Pikachu + Eevee (sprite sudah dibundel).
const MASCOTS: [u32; 5] = [1, 4, 7, 25, 133];

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Trivia,
    Pokemon,
}

struct QuizItem {
    prompt: String,
    hint: Option<String>,
    pokemon_id: Option<u32>,
    options: [String; 4],
    answer: usize,
}

#[derive(Clone, Copy)]
enum Phase {
    Answering,
    Answered { chosen: Option<usize> },
}

struct QuizState {
    mode: Mode,
    items: Vec<QuizItem>,
    idx: usize,
    score: i32,
    streak: i32,
    correct: usize,
    phase: Phase,
    deadline: Instant,
}

struct ResultState {
    mode: Mode,
    score: i32,
    correct: usize,
    total: usize,
    best: i32,
    new_best: bool,
}

enum Screen {
    Menu,
    Quiz(QuizState),
    Result(ResultState),
}

enum Action {
    None,
    StartTrivia,
    StartPokemon,
    Answer(Option<usize>),
    Next,
    PlayAgain(Mode),
    ToMenu,
}

struct KuisApp {
    screen: Screen,
    menu_category: &'static str,
    menu_difficulty: u8,
    best_trivia: i32,
    best_pokemon: i32,
    data_path: PathBuf,
    textures: HashMap<u32, (TextureHandle, TextureHandle)>,
}

impl KuisApp {
    fn new(cc: &eframe::CreationContext<'_>, data_path: PathBuf) -> Self {
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(34.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(22.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(22.0, FontFamily::Proportional),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(17.0, FontFamily::Proportional),
        );
        style.spacing.button_padding = Vec2::new(16.0, 14.0);
        style.spacing.item_spacing = Vec2::new(10.0, 12.0);
        // Palet v2.2: sudut membulat + tombol sedikit lebih terang dari latar,
        // pilihan terpilih memakai aksen. Panel dibuat transparan supaya
        // gradien latar (paint_background) terlihat di seluruh layar.
        style.visuals.panel_fill = Color32::TRANSPARENT;
        style.visuals.selection.bg_fill = ACCENT;
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(45, 52, 64);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(60, 68, 84);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(70, 80, 98);
        cc.egui_ctx.set_style(style);

        let (best_trivia, best_pokemon) = Self::load_best(&data_path);
        Self {
            screen: Screen::Menu,
            menu_category: "Umum",
            menu_difficulty: 0,
            best_trivia,
            best_pokemon,
            data_path,
            textures: HashMap::new(),
        }
    }

    fn best_path(&self) -> PathBuf {
        self.data_path.join("best.txt")
    }

    fn load_best(data_path: &PathBuf) -> (i32, i32) {
        match std::fs::read_to_string(data_path.join("best.txt")) {
            Ok(s) => {
                let mut it = s.lines();
                let t = it.next().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
                let p = it.next().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
                (t, p)
            }
            Err(_) => (0, 0),
        }
    }

    fn save_best(&self) {
        let _ = std::fs::write(
            self.best_path(),
            format!("{}\n{}\n", self.best_trivia, self.best_pokemon),
        );
    }

    fn start_trivia(&mut self) {
        let mut rng = rand::rng();
        // Pool kategori ini; soal tingkat terpilih didahulukan, sisanya pengisi.
        let mut pool: Vec<&questions_data::Question> = questions_data::QUESTIONS
            .iter()
            .filter(|q| q.category == self.menu_category)
            .collect();
        pool.shuffle(&mut rng);
        pool.sort_by_key(|q| q.difficulty != self.menu_difficulty);
        let mut items = Vec::new();
        for q in pool.into_iter().take(ROUND_LEN) {
            let mut opts: Vec<(usize, &&str)> =
                q.options.iter().enumerate().collect();
            opts.shuffle(&mut rng);
            let answer = opts.iter().position(|(i, _)| *i == q.answer).unwrap();
            let options: [String; 4] = opts
                .iter()
                .map(|(_, s)| s.to_string())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            items.push(QuizItem {
                prompt: q.text.to_string(),
                hint: None,
                pokemon_id: None,
                options,
                answer,
            });
        }
        self.screen = Screen::Quiz(QuizState {
            mode: Mode::Trivia,
            items,
            idx: 0,
            score: 0,
            streak: 0,
            correct: 0,
            phase: Phase::Answering,
            deadline: Instant::now() + Duration::from_secs(QUESTION_SECS),
        });
    }

    fn start_pokemon(&mut self) {
        let mut rng = rand::rng();
        let mut ids: Vec<u32> = (1..=151).collect();
        ids.shuffle(&mut rng);
        let mut items = Vec::new();
        for id in ids.into_iter().take(ROUND_LEN) {
            let p = &pokemon_data::POKEMON[(id - 1) as usize];
            let mut others: Vec<u32> = (1..=151).filter(|x| *x != id).collect();
            others.shuffle(&mut rng);
            let mut opts: Vec<String> = others
                .into_iter()
                .take(3)
                .map(|x| pokemon_data::POKEMON[(x - 1) as usize].name.to_string())
                .collect();
            opts.push(p.name.to_string());
            opts.shuffle(&mut rng);
            let answer = opts.iter().position(|n| n == p.name).unwrap();
            let options: [String; 4] = opts.try_into().unwrap();
            items.push(QuizItem {
                prompt: "Siapa Pokemon ini?".to_string(),
                hint: Some(format!(
                    "Tipe: {} \u{2022} Tinggi: {:.1} m \u{2022} Berat: {:.1} kg",
                    p.types, p.height_m, p.weight_kg
                )),
                pokemon_id: Some(id),
                options,
                answer,
            });
        }
        self.screen = Screen::Quiz(QuizState {
            mode: Mode::Pokemon,
            items,
            idx: 0,
            score: 0,
            streak: 0,
            correct: 0,
            phase: Phase::Answering,
            deadline: Instant::now() + Duration::from_secs(QUESTION_SECS),
        });
    }

    fn answer(&mut self, chosen: Option<usize>) {
        if let Screen::Quiz(q) = &mut self.screen {
            if !matches!(q.phase, Phase::Answering) {
                return;
            }
            let correct = chosen == Some(q.items[q.idx].answer);
            if correct {
                q.score += 10;
                q.streak += 1;
                q.correct += 1;
            } else {
                q.streak = 0;
            }
            q.phase = Phase::Answered { chosen };
        }
    }

    fn next(&mut self) {
        let screen = std::mem::replace(&mut self.screen, Screen::Menu);
        if let Screen::Quiz(mut q) = screen {
            q.idx += 1;
            if q.idx >= q.items.len() {
                let total = q.items.len();
                let (best, is_new) = match q.mode {
                    Mode::Trivia => {
                        let nb = q.score > self.best_trivia;
                        if nb {
                            self.best_trivia = q.score;
                        }
                        (self.best_trivia, nb)
                    }
                    Mode::Pokemon => {
                        let nb = q.score > self.best_pokemon;
                        if nb {
                            self.best_pokemon = q.score;
                        }
                        (self.best_pokemon, nb)
                    }
                };
                if is_new {
                    self.save_best();
                }
                self.screen = Screen::Result(ResultState {
                    mode: q.mode,
                    score: q.score,
                    correct: q.correct,
                    total,
                    best,
                    new_best: is_new,
                });
            } else {
                q.phase = Phase::Answering;
                q.deadline = Instant::now() + Duration::from_secs(QUESTION_SECS);
                self.screen = Screen::Quiz(q);
            }
        } else {
            self.screen = screen;
        }
    }
}

impl eframe::App for KuisApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Timer habis saat masih menjawab -> dihitung salah (tanpa pilihan).
        let mut timed_out = false;
        if let Screen::Quiz(q) = &mut self.screen {
            if matches!(q.phase, Phase::Answering) {
                if Instant::now() >= q.deadline {
                    timed_out = true;
                } else {
                    ctx.request_repaint_after(Duration::from_millis(100));
                }
            }
        }
        if timed_out {
            self.answer(None);
        }

        paint_background(ctx);

        // ScrollArea vertikal sebagai jaring pengaman: tidak ada lagi elemen
        // yang bisa terdorong ke luar viewport (bug menu v2.2). Konten yang
        // lebih tinggi dari layar bisa di-scroll; yang muat tetap tampil penuh.
        // v2.5: tiap layar punya id_salt sendiri — dan layar kuis punya id
        // per soal — supaya posisi scroll TIDAK terbawa dari soal/layar
        // sebelumnya (tanpa ini, soal baru bisa terbuka dalam keadaan sudah
        // ter-scroll ke bawah dengan header terpotong). Header kuis (tombol
        // Menu + skor + timer) dirender DI LUAR ScrollArea supaya tetap
        // terlihat saat jawaban di-scroll.
        let action = egui::CentralPanel::default()
            .show(ctx, |ui| {
                match &mut self.screen {
                    Screen::Menu => egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .id_salt("menu")
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                let cat = &mut self.menu_category;
                                let dif = &mut self.menu_difficulty;
                                let (bt, bp) = (self.best_trivia, self.best_pokemon);
                                show_menu(ctx, ui, cat, dif, bt, bp, &mut self.textures)
                            })
                            .inner
                        })
                        .inner,
                    Screen::Quiz(q) => {
                        let mut action = show_quiz_header(ui, q);
                        let body = egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .id_salt(("quiz", matches!(q.mode, Mode::Pokemon), q.idx))
                            .show(ui, |ui| {
                                ui.vertical_centered(|ui| {
                                    show_quiz_body(ctx, ui, q, &mut self.textures)
                                })
                                .inner
                            })
                            .inner;
                        if !matches!(body, Action::None) {
                            action = body;
                        }
                        action
                    }
                    Screen::Result(r) => egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .id_salt("result")
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                show_result(ctx, ui, r, &mut self.textures)
                            })
                            .inner
                        })
                        .inner,
                }
            })
            .inner;

        match action {
            Action::None => {}
            Action::StartTrivia => self.start_trivia(),
            Action::StartPokemon => self.start_pokemon(),
            Action::Answer(c) => self.answer(c),
            Action::Next => self.next(),
            Action::PlayAgain(m) => match m {
                Mode::Trivia => self.start_trivia(),
                Mode::Pokemon => self.start_pokemon(),
            },
            Action::ToMenu => self.screen = Screen::Menu,
        }
    }
}

fn paint_background(ctx: &egui::Context) {
    // Gradien vertikal 3 titik (biru gelap -> indigo -> ungu) menutupi
    // seluruh layar; satu mesh, tanpa aset gambar.
    // (egui 0.33: colored_vertex tidak mengembalikan indeks, dan
    // Shape::Mesh memegang Arc<Mesh>.)
    let rect = ctx.content_rect();
    let mid_y = rect.top() + rect.height() * 0.55;
    let mut mesh = Mesh::default();
    mesh.colored_vertex(rect.left_top(), GRAD_TOP);
    mesh.colored_vertex(rect.right_top(), GRAD_TOP);
    mesh.colored_vertex(egui::pos2(rect.left(), mid_y), GRAD_MID);
    mesh.colored_vertex(egui::pos2(rect.right(), mid_y), GRAD_MID);
    mesh.colored_vertex(rect.left_bottom(), GRAD_BOT);
    mesh.colored_vertex(rect.right_bottom(), GRAD_BOT);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(1, 3, 2);
    mesh.add_triangle(2, 3, 4);
    mesh.add_triangle(3, 5, 4);
    ctx.layer_painter(LayerId::background())
        .add(Shape::Mesh(std::sync::Arc::new(mesh)));
}

// Chip seleksi: terpilih berisi warna kategori, tidak terpilih gelap + teks berwarna.
fn chip(
    ui: &mut egui::Ui,
    width: f32,
    selected: bool,
    color: Color32,
    text: &str,
) -> egui::Response {
    let btn = Button::new(
        RichText::new(text)
            .size(20.0)
            .color(if selected { Color32::WHITE } else { color }),
    );
    let btn = if selected {
        btn.fill(color)
    } else {
        btn.fill(CHIP_DARK)
    };
    ui.add_sized(Vec2::new(width, 52.0), btn)
}

// Indikator progres ronde: titik per soal (terisi = sudah dijawab).
// Baris dipusatkan manual dengan spasi depan — JANGAN horizontal_centered:
// pola yang sama dengan kartu maskot v2.2, ia bisa mengembang dan mendorong
// konten di bawahnya (tombol LANJUT) ke luar layar.
fn progress_dots(ui: &mut egui::Ui, idx: usize, total: usize) {
    let total_w = total as f32 * 18.0 + (total.saturating_sub(1)) as f32 * 10.0;
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - total_w) / 2.0).max(0.0));
        for n in 0..total {
            let (rect, _) =
                ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::hover());
            let col = if n < idx {
                ACCENT
            } else if n == idx {
                Color32::WHITE
            } else {
                Color32::from_rgb(60, 68, 84)
            };
            ui.painter_at(rect).circle_filled(rect.center(), 7.0, col);
        }
    });
}

// Dorong strip bawah (maskot/footer) ke dasar layar: isi sisa ruang viewport
// dengan spasi. Aman di dalam ScrollArea karena memakai tinggi viewport
// nyata (ctx.content_rect), bukan available_height yang di dalam ScrollArea
// tak terbatas. Bila konten sudah lebih tinggi dari layar, gap <= 0 dan
// tidak ada spasi yang ditambahkan (ScrollArea yang menangani).
fn push_to_bottom(ctx: &egui::Context, ui: &mut egui::Ui, reserve: f32) {
    let gap = ctx.content_rect().bottom() - ui.cursor().min.y - reserve;
    if gap > 0.0 {
        ui.add_space(gap);
    }
}

// Deretan 5 maskot kecil di dasar layar (sprite sudah dibundel, tanpa aset
// baru). Dipusatkan manual — JANGAN horizontal_centered (lihat komentar
// progress_dots: ia bisa mengembang dan mendorong konten di bawahnya).
fn mascot_strip(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    textures: &mut HashMap<u32, (TextureHandle, TextureHandle)>,
    size: f32,
) {
    let total_w = 5.0 * size + 4.0 * 8.0;
    ui.horizontal(|ui| {
        ui.add_space(((ui.available_width() - total_w) / 2.0).max(0.0));
        for id in MASCOTS {
            let (ct, _) = get_textures(ctx, textures, id);
            ui.image((ct.id(), Vec2::new(size, size)));
        }
    });
}

fn show_menu(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    category: &mut &'static str,
    difficulty: &mut u8,
    best_trivia: i32,
    best_pokemon: i32,
    textures: &mut HashMap<u32, (TextureHandle, TextureHandle)>,
) -> Action {
    let mut action = Action::None;
    // Jarak TETAP (bukan fraksi available_height): di dalam ScrollArea
    // tinggi tersedia tidak terbatas, jadi fraksi tinggi layar akan
    // mengembang tak terkendali. Jarak tetap = layout deterministik.
    ui.add_space(12.0);
    // Banner judul ala game: kartu gelap + garis aksen + judul besar.
    Frame::default()
        .fill(CARD)
        .stroke(Stroke::new(2.0, ACCENT))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new("HOZIN KUIS")
                        .size(56.0)
                        .strong()
                        .color(ACCENT),
                );
                ui.label(
                    RichText::new("Trivia Bahasa Indonesia + Tebak Pokemon")
                        .size(18.0)
                        .color(MUTED),
                );
            });
        });
    ui.add_space(10.0);
    // Deretan maskot dari sprite yang sudah dibundel (tanpa aset baru).
    // AKAR BUG v2.2: Frame::group + horizontal_centered membuat kartu
    // mengembang setinggi layar (maskot terpusat vertikal di tengah kartu
    // raksasa itu) sehingga kartu skor, chip, tombol, dan footer terdorong
    // ke luar viewport. Pola pengganti: Frame::group + vertical_centered
    // (pola v2.1 yang terbukti tidak mengembang) + baris horizontal dengan
    // spasi depan manual supaya maskot tetap terpusat.
    let mascot_total = 5.0 * 64.0 + 4.0 * 10.0;
    egui::Frame::group(ui.style())
        .corner_radius(CornerRadius::same(14))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.horizontal(|ui| {
                    ui.add_space(((ui.available_width() - mascot_total) / 2.0).max(0.0));
                    for id in MASCOTS {
                        let (ct, _) = get_textures(ctx, textures, id);
                        ui.image((ct.id(), Vec2::new(64.0, 64.0)));
                    }
                });
            });
        });
    ui.add_space(10.0);
    egui::Frame::group(ui.style())
        .corner_radius(CornerRadius::same(14))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(format!("Skor terbaik Trivia: {best_trivia}")).size(20.0),
                );
                ui.label(
                    RichText::new(format!("Skor terbaik Pokemon: {best_pokemon}"))
                        .size(20.0),
                );
            });
        });
    ui.add_space(10.0);
    ui.label(RichText::new("Kategori Trivia:").size(20.0).strong());
    ui.add_space(4.0);
    let chip_w = (ui.available_width() - 20.0) / 3.0;
    for row in 0..2 {
        ui.horizontal(|ui| {
            for i in (row * 3)..(row * 3 + 3) {
                let cat = CATEGORIES[i];
                if chip(ui, chip_w, *category == cat, CAT_COLORS[i], cat).clicked() {
                    *category = cat;
                }
            }
        });
        ui.add_space(8.0);
    }
    ui.label(RichText::new("Tingkat kesulitan:").size(20.0).strong());
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        for (v, name) in [(0u8, "Mudah"), (1, "Sedang"), (2, "Sulit")] {
            if chip(ui, chip_w, *difficulty == v, DIFF_COLORS[v as usize], name).clicked() {
                *difficulty = v;
            }
        }
    });
    ui.add_space(16.0);
    if ui
        .add_sized(
            Vec2::new(ui.available_width(), 68.0),
            Button::new(
                RichText::new("MULAI TRIVIA")
                    .size(24.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(ACCENT),
        )
        .clicked()
    {
        action = Action::StartTrivia;
    }
    ui.add_space(10.0);
    if ui
        .add_sized(
            Vec2::new(ui.available_width(), 68.0),
            Button::new(
                RichText::new("MULAI TEBAK POKEMON")
                    .size(24.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(POKEMON_GOLD),
        )
        .clicked()
    {
        action = Action::StartPokemon;
    }
    // Footer didorong ke dasar layar (bukan spacer fraksi v2.2 yang rapuh):
    // push_to_bottom memakai tinggi viewport nyata, jadi footer selalu
    // menempel di bawah bila konten lebih pendek dari layar.
    ui.add_space(10.0);
    push_to_bottom(ctx, ui, 28.0);
    ui.label(
        RichText::new("Offline \u{2022} 120 soal \u{2022} 151 Pokemon Gen 1")
            .size(16.0)
            .color(MUTED),
    );
    action
}

fn decode_png(bytes: &[u8]) -> ColorImage {
    match image::load_from_memory(bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &rgba.into_raw())
        }
        Err(_) => ColorImage::new([96, 96], vec![Color32::TRANSPARENT; 96 * 96]),
    }
}

fn silhouette(img: &ColorImage) -> ColorImage {
    // Clone + mutasi supaya tetap kompatibel bila egui menambah field ColorImage.
    let mut out = img.clone();
    for p in out.pixels.iter_mut() {
        if p.a() > 0 {
            *p = Color32::BLACK;
        }
    }
    out
}

fn get_textures(
    ctx: &egui::Context,
    cache: &mut HashMap<u32, (TextureHandle, TextureHandle)>,
    id: u32,
) -> (TextureHandle, TextureHandle) {
    if let Some(t) = cache.get(&id) {
        return t.clone();
    }
    let bytes = SPRITES
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, b)| *b)
        .unwrap_or(&[]);
    let color = decode_png(bytes);
    let sil = silhouette(&color);
    let mut opts = TextureOptions::default();
    opts.magnification = egui::TextureFilter::Nearest;
    let ct = ctx.load_texture(format!("poke-{id}-color"), color, opts);
    let st = ctx.load_texture(format!("poke-{id}-sil"), sil, opts);
    cache.insert(id, (ct.clone(), st.clone()));
    (ct, st)
}

// Header kuis TETAP (v2.5): dirender di CentralPanel DI LUAR ScrollArea,
// jadi tombol kembali ke menu, nomor soal, skor, dan timer TIDAK ikut
// ke-scroll saat pengguna menggulir jawaban ke bawah (bug v2.4: header
// ikut ter-scroll ke luar layar — di screenshot user, tombol Menu dan
// timer tidak kelihatan sama sekali setelah scroll).
fn show_quiz_header(ui: &mut egui::Ui, q: &mut QuizState) -> Action {
    let mut action = Action::None;
    ui.add_space(12.0);
    // Baris atas: tombol kembali ke menu (kiri), nomor soal, skor (kanan).
    ui.horizontal(|ui| {
        if ui
            .add_sized(
                Vec2::new(104.0, 44.0),
                Button::new(RichText::new("\u{2190} Menu").size(20.0).strong()),
            )
            .clicked()
        {
            action = Action::ToMenu;
        }
        ui.add_space(8.0);
        ui.label(
            RichText::new(format!("Soal {}/{}", q.idx + 1, q.items.len())).size(20.0),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let skor = if q.streak >= 2 {
                format!("Skor: {}   Streak: {}x", q.score, q.streak)
            } else {
                format!("Skor: {}", q.score)
            };
            ui.label(RichText::new(skor).size(20.0).strong());
        });
    });
    ui.add_space(10.0);
    // Timer: bar dibuat 36px + teks 18pt supaya teks muat rapi DI DALAM bar
    // (bug v2.3: bar tinggi default ~20px dengan teks 22pt, teks menumpuk
    // di atas bar).
    if matches!(q.phase, Phase::Answering) {
        let remain = q.deadline.saturating_duration_since(Instant::now());
        let frac = remain.as_secs_f32() / QUESTION_SECS as f32;
        ui.add(
            ProgressBar::new(frac)
                .desired_height(36.0)
                .text(
                    RichText::new(format!("Sisa waktu: {} detik", remain.as_secs()))
                        .size(18.0),
                ),
        );
    } else {
        ui.add(
            ProgressBar::new(0.0)
                .desired_height(36.0)
                .text(RichText::new("Waktu berhenti").size(18.0)),
        );
    }
    ui.add_space(12.0);
    action
}

// Isi kuis yang bisa di-scroll (v2.5): label mode, kartu soal, siluet,
// petunjuk, jawaban A-D, pesan hasil + titik progres + LANJUT, dan strip
// maskot di ekor. ScrollArea-nya diberi id per soal (lihat update) supaya
// tiap soal baru mulai dari atas, tidak mewarisi posisi scroll soal lama.
fn show_quiz_body(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    q: &mut QuizState,
    textures: &mut HashMap<u32, (TextureHandle, TextureHandle)>,
) -> Action {
    let mut action = Action::None;
    let item = &q.items[q.idx];
    let answered = matches!(q.phase, Phase::Answered { .. });

    let mode_name = match q.mode {
        Mode::Trivia => "TRIVIA",
        Mode::Pokemon => "TEBAK POKEMON",
    };
    ui.label(RichText::new(mode_name).size(18.0).strong().color(ACCENT));
    ui.add_space(6.0);
    Frame::default()
        .fill(CARD)
        .stroke(Stroke::new(2.0_f32, ACCENT))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::symmetric(18, 16))
        .show(ui, |ui| {
            ui.set_min_height(110.0);
            ui.label(RichText::new(&item.prompt).size(30.0).strong());
        });
    ui.add_space(16.0);

    // PERBAIKAN SILUET (bug v1.0: hitam di atas hitam): sprite selalu digambar
    // di atas lingkaran LATAR TERANG, jadi siluet hitam pekat jelas kelihatan.
    if let Some(pid) = item.pokemon_id {
        let (ct, st) = get_textures(ctx, textures, pid);
        let tex = if answered { &ct } else { &st };
        let (rect, _) = ui.allocate_exact_size(Vec2::new(220.0, 220.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.circle_filled(
            rect.center(),
            104.0,
            Color32::from_rgb(232, 234, 240),
        );
        let img_rect =
            egui::Rect::from_center_size(rect.center(), Vec2::new(176.0, 176.0));
        painter.image(
            tex.id(),
            img_rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        ui.add_space(8.0);
    }
    if let Some(hint) = &item.hint {
        ui.label(RichText::new(hint).size(18.0));
        ui.add_space(8.0);
    }

    let letters = ["A", "B", "C", "D"];
    for (i, opt) in item.options.iter().enumerate() {
        let text = if matches!(q.mode, Mode::Pokemon) {
            opt.to_uppercase()
        } else {
            opt.clone()
        };
        let text = format!("{}. {}", letters[i], text);
        // Tombol jawaban 84px + tint gelap per huruf (A biru, B hijau,
        // C emas, D ungu) — v2.3: 64px abu-abu datar, tanpa identitas huruf.
        let mut btn = Button::new(RichText::new(text).size(24.0)).fill(LETTER_TINTS[i]);
        if let Phase::Answered { chosen } = &q.phase {
            if i == item.answer {
                btn = btn.fill(Color32::from_rgb(46, 160, 67));
            } else if Some(i) == *chosen {
                btn = btn.fill(Color32::from_rgb(200, 60, 60));
            }
        }
        if ui
            .add_sized(Vec2::new(ui.available_width(), 84.0), btn)
            .clicked()
            && matches!(q.phase, Phase::Answering)
        {
            action = Action::Answer(Some(i));
        }
        ui.add_space(12.0);
    }

    if let Phase::Answered { chosen } = &q.phase {
        let benar = item.options[item.answer].clone();
        let benar = if matches!(q.mode, Mode::Pokemon) {
            benar.to_uppercase()
        } else {
            benar
        };
        let msg = match chosen {
            None => format!("Waktu habis! Jawaban benar: {benar}"),
            Some(c) if *c == item.answer => {
                format!("Benar! +10   (Streak: {}x)", q.streak)
            }
            Some(_) => format!("Salah. Jawaban benar: {benar}"),
        };
        let msg_color = match chosen {
            Some(c) if *c == item.answer => Color32::from_rgb(80, 200, 110),
            _ => Color32::from_rgb(230, 110, 110),
        };
        ui.add_space(8.0);
        ui.label(RichText::new(msg).size(22.0).strong().color(msg_color));
        ui.add_space(8.0);
        progress_dots(ui, q.idx, q.items.len());
        ui.add_space(16.0);
        if ui
            .add_sized(
                Vec2::new(ui.available_width(), 60.0),
                Button::new(
                    RichText::new("LANJUT")
                        .size(22.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(ACCENT),
            )
            .clicked()
        {
            action = Action::Next;
        }
    } else {
        // Masih menjawab: indikator progres tepat setelah opsi (jarak tetap).
        ui.add_space(16.0);
        progress_dots(ui, q.idx, q.items.len());
    }
    // Ekor layar: strip maskot didorong ke dasar layar (void ungu besar
    // di bawah titik progres pada v2.3). Bila konten lebih tinggi dari
    // layar, push_to_bottom tidak menambah apa-apa dan ScrollArea menangani.
    ui.add_space(12.0);
    push_to_bottom(ctx, ui, 52.0);
    mascot_strip(ctx, ui, textures, 40.0);
    // Padding bawah (v2.5): tanpa ini, saat konten di-scroll sampai mentok,
    // elemen terakhir menempel di tepi bawah layar (bug v2.4: opsi D
    // kelihatan terjepit di tepi bawah tanpa ruang).
    ui.add_space(24.0);
    action
}

fn show_result(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    r: &ResultState,
    textures: &mut HashMap<u32, (TextureHandle, TextureHandle)>,
) -> Action {
    let mut action = Action::None;
    ui.add_space(24.0);
    ui.label(RichText::new("SELESAI!").size(44.0).strong());
    let mode_name = match r.mode {
        Mode::Trivia => "Trivia",
        Mode::Pokemon => "Tebak Pokemon",
    };
    ui.label(RichText::new(mode_name).size(22.0).color(MUTED));
    ui.add_space(12.0);
    egui::Frame::group(ui.style())
        .corner_radius(CornerRadius::same(14))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new(format!("Skor: {}", r.score)).size(36.0).strong());
                ui.label(
                    RichText::new(format!("Benar: {}/{}", r.correct, r.total)).size(24.0),
                );
                ui.label(RichText::new(format!("Skor terbaik: {}", r.best)).size(24.0));
            });
        });
    if r.new_best {
        ui.add_space(8.0);
        ui.label(
            RichText::new("SKOR TERBAIK BARU!")
                .size(24.0)
                .strong()
                .color(Color32::from_rgb(255, 200, 40)),
        );
    }
    ui.add_space(20.0);
    if ui
        .add_sized(
            Vec2::new(ui.available_width(), 68.0),
            Button::new(
                RichText::new("MAIN LAGI")
                    .size(24.0)
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(ACCENT),
        )
        .clicked()
    {
        action = Action::PlayAgain(r.mode);
    }
    ui.add_space(10.0);
    if ui
        .add_sized(
            Vec2::new(ui.available_width(), 68.0),
            Button::new(RichText::new("MENU UTAMA").size(24.0).strong()),
        )
        .clicked()
    {
        action = Action::ToMenu;
    }
    // Void di bawah tombol pada v2.3: kartu hasil + tombol hanya ~380pt.
    // Strip maskot didorong ke dasar layar seperti di layar kuis.
    ui.add_space(12.0);
    push_to_bottom(ctx, ui, 52.0);
    mascot_strip(ctx, ui, textures, 40.0);
    ui.add_space(8.0);
    action
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: android_activity::AndroidApp) {
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    log::info!("Hozin Kuis v2.5 (Rust) mulai");
    let data_path = app
        .internal_data_path()
        .unwrap_or_else(|| PathBuf::from("."));
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Hozin Kuis",
        options,
        Box::new(move |cc| Ok(Box::new(KuisApp::new(cc, data_path)))),
    );
}
