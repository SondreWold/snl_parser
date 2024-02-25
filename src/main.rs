use regex::Regex;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::{thread, time};

fn read_sitemap(path: &str) -> Result<String, Box<dyn Error>> {
    let sitemap: String = fs::read_to_string(&path)?.parse()?;
    Ok(sitemap)
}

fn get_urls(sitemap: &str) -> Result<Vec<&str>, Box<dyn Error>> {
    let re: Regex = Regex::new(r"(https://snl.no/[\w]+)")?;
    let mut urls: Vec<&str> = vec![];
    for (_, [url]) in re.captures_iter(sitemap).map(|caps| caps.extract()) {
        urls.push(url);
    }
    Ok(urls)
}

#[derive(Debug, Deserialize, Serialize)]
struct SnlArticle {
    title: String,
    id: usize,
    xhtml_body: String,
}

fn main() {
    println!("SNL Parser!");
    let sitemap_path = "data/sitemap.xml";

    let sitemap = match read_sitemap(sitemap_path) {
        Ok(file) => file,
        Err(error) => panic!("Problem opening the file: {:?}", error),
    };

    let urls = match get_urls(&sitemap) {
        Ok(list) => list,
        Err(error) => panic!("Failed to parse urls from sitemap: {:?}", error),
    };

    let number_of_urls: usize = urls.len();
    println!("Number of articles found on sitemap: {:?}", number_of_urls);
    let mut articles: Vec<SnlArticle> = vec![];
    let json_suffix = ".json";
    let delay = time::Duration::from_millis(500);
    let mut c = 0;
    let n = 10;
    for &url in urls.iter() {
        c = c + 1;
        if c > n {
            break;
        }
        println!("Parsing url: {}/{}", c, n);
        thread::sleep(delay);

        let full_url = url.to_owned() + json_suffix;
        let response = match reqwest::blocking::get(&full_url) {
            Ok(data) => data,
            Err(err) => {
                println!("Failed to request URL: {:?}", err);
                continue;
            }
        };

        let article = match response.json() {
            Ok(art) => art,
            Err(error) => {
                println!("Failed to parse document: {:?}", error);
                continue;
            }
        };
        articles.push(article);
    }
    println!("Finished parsing JSON files... saving...");
    let file = match File::create("snl_articles.json") {
        Ok(f) => f,
        Err(error) => panic!("Failed to create output file... {:?}", error),
    };

    let mut writer = BufWriter::new(file);
    match serde_json::to_writer(&mut writer, &articles) {
        Ok(()) => println!("Successfully wrote output file"),
        Err(error) => panic!("Failed to write output file: {:?}", error),
    }
    writer.flush().unwrap();
    println!("Finished");
}
