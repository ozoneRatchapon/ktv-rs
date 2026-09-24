use dioxus::prelude::*;
use crate::types::AppSettings;

#[component]
pub fn Settings(
    settings: Signal<AppSettings>,
) -> Element {
    let current_settings = settings();

    rsx! {
        div { class: "settings-container",
            div { class: "settings-card",
                div { class: "settings-header",
                    span { class: "settings-icon", "⚙️" }
                    div {
                        h3 { "การตั้งค่าระบบคาราโอเกะ (KTV Settings)" }
                        p { "ปรับแต่งพฤติกรรมของเครื่องเล่นและระบบข้ามอินโทรอัตโนมัติ" }
                    }
                }

                // Auto-skip intro toggle
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "⚡ ข้ามอินโทรแพลตฟอร์มอัตโนมัติ (Auto Skip Platform Intro)" }
                        div { class: "setting-sub", "ข้ามโลโก้และสปอตเปิดหัวเพลงของ YouTube Karaoke ทันทีที่เริ่มเล่น" }
                    }
                    button {
                        class: if current_settings.auto_skip_intro { "toggle-btn active" } else { "toggle-btn" },
                        onclick: move |_| {
                            let mut s = settings();
                            s.auto_skip_intro = !s.auto_skip_intro;
                            settings.set(s);
                        },
                        if current_settings.auto_skip_intro { "เปิดใช้งาน (ON)" } else { "ปิด (OFF)" }
                    }
                }

                // Default Intro Skip Offset
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "⏱️ เวลาข้ามอินโทรเริ่มต้น (Default Skip Duration)" }
                        div { class: "setting-sub", "สำหรับเพลงของ GMM Karaoke โลโก้เปิดหัวเพลงมาตรฐานมีความยาวประมาณ 13 วินาที" }
                    }
                    div { class: "setting-input-group",
                        input {
                            class: "number-input",
                            r#type: "number",
                            min: "0",
                            max: "40",
                            value: "{current_settings.default_intro_skip_secs}",
                            oninput: move |evt| {
                                if let Ok(val) = evt.value().parse::<u32>() {
                                    let mut s = settings();
                                    s.default_intro_skip_secs = val;
                                    settings.set(s);
                                }
                            }
                        }
                        span { class: "input-unit", "วินาที" }
                    }
                }

                // Room Name
                div { class: "setting-row",
                    div { class: "setting-desc",
                        div { class: "setting-title", "🏷️ ชื่อห้องคาราโอเกะ (Room Name)" }
                        div { class: "setting-sub", "กำหนดชื่อห้องสำหรับแสดงที่หน้าจอหลัก" }
                    }
                    input {
                        class: "text-input",
                        r#type: "text",
                        value: "{current_settings.room_name}",
                        oninput: move |evt| {
                            let mut s = settings();
                            s.room_name = evt.value();
                            settings.set(s);
                        }
                    }
                }
            }
        }
    }
}
