use libadwaita::prelude::*;
use libadwaita::{
    Application, ApplicationWindow, HeaderBar, NavigationSplitView, NavigationPage,
    Avatar, ToastOverlay, Toast
};
use gtk4::{
    Box, Orientation, Label, Entry, Button, ScrolledWindow, ListBox, ListBoxRow,
    Align, PolicyType, Separator
};

const APP_ID: &str = "com.github.subwatg.linelinux";

#[derive(Clone)]
struct ChatItem {
    name: String,
    last_msg: String,
    time: String,
    mid: String,
}

fn sample_chats() -> Vec<ChatItem> {
    vec![
        ChatItem {
            name: "ภาควิชาวิทยาการคำนวณและเทคโนโลยีฯ".to_string(),
            last_msg: "แจ้งกำหนดการประชุมคณะกรรมการประจำภาควิชา...".to_string(),
            time: "18:30".to_string(),
            mid: "c_dept_01".to_string(),
        },
        ChatItem {
            name: "อาจารย์กิตติพงศ์ (บันทึกส่วนตัว / Keep)".to_string(),
            last_msg: "บันทึก: โครงสร้างคำสั่ง e-Office 2569".to_string(),
            time: "17:15".to_string(),
            mid: "c_keep_01".to_string(),
        },
        ChatItem {
            name: "งานบริการการศึกษา มก. กพส.".to_string(),
            last_msg: "ส่งรายชื่อกรรมการคุมสอบไล่ ภาคต้น 69 เรียบร้อยแล้วค่ะ".to_string(),
            time: "15:42".to_string(),
            mid: "c_academic_01".to_string(),
        },
        ChatItem {
            name: "โครงการวิจัยคณิตศาสตร์".to_string(),
            last_msg: "Lemma L7 ผ่านการทดสอบเรียบร้อยครับอาจารย์".to_string(),
            time: "12:05".to_string(),
            mid: "c_math_01".to_string(),
        },
    ]
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

    // -------------------------------------------------------------
    // 1. Sidebar (Chat List)
    // -------------------------------------------------------------
    let sidebar_header = HeaderBar::builder()
        .title_widget(&Label::builder().label("LINE").css_classes(["title-3"]).build())
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

    let chats = sample_chats();
    for c in &chats {
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

        let title_box = Box::new(Orientation::Horizontal, 8);
        let name_lbl = Label::builder()
            .label(&c.name)
            .css_classes(["heading"])
            .halign(Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .hexpand(true)
            .build();
        let time_lbl = Label::builder()
            .label(&c.time)
            .css_classes(["caption", "dim-label"])
            .build();
        title_box.append(&name_lbl);
        title_box.append(&time_lbl);

        let msg_lbl = Label::builder()
            .label(&c.last_msg)
            .css_classes(["body", "dim-label"])
            .halign(Align::Start)
            .ellipsize(gtk4::pango::EllipsizeMode::End)
            .build();

        text_box.append(&title_box);
        text_box.append(&msg_lbl);
        hbox.append(&text_box);

        row.set_child(Some(&hbox));
        list_box.append(&row);
    }

    let sidebar_scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(PolicyType::Never)
        .vscrollbar_policy(PolicyType::Automatic)
        .child(&list_box)
        .vexpand(true)
        .build();

    let sidebar_box = Box::new(Orientation::Vertical, 0);
    sidebar_box.append(&sidebar_header);
    sidebar_box.append(&search_entry);
    sidebar_box.append(&Separator::new(Orientation::Horizontal));
    sidebar_box.append(&sidebar_scrolled);

    let sidebar_page = NavigationPage::builder()
        .title("Chats")
        .child(&sidebar_box)
        .build();

    // -------------------------------------------------------------
    // 2. Content Area (Active Chat & Call Interface)
    // -------------------------------------------------------------
    let content_header = HeaderBar::new();

    let chat_title_lbl = Label::builder()
        .label("ภาควิชาวิทยาการคำนวณและเทคโนโลยีฯ")
        .css_classes(["heading"])
        .build();
    content_header.set_title_widget(Some(&chat_title_lbl));

    // Call Button (LINE Planet VoIP!)
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

    // Call Button Action
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

    // Sample message bubbles
    let add_bubble = |container: &Box, text: &str, is_me: bool, time: &str| {
        let bubble_box = Box::new(Orientation::Horizontal, 6);
        let bubble = Box::new(Orientation::Vertical, 4);

        if is_me {
            bubble_box.set_halign(Align::End);
            bubble.set_css_classes(&["card"]);
            // Custom inline style can be added via CSS provider
        } else {
            bubble_box.set_halign(Align::Start);
            bubble.set_css_classes(&["card"]);
        }

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

    add_bubble(&msg_container, "เรียนอาจารย์ทุกท่าน แจ้งกำหนดการประชุมกรรมการประจำภาควิชาบ่ายวันพุธนี้ครับ", false, "18:25");
    add_bubble(&msg_container, "รับทราบครับ เดี๋ยวเตรียมเอกสารวาระการประชุมเสนอหลักสูตรไปให้ครับ", true, "18:28");
    add_bubble(&msg_container, "ขอบพระคุณอาจารย์กิตติพงศ์มากครับ", false, "18:30");

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
        .placeholder_text("พิมพ์ข้อความ...")
        .hexpand(true)
        .build();

    let send_btn = Button::builder()
        .icon_name("document-send-symbolic")
        .css_classes(["suggested-action", "circular"])
        .build();

    composer_box.append(&attach_btn);
    composer_box.append(&entry);
    composer_box.append(&send_btn);

    // Send action
    let entry_clone = entry.clone();
    let msg_cont_clone = msg_container.clone();
    send_btn.connect_clicked(move |_| {
        let text = entry_clone.text().to_string();
        if !text.trim().is_empty() {
            add_bubble(&msg_cont_clone, &text, true, "เมื่อสักครู่");
            entry_clone.set_text("");
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
    // 3. Navigation Split View (Responsive Master-Detail)
    // -------------------------------------------------------------
    let split_view = NavigationSplitView::builder()
        .sidebar(&sidebar_page)
        .content(&content_page)
        .min_sidebar_width(320.0)
        .max_sidebar_width(400.0)
        .build();

    toast_overlay.set_child(Some(&split_view));

    // -------------------------------------------------------------
    // 4. Main Window
    // -------------------------------------------------------------
    let window = ApplicationWindow::builder()
        .application(app)
        .title("LINE")
        .default_width(1000)
        .default_height(680)
        .content(&toast_overlay)
        .build();

    window.present();
}
