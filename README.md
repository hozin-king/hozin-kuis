# Hozin Kuis

Game kuis Android native (Kotlin, tanpa WebView).

## Mode

- **Trivia** — 10 soal pilihan ganda dari [Open Trivia DB](https://opentdb.com) (gratis, tanpa kunci).
  Kategori: Umum, Film, Sains, Komputer, Olahraga, Sejarah. Tingkat: Mudah / Sedang / Sulit.
- **Tebak Pokemon** — 10 ronde dari [PokeAPI](https://pokeapi.co) (gratis, tanpa kunci).
  Tebak nama Pokemon dari siluet + petunjuk (tipe, tinggi, berat).

Tiap soal: timer 30 detik, skor +10 per jawaban benar, streak, skor terbaik tersimpan lokal.
Bila offline: pesan "butuh internet untuk ambil soal", tanpa crash. Tanpa iklan, tanpa IAP, tanpa login.

## Build

APK hanya dibangun di GitHub Actions (alur temp-publik standar). Rilis otomatis saat push ke `main`.

- Package: `com.hozinking.kuis`
- minSdk 26, targetSdk 34, compileSdk 34
- Signing: keystore debug persisten (secret `DEBUG_KEYSTORE_BASE64`)
