pub struct Notification {
    pub title: String,
    pub message: String,
    pub to: Option<String>,
    pub from: Option<String>,
    pub level: String,
}

pub struct LevelStyle {
    pub label: &'static str,
    pub emoji: &'static str,
    pub color: u64,
    pub hex_color: &'static str,
}

pub fn level_style(level: &str) -> LevelStyle {
    match level {
        "error" => LevelStyle {
            label: "ERROR",
            emoji: "❌",
            color: 0xE74C3C,
            hex_color: "#E74C3C",
        },
        "info" => LevelStyle {
            label: "INFO",
            emoji: "✅",
            color: 0x2ECC71,
            hex_color: "#2ECC71",
        },
        "warn" => LevelStyle {
            label: "WARN",
            emoji: "⚠️",
            color: 0xF1C40F,
            hex_color: "#F1C40F",
        },
        _ => LevelStyle {
            label: "WARN",
            emoji: "⚠️",
            color: 0xF1C40F,
            hex_color: "#F1C40F",
        },
    }
}
