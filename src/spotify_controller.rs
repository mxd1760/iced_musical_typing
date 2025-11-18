use iced::futures::TryFutureExt;
use rspotify::{prelude::*,AuthCodeSpotify, OAuth,Credentials};


#[derive(Debug)]
pub struct SpotifyController{
  spotify:AuthCodeSpotify,
}

pub struct Song{

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
            std::fs::write(
                token_path,
                serde_json::to_string_pretty(token)?
            )?;
        }

        Ok(Self {
            spotify,
        })
    }

    pub async fn init_from_env(oauth: OAuth) -> anyhow::Result<Self> {
        Self::init(Credentials::from_env().unwrap(), oauth).await
    }

    pub async fn get_access_token(&self)->anyhow::Result<String>{
        let token_guard = self.spotify.token.lock().await.unwrap();
        let access_token = match &*token_guard {
            Some(token) => &token.access_token,
            None => {
                return Err(anyhow::anyhow!("No access token available"));
            }
        };
        Ok(access_token.to_string())
    }

    pub fn get_devices() {
        todo!();
    }

    pub fn search_by_title() {
        todo!();
    }

    pub async fn play(&mut self, track_uri_option: Option<String>) -> anyhow::Result<()> {

        let access_token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = "https://api.spotify.com/v1/me/player/play";
        let mut body = serde_json::json!({});
        if let Some(track_uri) = track_uri_option {
            body = serde_json::json!({
                "uris": [track_uri]
            });
        }

        let res = client
            .put(url)
            .bearer_auth(access_token)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        println!("Response {:#?}", res.status());
        Ok(())
    }

    pub async fn pause(&mut self) -> anyhow::Result<()> {

        let access_token = self.get_access_token().await?;
        let client = reqwest::Client::new();
        let url = "https://api.spotify.com/v1/me/player/pause";

        let res = client
            .put(url)
            .bearer_auth(access_token)
            .header("Content-Length", 0)
            .send()
            .await?;

        println!("Response {:#?}", res.status());
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