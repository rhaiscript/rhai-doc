use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Page {
    pub title: String,
    pub name: String,
    pub root: String,
    pub icon: String,
    pub functions: Option<Vec<Function>>,
    pub markdown: Option<String>,
    pub external_links: Vec<Link>,
    pub page_links: Vec<Link>,
    pub document_links: Vec<Link>,
}

#[derive(Deserialize, Serialize)]
pub struct Function {
    pub definition: String,
    pub markdown: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Link {
    pub name: String,
    pub link: String,
}
