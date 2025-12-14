use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
    thread,
};

use crate::TextType;

#[derive(Debug, Clone)]
pub struct TextController {
    loaded_lyrics: Vec<String>,
}

impl Default for TextController {
    fn default() -> Self {
        Self {
            loaded_lyrics: vec![],
        }
    }
}

pub const NUM_LINES: usize = 20;

impl TextController {
    pub async fn init() -> Self {
        Self::default()
    }

    pub async fn fetch_lyrics(&mut self, index: usize) -> Option<Vec<String>> {
        if index > self.loaded_lyrics.len() {
            None
        } else {
            let index_end = index + NUM_LINES;
            if index_end > self.loaded_lyrics.len() {
                Some(self.loaded_lyrics[index..].to_vec())
            } else {
                Some(self.loaded_lyrics[index..index + NUM_LINES].to_vec())
            }
        }
    }

    pub async fn load_lyrics(&mut self, mode: TextType, song_name: Option<String>) -> bool {
        match mode {
            TextType::LRCLIB => match song_name {
                Some(query) => match get_lrclib_lyrics(query).await {
                    Ok(v) => {
                        if v.is_empty() {
                            false
                        } else {
                            self.loaded_lyrics = v;
                            true
                        }
                    }
                    Err(_) => false,
                },
                None => false,
            },
            // TextType::Github => todo!(),
            TextType::ThisProject => {
                match load_new_lines(format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR"))).await {
                    Ok(v) => {
                        if v.is_empty() {
                            false
                        } else {
                            self.loaded_lyrics = v;
                            true
                        }
                    }
                    Err(_) => false,
                }
            }
        }
    }
}

async fn load_new_lines(file_name: impl Into<PathBuf>) -> anyhow::Result<Vec<String>> {
    let file_path = file_name.into();

    let line_handle = thread::spawn(move || -> Result<Vec<String>, io::Error> {
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .filter_map(Result::ok)
            .filter(|v| !v.is_empty())
            .map(|val| {
                let mut v = val.trim().to_string();
                v.push(' ');
                v
            })
            .collect();
        Ok(lines.clone())
    });

    if let Ok(result) = line_handle.join() {
        Ok(result?)
    } else {
        Err(anyhow::anyhow!("read file thread panicked"))
    }
}

#[derive(Debug, serde::Deserialize)]

struct LrclibObj {
    #[serde(rename = "id")]
    _id: i32,
    #[serde(rename = "name")]
    _name: String,
    #[serde(rename = "trackName")]
    _track_name: String,
    #[serde(rename = "artistName")]
    _artist_name: String,
    #[serde(rename = "albumName")]
    _album_name: String,
    #[serde(rename = "duration")]
    _duration: f32,
    #[serde(rename = "instrumental")]
    _instrumental: bool,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    _synced_lyrics: Option<String>,
}

async fn get_lrclib_lyrics(query: String) -> anyhow::Result<Vec<String>> {
    let client = reqwest::Client::new();
    let url = "https://lrclib.net/api/search?q=".to_owned() + query.as_str();

    let res: reqwest::Response = client.get(url).header("Content-Length", 0).send().await?;

    println!("LRCLIB Response {:#?}", res.status());
    // get plainLyrics
    let plain_lyrics = parse_lrclib_response(res).await.unwrap(); //?;//TODO

    Ok(plain_lyrics
        .split('\n')
        .filter(|v| !v.is_empty())
        .map(|v| v.to_owned() + " ")
        .collect())
}

async fn parse_lrclib_response(res: reqwest::Response) -> anyhow::Result<String> {
    let map = res.json::<Vec<LrclibObj>>().await.unwrap(); //?;//TODO
    Ok(map[0]
        .plain_lyrics
        .clone()
        .ok_or(anyhow::anyhow!("Plain Lyrics null"))?)
}
