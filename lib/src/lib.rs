use std::{collections::{HashMap}, time::Duration, io::Write};

use crate::amf::{amf0::{array, number, string}, Value, Amf0Value, packet::{Body, Packet, ReadAs}};

use crate::amf::{TryAsAmf0Object, TryAsBoolean, TryAsNumber};
use game::sys::{Quality, ChallengeType, QualityUpType};
use rand::Rng;
use reqwest::{header, Url};

pub use account::*;

pub type Result<T,E = ErrorKind> = std::result::Result<T,E>;

pub mod amf;
pub mod game;

mod account;

#[derive()]
pub enum ErrorKind {
    Static(&'static str),
    Owned(String),
}

impl From<String> for ErrorKind {
    fn from(s: String) -> Self {
        Self::Owned(s)
    }
}

impl From<&'static str> for ErrorKind {
    fn from(s: &'static str) -> Self {
        Self::Static(s)
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorKind::*;
        let s = match self {
            Static(s) => s,
            Owned(s) => s.as_str(),
        };
        s.fmt(f)
    }
}

pub struct Client {
    reqwest_client: reqwest::Client,
    #[allow(dead_code)] server: u8,
    server_url: Url,
    cookies: String,
}

static ERR_PARSE_AMF_OBJ: &'static str = "无法将返回的数据解析为`Amf0Value::Object`";

impl Client {
    
    #[inline]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn resolve_server(server_id: u8) -> Url {
        let url = if server_id < 12 {
            format!("http://pvz-s{}.youkia.com", server_id)
        } else {
            format!("http://s{}.youkia.pvz.youkia.com", server_id)
        };
        Url::parse(&url).expect("fail to parse server host!")
    }

    #[inline]
    pub(crate) fn amf_request_path(&self) -> Url {
        let url = self.server_url.join("/pvz/amf/").unwrap();
        url
    }

    pub fn server_url(&self) -> &Url {
        &self.server_url
    }

    pub fn cookies(&self) -> impl Iterator<Item = (&str,&str)> {
        self.cookies
            .split(";")
            .filter(|s| !s.is_empty())
            .map(|s| {
                if let Some(n) = s.find('=') {
                    return s.split_at(n);
                }
                unreachable!("key/value pairs are joined with '='");
            })
    }

    pub fn get_cookie(&self, key: &str) -> Option<&str> {
        self.cookies().find_map(|(k,v)| {
            (key == k).then(|| v)
        })
    }

    /// add or set the cookie by key, return old value
    pub fn set_cookie<S: AsRef<str>, V: AsRef<str>>(&mut self, key: S, new_value: V) -> Option<String> {
        let val_ref = self.get_cookie(key.as_ref());

        if let Some(val) = val_ref {
            let old_val = val.to_owned();
            unsafe {
                let start = val.as_ptr().offset_from(self.cookies.as_ptr()) as usize;
                let end = start + val.len();
                self.cookies.replace_range(start..end, new_value.as_ref());
            }
            return Some(old_val);
        }

        None
    }

    pub(crate) async fn send_amf<V: Into<Value>>(
        &self,
        target_uri: &str,
        response_uri: &str,
        data: V
    ) -> Result<Packet<'_>> {
        let req_packet = Packet::builder()
            .with_default_version()
            .body(target_uri, response_uri, data)
            .build()
            .map_err(|e|{
                format!("fail to build packet: {}", e)
            })?;
        let resp: Packet<'_> = self.reqwest_client
            .post(self.amf_request_path())
            .header(header::COOKIE, &self.cookies)
            .header("x-flash-version", "34,0,0,192")
            .header(header::CONTENT_TYPE, "application/x-amf")
            .body(req_packet)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .bytes()
            .await
            .map_err(|e| e.to_string())?
            .read_as()
            .map_err(|e| {
                format!("fail to parse response as AMF packet: {}", e)
            })?;
        Ok(resp)
    } 

    /// 技能升级
    /// 
    /// **@return**: now_skill_id
    pub async fn skill_up(
        &self,
        plant_id: f64,
        skill_id: f64,
    ) -> Result<f64> {

        let res: Packet = self.send_amf(
            "api.apiorganism.skillUp", 
            "/1",
            array(vec![number(plant_id), number(skill_id)])
        ).await?;

        let Body { data, .. } = res.bodies
            .first()
            .ok_or("response packet body is empty.")?;

        let data = data.try_as_amf0_object().ok_or(ERR_PARSE_AMF_OBJ)?;

        let now_id = data.get("now_id");

        if now_id.is_none() {
            return get_error_from_map(&data, "返回数据中无所需的Number'now_id'".into());
        }

        match now_id.unwrap() {
            &&Amf0Value::Number(now_id) => Ok(now_id),
            &&Amf0Value::String(ref now_id) => Ok(now_id.parse().unwrap()),
            _ => Err("无法将'now_id'解析为数字".into()),
        }

    }

    pub async fn skill_up_to(
        &self,
        plant_id: f64,
        mut skill_id: f64,
        until: impl Fn(usize, u32)->bool,
    ) -> Result<()> {
        'outer: for up in 0.. {
            for i in 1.. {
                if until(i, up) {
                    break 'outer;
                }
                if i != 1 {
                    wait_a_moment().await;
                }
                let new_skill_id = self.skill_up(plant_id, skill_id).await?;
                if new_skill_id != skill_id {
                    println!("\rtry {:-3} : {} -> {} !", i, skill_id, new_skill_id);
                    skill_id = new_skill_id;
                    break;
                }
                print!("\rtry {:-3} : {}", i, new_skill_id);
                std::io::stdout().flush().map_err(|e| format!("fail to flush stdout: {}", e))?;
            }
        }
        Ok(())
    }

    /// 刷新品质
    /// 
    /// **@return**: 刷新后的值
    pub async fn quality_up(
        &self,
        quality_up_type: QualityUpType,
        plant_id: f64,
    ) -> Result<Quality> {

        let res: Packet = self.send_amf(
            match quality_up_type {
                QualityUpType::General => "api.apiorganism.qualityUp",
                QualityUpType::Moshen => "api.apiorganism.quality12Up",
            }, 
            "/1",
            array(vec![number(plant_id)])
        ).await?;

        let Body { data, .. } = res.bodies
            .first()
            .ok_or("response packet body is empty.")?;

        if let Value::Amf0(Amf0Value::Object{entries, ..}) = data {
            let entries = entries.into_iter()
                .map(|p| (p.key.as_str(), &p.value));
            let data: HashMap<&str, &Amf0Value> = HashMap::from_iter(entries);
            if let Some(Amf0Value::String(newq)) = data.get("quality_name") {
                return Ok(newq.parse()?);
            }
            return get_error_from_map(&data, "未知错误：返回数据中无'quality_name'".into());
        }

        Err(ERR_PARSE_AMF_OBJ.into())
    }

    pub async fn quality_up_to(
        &self,
        quality_up_type: QualityUpType,
        plant_id: f64,
        until: impl Fn(usize, Quality) -> bool,
    ) -> Result<()> {
        println!("------ START {} ------", plant_id);
        let mut pre = None;
        for i in 1.. {
            if i != 1 {
                wait_a_moment().await;
            }
            let new_quality = self.quality_up(quality_up_type, plant_id).await?;
            if pre.is_some() && new_quality != pre.unwrap() {
                println!("\rtry {:-3} : -> {} !", i, new_quality);
            } else {
                print!("\rtry {:-3} : {}", i, new_quality);
            }
            std::io::stdout().flush().map_err(|e| format!("fail to flush stdout: {}", e))?;

            if until(i, new_quality) {
                break;
            }

            pre = Some(new_quality);
        }
        Ok(())
    }

    pub async fn open_box(
        &self,
        box_id: f64,
        amount: u32,
    ) -> Result<()> {
        let res = self.send_amf(
            "api.reward.openbox",
            "/1",
            array(vec![number(box_id), number(amount as f64)]),
        ).await?;

        if res.bodies.len() == 0 {
            Err("response packet body is empty.")?;
        }
        let ref data = res.bodies[0].data;

        let data = data.try_as_amf0_object().ok_or(ERR_PARSE_AMF_OBJ)?;

        if data.get("tools").is_some() {
            return Ok(());
        }

        get_error_from_map(&data, "未知错误：返回数据中无`tools`".into())
    }

    pub async fn open_box_repeat(
        &self,
        box_id: f64,
        amount: u32,
        repeat: usize,
    ) -> Result<()> {
        println!("warning: 提示信息有待优化");
        for i in 1..=repeat {
            if i != 1 {
                wait_a_moment().await;
            }
            self.open_box(box_id, amount).await?;
            println!("\rNo.{:-4 } 成功开启{}个", i, amount);
        }
        println!("共开启{}个", amount as usize * repeat);
        Ok(())
    }

    pub async fn get_duty_reward(
        &self,
        duty_id: f64,
        duty_catogary_id: f64
    ) -> Result<()> {
        let res = self.send_amf(
            "api.duty.reward",
            "/1",
            array(vec![number(duty_id), number(duty_catogary_id)]),
        ).await?;

        let Body { data, .. } = res.bodies
            .first()
            .ok_or("response packet body is empty.")?;

        let data = data.try_as_amf0_object().ok_or(ERR_PARSE_AMF_OBJ)?;

        if data.get("user_exp").is_some() {
            return Ok(());
        }

        get_error_from_map(&data, "未知错误：返回数据中无`user_exp`".into())
    }

    pub async fn get_duty_rewards(
        &self,
        duty_ids: impl Iterator<Item = f64>,
        duty_catogary_id: f64
    ) -> Result<()> {
        for (i, duty_id) in duty_ids.enumerate() {
            if i != 0 {
                wait_a_moment().await;
                // tokio::time::sleep(Duration::from_millis(500)).await;
            }
            let res = self.get_duty_reward(duty_id, duty_catogary_id).await;

            if let Err(e) = res {
                eprintln!("{}",e)
            } else {
                println!("get reward : {:-5} in {}", duty_id, duty_catogary_id);
            }
        }
        Ok(())
    }

    /// **@param award_type**: `"medal"` or `""`
    /// 
    /// **@return next**
    pub async fn get_fuben_award(
        &self,
        award_type: impl AsRef<str>,
        fuben_id: f64
    ) -> Result<f64> {
        let res = self.send_amf(
            "api.fuben.award",
            "/1",
            array(vec![string(award_type.as_ref()), number(fuben_id)]),
        ).await?;

        let Body { data, .. } = res.bodies
            .first()
            .ok_or("response packet body is empty.")?;

        let data = data.try_as_amf0_object().ok_or(ERR_PARSE_AMF_OBJ)?;

        match data.get("next") {
            Some(n) => n.try_as_f64().ok_or("无法将`next`解析为数字".into()),
            None => get_error_from_map(&data, "未知错误：返回数据中无`next`".into())
        }

    }

    pub async fn get_fuben_reward(
        &self,
        fuben_id: impl Into<Amf0Value>,
    ) -> Result<(usize, usize)> {
        let fuben_id: Amf0Value = fuben_id.into();
        let res = self.send_amf(
            "api.fuben.reward",
            "/1",
            array(vec![fuben_id]),
        ).await?;

        if res.bodies.len() == 0 {
            Err("response packet body is empty.")?;
        }
        let ref data = res.bodies[0].data;

        let data = data.try_as_amf0_object().ok_or(ERR_PARSE_AMF_OBJ)?;
    
        let integral = data.get("integral")
            .ok_or("未知错误：返回数据中无`integral`")?
            .try_as_number()
            .ok_or("无法将`integral`字段解析为Number")?;

        let medal = data.get("medal")
            .ok_or("返回数据中无`medal`字段")?
            .try_as_amf0_object()
            .ok_or("无法将`medal`字段解析为Object")?
            .get("amount")
            .ok_or("Object`medal`中没有`amount`")?
            .try_as_number()
            .ok_or("无法将``字段解析为Number")?;

        Ok((integral as usize, medal as usize))

    }

    pub async fn reset_fuben_reward(
        &self,
        fuben_id: f64,
    ) -> Result<()> {
        let fuben_id = format!("{}.9999999999111", fuben_id);
        self.get_fuben_reward(string(fuben_id)).await?;
        Ok(())
    }

    pub async fn reset_and_get_fuben_reward(
        &self,
        fuben_id: f64,
        times: usize,
    ) -> Result<()> {
        let (_, medal) = self.get_fuben_reward(number(fuben_id)).await?;
        println!("--- current medals: {}", medal);
        for i in 0..times {
            if i != 0 {
                wait_a_moment().await;
            }
            self.reset_fuben_reward(fuben_id).await?;
            print!("No.{:-3} : reset", i);
            std::io::stdout().flush().map_err(|e| format!("fail to flush stdout: {}", e))?;
            for j in 1.. {
                wait_a_moment().await;
                let next = self.get_fuben_award("medal", fuben_id).await?;
                print!(" : get-{}", j);
                std::io::stdout().flush().map_err(|e| format!("fail to flush stdout: {}", e))?;
                if next == 0. || next > medal as f64 {
                    println!(" : ok");
                    break;
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn get_reward(&self, awards_key: &str) -> Result<()> {
        self.send_amf(
            "api.reward.lottery",
            "/1",
            array(vec![string(awards_key)]),
        ).await?;
        Ok(())
    }

    pub async fn challenge(
        &self,
        challenge_type: ChallengeType,
        challenge_id: f64,
        plant_ids: impl Iterator<Item = f64>,
    ) -> Result<bool> {
        let target_uri = ChallengeType::get_amf_target(&challenge_type);
        let plant_ids = plant_ids.map(number).collect();
        let res = self.send_amf(
            target_uri,
            "/1",
            array(vec![number(challenge_id), array(plant_ids)]),
        ).await?;

        let Body { data, .. } = res.bodies
            .first()
            .ok_or("response packet body is empty.")?;

        let data = data.try_as_amf0_object().ok_or(ERR_PARSE_AMF_OBJ)?;

        let win = data.get("is_winning");

        if let Some(win) = win {
            win.try_as_bool().ok_or("无法将`is_winning`解析为bool")?;

            let awards_key = data.get("awards_key")
                .ok_or("返回数据中无`awards_key`")?
                .try_as_str()
                .ok_or("无法将`awards_key`解析为&str")?;

            self.get_reward(awards_key).await?;
        }

        get_error_from_map(&data, "返回数据中无`is_winning`".into())

    }

    pub async fn challenge_fuben_repeat(
        &self,
        fuben_id: f64,
        plant_ids: Vec<f64>,
        times: usize,
    ) -> Result<()> {
        use ChallengeType::*;

        for i in 1..=times {
            if i != 1 {
                wait_a_moment().await;
            }
            let plant_ids = plant_ids.iter().map(ToOwned::to_owned);
            let win = self.challenge(Fuben, fuben_id, plant_ids).await?;
            println!("repeat {:-3} : win={}", i, win);
        }
        Ok(())
    }


}

fn get_error_from_map<T>(data: &HashMap<&str,&Amf0Value>, or: ErrorKind) -> Result<T> {
    let error = data.get("desctiption").and_then(|e| e.try_as_str());

    Err(if error.is_some() {
        let error = error.unwrap();
        if error.is_ascii() {
            or // 替换为更友好的错误信息
        } else {
            error.to_owned().into()
        }
    } else {
        or
    })
}

pub struct ClientBuilder {
    server: Option<u8>,
    cookies: HashMap<String, String>,
}

impl ClientBuilder {
    pub(crate) fn new () -> Self {
        ClientBuilder {
            server: None,
            cookies: HashMap::new(),
        }
    }

    pub fn build(self) -> Result<Client> {
        use header::{REFERER,};

        let server = self.server.ok_or("您必须给定登录的服务器")?;

        let server_url = Client::resolve_server(server);
        let referer = server_url.join("main.swf").unwrap();

        let reqwest_client = 
            reqwest::Client::builder()
                .default_headers({
                    let mut headers = header::HeaderMap::new();
                    headers.append(REFERER, referer.as_str().parse().unwrap());
                    headers
                })
                .build()
                .map_err(|e|{
                    format!("internal reqwest client building error: {}", e)
                })?;

        let cookies = {
            let total_bytes = self.cookies
                .iter()
                .map(|(k,v)| k.len() + v.len() + "=".len() + ";".len())
                .sum::<usize>();
            let mut cookies = String::with_capacity(total_bytes);
            for (key, val) in self.cookies {
                cookies.push_str(&key);
                cookies.push('=');
                cookies.push_str(&val);
                cookies.push(';');
            }
            cookies
        };

        Ok(Client {
            reqwest_client,
            server,
            server_url,
            cookies,
        })
    }

    #[allow(unused_mut)]
    pub fn account(mut self, account: AccountInfo) -> Self {
        let AccountInfo { server, cookies} = account;
        self.server(server as u8)
            .cookies(cookies.into_iter())
    }

    pub fn server(mut self, server: u8) -> Self{
        self.server.replace(server);
        self
    }

    pub fn cookie(mut self, key: impl ToString, value: impl ToString) -> Self {
        self.cookies.insert(key.to_string(), value.to_string());
        self
    }

    pub fn cookies(mut self, entries: impl Iterator<Item = (String, String)>) -> Self {
        self.cookies.reserve(entries.size_hint().0);
        for (k, v) in entries {
            self.cookies.insert(k, v);
        }
        self
    }
}

pub async fn wait_a_moment() -> () {
    let duration: u64 = rand::thread_rng()
        .gen_range(800..1400);
    tokio::time::sleep(Duration::from_millis(duration)).await;
}
