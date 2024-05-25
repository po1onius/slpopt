use std::fs;
use md5::{Digest, Md5};
use reqwest;
use rand::Rng;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct TransRequest {
    language: String,
    vendor: String,
    baidu: Option<Baidu>,
    google: Option<Google>,
    bing: Option<Bing>
}

#[derive(Deserialize)]
#[derive(Clone)]
struct Baidu {
    appid: String,
    token: String
}

#[derive(Deserialize)]
struct Google {

}

#[derive(Deserialize)]
struct Bing {

}

impl TransRequest {
    pub fn from_config() ->Self {
        let config = fs::read_to_string("config.toml").unwrap();
        let config = toml::from_str(&config).unwrap();
        config
    }

    pub async fn request(&self, text: &str) ->String {
        match self.vendor.as_str() {
            "baidu" => {
                let baidu = self.baidu.clone().unwrap();

                let mut rng = rand::thread_rng();
                let salt = rng.gen_range(32768..65536);
                let h = format!("{}{}{}{}", &baidu.appid, text, salt, &baidu.token);
                
                let mut hasher = Md5::new();
                hasher.update(h.as_bytes());
                let result = hasher.finalize();
                
                let url = format!("http://api.fanyi.baidu.com/api/trans/vip/translate?q={}&from=en&to=zh&appid={}&salt={}&sign={:x}", text, &baidu.appid, salt, result);
                let response = reqwest::get(url).await.unwrap();
                let v: serde_json::Value = serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
                let res = v["trans_result"][0]["dst"].as_str();
                if res.is_some() {
                    return res.unwrap().to_string();
                } else {
                    return "error".to_string();
                }
            }
            _ => "error".to_string()
        }
    }
}

