use std::sync::{Arc, Mutex};
use libadwaita::prelude::*;
use libadwaita::{
    Application, ApplicationWindow, HeaderBar, NavigationSplitView, NavigationPage,
    Avatar, ToastOverlay, Toast, StatusPage
};
use gtk4::{
    Box, Orientation, Label, Entry, Button, ScrolledWindow, ListBox, ListBoxRow,
    Align, PolicyType, Separator
};
use serde::{Deserialize, Serialize};

const APP_ID: &str = "com.github.subwatg.linelinux";
const ENGINE_URL: &str = "http://127.0.0.1:4884";

#[derive(Clone, Debug, Deserialize, Serialize)]
struct UserProfile {
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "statusMessage")]
    status_message: String,
    mid: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ChatItem {
    mid: String,
    name: String,
    #[serde(rename = "lastMessage")]
    last_message: String,
}

#[derive(Serialize)]
struct SendMessageReq {
    text: String,
}

fn fetch_profile() -> Option<UserProfile> {
    ureq::get(&format!("{}/api/me", ENGINE_URL))
        .timeout(std::time::Duration::from_millis(800))
        .call()
        .ok()?
        .into_json::<UserProfile>()
        .ok()
}

fn fetch_chats() -> Vec<ChatItem> {
    ureq::get(&format!("{}/api/chats", ENGINE_URL))
        .timeout(std::time::Duration::from_millis(1500))
        .call()
        .ok()
        .and_then(|r| r.into_json::<Vec<ChatItem>>().ok())
        .unwrap_or_default()
}

fn send_chat_message(mid: &str, text: &str) -> bool {
    let req = SendMessageReq { text: text.to_string() };
    ureq::post(&format!("{}/api/chats/{}/send", ENGINE_URL, mid))
        .timeout(std::time::Duration::from_millis(3000))
        .send_json(req)
        .is_ok()
}

fn main() {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let toast_overlay = ToastOverlay::new();

    // Check engine connection
    let profile = fetch_profile();
    let real_chats = if profile.is_some() { fetch_chats() } else { Vec::new() };

    let selected_chat_mid = Arc::new(Mutex::new(
        real_chats.first().map(|c| c.mid.clone()).unwrap_or_default()
    ));

    // -------------------------------------------------------------
    // 1. Sidebar (Chat List & User Profile)
    // -------------------------------------------------------------
    let user_name = profile.as_ref().map(|p| p.display_name.as_str()).unwrap_or("LINE");
    let sidebar_title = Label::builder().label(user_name).css_classes(["title-3"]).build();
    let sidebar_header = HeaderBar::builder()
        .title_widget(&sidebar_title)
        .build();

    let search_entry = Entry::builder()
        .placeholder_text("ค้นหาแชตหรือเพื่อน...")
        .margin_start(12)
        .margin_end(12)
        .margin_top(8)
        .margin_bottom(8)
        .build();

    let list_box = ListBox::builder()
        .css_classes(["navigation-sidebar"])
        .selection_mode(gtk4::SelectionMode::Single)
        .build();

    let sidebar_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .vexpand(true)
        .build();

    let sidebar_box = Box::new(Orientation::Vertical, 0);
    sidebar_box.append(&sidebar_header);

    if profile.is_none() {
        // Show status page instructing how to connect real LINE
        let status_page = StatusPage::builder()
            .icon_name("network-offline-symbolic")
            .title("ยังไม่ได้เชื่อมต่อบัญชี LINE จริง")
            .description("กรุณารันคำสั่ง ./run_engine.sh ในเทอร์มินัลเพื่อสแกน QR Code เข้าสู่ระบบ")
            .build();
        sidebar_box.append(&status_page);
    } else {
        sidebar_box.append(&search_entry);
        sidebar_box.append(&Separator::new(Orientation::Horizontal));

        for c in &real_chats {
            let row = ListBoxRow::new();
            let hbox = Box::new(Orientation::Horizontal, 12);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);
            hbox.set_margin_top(10);
            hbox.set_margin_bottom(10);

            let avatar = Avatar::builder()
                .text(&c.name)
                .size(42)
                .show_initials(true)
                .build();
            hbox.append(&avatar);

            let text_box = Box::new(Orientation::Vertical, 4);
            text_box.set_hexpand(true);

            let name_lbl = Label::builder()
                .label(&c.name)
                .css_classes(["heading"])
                .halign(Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .hexpand(true)
                .build();
            text_box.append(&name_lbl);

            let msg_lbl = Label::builder()
                .label(&c.last_message)
                .css_classes(["body", "dim-label"])
                .halign(Align::Start)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            text_box.append(&msg_lbl);

            hbox.append(&text_box);
            row.set_child(Some(&hbox));
            list_box.append(&row);
        }

        sidebar_scrolled.set_child(Some(&list_box));
        sidebar_box.append(&sidebar_scrolled);
    }

    let sidebar_page = NavigationPage::builder()
        .title("Chats")
        .child(&sidebar_box)
        .build();

    // -------------------------------------------------------------
    // 2. Content Area (Active Chat & VoIP Call)
    // -------------------------------------------------------------
    let content_header = HeaderBar::new();

    let active_chat_name = real_chats.first().map(|c| c.name.clone()).unwrap_or_else(|| "ข้อความ".to_string());
    let chat_title_lbl = Label::builder()
        .label(&active_chat_name)
        .css_classes(["heading"])
        .build();
    content_header.set_title_widget(Some(&chat_title_lbl));

    // Call Button (LINE Planet WebRTC)
    let call_btn = Button::builder()
        .icon_name("call-start-symbolic")
        .tooltip_text("LINE Planet Voice Call")
        .css_classes(["flat"])
        .build();

    let video_btn = Button::builder()
        .icon_name("camera-web-symbolic")
        .tooltip_text("LINE Planet Video Call")
        .css_classes(["flat"])
        .build();

    content_header.pack_end(&video_btn);
    content_header.pack_end(&call_btn);

    let toast_call = toast_overlay.clone();
    call_btn.connect_clicked(move |_| {
        let t = Toast::new("📞 เริ่มต้นสายโทรเสียง LINE Planet WebRTC...");
        t.set_timeout(3);
        toast_call.add_toast(t);
    });

    let toast_vid = toast_overlay.clone();
    video_btn.connect_clicked(move |_| {
        let t = Toast::new("📹 กำลังเชื่อมต่อสัญญาณกล้องวิดีโอคอล...");
        t.set_timeout(3);
        toast_vid.add_toast(t);
    });

    // Message History View
    let msg_container = Box::new(Orientation::Vertical, 14);
    msg_container.set_margin_start(18);
    msg_container.set_margin_end(18);
    msg_container.set_margin_top(18);
    msg_container.set_margin_bottom(18);
    msg_container.set_vexpand(true);

    let add_bubble = |container: &Box, text: &str, is_me: bool, time: &str| {
        let bubble_box = Box::new(Orientation::Horizontal, 6);
        let bubble = Box::new(Orientation::Vertical, 4);

        bubble_box.set_halign(if is_me { Align::End } else { Align::Start });
        bubble.set_css_classes(&["card"]);

        bubble.set_margin_start(8);
        bubble.set_margin_end(8);
        bubble.set_margin_top(6);
        bubble.set_margin_bottom(6);

        let lbl = Label::builder()
            .label(text)
            .wrap(true)
            .max_width_chars(50)
            .halign(if is_me { Align::End } else { Align::Start })
            .build();

        let time_lbl = Label::builder()
            .label(time)
            .css_classes(["caption", "dim-label"])
            .halign(if is_me { Align::End } else { Align::Start })
            .build();

        bubble.append(&lbl);
        bubble.append(&time_lbl);
        bubble_box.append(&bubble);
        container.append(&bubble_box);
    };

    if profile.is_some() {
        add_bubble(&msg_container, "เชื่อมต่อบัญชีจริงสำเร็จ สามารถพิมพ์ส่งข้อความจริงเข้าแชต LINE ได้เลยครับ", false, "ระบบ");
    }

    let msg_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&msg_container)
        .vexpand(true)
        .build();

    // Message Composer (Bottom Bar)
    let composer_box = Box::new(Orientation::Horizontal, 8);
    composer_box.set_margin_start(16);
    composer_box.set_margin_end(16);
    composer_box.set_margin_top(12);
    composer_box.set_margin_bottom(16);

    let attach_btn = Button::builder()
        .icon_name("mail-attachment-symbolic")
        .css_classes(["flat", "circular"])
        .build();

    let entry = Entry::builder()
        .placeholder_text("พิมพ์ข้อความส่งเข้า LINE...")
        .hexpand(true)
        .build();

    let send_btn = Button::builder()
        .icon_name("document-send-symbolic")
        .css_classes(["suggested-action", "circular"])
        .build();

    composer_box.append(&attach_btn);
    composer_box.append(&entry);
    composer_box.append(&send_btn);

    // Send action: Wire directly to real LINE API!
    let entry_clone = entry.clone();
    let msg_cont_clone = msg_container.clone();
    let mid_clone = Arc::clone(&selected_chat_mid);
    let toast_send = toast_overlay.clone();

    send_btn.connect_clicked(move |_| {
        let text = entry_clone.text().to_string();
        if !text.trim().is_empty() {
            let mid = mid_clone.lock().unwrap().clone();
            if !mid.is_empty() {
                let success = send_chat_message(&mid, &text);
                if success {
                    add_bubble(&msg_cont_clone, &text, true, "ส่งแล้ว");
                    entry_clone.set_text("");
                } else {
                    toast_send.add_toast(Toast::new("❌ ส่งข้อความไม่สำเร็จ"));
                }
            } else {
                toast_send.add_toast(Toast::new("⚠️ กรุณาเลือกห้องแชตก่อนส่ง"));
            }
        }
    });

    let content_box = Box::new(Orientation::Vertical, 0);
    content_box.append(&content_header);
    content_box.append(&msg_scrolled);
    content_box.append(&Separator::new(Orientation::Horizontal));
    content_box.append(&composer_box);

    let content_page = NavigationPage::builder()
        .title("Chat")
        .child(&content_box)
        .build();

    // -------------------------------------------------------------
    // 3. Navigation Split View & Main Window
    // -------------------------------------------------------------
    let split_view = NavigationSplitView::builder()
        .sidebar(&sidebar_page)
        .content(&content_page)
        .min_sidebar_width(320.0)
        .max_sidebar_width(400.0)
        .build();

    toast_overlay.set_child(Some(&split_view));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("LINE")
        .default_width(1000)
        .default_height(680)
        .content(&toast_overlay)
        .build();

    window.present();
}
