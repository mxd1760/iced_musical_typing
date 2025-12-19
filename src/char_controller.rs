use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct CharController {
    // search_map:HashMap<String,Vec<String>>,
    play_map: HashMap<String, Vec<String>>,
}

impl CharController {
    pub async fn init(
        //search_chars:Vec<String>,
        play_chars: Vec<&str>,
    ) -> Self {
        let mut play_map: HashMap<String, Vec<String>> = HashMap::new();
        for file in play_chars {
            load_chars(&mut play_map, file).await;
        }
        Self { play_map }
    }

    pub fn get_play_char(&self, key: &str) -> Option<Vec<String>> {
        match self.play_map.get(key) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }

    pub fn check_special_char(&self, key: &str) -> Option<Vec<String>> {
        if key.chars().nth(0) == Some('っ') {
            let k = key.chars().nth(1).unwrap_or(' ').to_string();
            match self.play_map.get(k.as_str()) {
                Some(v) => {
                    if let Some(romaji) = v.get(0) {
                        Some(vec![romaji.chars().enumerate().fold(
                            "".into(),
                            |mut out, (i, v)| {
                                if i == 0 {
                                    out.push(v);
                                    out.push(v);
                                    out
                                } else {
                                    out.push(v);
                                    out
                                }
                            },
                        )])
                    } else {
                        None
                    }
                }
                None => None,
            }
        } else {
            match key.chars().nth(1) {
                Some('ゃ' | 'ャ' | 'ょ' | 'ョ' | 'ゅ' | 'ュ') => {
                    let k1 = key.chars().nth(0).unwrap_or(' ').to_string();
                    let k2 = key.chars().nth(1).unwrap_or(' ').to_string();
                    if let Some(a) = self.get_play_char(k1.as_str()) {
                        if let Some(b) = self.get_play_char(k2.as_str()) {
                            let romaji = a
                                .iter()
                                .nth(0)
                                .unwrap_or(&" ".to_owned())
                                .chars()
                                .zip(b.iter().nth(0).unwrap_or(&" ".to_owned()).chars())
                                .enumerate()
                                .fold("".into(), |mut out: String, (i, (c, d))| {
                                    if i == 0 {
                                        out.push(c);
                                        out.push(d);
                                        out
                                    } else {
                                        out.push(d);
                                        out
                                    }
                                });
                            Some(vec![romaji])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
                Some('々') => {
                    let k = key.chars().nth(0).unwrap_or(' ').to_string();
                    if let Some(v) = self.get_play_char(k.as_str()) {
                        Some(v.iter().map(|v| v.to_owned() + v).collect())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }

    // fn convert_search_str(&self,start:String)->String{
    //
    // }
}

async fn load_chars(map: &mut HashMap<String, Vec<String>>, json: &str) {
    match serde_json::from_str::<HashMap<String, Vec<String>>>(json) {
        Ok(v) => {
            map.extend(v);
        }
        Err(e) => {
            log::error!("SOURCE: {}\nFile failed to load: {}\n", json, e);
        }
    }
}
