use std::collections::HashMap;

use rspotify::{AuthCodeSpotify, Credentials, OAuth, prelude::*};

#[derive(Debug)]
pub struct SpotifyController {
    spotify: AuthCodeSpotify,
    device_id: String,
    song_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Song {
    pub name: String,
    pub id: String,
    // artists:Vec<Artist>,
    pub artist: String,
}

// pub struct Artist {
//     name: String,
//     id: String,
// }

impl Song {
    pub fn mock_songs() -> Vec<Song> {
        vec![
            Song {
                name: "Yumeyume".into(),
                id: "05ReuhxWC85vxG530BGty7".into(),
                artist: "DECO*27".into(),
            },
            Song {
                name: "Crazy for you".into(),
                id: "0xIW9Iex1ziifoFcRL1JVS".into(),
                artist: "焼塩檸檬".into(),
            },
            Song {
                name: "仮死化".into(),
                id: "4sVdacv8Qflef5SDiYXUpg".into(),
                artist: "Vivid BAD SQUAD".into(),
            },
            Song {
                name: "メリュー".into(),
                id: "6Tl3V1vOgah4pAwXUGeuI3".into(),
                artist: "25時".into(),
            },
            Song {
              name: "Hai Yorikonde".into(),
              id: "6woV8uWxn7rcLZxJKYruS1".into(),
              artist: "Kocchi no Kento".into()
            },
            Song {
                name: "Golden".into(),
                id: "1CPZ5BxNNd0n0nF4Orb9JS".into(),
                artist: "HUNTR/X".into(),
            },
            Song {
                name: "Mr. Brightside".into(),
                id: "003vvx7Niy0yvhvHt4a68B".into(),
                artist: "The Killers".into(),
            },
            Song {
                name: "100 bad days".into(),
                id: "4rnyUV17cSZGsz18xJNdjL".into(),
                artist: "AJR".into(),
            },
            Song {
                name: "115".into(),
                id: "725NSbIej5lP3GfhLC7So3".into(),
                artist: "Kevin Sherwood".into(),
            },
            Song {
                name: "Nobody".into(),
                id: "3SiVMpHxTS1gspWzRZE50S".into(),
                artist: "OneRepublic".into(),
            },
            Song {
                name: "Timber".into(),
                id: "3cHyrEgdyYRjgJKSOiOtcS".into(),
                artist: "Pitbull".into(),
            },
            Song {
                name: "I Want It That Way".into(),
                id: "47BBI51FKFwOMlIiX6m8ya".into(),
                artist: "Backstreet Boys".into(),
            },
            Song {
                name: "Shivers".into(),
                id: "78AjaULHLHTUs2UhaTCM8N".into(),
                artist: "Ed Sheeran".into(),
            },
            Song {
                name: "It Ends Tonight".into(),
                id: "1FMHNVeJ9s1x1l1WlaRs2I".into(),
                artist: "The All-American Rejects".into(),
            },
        ]
    }
}

impl SpotifyController {
    pub async fn init(creds: Credentials, oauth: OAuth) -> anyhow::Result<Self> {
        let token_path = "target/spotify_token.json";
        let spotify = AuthCodeSpotify::new(creds, oauth);
        // if let Ok(saved) = std::fs::read_to_string(token_path) {
        //     if let Ok(token) = serde_json::from_str(&saved) {
        //         {
        //             let mut guard = spotify.token.lock().await.unwrap();
        //             *guard = Some(token);
        //         }
        //         return Ok(Self { spotify });
        //     }
        // }
        // else
        let url = spotify.get_authorize_url(false).unwrap();
        println!("Allow us to control playback on spotify: {url}");
        spotify.prompt_for_token(&url).await?;

        // save token
        if let Some(token) = &*spotify.token.lock().await.unwrap() {
            std::fs::write(token_path, serde_json::to_string_pretty(token)?)?;
        }

        Ok(Self {
            spotify,
            device_id: "".into(),
            song_id: None,
        })
    }

    pub async fn init_from_env(oauth: OAuth) -> anyhow::Result<Self> {
        Self::init(Credentials::from_env().unwrap(), oauth).await
    }

    pub fn set_device_id(&mut self, new_id: String) {
        self.device_id = new_id;
    }
    pub fn set_song_id(&mut self, new_id: String) {
        self.song_id = Some(new_id);
    }

    pub async fn get_access_token(&self) -> anyhow::Result<String> {
        let token_guard = self.spotify.token.lock().await.unwrap();
        let access_token = match &*token_guard {
            Some(token) => &token.access_token,
            None => {
                return Err(anyhow::anyhow!("No access token available"));
            }
        };
        Ok(access_token.to_string())
    }

    pub async fn get_devices(&self) -> anyhow::Result<Vec<(String, String)>> {
        let access_token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = "https://api.spotify.com/v1/me/player/devices";

        let res: reqwest::Response = client
            .get(url)
            .bearer_auth(access_token)
            .header("Content-Length", 0)
            .send()
            .await?;

        println!("Spotify Response {:#?}", res.status());
        Ok(parse_devices_response(res).await)
    }

    pub async fn search(&self, query: String) -> Option<Vec<Song>> {
        let access_token = match self.get_access_token().await {
            Ok(t) => t,
            Err(_) => {
                return None;
            }
        };
        let client = reqwest::Client::new();
        let url = format!("https://api.spotify.com/v1/search?type=track&q={}", query);

        let res = match client
            .get(url)
            .bearer_auth(access_token)
            .header("Content-Length", 0)
            .send()
            .await
        {
            Ok(v) => v,
            Err(_) => {
                return None;
            }
        };

        println!("Spotify Search Response {:#?}", res.status());
        println!("{:#?}", res);
        parse_search_response(res).await
    }

    pub async fn play(&mut self) -> anyhow::Result<()> {
        let access_token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = "https://api.spotify.com/v1/me/player";
        let body = serde_json::json!({
          "device_ids":[self.device_id],
          "play":true
        });

        let res = client
            .put(url)
            .bearer_auth(access_token.clone())
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        println!("Spotify Response {:#?}", res.status());

        if let Some(track_uri) = &self.song_id {
            let url = "https://api.spotify.com/v1/me/player/play";
            let body2 = serde_json::json!({
                "uris":[format!("spotify:track:{}",track_uri)],
                "play":true
            });

            let res = client
                .put(url)
                .bearer_auth(access_token)
                .header("Content-Type", "application/json")
                .json(&body2)
                .send()
                .await?;
            println!("2nd Response {:#?}", res.status());
        }
        self.song_id = None;
        Ok(())
    }

    pub async fn pause(&self) -> anyhow::Result<()> {
        let access_token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = "https://api.spotify.com/v1/me/player/pause";

        let res = client
            .put(url)
            .bearer_auth(access_token)
            .header("Content-Length", 0)
            .send()
            .await?;

        println!("Spotify Response {:#?}", res.status());
        Ok(())
    }

    // pub fn skip() {
    //     todo!();
    // }

    // pub fn back() {
    //     todo!();
    // }

    // pub fn shuffle() {
    //     todo!();
    // }

    // pub fn replay() {
    //     todo!();
    // }

    // pub fn change_account() {
    //     todo!();
    // }
}

#[derive(Debug, serde::Deserialize)]
struct SpotifyDevice {
    id: String,
    #[serde(rename = "is_active")]
    _is_active: bool,
    #[serde(rename = "is_private_session")]
    _is_private_session: bool,
    #[serde(rename = "is_restricted")]
    _is_restricted: bool,
    name: String,
    #[serde(rename = "supports_volume")]
    _supports_volume: bool,
    #[serde(rename = "type")]
    _type: String,
    #[serde(rename = "volume_percent")]
    _volume_percent: i32,
}

async fn parse_devices_response(res: reqwest::Response) -> Vec<(String, String)> {
    let map = match res.json::<HashMap<String, Vec<SpotifyDevice>>>().await {
        Ok(map) => map,
        Err(s) => {
            log::error!("error parsing response: {}", s);
            return vec![];
        }
    };
    let devices = match map.get("devices") {
        Some(devices) => devices,
        None => {
            log::warn!("could not fetch devices");
            return vec![];
        }
    };
    devices
        .iter()
        .map(|v| (v.name.clone(), v.id.clone()))
        .collect()
}

#[derive(Debug, serde::Deserialize)]
struct SearchResults {
    #[serde(rename = "href")]
    _href: String,
    #[serde(rename = "limit")]
    _limit: i32,
    #[serde(rename = "next")]
    _next: Option<String>,
    #[serde(rename = "offset")]
    _offset: i32,
    #[serde(rename = "previous")]
    _previous: Option<String>,
    #[serde(rename = "total")]
    _total: i32,
    #[serde(rename = "items")]
    items: Vec<SearchResultItem>,
}

#[derive(Debug, serde::Deserialize)]
struct SearchResultItem {
    #[serde(rename = "album")]
    _album: Album,
    #[serde(rename = "artists")]
    _artists: Vec<Artist>,
    #[serde(rename = "available_markets")]
    _markets: Vec<String>,
    #[serde(rename = "disc_number")]
    _disc_number: i32,
    #[serde(rename = "duration_ms")]
    _duration: i64,
    #[serde(rename = "explicit")]
    _explicit: bool,
    #[serde(rename = "external_ids")]
    _external_ids: HashMap<String, String>,
    #[serde(rename = "external_urls")]
    _external_urls: HashMap<String, String>,
    #[serde(rename = "href")]
    _href: String,
    #[serde(rename = "id")]
    id: String,
    #[serde(rename = "is_local")]
    _is_local: bool,
    #[serde(rename = "is_playable")]
    _is_playable: bool,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "popularity")]
    _popularity: i32,
    #[serde(rename = "preview_url")]
    _preview_url: Option<String>,
    #[serde(rename = "track_number")]
    _track_number: i32,
    #[serde(rename = "type")]
    _type: String,
    #[serde(rename = "uri")]
    _uri: String,
}

#[derive(Debug, serde::Deserialize)]
struct Album {
    #[serde(rename = "album_type")]
    _album_type: String,
    #[serde(rename = "artists")]
    _artists: Vec<Artist>,
    #[serde(rename = "available_markets")]
    _markets: Vec<String>,
    #[serde(rename = "external_urls")]
    _external_urls: HashMap<String, String>,
    #[serde(rename = "href")]
    _href: String,
    #[serde(rename = "id")]
    _id: String,
    #[serde(rename = "images")]
    _images: Vec<AlbumImage>,
    #[serde(rename = "is_playable")]
    _is_playable: bool,
    #[serde(rename = "name")]
    _name: String,
    #[serde(rename = "release_date")]
    _release_date: String,
    #[serde(rename = "release_date_precision")]
    _rdb: String,
    #[serde(rename = "total_tracks")]
    _total_tracks: i32,
    #[serde(rename = "type")]
    _type: String,
    #[serde(rename = "uri")]
    _uri: String,
}

#[derive(Debug, serde::Deserialize)]
struct Artist {
    #[serde(rename = "external_urls")]
    _external_urls: HashMap<String, String>,
    #[serde(rename = "href")]
    _href: String,
    #[serde(rename = "id")]
    _id: String,
    #[serde(rename = "name")]
    name: String,
    #[serde(rename = "type")]
    _type: String,
    #[serde(rename = "uri")]
    _uri: String,
}

#[derive(Debug, serde::Deserialize)]
struct AlbumImage {
    #[serde(rename = "height")]
    _height: i32,
    #[serde(rename = "width")]
    _width: i32,
    #[serde(rename = "url")]
    _url: String,
}

async fn parse_search_response(res: reqwest::Response) -> Option<Vec<Song>> {
    let body = match res.text().await{
      Ok(text)=>text,
      Err(s)=>{
        log::error!("Error reading text: {}",s);
        return None;
      }
    };
    let map: HashMap<String,SearchResults> = match serde_json::from_str(&body){
      Ok(map)=>map,
      Err(s)=>{
        log::error!("Error parsing response: {}",s);
        return None;
      }
    };
    let tracks = match map.get("tracks") {
        Some(tracks) => tracks,
        None => {
            log::warn!("could not fetch tracks");
            return None;
        }
    };
    Some(
        tracks
            .items
            .iter()
            .enumerate()
            .filter(|(i, _)| *i <= 10 as usize)
            .map(|(_, v)| Song {
                name: v.name.clone(),
                id: v.id.clone(),
                artist: v._artists[0].name.clone(),
            })
            .collect(),
    )
    
}
