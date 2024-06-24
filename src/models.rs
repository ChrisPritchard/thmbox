
#[derive(serde::Deserialize)]
pub struct RunningResponse {
    pub status: String,
    pub data: Vec<VmData>
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