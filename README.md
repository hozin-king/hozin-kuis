# Hozin Kuis

Game kuis Android, ditulis **penuh dalam Rust** (cargo-apk + android-activity + egui).
Tanpa Kotlin/Java di aplikasi, tanpa WebView, **100% offline**.

## Mode

- **Trivia (Bahasa Indonesia)** — 10 soal pilihan ganda dari bank soal lokal
  (`assets/questions.json`, 120 soal: 6 kategori × 20 — Umum, Sains, Sejarah,
  Olahraga, Film, Komputer; tiap kategori 6 mudah + 7 sedang + 7 sulit).
- **Tebak Pokemon** — 10 ronde, 151 Pokemon Gen 1. Data (nama, tipe Bahasa
  Indonesia, tinggi, berat) dari PokeAPI; sprite PNG dibundel di
  `assets/pokemon/<id>.png`. Siluet tampil hitam pekat di atas **lingkaran
  latar terang** (perbaikan bug v1.0: siluet hitam di atas hitam).

Tiap soal: timer 30 detik (habis = salah), skor +10 per jawaban benar, streak,
skor terbaik per mode tersimpan lokal (`best.txt` di internal storage).
Tanpa iklan, tanpa IAP, tanpa login, tanpa internet.

## Build

APK hanya dibangun di GitHub Actions. Rilis otomatis saat push ke `main`.

- Package: `com.hozinking.kuis`
- minSdk 26, targetSdk 34
- Target: `aarch64-linux-android` (arm64-v8a)
- Signing: keystore debug persisten (secret `DEBUG_KEYSTORE_BASE64`,
  `CARGO_APK_RELEASE_KEYSTORE` + password `android` saat build)
- Versi: Cargo `2.3.0` → versionName `2.3.0`, versionCode dari semver
  (cargo-apk; tidak bisa di-override di TOML). Rilis GitHub: tag `v2.3`.

## Riwayat

- **v1.0 (code 1)** — Kotlin, soal Trivia dari Open Trivia DB (Inggris) +
  Tebak Pokemon dari PokeAPI (butuh internet).
- **v2.0 (code 2)** — tulis ulang penuh Rust; soal Indonesia + offline penuh;
  siluet diperbaiki (latar terang).
- **v2.1** — polish UI saja: palet aksen, tombol utama, sudut membulat,
  kartu skor; fungsi tidak berubah.
- **v2.2** — redesain UI benar-benar: latar gradien, layout isi layar penuh,
  banner judul, maskot sprite Pokemon, chip berwarna per kategori/kesulitan,
  tombol Trivia biru / Pokemon emas, kartu soal, jawaban A-D, indikator
  progres; fungsi tidak berubah.
- **v2.3** — perbaikan menu v2.2 yang rusak di HP (kartu maskot mengembang
  setinggi layar dan mendorong semua elemen menu ke luar viewport):
  layout pakai ScrollArea vertikal + jarak tetap + pola kartu yang tidak
  mengembang; bahasa visual v2.2 dipertahankan. BARU: ikon launcher proper
  (Pikachu di atas gradien aplikasi, mipmap 48-192px; dihasilkan
  `tools/gen_icon.py` dari sprite yang sudah dibundel).

## Data

- `assets/questions.json` — sumber bank soal (dibaca `tools/gen_data.py` ->
  `src/questions_data.rs`).
- `src/pokemon_data.rs` — data 151 Pokemon Gen 1 (dibuat `tools/gen_data.py`
  dari PokeAPI; tipe diterjemahkan ke Bahasa Indonesia).
- `build.rs` — membundel sprite `assets/pokemon/*.png` ke binary via
  `include_bytes!` (dicek: tepat 151 file PNG valid).
