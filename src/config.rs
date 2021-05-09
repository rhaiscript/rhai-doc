use crate::data::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub name: Option<String>,
    pub color: Option<Rgb>,
    pub icon: Option<String>,
    pub stylesheet: Option<String>,
    pub code_theme: Option<String>,
    pub code_lang: Option<String>,
    pub root: Option<String>,
    pub index: Option<String>,
    pub extension: Option<String>,
    pub links: Vec<Link>,
    pub google_analytics: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
    pub fn to_alpha(&self, alpha: u8) -> Rgba {
        Rgba(self.0, self.1, self.2, alpha)
    }
}

impl ToString for Rgb {
    fn to_string(&self) -> String {
        format!("rgb({r}, {g}, {b})", r = self.0, g = self.1, b = self.2)
    }
}

pub struct Rgba(pub u8, pub u8, pub u8, pub u8);

impl ToString for Rgba {
    fn to_string(&self) -> String {
        format!(
            "rgba({r}, {g}, {b}, {a})",
            r = self.0,
            g = self.1,
            b = self.2,
            a = (self.3 as f32 / u8::MAX as f32)
        )
    }
}
