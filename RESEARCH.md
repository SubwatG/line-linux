# LINE Native Linux Client (Research & PoC)

โปรเจกต์สำรวจและสร้าง **LINE Client แบบ Native บน Linux** (สำหรับ CachyOS / Ubuntu / Arch) เพื่อแก้ปัญหาข้อจำกัดของ LINE Chrome Extension โดยเฉพาะฟังก์ชัน **"LINE Call (VoIP เสียงและวิดีโอ)"** และการกินทรัพยากรสูง

---

## 1. ผลการ Reverse Engineer สถาปัตยกรรม LINE Protocol

จากการสำรวจและทดสอบโค้ดระดับ Protocol จากโปรเจกต์ `linejs`, `linethrift`, และ `line-protocol`:

### ก. ระบบรับ-ส่งข้อความ (Talk Service)
- **Wire Protocol:** ใช้ **Apache Thrift CompactProtocol (`TCompactProtocol`)** ผ่าน HTTPS/HTTP2 ไปยังเกตเวย์ LEGY เช่น `legy-jp.line-apps.com`
- **การเข้าสู่ระบบ (Authentication):** รองรับระบบ **Secondary Device QR Code Login** (`/ACCT/lgn/sq/v1`)
  - สร้าง Session + สร้าง ECDH Secret สำหรับ E2EE
  - แสดงผล QR Code ให้ผู้ใช้นำแอป LINE บนมือถือมาสแกน
  - ยืนยันตัวตนด้วย PIN Code 4 หลักครั้งแรก โดยไม่ต้องใช้รหัสผ่านหรือ OTP SMS
- **การเข้ารหัสปลายทาง (E2EE / Letter Sealing):**
  - ใช้ **Curve25519 (X25519)** แลกเปลี่ยนกุญแจระหว่างอุปกรณ์
  - เข้ารหัสเนื้อหาด้วย **AES-GCM / AES-CBC** และ SHA-256 HMAC

### ข. ความลับของระบบโทร (LINE Call / VoIP) — *จุดที่ Chrome Extension ทำไม่ได้*
- LINE ใช้แพลตฟอร์ม VoIP ภายในที่เรียกว่า **LINE Planet (PlanetKit)**
- **ความจริงที่ค้นพบ:** บนเว็บ LINE มี SDK ทางการชื่อ **WebPlanetKit (`@line/planet-kit`)** ซึ่งพัฒนาบน **มาตรฐาน WebRTC ล้วน ๆ!**
- **เส้นทางของสายสนทนา:**
  1. `acquireCallRoute (/V4)`: ขอ Route การโทรจาก LINE Server ผ่าน Talk Token
  2. เชื่อมต่อ UDP ไปยัง PLANET Relay
  3. แลกเปลี่ยนสัญญาณควบคุมการโทร (Cassini Call-Control Messages)
  4. ถอดรหัส/สร้างกุญแจมีเดีย **SRTP (`AES_CM_128_HMAC_SHA1_80`)**
  5. ส่งและรับเสียงพูดด้วย **Opus Codec (48 kHz Mono, 20ms Frame)**
- **ผลลัพธ์:** บน Linux เราสามารถคุยสายผ่าน **PipeWire / ALSA** ร่วมกับ WebRTC ได้ 100% โดยไม่ต้องพึ่งพา Windows/macOS!

---

## 2. โครงสร้าง PoC ในเครื่อง (`/home/kitti/Projects/line-linux/`)

```text
/home/kitti/Projects/line-linux/
├── linejs/                  # แกนหลัก Protocol Engine (Apache Thrift + PlanetKit WebRTC)
│   ├── line_poc.ts          # สคริปต์ทดสอบล็อกอิน QR Code, ดึงแชต, และดักฟังสัญญาณโทร
│   └── session_storage.json # จัดเก็บ Token อัตโนมัติหลังสแกนสำเร็จ
├── run_poc.sh               # ตัวรัน PoC สดในคลิกเดียว (วาด QR Code ใน Terminal)
└── RESEARCH.md              # เอกสารสถาปัตยกรรมฉบับนี้
```

---

## 3. วิธีทดสอบ PoC สด (Live Terminal QR Code Login)

เปิดหน้าต่าง Kitty Terminal อีกแท็บหนึ่ง แล้วรัน:

```bash
/home/kitti/Projects/line-linux/run_poc.sh
```

1. หน้าจอ Terminal จะวาดภาพ **QR Code** ขึ้นมาทันที
2. หยิบมือถือ เปิดแอป LINE $\to$ กดไอคอนสแกนคิวอาร์โค้ด
3. สแกนภาพบนหน้าจอ และกดพิมพ์รหัส **PIN 4 หลัก** บนมือถือ
4. ตัวโปรแกรมจะล็อกอินสำเร็จ แสดงชื่อโปรไฟล์ และรายชื่อห้องแชตทั้งหมดทันที!
5. พร้อมดักฟังสัญญาณ **Incoming Call** เมื่อมีคนโทรเข้ามาใน LINE!

---

## 4. แผนการต่อยอดเป็นแอป GUI ด้วย Rust + Tauri (The Next Step)

- **Backend (Rust):**
  - ใช้ Crate `thrift` หรือขับเคลื่อน Engine ผ่าน IPC
  - เชื่อมต่อ `notify-rust` สำหรับป๊อปอัปแจ้งเตือน Native บน GNOME
  - จัดเก็บประวัติแชตใน SQLite ภายในเครื่อง
- **Frontend (Tauri + Svelte):**
  - หน้าต่างแชตแบบ Native น้ำหนักเบา (RAM ~50MB แทนที่จะเป็น 1.5GB ของ Chrome)
  - หน้าจอรับสายโทรเข้า-ออกด้วย WebRTC ผ่านไมค์ Jieli และลำโพงของเครื่อง
