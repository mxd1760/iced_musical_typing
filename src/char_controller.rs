use std::collections::HashMap;


#[derive(Debug,Clone)]
pub struct CharController{
  // search_map:HashMap<String,Vec<String>>,
  play_map:HashMap<String,Vec<String>>,
}


impl CharController{
  pub async fn init(
    //search_chars:Vec<String>,
    play_chars:Vec<&str>) -> Self{
    let mut play_map:HashMap<String,Vec<String>> = HashMap::new();
    for file in play_chars{
      load_chars(&mut play_map,file).await;
    }
    Self{
      play_map
    }
  }

  pub fn get_play_char(&self,key:&str)->Option<Vec<String>>{
    match self.play_map.get(key){
        Some(v) => {
          Some(v.clone())
        },
        None => None,
    }
  }

  // fn convert_search_str(&self,start:String)->String{
  //
  // }
}

async fn load_chars(map:&mut HashMap<String,Vec<String>>,json:&str){
    match serde_json::from_str::<HashMap<String,Vec<String>>>(json){
        Ok(v) => {
          map.extend(v);
        },
        Err(e) => {
          log::error!("SOURCE: {}\nFile failed to load: {}\n",json,e);
        },
    } 
}