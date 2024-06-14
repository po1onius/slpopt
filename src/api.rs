use md5::{Digest, Md5};
use rand::Rng;
use reqwest::{
    self,
    header::{HeaderValue, CONTENT_TYPE},
};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
pub struct Config {
    language: String,
    vendor: String,
    baidu: Option<Baidu>,
    google: Option<Google>,
    bing: Option<Bing>,
}

#[derive(Deserialize, Clone)]
struct Baidu {
    appid: String,
    token: String,
}

#[derive(Deserialize)]
struct Google {}

#[derive(Deserialize)]
struct Bing {}

pub struct TransRequest {
    config: Config,
    req_client: reqwest::Client,
}

const BAIDU_URL: &str = "http://api.fanyi.baidu.com/api/trans/vip/translate";

impl TransRequest {
    pub fn from_config() -> Self {
        let config = fs::read_to_string("config.toml").unwrap();
        let config = toml::from_str(config.as_str()).unwrap();
        Self {
            config,
            req_client: reqwest::Client::new(),
        }
    }

    async fn baidu(&self, text: &str) -> String {
        let baidu = self.config.baidu.clone().unwrap();

        let mut rng = rand::thread_rng();
        let salt = rng.gen_range(32768..65536).to_string();
        let h = format!(
            "{}{}{}{}",
            baidu.appid.as_str(),
            text,
            salt,
            baidu.token.as_str()
        );

        let mut hasher = Md5::new();
        hasher.update(h.as_bytes());
        let sign = hasher.finalize();

        //let url = format!("http://api.fanyi.baidu.com/api/trans/vip/translate?q={}&from=auto&to={}&appid={}&salt={}&sign={:x}", text, &self.config.language, &baidu.appid, salt, result);
        let sign = format!("{:x}", sign);

        let body = [
            ("q", text),
            ("from", "auto"),
            ("to", self.config.language.as_str()),
            ("appid", baidu.appid.as_str()),
            ("salt", salt.as_str()),
            ("sign", sign.as_str()),
        ];

        let response = self
            .req_client
            .post(BAIDU_URL)
            .form(&body)
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            )
            .send()
            .await
            .unwrap();

        //let response = reqwest::get(url).await.unwrap();
        let v: serde_json::Value =
            serde_json::from_str(response.text().await.unwrap().as_str()).unwrap();
        let res = v["trans_result"][0]["dst"].as_str();
        if res.is_some() {
            res.unwrap().to_string()
        } else {
            v["error_msg"].as_str().unwrap().to_string()
        }
    }

    pub async fn request(&self, text: &str) -> String {
        match self.config.vendor.as_str() {
            "baidu" => self.baidu(text).await,
            _ => "error".to_string(),
        }
    }
}
