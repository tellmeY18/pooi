use std::{env, fs, io::Write, path::Path};

use anyhow::{bail, Result};
use chrono::prelude::Local;
use glob::glob;

pub fn fetch(query: String, lang: String) -> Result<String, ureq::Error> {
    let x = ureq::get("https://google.com/search")
        .set(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:53.0) Gecko/20100101 Firefox/53.0"
        )
        .query("q", &query)
        .query("hl", &lang)
        .call()?
        .into_string()
        .unwrap();
    Ok(x)
}

pub fn cached_html(os_type: &str) -> Result<String> {
    let files = get_file_list(os_type)?;
    let html = fs::read_to_string(&files[(files.len() - 1)])?;
    Ok(html)
}

pub fn save_html(query: &[&str], html: &str, os_type: &str) -> Result<String> {
    let cache_path = get_cache_path(os_type)?;
    let file_date = Local::now().format("%s").to_string();
    let file_query = query.join("_");
    let sep = sep_type(os_type);

    let mut x: Vec<&str> = vec![&cache_path, sep, "oi"];
    if !Path::new(&x.join("")).is_dir() {
        fs::create_dir(&x.join(""))?
    }
    x.push(sep);
    x.push(&file_date);
    x.push("-");
    x.push(&file_query);
    x.push(".html");

    let full_path = x.join("");
    let mut file = fs::File::create(&full_path)?;
    file.write_all(html.as_bytes())?;
    Ok(full_path)
}

pub fn clean_cache(os_type: &str) -> Result<String> {
    let sep = sep_type(os_type);
    let target = [&get_cache_path(os_type)?, sep, "oi"].join("");
    fs::remove_dir_all(&target)?;
    Ok(target)
}

// this isn't strictly necessary but the mix of slashes on Windows looks messy
fn sep_type(os_type: &str) -> &str {
    match os_type {
        "Windows" => "\\",
        _ => "/"
    }
}

fn get_cache_path(os_type: &str) -> Result<String> {
    let cache_path: String = match os_type {
        "Bsd" | "Linux" => match env::var("XDG_CACHE_HOME") {
            Ok(x) => x,
            Err(_) => {
                let home_path = env::var("HOME")?;
                let x = [home_path, "/.cache".to_string()];
                x.join("")
            }
        },
        "MacOS" => {
            let home_path = env::var("HOME")?;
            let x = [home_path, "/Library/Application Support".to_string()];
            x.join("")
        }
        "Windows" => env::var("LOCALAPPDATA")?,
        &_ => bail!("This feature is not supported on your platform, sorry!")
    };
    Ok(cache_path)
}

fn get_file_list(os_type: &str) -> Result<Vec<String>> {
    let sep = sep_type(os_type);
    let cache_path = [&get_cache_path(os_type)?, sep, "oi", sep, "*.html"].join("");

    let mut files: Vec<String> = vec![];
    for x in glob(&cache_path).unwrap() {
        match x {
            Ok(x) => files.push(x.display().to_string()),
            Err(_) => panic!("get_file_list: glob search failed!?!")
        }
    }

    match files.len() {
        0 => bail!("Can't find any cached results, sorry!"),
        _ => Ok(files)
    }
}