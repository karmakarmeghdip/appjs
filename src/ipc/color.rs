use serde::{Deserialize, Deserializer, Serialize};

/// Represents a parsed color value
#[derive(Debug, Clone, Serialize)]
pub enum ColorValue {
    /// RGBA color (0-255 per channel)
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    /// Named color string
    Named(String),
}

impl ColorValue {
    /// Parse a color string like "#RRGGBB", "#RRGGBBAA", "rgb(r,g,b)", "rgba(r,g,b,a)",
    /// or named CSS colors.
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.starts_with('#') {
            let hex = &s[1..];
            match hex.len() {
                6 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    Some(ColorValue::Rgba { r, g, b, a: 255 })
                }
                8 => {
                    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                    let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                    Some(ColorValue::Rgba { r, g, b, a })
                }
                _ => None,
            }
        } else if s.starts_with("rgb(") || s.starts_with("rgba(") {
            let inner = s
                .trim_start_matches("rgba(")
                .trim_start_matches("rgb(")
                .trim_end_matches(')');
            let parts: Vec<&str> = inner.split(',').collect();
            if parts.len() >= 3 {
                let r = parts[0].trim().parse::<u8>().ok()?;
                let g = parts[1].trim().parse::<u8>().ok()?;
                let b = parts[2].trim().parse::<u8>().ok()?;
                let a = if parts.len() >= 4 {
                    let af = parts[3].trim().parse::<f32>().ok()?;
                    (af * 255.0) as u8
                } else {
                    255
                };
                Some(ColorValue::Rgba { r, g, b, a })
            } else {
                None
            }
        } else {
            // Try known named colors
            match s.to_lowercase().as_str() {
                "white" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 255,
                }),
                "black" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                "red" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 0,
                    b: 0,
                    a: 255,
                }),
                "green" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 128,
                    b: 0,
                    a: 255,
                }),
                "blue" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 0,
                    b: 255,
                    a: 255,
                }),
                "yellow" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 255,
                    b: 0,
                    a: 255,
                }),
                "cyan" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 255,
                    b: 255,
                    a: 255,
                }),
                "magenta" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 0,
                    b: 255,
                    a: 255,
                }),
                "orange" => Some(ColorValue::Rgba {
                    r: 255,
                    g: 165,
                    b: 0,
                    a: 255,
                }),
                "purple" => Some(ColorValue::Rgba {
                    r: 128,
                    g: 0,
                    b: 128,
                    a: 255,
                }),
                "gray" | "grey" => Some(ColorValue::Rgba {
                    r: 128,
                    g: 128,
                    b: 128,
                    a: 255,
                }),
                "transparent" => Some(ColorValue::Rgba {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                }),
                other => Some(ColorValue::Named(other.to_string())),
            }
        }
    }
}

impl<'de> Deserialize<'de> for ColorValue {
    fn deserialize<D>(deserializer: D) -> Result<ColorValue, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        ColorValue::parse(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("Invalid color string: {}", s)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_parse_hex() {
        if let Some(ColorValue::Rgba { r, g, b, a }) = ColorValue::parse("#ff0080") {
            assert_eq!((r, g, b, a), (255, 0, 128, 255));
        } else {
            panic!("Failed to parse 6-char hex");
        }

        if let Some(ColorValue::Rgba { r, g, b, a }) = ColorValue::parse("#ff008080") {
            assert_eq!((r, g, b, a), (255, 0, 128, 128));
        } else {
            panic!("Failed to parse 8-char hex");
        }
    }

    #[test]
    fn test_color_parse_rgb() {
        if let Some(ColorValue::Rgba { r, g, b, a }) = ColorValue::parse("rgb(10, 20, 30)") {
            assert_eq!((r, g, b, a), (10, 20, 30, 255));
        } else {
            panic!("Failed to parse rgb");
        }

        if let Some(ColorValue::Rgba { r, g, b, a }) = ColorValue::parse("rgba(10, 20, 30, 0.5)") {
            assert_eq!((r, g, b, a), (10, 20, 30, 127)); // 0.5 * 255 = 127.5 -> 127
        } else {
            panic!("Failed to parse rgba");
        }
    }

    #[test]
    fn test_color_parse_named() {
        if let Some(ColorValue::Rgba { r, g, b, a }) = ColorValue::parse("red") {
            assert_eq!((r, g, b, a), (255, 0, 0, 255));
        } else {
            panic!("Failed to parse named color 'red'");
        }

        if let Some(ColorValue::Named(name)) = ColorValue::parse("papayawhip") {
            assert_eq!(name, "papayawhip");
        } else {
            panic!("Failed to parse unknown named color");
        }
    }

    #[test]
    fn test_color_deserialize() {
        let json = "\"#00ff00\"";
        let color: ColorValue = serde_json::from_str(json).unwrap();
        if let ColorValue::Rgba { r, g, b, a } = color {
            assert_eq!((r, g, b, a), (0, 255, 0, 255));
        } else {
            panic!("Deserialization failed");
        }
    }
}
