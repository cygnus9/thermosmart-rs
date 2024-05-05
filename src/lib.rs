use bytes::Buf;
use reqwest::{redirect::Policy, Client, Url};
use url::ParseError;

pub struct Thermostat {
    client: Client,
    base_url: Url,
}

impl Thermostat {
    pub fn new(endpoint: &str) -> Self {
        let base_url = Url::parse(endpoint)
            .or_else(|e| match e {
                ParseError::RelativeUrlWithoutBase => Url::parse(&format!("http://{endpoint}")),
                err => Err(err),
            })
            .unwrap();
        Self {
            client: Client::builder()
                .redirect(Policy::limited(1))
                .build()
                .unwrap(),
            base_url,
        }
    }

    pub async fn get_status(&self) -> RawThermostatData {
        let body = self
            .client
            .get(self.base_url.join("thermostat.xml").unwrap())
            .send()
            .await
            .unwrap()
            .bytes()
            .await
            .unwrap();
        serde_xml_rs::from_reader(body.reader()).unwrap()
    }
}

#[allow(unused)]
#[derive(serde::Deserialize, Debug)]
pub struct RawThermostatData {
    pub setpoint: f64,
    pub temperature: f64,
    #[serde(deserialize_with = "de::opt_f64")]
    pub outside: Option<f64>,
    pub pause: bool,
    pub heating: bool,
}

mod de {
    use serde::{de::Error, Deserialize, Deserializer};

    pub fn opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value == "---" {
            Ok(None)
        } else if let Ok(value) = value.parse() {
            Ok(Some(value))
        } else {
            Err(D::Error::custom(format!(
                "Unable to parse '{value}' as optional float"
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_message() {
        let xml_message = r#"<?xml version="1.0" encoding="utf-8"?>
        <thermostat>
                <setpoint>14.0</setpoint>
                <temperature>20.3</temperature>
                <outside>---</outside>
                <pause>1</pause>
                <heating>0</heating>
        </thermostat>"#;

        let message: RawThermostatData = serde_xml_rs::from_str(xml_message).unwrap();
        println!("{message:?}");
    }
}
