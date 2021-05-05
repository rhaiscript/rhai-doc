#[macro_use]
extern crate clap;

use clap::App;
use glob::glob;
use handlebars::Handlebars;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use rhai::*;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

mod config;
mod data;
mod error;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LinkInfo {
    pub path: PathBuf,
    pub active: bool,
    pub name: String,
    pub link: String,
    pub sub_links: Vec<data::Link>,
    #[serde(skip)]
    pub ast: Option<AST>,
}

fn write_styles(
    config: &config::Config,
    _source: &PathBuf,
    destination: &PathBuf,
) -> Result<(), error::RhaiDocError> {
    let mut handlebars = Handlebars::new();
    let mut styles = destination.clone();
    let mut data: BTreeMap<String, String> = BTreeMap::new();

    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_template_string(
        "styles",
        std::str::from_utf8(include_bytes!("../assets/styles.tpl.css"))?,
    )?;

    styles.push("styles.css");

    let color = config
        .color
        .clone()
        .unwrap_or_else(|| config::Rgb(246, 119, 2));
    data.insert("color".into(), color.to_string());
    data.insert("color_alpha".into(), color.to_alpha(45).to_string());

    let mut file = File::create(&styles)?;
    file.write_all(handlebars.render("styles".into(), &data)?.as_ref())?;

    Ok(())
}

fn write_icon(
    config: &config::Config,
    source: &PathBuf,
    destination: &PathBuf,
) -> Result<String, error::RhaiDocError> {
    let icon_default = include_bytes!("../assets/logo.svg");
    let mut source = source.clone();
    let mut destination = destination.clone();

    if let Some(icon) = config.icon.clone() {
        source.push(&icon);
        let mut file = match File::open(&source) {
            Ok(f) => f,
            Err(error) => {
                eprintln!(
                    "Cannot load icon `{file}`: {error}",
                    file = source.to_string_lossy(),
                    error = error
                );
                return Err(error.into());
            }
        };
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

fn comments_to_string(comments: &[&str]) -> String {
    let mut md = String::new();
    for comment in comments.iter() {
        md += &format!("{}\n", &comment[3..comment.len()]);
    }
    md
}

fn html_from_pathbuf(path: &Path, root: &Path) -> String {
    let mut new_path = match path.strip_prefix(root) {
        Ok(path) => PathBuf::from(path),
        Err(_) => PathBuf::from(path),
    };
    new_path.set_extension("html");
    new_path
        .iter()
        .map(|item| item.to_string_lossy())
        .collect::<Vec<_>>()
        .join("--")
}

fn gen_hash_name(function: &ScriptFnMetadata) -> String {
    if function.params.is_empty() {
        function.name.to_string()
    } else {
        format!("{}-{}", function.name, function.params.len())
    }
}

fn main() -> Result<(), error::RhaiDocError> {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let verbose = matches.is_present("verbose");
    let config_file = matches.value_of("config").unwrap_or("rhai.toml");
    let destination = matches.value_of("destination").unwrap_or("dist");
    let directory_source = matches.value_of("directory").unwrap_or(".");
    let directory_pages_string = format!("{directory}/pages", directory = directory_source);
    let directory_pages = matches.value_of("pages").unwrap_or(&directory_pages_string);

    if verbose {
        println!(
            "Rhai documentation tool (version {})",
            env!("CARGO_PKG_VERSION")
        );
    }

    let mut path_toml = PathBuf::from(directory_source);
    path_toml.push(config_file);

    if verbose {
        println!("Config file: `{}`", path_toml.to_string_lossy());
    }

    let source = PathBuf::from(directory_source);
    let mut path_glob_source = source.clone();
    path_glob_source.push("**/*.rhai");

    if verbose {
        println!("Source directory: `{}`", source.to_string_lossy());
    }

    let mut path_glob_documents = PathBuf::from(&directory_pages);
    path_glob_documents.push("*.md");

    if verbose {
        println!(
            "MarkDown pages: `{}`",
            path_glob_documents.to_string_lossy()
        );
    }

    let destination = PathBuf::from(destination);
    std::fs::create_dir_all(&destination)?;

    if verbose {
        println!("Destination directory: `{}`", destination.to_string_lossy());
    }

    match File::open(&path_toml) {
        Ok(mut config_file) => {
            let mut page_links = Vec::new();
            let mut document_links = Vec::new();
            let mut handlebars = Handlebars::new();
            let mut config_file_output = String::new();

            let mut options = Options::all();
            options.insert(Options::ENABLE_SMART_PUNCTUATION);
            options.insert(Options::ENABLE_TABLES);
            let engine = Engine::default();

            let mut pages: Vec<(String, PathBuf, String)> = Vec::new();

            config_file.read_to_string(&mut config_file_output)?;
            let config: config::Config = toml::from_str(&config_file_output)?;

            if verbose {
                println!("Config: {:?}", config);
            }

            if let Some(extension) = &config.extension {
                path_glob_source.set_extension(extension);
            }

            if verbose {
                println!(
                    "Source file pattern: `{}`",
                    path_glob_source.to_string_lossy()
                );
            }

            handlebars.register_escape_fn(handlebars::no_escape);
            handlebars.register_template_string(
                "page",
                std::str::from_utf8(include_bytes!("../assets/page.html.hbs"))?,
            )?;
            handlebars.register_partial(
                "fn-block",
                std::str::from_utf8(include_bytes!("../assets/fn-block.html.hbs"))?,
            )?;

            if verbose {
                println!("Registered handlebars templates.");
            }

            //
            //  WRITE FILES
            //
            write_styles(&config, &source, &destination)?;
            let icon = write_icon(&config, &source, &destination)?;

            let stylesheet_filename = if let Some(ref ss) = config.stylesheet {
                let ss = PathBuf::from(ss);

                if ss.is_file() {
                    if verbose {
                        println!("Custom stylesheet: `{}`", ss.to_string_lossy());
                    }
                    let mut ss_source = source.clone();
                    ss_source.push(&ss);
                    let mut ss_dest = destination.clone();
                    let filename = ss.file_name().unwrap().to_string_lossy().into_owned();
                    ss_dest.push(&filename);

                    let mut file = match File::open(&ss_source) {
                        Ok(f) => f,
                        Err(error) => {
                            eprintln!(
                                "Cannot load icon `{file}`: {error}",
                                file = ss_source.to_string_lossy(),
                                error = error
                            );
                            return Err(error.into());
                        }
                    };
                    let mut content = Vec::<u8>::new();
                    file.read_to_end(&mut content)?;
                    let mut file = File::create(&ss_dest)?;
                    file.write_all(&content)?;
                    Some(filename)
                } else {
                    None
                }
            } else {
                None
            };

            if verbose {
                println!("Written styles and icon.")
            }

            //
            //  PAGE LINKS
            //
            if verbose {
                println!(
                    "Processing MarkDown pages from `{}`...",
                    path_glob_documents.to_string_lossy()
                );
            }

            let mut files_list = glob(&path_glob_documents.to_string_lossy())?
                .into_iter()
                .filter(|p| p.is_ok())
                .map(|p| p.unwrap())
                .collect::<Vec<_>>();
            files_list.sort();

            let mut index_file = PathBuf::from(&directory_pages);
            index_file.push(PathBuf::from(config.index));
            let index_file = index_file.canonicalize().unwrap();

            if let Some(n) = files_list.iter().enumerate().find_map(|(i, p)| {
                if p.canonicalize().unwrap() == index_file {
                    Some(i)
                } else {
                    None
                }
            }) {
                let file = files_list.remove(n);
                files_list.insert(0, file);
            }

            for src_path in files_list {
                if verbose {
                    println!(
                        "> Writing MarkDown page `{}`...",
                        src_path.to_string_lossy()
                    );
                }
                let mut markdown_string = String::new();
                let mut dest_path = destination.clone();
                let mut file_name = html_from_pathbuf(&src_path, &PathBuf::from(directory_source));
                let mut markdown = File::open(&src_path)?;
                markdown.read_to_string(&mut markdown_string)?;

                dest_path.push(&file_name);
                let mut html_output = String::new();
                let mut parser_header = Parser::new_ext(&markdown_string, options);
                let parser_html = Parser::new_ext(&markdown_string, options);
                html::push_html(&mut html_output, parser_html);

                // Don't create the page unless it has a heading
                if parser_header.next() == Some(Event::Start(Tag::Heading(1))) {
                    if let Some(Event::Text(text)) = parser_header.next() {
                        let name: String = text.to_owned().to_string();

                        if src_path.canonicalize().unwrap() == index_file {
                            dest_path.set_file_name("index.html");
                            file_name = "index.html".into();
                        }

                        page_links.push(LinkInfo {
                            path: src_path,
                            active: false,
                            name: name.clone(),
                            link: file_name,
                            sub_links: Default::default(),
                            ast: None,
                        });
                        pages.push((name, dest_path, html_output));
                    }
                }
            }

            //
            //  DOCUMENT LINKS
            //
            if verbose {
                println!(
                    "Scanning for Rhai scripts from `{}`...",
                    path_glob_source.to_string_lossy()
                );
            }
            for entry in glob(&path_glob_source.to_string_lossy())? {
                match entry {
                    Ok(path) => {
                        if verbose {
                            println!("> Found Rhai script `{}`", path.to_string_lossy());
                        }

                        let mut name = path.clone();
                        name.set_extension("");

                        let name = match name.strip_prefix(&source) {
                            Ok(name) => name,
                            Err(_) => &name,
                        }
                        .components()
                        .map(|c| c.as_os_str().to_string_lossy())
                        .collect::<Vec<_>>()
                        .join("/");

                        let ast = engine.compile_file(path.clone())?;

                        document_links.push(LinkInfo {
                            path: path.clone(),
                            name,
                            active: false,
                            link: html_from_pathbuf(&path, &PathBuf::from(directory_source)),
                            sub_links: Default::default(),
                            ast: Some(ast),
                        })
                    }
                    Err(error) => eprintln!(
                        "Error loading script files `{pattern}`: {error}",
                        pattern = path_glob_source.to_string_lossy(),
                        error = error
                    ),
                }
            }

            //
            //  PAGES
            //
            if verbose {
                println!("Processing HTML pages...");
            }
            document_links
                .iter_mut()
                .for_each(|LinkInfo { active, .. }| *active = false);

            for (i, (name, dest_path, markdown)) in pages.into_iter().enumerate() {
                if verbose {
                    println!("> Writing HTML page `{}`...", dest_path.to_string_lossy());
                }

                let mut links_clone = page_links.clone();
                links_clone[i].active = true;

                let page: data::Page = data::Page {
                    title: config.name.clone().unwrap_or_default(),
                    name,
                    root: config.root.clone().unwrap_or_default(),
                    icon: icon.clone(),
                    stylesheet: stylesheet_filename.clone(),
                    functions: None,
                    markdown: Some(markdown),
                    external_links: config.links.clone(),
                    page_links: links_clone,
                    document_links: document_links.clone(),
                    google_analytics: config.google_analytics.clone(),
                };
                let mut file = File::create(&dest_path)?;

                file.write_all(handlebars.render("page".into(), &page)?.as_ref())?;
            }

            //
            //  DOCUMENTS
            //
            page_links
                .iter_mut()
                .for_each(|LinkInfo { active, .. }| *active = false);

            for i in 0..document_links.len() {
                let LinkInfo { path, ast, .. } = &document_links[i];

                let mut new_path = destination.clone();
                let file_name = html_from_pathbuf(&path, &PathBuf::from(directory_source));
                new_path.push(&file_name);

                if verbose {
                    println!(
                        "Processing Rhai script `{}` into `{}`...",
                        path.to_string_lossy(),
                        new_path.to_string_lossy()
                    );
                }

                let mut functions = ast.as_ref().unwrap().iter_functions().collect::<Vec<_>>();
                functions.sort_by(|a, b| match a.name.partial_cmp(b.name).unwrap() {
                    Ordering::Equal => a.params.len().partial_cmp(&b.params.len()).unwrap(),
                    cmp => cmp,
                });

                let mut links_clone = document_links.clone();
                links_clone[i].active = true;
                links_clone[i].sub_links = functions
                    .iter()
                    .map(|f| data::Link {
                        name: f.to_string(),
                        link: gen_hash_name(f),
                    })
                    .collect();

                let mut page: data::Page = data::Page {
                    title: config.name.clone().unwrap_or_default(),
                    name: file_name,
                    root: config.root.clone().unwrap_or_default(),
                    icon: icon.clone(),
                    stylesheet: stylesheet_filename.clone(),
                    functions: Some(Vec::new()),
                    markdown: None,
                    external_links: config.links.clone(),
                    page_links: page_links.clone(),
                    document_links: links_clone,
                    google_analytics: config.google_analytics.clone(),
                };

                let functions = functions
                    .into_iter()
                    .map(|function| {
                        if verbose {
                            println!("> Writing function `{}`...", function);
                        }

                        let mut html_output = String::new();
                        let markdown = comments_to_string(&function.comments);
                        let parser = Parser::new_ext(&markdown, options);

                        html::push_html(
                            &mut html_output,
                            parser.into_iter().map(|event| match event {
                                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                                    if lang.is_empty() =>
                                {
                                    Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(
                                        "rust".into(),
                                    )))
                                }
                                _ => event,
                            }),
                        );

                        data::Function {
                            id: gen_hash_name(&function),
                            definition: format!("fn {}", function),
                            markdown: html_output,
                        }
                    })
                    .collect();

                page.functions = Some(functions);

                let mut file = File::create(&new_path)?;

                file.write_all(handlebars.render("page".into(), &page)?.as_ref())?;
            }
        }
        Err(error) => eprintln!(
            "Cannot load `{file}`: {error}",
            file = path_toml.to_string_lossy(),
            error = error
        ),
    }

    if verbose {
        println!("Done.");
    }

    Ok(())
}
