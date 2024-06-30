use crate::config;
use md5::{Digest, Md5};
use rand::Rng;
use reqwest::{
    self,
    header::{HeaderValue, CONTENT_TYPE},
};

pub struct TransRequest {
    req_client: reqwest::Client,
}

const BAIDU_URL: &str = "http://api.fanyi.baidu.com/api/trans/vip/translate";

impl TransRequest {
    pub fn new() -> Self {
        Self {
            req_client: reqwest::Client::new(),
        }
    }

    async fn baidu(&self, text: &str, target_language: &str) -> String {
        let config = config::get_config();
        if let Some(baidu) = &config.baidu {
            
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
                ("to", target_language),
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
        } else {
            "no baidu token".into()
        }
    }

    pub async fn request(&self, text: &str, vendor: &str, target_language: &str) -> String {
        match vendor {
            "baidu" => self.baidu(text, target_language).await,
            _ => "error".to_string(),
        }
    }
}
