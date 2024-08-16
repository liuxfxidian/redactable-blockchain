use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo{
    pub id: u16,// assigned id
    pub address: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxySetupPhaseBroadcastMsg{
    pub node_info_vec: Vec<NodeInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxySetupPhaseFinishFlag
{
    pub sender:u16,
    pub role: String
}