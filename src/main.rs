#[macro_use]
extern crate clap;

use clap::App;
use glob::glob;
use handlebars::Handlebars;
use pulldown_cmark::{html, Event, Options, Parser, Tag};
use rhai::*;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

mod config;
mod data;
mod error;

fn write_styles(config: &config::Config) -> Result<(), error::RhaiDocError> {
    let mut handlebars = Handlebars::new();
    let mut path = PathBuf::new();
    let mut data: BTreeMap<String, String> = BTreeMap::new();

    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_template_string(
        "styles",
        std::str::from_utf8(include_bytes!("../assets/styles.tpl.css"))?,
    )?;

    path.push("dist");
    path.push("styles.css");

    data.insert("colour".into(), config.colour.to_string());
    data.insert(
        "colour_alpha".into(),
        config.colour.to_alpha(45).to_string(),
    );

    let mut file = File::create(&path)?;
    file.write_all(handlebars.render("styles".into(), &data)?.as_ref())?;

    Ok(())
}

fn write_icon(
    config: &config::Config,
    directory: &PathBuf,
    destination: &PathBuf,
) -> Result<String, error::RhaiDocError> {
    let icon_default = include_bytes!("../assets/logo.svg");
    let mut directory = directory.clone();
    let mut destination = destination.clone();

    if let Some(icon) = config.icon.clone() {
        directory.push(&icon);
        let mut file = File::open(&directory)?;
        let mut logo = Vec::new();
        destination.push("logo");

        if let Some(extension) = PathBuf::from(&icon).extension() {
            destination.set_extension(extension);
            file.read_to_end(&mut logo)?;

            let mut new_file = File::create(destination)?;
            new_file.write_all(&logo)?;

            return Ok(format!(
                "logo.{extension}",
                extension = extension.to_string_lossy()
            ));
        } else {
            return Err(error::RhaiDocError::Icon(
                "Icon must have an extension".into(),
            ));
        }
    }

    destination.push("logo.svg");
    let mut new_file = File::create(destination)?;
    new_file.write_all(icon_default)?;
    Ok("logo.svg".into())
}

fn comments_to_string(comments: &Vec<&str>) -> String {
    let mut md = String::new();
    for comment in comments.iter() {
        md += &format!("{}\n", &comment[3..comment.len()]);
    }
    md
}

fn html_from_pathbuf(path: &PathBuf) -> String {
    let mut new_path = path.clone();
    new_path.set_extension("html");
    new_path
        .iter()
        .map(|item| item.to_string_lossy())
        .collect::<Vec<Cow<'_, str>>>()
        .join("-")
}

fn main() -> Result<(), error::RhaiDocError> {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let destination = matches.value_of("destination").unwrap_or("dist");
    let directory_source = matches.value_of("directory").unwrap_or(".");
    let directory_pages_string = format!("{directory}/pages", directory = directory_source);
    let directory_pages = matches.value_of("pages").unwrap_or(&directory_pages_string);

    let mut path_toml = PathBuf::from(directory_source);
    path_toml.push("rhai.toml");

    let mut path_glob_source = PathBuf::from(directory_source);
    path_glob_source.push("**/*.rhai");

    let mut path_glob_documents = PathBuf::from(&directory_pages);
    path_glob_documents.push("*.md");

    std::fs::create_dir_all(destination)?;

    match File::open(path_toml) {
        Ok(mut config_file) => {
            let mut page_links: Vec<data::Link> = Vec::new();
            let mut document_links: Vec<data::Link> = Vec::new();
            let mut handlebars = Handlebars::new();
            let mut config_file_output = String::new();

            let options = Options::all();
            let engine = Engine::default();

            let mut pages: Vec<(String, PathBuf, String)> = Vec::new();

            config_file.read_to_string(&mut config_file_output)?;
            let config: config::Config = toml::from_str(&config_file_output)?;

            handlebars.register_escape_fn(handlebars::no_escape);
            handlebars.register_template_string(
                "page",
                std::str::from_utf8(include_bytes!("../assets/page.html.hbs"))?,
            )?;
            handlebars.register_partial(
                "fn-block",
                std::str::from_utf8(include_bytes!("../assets/fn-block.html.hbs"))?,
            )?;

            if let Some(extension) = &config.extension {
                path_glob_source.set_extension(extension);
            }
            //
            //  WRITE FILES
            //
            write_styles(&config)?;
            let icon = write_icon(
                &config,
                &PathBuf::from(directory_source),
                &PathBuf::from(destination),
            )?;

            //
            //  PAGE LINKS
            //
            for entry in glob(&path_glob_documents.to_string_lossy())? {
                if let Ok(path) = entry {
                    let mut markdown_string = String::new();
                    let mut new_path = PathBuf::from(destination);
                    let mut file_name = html_from_pathbuf(&path);
                    let mut markdown = File::open(&path)?;
                    markdown.read_to_string(&mut markdown_string)?;

                    new_path.push(&file_name);

                    let mut html_output = String::new();
                    let mut parser_header = Parser::new_ext(&markdown_string, options);
                    let parser_html = Parser::new_ext(&markdown_string, options);
                    html::push_html(&mut html_output, parser_html);

                    // Don't create the page unless it has a heading
                    if parser_header.next() == Some(Event::Start(Tag::Heading(1))) {
                        if let Some(Event::Text(text)) = parser_header.next() {
                            let name: String = text.to_owned().to_string();

                            if path.file_name() == Some(OsStr::new(&config.index)) {
                                new_path.set_file_name("index.html");
                                file_name = "index.html".into();
                            }

                            page_links.push(data::Link {
                                name: name.clone(),
                                link: file_name,
                            });
                            pages.push((name, new_path, html_output));
                        }
                    }
                }
            }

            //
            //  DOCUMENT LINKS
            //
            for entry in glob(&path_glob_source.to_string_lossy())? {
                match entry {
                    Ok(path) => document_links.push(data::Link {
                        name: path
                            .strip_prefix(directory_source)?
                            .to_string_lossy()
                            .into(),
                        link: html_from_pathbuf(&path),
                    }),
                    Err(_) => {}
                }
            }

            //
            //  PAGES
            //
            for (name, path, markdown) in pages {
                let page: data::Page = data::Page {
                    title: config.name.clone(),
                    name: name,
                    root: config.root.clone(),
                    icon: icon.clone(),
                    functions: None,
                    markdown: Some(markdown),
                    external_links: config.links.clone(),
                    page_links: page_links.clone(),
                    document_links: document_links.clone(),
                };
                let mut file = File::create(&path)?;

                file.write_all(handlebars.render("page".into(), &page)?.as_ref())?;
            }

            //
            //  DOCUMENTS
            //
            for entry in glob(&path_glob_source.to_string_lossy())? {
                match entry {
                    Ok(path) => {
                        let ast = engine.compile_file(path.clone())?;
                        let mut new_path = PathBuf::from(destination);
                        let file_name = html_from_pathbuf(&path);

                        new_path.push(&file_name);

                        let mut page: data::Page = data::Page {
                            title: config.name.clone(),
                            name: file_name,
                            root: config.root.clone(),
                            icon: icon.clone(),
                            functions: Some(Vec::new()),
                            markdown: None,
                            external_links: config.links.clone(),
                            page_links: page_links.clone(),
                            document_links: document_links.clone(),
                        };

                        let functions = ast
                            .iter_functions()
                            .map(|function| {
                                let mut html_output = String::new();
                                let markdown = comments_to_string(&function.comments);
                                let parser = Parser::new_ext(&markdown, options);

                                html::push_html(&mut html_output, parser);

                                data::Function {
                                    definition: format!("fn {}", function),
                                    markdown: html_output,
                                }
                            })
                            .collect();

                        page.functions = Some(functions);

                        let mut file = File::create(&new_path)?;

                        file.write_all(handlebars.render("page".into(), &page)?.as_ref())?;
                    }
                    Err(error) => println!("Error loading file: {error}", error = error),
                }
            }
        }
        Err(error) => println!("Cannot load `rhai.toml`: {error}", error = error),
    }

    Ok(())
}
