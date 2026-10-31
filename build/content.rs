use serde::Deserialize;

use crate::lang::Lang;

/// Facts about this build, shown in the footer.
pub struct Build {
    pub version: String,
    pub commit: String,
}

#[derive(Deserialize)]
pub struct Content {
    pub name: String,
    pub role: String,
    pub headline: String,
    pub intro: String,
    pub location: String,
    pub email: String,
    pub url: String,
    pub ui: Ui,
    #[serde(default)]
    pub links: Vec<Link>,
    pub about: About,
    /// Price list; a language without it shows no prices section and no nav link.
    pub prices: Option<Prices>,
    #[serde(default)]
    pub projects: Vec<Project>,
    #[serde(default)]
    pub experience: Vec<Job>,
    #[serde(default)]
    pub education: Vec<Education>,
    #[serde(default)]
    pub skills: Vec<SkillGroup>,
}

#[derive(Deserialize)]
pub struct Ui {
    pub nav_about: String,
    pub nav_work: String,
    pub nav_resume: String,
    pub nav_contact: String,
    pub see_work: String,
    pub get_in_touch: String,
    pub work_title: String,
    pub work_lead: String,
    pub learn_more: String,
    pub resume_title: String,
    pub skills: String,
    pub education: String,
    pub contact_title: String,
    pub contact_lead: String,
    pub footer_note: String,
    pub source: String,
    pub not_found_title: String,
    pub not_found_lead: String,
    pub not_found_back: String,
}

#[derive(Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Deserialize)]
pub struct About {
    pub title: String,
    pub lead: String,
    #[serde(default)]
    pub items: Vec<AboutItem>,
}

#[derive(Deserialize)]
pub struct AboutItem {
    pub symbol: String,
    pub title: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct Prices {
    pub nav: String,
    pub title: String,
    pub lead: String,
    pub items: Vec<PriceItem>,
    pub included_title: String,
    #[serde(default)]
    pub included: Vec<String>,
    pub note: String,
}

#[derive(Deserialize)]
pub struct PriceItem {
    pub name: String,
    pub price: String,
    pub term: String,
    pub text: String,
}

#[derive(Deserialize)]
pub struct Project {
    pub name: String,
    pub tagline: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub theme: Theme,
}

#[derive(Deserialize, Default, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    #[default]
    Light,
    Blue,
}

impl Theme {
    pub fn class(self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Blue => "blue",
        }
    }
}

#[derive(Deserialize)]
pub struct Job {
    pub role: String,
    pub company: String,
    pub period: String,
    pub location: Option<String>,
    #[serde(default)]
    pub highlights: Vec<String>,
}

#[derive(Deserialize)]
pub struct Education {
    pub degree: String,
    pub school: String,
    pub period: String,
}

#[derive(Deserialize)]
pub struct SkillGroup {
    pub group: String,
    pub items: Vec<String>,
}

fn source(lang: Lang) -> &'static str {
    match lang {
        Lang::En => include_str!("../content/en.toml"),
        Lang::Ru => include_str!("../content/ru.toml"),
    }
}

pub fn load(lang: Lang) -> Content {
    toml::from_str(source(lang))
        .unwrap_or_else(|e| panic!("content/{}.toml is invalid: {e}", lang.code()))
}
