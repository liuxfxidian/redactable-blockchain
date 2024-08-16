use message::proxy::setup_msg::NodeInfo;
use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use tokio_util::codec::{Framed, LinesCodec};
use tokio_stream::StreamExt;
use std::net::SocketAddr;
use log::{error, warn};
use message::common_msg::GSTBKMsg;

///接收并序列化消息
pub async fn get_message(mut framed:Framed<TcpStream,LinesCodec>) -> Result<GSTBKMsg, Box<dyn std::error::Error>>
{
    let message = match framed.next().await 
    {
        Some(Ok(m)) => m,
        Some(Err(e)) => 
        {
            error!("Failed to get from framed: {:?}",e);
            return Err(Box::new(e));
        }
        None => 
        {
            error!("Failed to get a message.");
            return Err("Failed to get a message.".into());
        }
    };
    let result: Result<GSTBKMsg,_> =  serde_json::from_str(&message);
    let msg = match result 
    {
        Ok(v) => v,
        Err(e) => 
        {
            error!("Error deserializing JSON: {:?}", e);
            return Err(Box::new(e));
        }
    };
    return  Ok(msg);
}

/// 序列化setup阶段的消息
pub fn setup_to_gstbk(msg_setup : message::node::common_msg::SetupMsg) -> String 
{
    let msg_str = serde_json::to_string(&message::common_msg::GSTBKMsg::GSTBKMsgN(message::node::common_msg::GSTBKMsg::SetupMsg(msg_setup))).unwrap();
    return msg_str;
}

///Keygen阶段序列化消息
pub fn keygen_to_gstbk(msg_keygen : message::node::common_msg::KeyGenMsg)->String
{
    let msg_str = serde_json::to_string(&message::common_msg::GSTBKMsg::GSTBKMsgN(message::node::common_msg::GSTBKMsg::KeyGenMsg(msg_keygen))).unwrap();
    return msg_str;
}

///p2p信道
pub async fn p2p(msg : String,str_add : String) -> Result<(), anyhow::Error> 
{
    let add : SocketAddr = str_add.parse()?;
    let mut tcp_stream = TcpStream::connect(add).await?;
    tcp_stream.write_all(msg.as_bytes()).await?; 
    tcp_stream.shutdown().await?;
    Ok(())
}

///node之间的p2p信到
pub async fn to_node(msg : String,id : u16,node_list : Vec<NodeInfo>) -> Result<(), anyhow::Error> 
{
    if let Some(node) = node_list.iter().find(|&node_info| node_info.id == id) 
    {
        let add : SocketAddr = node.address.parse()?;
        let mut node_stream = TcpStream::connect(add).await?;
        node_stream.write_all(msg.as_bytes()).await?;
        node_stream.shutdown().await?;
    }
    else 
    {
        warn!("Nodelist with id {} not found.", id);
    }
    Ok(())
}

///广播信道
pub async fn broadcast(msg : String,node_list : Vec<NodeInfo>,node_id : u16) -> Result<(), anyhow::Error> 
{
    for node in node_list 
    {
            if node_id == node.id  
            {
                continue;
            }
            else 
            {
                let add : SocketAddr = node.address.parse()?;
                let mut tcp_stream = TcpStream::connect(add).await?;
                tcp_stream.write_all(msg.as_bytes()).await?; 
                tcp_stream.shutdown().await?;
            }    
    }
    Ok(())
}
