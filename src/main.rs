use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env;
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

fn clean_html(html: &str) -> String {
    let re = Regex::new(r"<[^<]+?>").unwrap();
    let result = re.replace_all(html, "");
    String::from(result)
}

#[derive(Debug, Deserialize, Serialize)]
struct SnlArticle {
    id: usize,
    url: String,
    title: String,
    subject_title: String,
    xhtml_body: String,
}

struct Config {
    sitemap: String,
    output_path: String,
    n: usize,
}

impl Config {
    fn new(args: &[String]) -> Config {
        let sitemap = args[1].clone();
        let output_path = args[2].clone();
        let n = match args[3].clone().parse() {
            Ok(n) => n,
            Err(error) => panic!("Failed to provide a digit for the n argument: {:?}", error),
        };
        Config {
            sitemap,
            output_path,
            n,
        }
    }
}

fn main() {
    println!("SNL Parser!");
    let args: Vec<String> = env::args().collect();
    let config: Config = Config::new(&args);
    let sitemap_path = config.sitemap;

    let sitemap = match read_sitemap(&sitemap_path) {
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
    let delay = time::Duration::from_millis(500);
    let mut c = 0;
    let n = config.n;
    for &url in urls.iter() {
        c = c + 1;
        if c > n {
            break;
        }
        println!("Parsing url: {}/{}", c, n);
        thread::sleep(delay);

        let full_url = format!("{}.json", url);
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

    for article in &mut articles {
        article.xhtml_body = clean_html(&article.xhtml_body);
    }

    println!("Finished parsing JSON files... saving...");
    let file = match File::create(config.output_path) {
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
