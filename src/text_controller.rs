use std::{
    fs::File,
    io::{self, BufRead},
    path::PathBuf,
    thread,
};

use iced::futures::TryFutureExt;

use crate::{TextControllerData, TextType};

#[derive(Debug, Clone)]
pub struct TextController {}

impl Default for TextController {
    fn default() -> Self {
        Self {}
    }
}

const NUM_LINES: i32 = 15;

impl TextController {
    pub async fn init() -> Self {
        Self::default()
    }

    pub async fn get_new_data(
        mode: TextType,
        index: i32,
        song_name: Option<String>,
    ) -> TextControllerData {
        match mode {
            TextType::LRCLIB => {
                let lyrics: Vec<String> = match song_name {
                    Some(query) => {
                        match get_lrclib_lyrics(query, index as usize, (index + NUM_LINES) as usize)
                            .await{
                                Ok(v) => if v.is_empty() {
                                    vec![
                                      "No more lyrics".into(),
                                      "All the lyrics are gone".into(),
                                      "No more lyrics".into(),
                                      "All the lyrics are gone".into(),
                                      "No more lyrics".into(),
                                    ]
                                }else{
                                  v
                                },
                                Err(_) => vec![
                                "Something went wrong".into(),
                                "LRCLIB fetch failed".into(),
                                "Something went wrong".into(),
                                "LRCLIB fetch failed".into(),
                                "Something went wrong".into(),
                            ],
                            }
                    }
                    None => {
                        vec![
                            "Something went wrong".into(),
                            "You have no current song".into(),
                            "Something went wrong".into(),
                            "You have no current song".into(),
                            "Something went wrong".into(),
                        ]
                    }
                };
                TextControllerData {
                    text_type: TextType::LRCLIB,
                    lyrics,
                    current_line: 0,
                    next_fetch_line: index + NUM_LINES,
                }
            }
            TextType::Github => todo!(),
            TextType::ThisProject => {
                let lyrics = match load_new_lines(
                    format!("{}/src/main.rs", env!("CARGO_MANIFEST_DIR")),
                    index as usize,
                    (index + NUM_LINES) as usize,
                )
                .await
                {
                    Ok(v) => if v.is_empty(){
                      vec![
                        "No more lyrics".into(),
                        "All the lyrics are gone".into(),
                        "No more lyrics".into(),
                        "All the lyrics are gone".into(),
                        "No more lyrics".into(),
                    ]
                    }else{v},
                    Err(_) => vec![
                        "Something went wrong".into(),
                        "Fetching the lyrics from this file".into(),
                        "Something went wrong".into(),
                        "fetching the lyrics from this file".into(),
                        "Something went wrong".into(),
                    ],
                };
                TextControllerData {
                    text_type: TextType::ThisProject,
                    lyrics,
                    current_line: 0,
                    next_fetch_line: index + NUM_LINES,
                }
            }
        }
    }
}

async fn load_new_lines(
    file_name: impl Into<PathBuf>,
    line_start: usize,
    line_end: usize,
) -> anyhow::Result<Vec<String>> {
    let file_path = file_name.into();
    
    let line_handle = thread::spawn(move || -> Result<Vec<String>, io::Error> {
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .filter_map(Result::ok)
            .filter(|v| !v.is_empty())
            .enumerate()
            .filter(|(i, _)| (*i >= line_start && *i < line_end))
            .map(|(_, val)| {
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
    id: i32,
    name: String,
    #[serde(rename = "trackName")]
    track_name: String,
    #[serde(rename = "artistName")]
    artist_name: String,
    #[serde(rename = "albumName")]
    album_name: String,
    duration: f32,
    instrumental: bool,
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

async fn get_lrclib_lyrics(
    query: String,
    index_start: usize,
    index_end: usize,
) -> anyhow::Result<Vec<String>> {
    let client = reqwest::Client::new();
    let url = "https://lrclib.net/api/search?q=".to_owned() + query.as_str();

    let res: reqwest::Response = client.get(url).header("Content-Length", 0).send().await?;

    println!("LRCLIB Response {:#?}", res.status());
    // get plainLyrics
    let plain_lyrics = parse_lrclib_response(res).await.unwrap(); //?;//TODO

    Ok(plain_lyrics
        .split('\n')
        .filter(|v| !v.is_empty())
        .enumerate()
        .filter(|(i, _)| *i >= index_start && *i < index_end)
        .map(|(_, v)| (v.to_owned() + " "))
        .collect())
}

async fn parse_lrclib_response(res: reqwest::Response) -> anyhow::Result<String> {
    let map = res.json::<Vec<LrclibObj>>().await.unwrap(); //?;//TODO
    Ok(map[0]
        .plain_lyrics
        .clone()
        .ok_or(anyhow::anyhow!("Plain Lyrics null"))?)
}
