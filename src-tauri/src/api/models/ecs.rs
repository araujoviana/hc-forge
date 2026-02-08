use serde::Serialize;

#[derive(Serialize)]
pub struct CreateEcsRequest {
    pub server: Server,
}

#[derive(Serialize)]
pub struct Server {
    pub name: String,

    #[serde(rename = "imageRef")]
    pub image_ref: String,

    #[serde(rename = "flavorRef")]
    pub flavor_ref: String,

    pub vpcid: String,
    pub nics: Vec<Nic>,

    #[serde(rename = "root_volume")]
    pub root_volume: RootVolume,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publicip: Option<PublicIp>,
}

#[derive(Serialize)]
pub struct Nic {
    pub subnet_id: String,
}

#[derive(Serialize)]
pub struct RootVolume {
    pub volumetype: String,
    pub size: u32,
}

#[derive(Serialize)]
pub struct PublicIp {
    pub eip: Eip,
}

#[derive(Serialize)]
pub struct Eip {
    #[serde(rename = "ip_type")]
    pub ip_type: String,
    pub bandwidth: Bandwidth,
}

#[derive(Serialize)]
pub struct Bandwidth {
    pub size: u32,

    #[serde(rename = "share_type")]
    pub share_type: String,

    #[serde(rename = "charge_mode")]
    pub charge_mode: String,
}
