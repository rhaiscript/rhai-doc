use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct Page {
    pub title: String,
    pub name: String,
    pub root: String,
    pub icon: String,
    pub stylesheet: Option<String>,
    pub functions: Option<Vec<Function>>,
    pub markdown: Option<String>,
    pub external_links: Vec<Link>,
    pub page_links: Vec<super::LinkInfo>,
    pub document_links: Vec<super::LinkInfo>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct Function {
    pub definition: String,
    pub markdown: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Hash)]
pub struct Link {
    pub name: String,
    pub link: String,
}
