# Spesifikasi YAN v1.0

**Yet Another Notation**

Format pertukaran data yang dapat dibaca manusia, dapat ditulis manusia, dan dapat dibaca mesin.

---

## 1. Pendahuluan

YAN adalah format pertukaran data yang ringan, berbasis teks, dan independen bahasa pemrograman. YAN dirancang agar mudah dibaca dan ditulis oleh manusia, sambil tetap sederhana untuk diparse dan dibuat oleh mesin.

YAN independen dari bahasa pemrograman. YAN tidak diturunkan dari bahasa pemrograman tertentu.

### 1.1 Tujuan Desain

- **Dapat dibaca manusia**: Tanpa tanda kutip, kurung, atau escaping yang tidak perlu.
- **Dapat ditulis manusia**: Sintaks intuitif dengan komentar dan format yang fleksibel.
- **Dapat dibaca mesin**: Tata bahasa yang tidak ambigu dengan spesifikasi formal.
- **Independen bahasa**: Tidak terikat pada bahasa pemrograman tertentu.

### 1.2 Konvensi dalam Dokumen Ini

Kata kunci "HARUS" (MUST), "TIDAK BOLEH" (MUST NOT), "DIBUTUHKAN" (REQUIRED), "AKAN" (SHALL), "TIDAK AKAN" (SHALL NOT), "SEHARUSNYA" (SHOULD), "SEHARUSNYA TIDAK" (SHOULD NOT), "DIREKOMENDASIKAN" (RECOMMENDED), "BOLEH" (MAY), dan "OPSIONAL" (OPTIONAL) dalam dokumen ini diinterpretasikan seperti yang dijelaskan dalam RFC 2119.

---

## 2. Tata Bahasa (Grammar)

### 2.1 Encoding

Dokumen YAN HARUS di-encode dalam UTF-8. Encoding lain TIDAK DIREKOMENDASIKAN dan BOLEH ditolak oleh parser.

### 2.2 Whitespace

Whitespace didefinisikan sebagai salah satu dari Unicode code point berikut:

- `U+0020` SPACE (spasi)
- `U+0009` CHARACTER TABULATION (tab)
- `U+000A` LINE FEED (LF)
- `U+000D` CARRIAGE RETURN (CR)

Parser YAN HARUS menormalisasi `CRLF` (`U+000D U+000A`) dan `CR` mandiri (`U+000D`) menjadi `LF` (`U+000A`) sebelum parsing.

### 2.3 Indentasi

Indentasi digunakan untuk menandai objek bersarang (nested) pada level blok. Blok indentasi HARUS menggunakan spasi atau tab, tetapi TIDAK BOLEH mencampur keduanya dalam satu dokumen.

Indentasi yang DIREKOMENDASIKAN adalah 2 spasi per level.

### 2.4 Tata Bahasa ABNF

```abnf
yan-document    = *ws *(comment / pair / ws) *ws

pair            = key ws ":" ws value

key             = 1*key-char
key-char        = ALPHA / DIGIT / "_" / "-"

value           = string
                / number
                / boolean
                / null-value
                / array
                / object-block
                / object-inline
                / type-hint
                / unquoted-string

string          = DQUOTE *char DQUOTE
char            = unescaped / escape
unescaped       = %x20-21 / %x23-5B / %x5D-10FFFF
escape          = "\\" (DQUOTE / "\\" / "b" / "f" / "n" / "r" / "t" / "u" HEXDIG HEXDIG HEXDIG HEXDIG)

unquoted-string = 1*unquoted-char
unquoted-char   = %x21-3A / %x3C-5B / %x5D-7E
                ; printable ASCII excluding : ; { } [ ] @ " \ dan whitespace

number          = ["-"] int [frac] [exp]
int             = "0" / ("1"-"9" *DIGIT)
frac            = "." 1*DIGIT
exp             = ("e" / "E") ["-" / "+"] 1*DIGIT

boolean         = "true" / "false" / "yes" / "no" / "on" / "off"

null-value      = "null" / "nil" / "_" / "~"

array           = value *(ws ";" ws value)

object-block    = *(pair ws)
                ; pasangan yang dipisahkan oleh baris baru dengan indentasi yang meningkat

object-inline   = "{" ws *(pair [ws (";" / ",") ws]) ws "}"

type-hint       = "@" type-name ws value
type-name       = 1*(ALPHA / DIGIT / "_" / "-")

comment         = line-comment / block-comment
line-comment    = "#" *VCHAR
block-comment   = "/*" *(*VCHAR / ws) "*/"

ws              = *(SP / HTAB / LF)
```

---

## 3. Tipe Data

### 3.1 String

String adalah urutan dari nol atau lebih karakter Unicode.

String BOLEH dikutip dengan tanda kutip ganda (`"`). String yang dikutip HARUS menggunakan `"` untuk escaping tanda kutip ganda dan `\\` untuk escaping backslash.

String tanpa kutip diperbolehkan ketika nilai tidak mengandung whitespace, tidak mengandung karakter spesial, dan tidak bertabrakan dengan tipe nilai lain. String tanpa kutip TIDAK BOLEH diawali dengan `@`, `{`, `[`, `"`, `-` (kecuali angka negatif), atau digit (kecuali angka).

```yan
name: Budi
greeting: "Hello, World!"
path: "/usr/local/bin"
```

### 3.2 Number (Angka)

Angka direpresentasikan dalam basis 10. Angka BOLEH berupa integer atau floating-point.

```yan
age: 25
pi: 3.14159
negative: -42
scientific: 1.23e-4
```

### 3.3 Boolean

Nilai boolean merepresentasikan keadaan benar/salah. Literal berikut dikenali:

| Nilai | Arti |
|-------|------|
| `true` | Benar |
| `false` | Salah |
| `yes` | Benar |
| `no` | Salah |
| `on` | Benar |
| `off` | Salah |

```yan
debug: off
ssl: yes
active: true
```

### 3.4 Null (Nol)

Nilai null merepresentasikan tidak adanya nilai. Literal berikut dikenali:

| Nilai | Arti |
|-------|------|
| `null` | Null |
| `nil` | Null |
| `_` | Null |
| `~` | Null |

```yan
data: null
author: _
optional: ~
```

### 3.5 Array (Larik)

Array adalah daftar terurut dari nilai-nilai yang dipisahkan oleh titik koma (`;`).

```yan
hobbies: makan; tidur; ngoding
numbers: 1; 2; 3; 5; 8
```

Array BOLEH mengandung nilai dengan tipe berbeda:

```yan
mixed: hello; 42; true; null
```

### 3.6 Object (Objek)

Objek adalah koleksi tidak terurut dari pasangan key-value. Objek dapat direpresentasikan dalam dua bentuk:

#### Bentuk Blok (Indentasi)

```yan
person:
  name: Budi
  age: 25
  address:
    city: Jakarta
    country: Indonesia
```

#### Bentuk Inline (Kurung Kurawal)

```yan
person: {name: Budi; age: 25}
```

Objek inline BOLEH menggunakan titik koma (`;`) atau koma (`,`) sebagai pemisah. Pemisah di akhir (trailing separator) diperbolehkan dan HARUS diabaikan oleh parser.

```yan
config: {host: localhost; port: 8080;}
```

### 3.7 Type Hints (Petunjuk Tipe)

Type hints memberikan informasi tipe eksplisit untuk nilai. Diawali dengan `@` diikuti nama tipe dan nilai.

```yan
created: @datetime 2026-06-27T13:00:00Z
birthdate: @date 2000-05-15
hash: @hex a1b2c3d4
```

#### Type Hints Inti (Core Type Hints)

| Hint | Deskripsi | Contoh Output |
|------|-----------|---------------|
| `@int` | Integer | `@int 42` |
| `@float` | Floating-point | `@float 3.14` |
| `@date` | Tanggal (ISO 8601) | `@date 2026-06-27` |
| `@datetime` | Tanggal dan waktu (ISO 8601) | `@datetime 2026-06-27T13:00:00Z` |
| `@hex` | String heksadesimal | `@hex deadbeef` |
| `@base64` | String Base64 | `@base64 SGVsbG8=` |
| `@uuid` | UUID | `@uuid 550e8400-e29b-41d4-a716-446655440000` |
| `@url` | URL | `@url https://example.com` |
| `@regex` | Ekspresi reguler | `@regex [a-z]+` |
| `@bool` | Boolean eksplisit | `@bool yes` |

Parser SEBAIKNYA mendukung semua type hints inti. Parser BOLEH mendukung type hints tambahan.

---

## 4. Komentar

### 4.1 Komentar Baris

Komentar baris dimulai dengan `#` dan berlanjut sampai akhir baris.

```yan
# Ini adalah komentar
name: Budi  # Ini juga komentar
```

### 4.2 Komentar Blok

Komentar blok dimulai dengan `/*` dan diakhiri dengan `*/`. Komentar blok BOLEH melintasi beberapa baris. Komentar blok TIDAK BOLEH bersarang (nested).

```yan
/* Ini adalah
   komentar multi-baris */
name: Budi
```

### 4.3 Penanganan Komentar

Komentar HARUS diperlakukan sebagai whitespace oleh parser. Komentar TIDAK BOLEH muncul dalam output yang diparse.

---

## 5. Struktur Dokumen

### 5.1 Dokumen Tunggal

File YAN berisi satu dokumen yang terdiri dari pasangan key-value.

```yan
app:
  name: MyApp
  version: 1.0.0

database:
  host: localhost
  port: 5432
```

### 5.2 Multi-Dokumen (Opsional)

Beberapa dokumen BOLEH dipisahkan oleh penanda dokumen `---` pada barisnya sendiri.

```yan
---
name: Dokumen Satu
value: 1
---
name: Dokumen Dua
value: 2
```

Dukungan multi-dokumen bersifat OPSIONAL. Parser yang tidak mendukungnya SEBAIKNYA memperlakukan `---` sebagai string tanpa kutip biasa.

---

## 6. Penanganan Error

### 6.1 Parse Error

Parser YAN HARUS menghasilkan pesan error yang jelas dan dapat ditindaklanjuti. Pesan error SEBAIKNYA mencakup:

- Nomor baris
- Nomor kolom (jika berlaku)
- Deskripsi error
- Token yang diharapkan vs. token yang ditemukan

### 6.2 Error Umum

| Error | Deskripsi |
|-------|-----------|
| `UNEXPECTED_TOKEN` | Karakter atau token yang tidak sesuai dengan tata bahasa |
| `UNCLOSED_STRING` | String yang dikutip tanpa penutup `"` |
| `UNCLOSED_BLOCK` | Objek inline `{` tanpa pasangan `}` |
| `INDENTATION_ERROR` | Tab dan spasi tercampur, atau indentasi tidak konsisten |
| `DUPLICATE_KEY` | Key yang sama muncul dua kali dalam satu objek |
| `INVALID_TYPE_HINT` | Type hint yang tidak dikenal atau formatnya salah |

### 6.3 Key Duplikat

Ketika key muncul lebih dari sekali dalam objek yang sama, parser SEBAIKNYA menggunakan kemunculan terakhir. Alternatifnya, parser BOLEH melempar error `DUPLICATE_KEY`.

---

## 7. Tingkat Kepatuhan (Compliance Levels)

### Level 1: Dasar

Parser HARUS mendukung:
- Pasangan key-value
- String (dikutip dan tanpa kutip)
- Angka (integer dan float)
- Boolean (`true`, `false`)
- Null (`null`)
- Array dengan pemisah titik koma
- Objek blok (indentasi)
- Objek inline (kurung kurawal)

### Level 2: Lengkap

Parser HARUS mendukung semua Level 1, ditambah:
- Semua alias boolean (`yes`, `no`, `on`, `off`)
- Semua alias null (`nil`, `_`, `~`)
- Komentar (`//` dan `/* */`)
- Semua type hints inti (`@int`, `@float`, `@date`, `@datetime`, `@hex`, `@base64`, `@uuid`, `@url`, `@regex`, `@bool`)
- Pemisah di akhir (trailing separators) pada objek inline dan array

### Level 3: Lanjutan (Opsional)

Parser BOLEH mendukung:
- File multi-dokumen (pemisah `---`)
- Validasi skema (schema validation)
- Parse streaming (YANL)
- Type hints kustom

---

## 8. Media Type

Media type yang DIREKOMENDASIKAN untuk dokumen YAN adalah:

```
application/yan
```

Ekstensi file yang DIREKOMENDASIKAN adalah:

```
.yan
```

Untuk YAN Lines (dokumen satu-baris), media type yang DIREKOMENDASIKAN adalah:

```
application/yanl
```

Ekstensi file yang DIREKOMENDASIKAN adalah:

```
.yanl
```

---

## 9. Perbandingan dengan Format Lain

| Fitur | JSON | YAML | TOML | YAN |
|-------|------|------|------|-----|
| Kutip pada key | Diperlukan | Opsional | Opsional | Tidak diperlukan |
| Komentar | Tidak | Ya | Ya | Ya |
| Trailing commas | Tidak | Ya | Ya | Ya |
| Type hints | Tidak | Tidak | Tidak | Ya |
| Sintaks array | `[a, b]` | `- a` | `[a, b]` | `a; b` |
| Objek bersarang | `{}` | Indentasi | `[table]` | Indentasi + `{}` |
| Dapat dibaca manusia | Sedang | Tinggi | Tinggi | Tinggi |
| Dapat dibaca mesin | Tinggi | Sedang | Tinggi | Tinggi |

---

## 10. Contoh

### 10.1 Konfigurasi Sederhana

```yan
// Konfigurasi server
server:
  host: 0.0.0.0
  port: @int 8080
  debug: off

// Pengaturan database
database:
  driver: postgresql
  host: localhost
  port: 5432
  username: admin
  password: "secret123!"
  pool: {min: 5; max: 20;}
```

### 10.2 Dokumen Kompleks

```yan
/*
 * Dokumen profil pengguna
 * Versi: 1.0
 */
user:
  id: @uuid 550e8400-e29b-41d4-a716-446655440000
  name: Budi Santoso
  email: budi@example.com
  active: yes
  role: admin

  profile:
    bio: "Software engineer dari Jakarta"
    avatar: @url https://cdn.example.com/avatars/budi.png
    joined: @datetime 2020-01-15T08:30:00Z

  preferences:
    theme: dark
    notifications: {email: yes; push: no; sms: _}

  tags: developer; backend; golang; yan
```

### 10.3 YANL (Satu Baris)

```yanl
// access.log.yanl
ts: @datetime 2026-06-27T12:00:00Z; method: GET; path: /api/users; status: 200; duration_ms: 45
ts: @datetime 2026-06-27T12:00:01Z; method: POST; path: /api/login; status: 401; error: "Invalid credentials"
```

---

## 11. Pertimbangan Keamanan

Parser YAN SEBAIKNYA berhati-hati saat mengevaluasi type hints, terutama `@regex` dan `@url`, untuk menghindari kerentanan keamanan seperti ReDoS (Regular Expression Denial of Service) atau SSRF (Server-Side Request Forgery).

Parser TIDAK BOLEH mengeksekusi kode arbitrer yang tertanam dalam dokumen YAN.

---

## 12. Referensi

- RFC 2119: Key words for use in RFCs to Indicate Requirement Levels
- RFC 8259: The JavaScript Object Notation (JSON) Data Interchange Format
- ECMA-404: The JSON Data Interchange Format
- ISO/IEC 21778:2017: Information technology — The JSON data interchange syntax

---

## 13. Penulis & Kontributor

YAN dirancang oleh [kontributor].

Untuk versi terbaru dari spesifikasi ini, kunjungi:
https://github.com/yan-notation/yan-spec

---

*Spesifikasi ini dirilis di bawah lisensi CC0 1.0 Universal.*
