use glob::glob;
use handlebars::Handlebars;
use pulldown_cmark::{html, CodeBlockKind, Event, Options, Parser, Tag};
use rhai::{Engine, FnAccess, ScriptFnMetadata, AST};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::prelude::{Read, Write};
use std::path::{Path, PathBuf};

mod cli;
mod config;
mod data;
mod error;

macro_rules! write_log {
    ($debug:expr, $fmt:expr, $(@ $args:expr),*) => {
        if $debug { println!($fmt, $($args.to_string_lossy()),*); }
    };
    ($debug:expr, $($args:expr),*) => {
        if $debug { println!($($args),*); }
    };
}

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

    styles.push("rhai-doc-styles.css");

    let color = config.color.clone();
    let color = color.unwrap_or_else(|| config::Rgb(246, 119, 2));
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

        return match PathBuf::from(&icon).extension() {
            Some(extension) => {
                destination.set_extension(extension);
                file.read_to_end(&mut logo)?;

                let mut new_file = File::create(destination)?;
                new_file.write_all(&logo)?;

                Ok(format!(
                    "logo.{extension}",
                    extension = extension.to_string_lossy()
                ))
            }
            None => Err(error::RhaiDocError::Icon(
                "Icon must have an extension".into(),
            )),
        };
    }

    destination.push("logo.svg");

    let mut new_file = File::create(destination)?;
    new_file.write_all(icon_default)?;

    Ok("logo.svg".into())
}

fn comments_to_string(comments: &[&str]) -> String {
    comments
        .iter()
        .map(|s| {
            if s.starts_with("///") || s.starts_with("/**") {
                if s.ends_with("**/") {
                    &s[3..s.len() - 3]
                } else if s.ends_with("*/") {
                    &s[3..s.len() - 2]
                } else {
                    &s[3..]
                }
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn html_from_pathbuf(path: &Path, root: &Path) -> PathBuf {
    let mut new_path = path
        .strip_prefix(root)
        .map_or_else(|_| PathBuf::from(path), PathBuf::from);
    new_path.set_extension("html");
    new_path
}

fn gen_hash_name(function: &ScriptFnMetadata) -> String {
    if function.params.is_empty() {
        function.name.to_string()
    } else {
        format!("{}-{}", function.name, function.params.len())
    }
}

fn main() -> Result<(), error::RhaiDocError> {
    let app = {
        use clap::Parser;
        cli::Cli::parse()
    };

    let (quiet, debug) = match app.verbose {
        1 => (true, false),
        3.. => (false, true),
        0 | 2 | _ => (false, false),
    };
    let config_file = app.config;
    let skip_private = !app.all;
    let source = app.directory;
    let dir_destination = app.destination;
    let dir_pages = app.pages;
    let command = app.command;

    write_log!(
        !quiet,
        "{} - Rhai documentation tool (version {})",
        "rhai-doc",
        "1.0" //app.name,
              //app.version
    );

    write_log!(!quiet, "Source directory: `{}`", @source);

    match command {
        Some(cli::RhaiDocCommand::New { config }) => {
            let mut path_toml = source.clone();
            path_toml.push(config);
            let mut config_file = match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path_toml)
            {
                Ok(f) => f,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    eprintln!(
                        "Configuration file `{file}` already exists",
                        file = path_toml.to_string_lossy(),
                    );
                    return Err(error.into());
                }
                Err(error) => {
                    eprintln!(
                        "Cannot create configuration file `{file}`: {error}",
                        file = path_toml.to_string_lossy(),
                        error = error
                    );
                    return Err(error.into());
                }
            };
            write_log!(!quiet, "Writing configuration file `{}`...", @path_toml);
            let toml = std::str::from_utf8(include_bytes!("../assets/rhai.toml"))?;
            config_file.write_all(toml.as_bytes())?;
            write_log!(!quiet, "Configuration file generated.");
            return Ok(());
        }
        None => (),
    }

    let mut path_toml = source.clone();
    path_toml.push(config_file);

    write_log!(!quiet, "Config file: `{}`", @path_toml);

    let mut config_file = match File::open(&path_toml) {
        Ok(f) => f,
        Err(error) => {
            eprintln!(
                "Cannot load `{file}`: {error}",
                file = path_toml.to_string_lossy(),
                error = error
            );
            return Err(error.into());
        }
    };
    let mut config_file_output = String::new();
    config_file.read_to_string(&mut config_file_output)?;
    let config: config::Config = toml::from_str(&config_file_output)?;

    write_log!(debug, "{:#?}", config);

    let mut path_glob_source = source.clone();
    path_glob_source.push("**");
    path_glob_source.push("*.rhai");

    if let Some(extension) = &config.extension {
        path_glob_source.set_extension(if extension.starts_with('.') {
            &extension[1..]
        } else {
            extension
        });
    }

    write_log!(!quiet, "Script files pattern: `{}`", @path_glob_source);

    let mut path_pages = source.clone();
    path_pages.push(dir_pages);

    let index_file = config.index.as_ref().map(|index| {
        let mut file = path_pages.clone();
        file.push(index);
        file
    });

    path_pages.push("**");
    path_pages.push("*.md");

    write_log!(!quiet, "MarkDown pages: `{}`", @path_pages);

    let mut destination = source.clone();
    destination.push(dir_destination);
    std::fs::create_dir_all(&destination)?;

    write_log!(!quiet, "Destination directory: `{}`", @destination);

    let mut page_links = Vec::new();
    let mut script_links = Vec::new();
    let mut handlebars = Handlebars::new();

    let mut options = Options::all();
    options.insert(Options::ENABLE_SMART_PUNCTUATION);
    options.insert(Options::ENABLE_TABLES);
    let engine = Engine::default();

    let mut pages: Vec<(String, PathBuf, String)> = Vec::new();

    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_template_string(
        "page",
        std::str::from_utf8(include_bytes!("../assets/page.html.hbs"))?,
    )?;
    handlebars.register_partial(
        "fn-block",
        std::str::from_utf8(include_bytes!("../assets/fn-block.html.hbs"))?,
    )?;

    write_log!(!quiet, "Registered handlebars templates.");

    //
    //  WRITE FILES
    //
    write_styles(&config, &source, &destination)?;
    let icon = write_icon(&config, &source, &destination)?;

    let stylesheet_filename = if let Some(stylesheet) = config.stylesheet {
        let mut css = source.clone();
        css.push(stylesheet);

        if css.is_file() {
            write_log!(!quiet, "Custom stylesheet: `{}`", @css);

            let mut ss_source = source.clone();
            ss_source.push(&css);
            let mut ss_dest = destination.clone();
            let filename = css.file_name().unwrap().to_string_lossy().into_owned();
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

    write_log!(!quiet, "Written styles and icon.");

    //
    //  PAGE LINKS
    //
    write_log!(!quiet, "Scanning for MarkDown pages from `{}`...", @path_pages);

    let mut files_list = glob(&path_pages.to_string_lossy())?
        .into_iter()
        .filter(|p| p.is_ok())
        .map(|p| p.unwrap())
        .collect::<Vec<_>>();
    files_list.sort();

    // Move the home page to the front
    let mut has_index = false;

    if let Some(ref index_file) = index_file {
        if let Some(n) =
            files_list.iter().enumerate().find_map(
                |(i, p)| {
                    if p == index_file {
                        Some(i)
                    } else {
                        None
                    }
                },
            )
        {
            let file = files_list.remove(n);
            files_list.insert(0, file);
            has_index = true;
        }
    }

    for src_path in files_list {
        write_log!(!quiet, "> Generating HTML from MarkDown page `{}`...", @src_path);

        let mut markdown_string = String::new();
        let mut dest_path = destination.clone();
        let mut file_path = html_from_pathbuf(&src_path, &source);
        let mut markdown = File::open(&src_path)?;
        markdown.read_to_string(&mut markdown_string)?;

        dest_path.push(&file_path);
        let mut html_output = String::new();
        let mut parser_header = Parser::new_ext(&markdown_string, options);
        let parser_html = Parser::new_ext(&markdown_string, options);
        html::push_html(&mut html_output, parser_html);

        // Don't create the page unless it has a heading
        if parser_header.next() == Some(Event::Start(Tag::Heading(1))) {
            if let Some(Event::Text(text)) = parser_header.next() {
                let name: String = text.to_owned().to_string();

                if let Some(ref index_file) = index_file {
                    if &src_path == index_file {
                        file_path = PathBuf::from("index.html");
                        dest_path = destination.clone();
                        dest_path.push(&file_path);
                    }
                }

                let link = file_path
                    .components()
                    .map(|s| s.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/")
                    .to_string();

                page_links.push(LinkInfo {
                    path: src_path,
                    active: false,
                    name: name.clone(),
                    link,
                    sub_links: Default::default(),
                    ast: None,
                });
                pages.push((name, dest_path, html_output));
            }
        }
    }

    //
    //  SCRIPT LINKS
    //
    write_log!(!quiet, "Scanning for Rhai scripts from `{}`...", @path_glob_source);

    for entry in glob(&path_glob_source.to_string_lossy())? {
        match entry {
            Ok(path) => {
                write_log!(!quiet, "> Found Rhai script `{}`", @path);

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

                if ast
                    .iter_functions()
                    .filter(|f| !skip_private || f.access != FnAccess::Private)
                    .count()
                    == 0
                {
                    write_log!(!quiet, "  ... which contains no functions. Skipped.");
                    continue;
                }

                let doc_path = html_from_pathbuf(&path, &source);

                let link = doc_path
                    .components()
                    .map(|s| s.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/")
                    .to_string();

                write_log!(!quiet, "  -> {}", link);

                script_links.push(LinkInfo {
                    path: path.clone(),
                    name,
                    active: false,
                    link,
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
    write_log!(!quiet, "Writing HTML pages...");

    for (i, (name, dest_path, markdown)) in pages.into_iter().enumerate() {
        write_log!(!quiet, "  -> HTML page `{}`...", @dest_path);

        let mut links_clone = page_links.clone();
        links_clone[i].active = true;

        let root = if let Some(ref r) = config.root {
            r.clone()
        } else {
            match dest_path.strip_prefix(&destination)?.ancestors().count() {
                0..=1 => String::new(),
                levels => std::iter::repeat("../")
                    .take(levels - 2)
                    .collect::<Vec<_>>()
                    .join(""),
            }
        };

        let page: data::Page = data::Page {
            title: config.name.clone().unwrap_or_default(),
            name,
            root,
            icon: icon.clone(),
            stylesheet: stylesheet_filename.clone(),
            code_theme: config.code_theme.clone().unwrap_or("default".to_string()),
            code_lang: config.code_lang.clone().unwrap_or("ts".to_string()),
            functions: None,
            markdown: Some(markdown),
            external_links: config.links.clone(),
            page_links: links_clone,
            script_links: script_links.clone(),
            google_analytics: config.google_analytics.clone(),
        };
        if let Some(dir) = dest_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut file = File::create(&dest_path)?;

        file.write_all(handlebars.render("page".into(), &page)?.as_ref())?;
    }

    if !has_index {
        let mut dest_path = destination.clone();
        dest_path.push("index.html");

        write_log!(!quiet, "  -> index page `{}`...", @dest_path);

        let page: data::Page = data::Page {
            title: config.name.clone().unwrap_or_default(),
            name: "index.html".to_string(),
            root: config.root.clone().unwrap_or_default(),
            icon: icon.clone(),
            stylesheet: stylesheet_filename.clone(),
            code_theme: config.code_theme.clone().unwrap_or("default".to_string()),
            code_lang: config.code_lang.clone().unwrap_or("ts".to_string()),
            functions: None,
            markdown: None,
            external_links: config.links.clone(),
            page_links: page_links.clone(),
            script_links: script_links.clone(),
            google_analytics: config.google_analytics.clone(),
        };
        if let Some(dir) = dest_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut file = File::create(&dest_path)?;

        file.write_all(handlebars.render("page".into(), &page)?.as_ref())?;
    }

    //
    //  SCRIPTS
    //
    write_log!(!quiet, "Writing Rhai scripts...");

    for i in 0..script_links.len() {
        let LinkInfo { path, ast, .. } = &script_links[i];

        let mut new_path = destination.clone();
        let file_name = html_from_pathbuf(&path, &source);
        new_path.push(&file_name);

        write_log!(!quiet, "> `{}` -> `{}`...", @path, @new_path);

        let mut functions = ast
            .as_ref()
            .unwrap()
            .iter_functions()
            .filter(|f| !skip_private || f.access != FnAccess::Private)
            .collect::<Vec<_>>();

        functions.sort_by(|a, b| match a.name.partial_cmp(b.name).unwrap() {
            Ordering::Equal => a.params.len().partial_cmp(&b.params.len()).unwrap(),
            cmp => cmp,
        });

        let mut links_clone = script_links.clone();
        links_clone[i].active = true;
        links_clone[i].sub_links = functions
            .iter()
            .map(|f| data::Link {
                name: f.to_string(),
                link: gen_hash_name(f),
            })
            .collect();

        let root = if let Some(ref r) = config.root {
            r.clone()
        } else {
            match new_path.strip_prefix(&destination)?.ancestors().count() {
                0..=1 => String::new(),
                levels => std::iter::repeat("../")
                    .take(levels - 2)
                    .collect::<Vec<_>>()
                    .join(""),
            }
        };

        let mut page: data::Page = data::Page {
            title: config.name.clone().unwrap_or_default(),
            name: file_name.to_string_lossy().to_string(),
            root,
            icon: icon.clone(),
            stylesheet: stylesheet_filename.clone(),
            code_theme: config.code_theme.clone().unwrap_or("default".to_string()),
            code_lang: config.code_lang.clone().unwrap_or("ts".to_string()),
            functions: Some(Vec::new()),
            markdown: None,
            external_links: config.links.clone(),
            page_links: page_links.clone(),
            script_links: links_clone,
            google_analytics: config.google_analytics.clone(),
        };

        let mut last_link = "";
        let fn_links = functions
            .iter()
            .filter(|f| {
                if f.name != last_link {
                    last_link = f.name;
                    true
                } else {
                    false
                }
            })
            .map(|f| format!("[`{}`]: #{}\n", f.name, gen_hash_name(f)))
            .collect::<Vec<_>>()
            .join("");
        let fn_links = fn_links.trim();

        let functions = functions
            .into_iter()
            .map(|function| {
                if function.access == FnAccess::Private {
                    write_log!(
                        debug,
                        "    -> {}...",
                        function.to_string().replace("private ", "private fn ")
                    );
                } else {
                    write_log!(debug, "    -> fn {}...", function);
                }

                let mut html_output = String::new();
                let mut markdown = comments_to_string(&function.comments);
                if !fn_links.is_empty() {
                    markdown.push_str("\n\n");
                    markdown.push_str(&fn_links);
                }
                let parser = Parser::new_ext(&markdown, options);

                html::push_html(
                    &mut html_output,
                    parser.into_iter().map(|event| match event {
                        Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang)))
                            if lang.is_empty() =>
                        {
                            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced("ts".into())))
                        }
                        _ => event,
                    }),
                );

                data::Function {
                    id: gen_hash_name(&function),
                    definition: if function.access == FnAccess::Private {
                        function.to_string().replace("private", "private fn")
                    } else {
                        format!("fn {}", function)
                    },
                    is_private: function.access == FnAccess::Private,
                    markdown: html_output,
                }
            })
            .collect();

        page.functions = Some(functions);

        if let Some(dir) = new_path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let mut file = File::create(&new_path)?;

        file.write_all(handlebars.render("page".into(), &page)?.as_ref())?;
    }

    write_log!(
        !quiet,
        "Done - documentation generated under `{}`",
        @destination
    );

    Ok(())
}
