const LIGHT_THEME: &str = include_str!("../../themes/default/layout.css");
const DARK_THEME: &str = include_str!("../../themes/dark/layout.css");

#[test]
fn markdown_dividers_use_the_accessible_violet_theme_token() {
    assert!(LIGHT_THEME.contains("color-mix(in srgb, var(--markhola-violet) 75%, transparent)"));
    assert!(DARK_THEME.contains("color-mix(in srgb, var(--markhola-violet) 85%, transparent)"));
    assert!(LIGHT_THEME.contains("background: linear-gradient("));
    assert!(DARK_THEME.contains("background: linear-gradient("));

    assert!(contrast("#6657E8", "#FAFEFD") >= 3.0);
    assert!(contrast("#6657E8", "#0D1117") >= 3.0);
    assert!(contrast("#8B81ED", "#FAFEFD") >= 3.0);
    assert!(contrast("#594DC9", "#0D1117") >= 3.0);
}

fn contrast(foreground: &str, background: &str) -> f64 {
    let foreground = luminance(foreground);
    let background = luminance(background);
    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
}

fn luminance(hex: &str) -> f64 {
    let value = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap();
    let channels = [
        ((value >> 16) & 0xff) as f64 / 255.0,
        ((value >> 8) & 0xff) as f64 / 255.0,
        (value & 0xff) as f64 / 255.0,
    ];
    let linear = channels.map(|channel| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2]
}
