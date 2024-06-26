use chrono::{DateTime, Utc};


#[derive(serde::Deserialize)]
pub struct RunningResponse {
    pub status: String,
    pub message: Option<String>,
    pub data: Option<Vec<VmData>>
}

#[derive(serde::Deserialize)]
pub struct VmData {
    pub id: String,
    pub title: String,
    pub expires: String,
    #[serde(alias = "internalIP")]
    pub internal_ip: String,
    pub credentials: Option<VmCredentials>,
    pub remote: VmRemote
}

impl VmData {
    pub fn minutes_remaining(self: &Self) -> i64 {
        let expires_parsed = DateTime::parse_from_rfc3339(&self.expires)
            .expect("Failed to parse ISO date")
            .with_timezone(&Utc);

        let now = Utc::now();
        let duration = expires_parsed.signed_duration_since(now);
        
        duration.num_minutes()
    }
}

#[derive(serde::Deserialize)]
pub struct VmCredentials {
    pub username: String,
    pub password: String
}

#[derive(serde::Deserialize)]
pub struct VmRemote {
    #[serde(alias = "privateIP")]
    pub private_ip: Option<String>
}