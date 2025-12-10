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
                artist: "Pitbull".into()
            },
            Song{
              name:"I Want It That Way".into(),
              id: "47BBI51FKFwOMlIiX6m8ya".into(),
              artist:"Backstreet Boys".into()
            },
            Song{
              name:"Shivers".into(),
              id: "78AjaULHLHTUs2UhaTCM8N".into(),
              artist:"Ed Sheeran".into()
            },
            Song{
              name:"It Ends Tonight".into(),
              id: "1FMHNVeJ9s1x1l1WlaRs2I".into(),
              artist:"The All-American Rejects".into()
            }
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

    pub fn search_by_title() {
        todo!();
    }

    pub async fn play(&mut self) -> anyhow::Result<()> {
        let access_token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = "https://api.spotify.com/v1/me/player";
        let mut body = serde_json::json!({});
        body = serde_json::json!({
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
            let mut body2 = serde_json::json!({});
            body2 = serde_json::json!({
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

    pub fn skip() {
        todo!();
    }

    pub fn back() {
        todo!();
    }

    pub fn shuffle() {
        todo!();
    }

    pub fn replay() {
        todo!();
    }

    pub fn change_account() {
        todo!();
    }
}

#[derive(Debug, serde::Deserialize)]
struct SpotifyDevice {
    id: String,
    is_active: bool,
    is_private_session: bool,
    is_restricted: bool,
    name: String,
    supports_volume: bool,
    #[serde(rename = "type")]
    _type: String,
    volume_percent: i32,
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
