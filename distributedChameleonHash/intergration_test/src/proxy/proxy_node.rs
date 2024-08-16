use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LinesCodec};
use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use std::thread::sleep;
use std::time::Duration;
use std::env;
use log::{error, info, warn};

use proxy::communication::communication::*;
use message::proxy::common_msg::{KeyGenMsg, SetupMsg};
use proxy::proxy::Proxy;
use proxy::config::config::Config;
use message::node::setup_msg::{NodeToProxySetupPhaseP2PMsg,NodeSetupPhaseFinishFlag};
use message::node::keygen_msg::NodeKeyGenPhaseOneBroadcastMsg;
use message::common_msg::GSTBKMsg;



#[tokio::main]
pub async fn main () -> Result<(), anyhow::Error> 
{
    // 初始化 日志记录器
    let log_path = String::from(env::current_dir().unwrap().as_path().to_str().unwrap())+"/src/proxy/config/config_file/log4rs.yaml";
    log4rs::init_file(log_path, Default::default()).unwrap();
    
    // 初始化
    let gs_tbk_config_path  = String::from(std::env::current_dir().unwrap().as_path().to_str().unwrap())+"/src/proxy/config/config_file/proxy_config.json";
    let gs_tbk_config:Config = serde_json::from_str(&Config::load_config(&gs_tbk_config_path)).unwrap();
    let proxy = Proxy::init(gs_tbk_config);

    // 创建setup阶段的一些共享变量
    let shared_node_setup_p2p_msg_vec = Arc::new(TokioMutex::new(Vec::<NodeToProxySetupPhaseP2PMsg>::new()));
    let setup_msg_num = Arc::new(TokioMutex::new(0));
    let setup_finish_num = Arc::new(TokioMutex::new(0));
    let shared_node_setup_finish_vec = Arc::new(TokioMutex::new(Vec::<NodeSetupPhaseFinishFlag>::new()));
    // 创建keygen阶段的共享变量
    let shared_get_s_msg_vec = Arc::new(TokioMutex::new(Vec::<NodeKeyGenPhaseOneBroadcastMsg>::new()));
    // 开启代理的监听端口
    let proxy_addr:SocketAddr = proxy.address.parse()?;
    let listener = TcpListener::bind(proxy_addr).await?;
    info!("Proxy_node is listening on {}",proxy_addr);
    let shared_proxy = Arc::new(TokioMutex::new(proxy));// 定义共享
    
    // 循环接收消息
    while let Result::Ok(( tcp_stream,_)) = listener.accept().await 
    {
        // 拷贝共享代理结构体
        let proxy_clone = shared_proxy.clone();

        // 拷贝共享变量
        let shared_node_setup_p2p_msg_vec_clone = shared_node_setup_p2p_msg_vec.clone();
        let msg_num_clone = setup_msg_num.clone();                            
        let finish_num_clone = setup_finish_num.clone();
        let node_setup_finish_vec_clone = shared_node_setup_finish_vec.clone();
        let get_s_msg_vec_clone = shared_get_s_msg_vec.clone();
        //let open_two_vec_clone = shared_ntp_open_two_vec.clone();
        tokio::spawn(async move
        {
            let proxy = proxy_clone.clone();
            //接收并拆分出消息
            let framed = Framed::new( tcp_stream,LinesCodec::new());
            let message = match get_message(framed).await 
            {
                Ok(v) => v,
                Err(e) => 
                {
                    error!("Failed to get node's message: {:?}",e);
                    return ;
                } 
            };
            //对不同的消息类型做处理
            match message 
            {
                GSTBKMsg::GSTBKMsgN(gstbkn_msg) => 
                {
                    match gstbkn_msg 
                    {
                        message::node::common_msg::GSTBKMsg::SetupMsg(setup_msg) =>  
                        { 
                            match setup_msg 
                            {
                                message::node::common_msg::SetupMsg::NodeToProxySetupPhaseP2PMsg(msg) => 
                                {
                                    info!("From Role : {}, Get NodeToProxySetupPhaseP2PMsg", msg.role);
                                    let node_setup_p2p_msg_vec = shared_node_setup_p2p_msg_vec_clone.clone();
                                    let msg_num = msg_num_clone.clone(); 
                                    let mut locked_proxy = proxy.lock().await;                           
                                    handle_setup_msg(msg,&node_setup_p2p_msg_vec,&msg_num).await;
                                    //判断收到的消息是否达到了n
                                    if *msg_num.lock().await == (locked_proxy.threashold_param.share_counts as i32) 
                                    {
                                        let duration = Duration::from_secs(1);
                                        sleep(duration); 
                                        //生成proxy_setup_msg 
                                        let msg_vec = (*node_setup_p2p_msg_vec.lock().await).clone();
                                        let setup_msg_str = setup_to_gstbk(SetupMsg::ProxySetupPhaseBroadcastMsg(locked_proxy.setup_phase_one(msg_vec)));
                                        //广播
                                        let node_list = locked_proxy.node_info_vec.clone().unwrap(); 
                                        match broadcast(setup_msg_str, node_list).await{
                                            Ok(_) => 
                                            {
                                                //println!("ProxySetupBroadcastMsg have send");
                                            }
                                            Err(e) => 
                                            {
                                                error!("Error!: {}, ProxySetupBroadcastMsg can not send ",e);
                                                return ;
                                            }
                                        };
                                    }
                                    else 
                                    {
                                        warn!("Insufficient number of messages, and current number is {:?}", msg_num);
                                        return;
                                    }
                                }
                                message::node::common_msg::SetupMsg::NodeSetupPhaseFinishFlag(msg) => 
                                {
                                    info!("From id : {}, Role : {}, Get NodeSetupPhaseFinishFlag",msg.sender,msg.role);
                                    let node_setup_finish_vec = node_setup_finish_vec_clone.clone();
                                    let finish_num = finish_num_clone.clone();
                                    let locked_proxy = proxy.lock().await;
                                    handle_setup_tag(msg,&node_setup_finish_vec,&finish_num).await;
                                    //判断是否所有节点都发了
                                    if *finish_num.lock().await == (locked_proxy.threashold_param.share_counts as i32) 
                                    {
                                        let setup_finish_flag_str = setup_to_gstbk(SetupMsg::ProxySetupPhaseFinishFlag(locked_proxy.setup_phase_two((*node_setup_finish_vec.lock().await).clone())));
                                        //广播
                                        let node_list = locked_proxy.node_info_vec.clone().unwrap(); 
                                        match broadcast(setup_finish_flag_str, node_list).await
                                        {
                                            Ok(_) => {
                                                //println!("ProxySetupFinishMsg have send");
                                            }
                                            Err(e) => {
                                                error!("Error: {}, ProxySetupFinishMsg can not sent ",e);
                                                return ;
                                            }
                                        };
                                    }
                                    else 
                                    {
                                        warn!("Insufficient number of messages, and current number is {:?}", finish_num);
                                        return;
                                    }//let s_total_msg = locked_proxy.keygen_get_s(&msg_vec).unwrap();
                                }
                                
                            }
                        }
                        message::node::common_msg::GSTBKMsg::KeyGenMsg(keygen_msg) => 
                        {
                            match keygen_msg {
                                message::node::common_msg::KeyGenMsg::NodeKeyGenPhaseOneBroadcastMsg(msg) => 
                                {
                                    info!("From id : {}, Role : {}, Get NodeKeyGenPhaseOneBroadcastMsg",msg.sender,msg.role);
                                    let get_s_msg_vec = get_s_msg_vec_clone.clone();
                                    let mut msg_vec = get_s_msg_vec.lock().await;
                                    let mut locked_proxy = proxy.lock().await;
                                    msg_vec.push(msg);
                                    if msg_vec.len() == locked_proxy.threashold_param.share_counts as usize {
                                        let s_total_msg = keygen_to_gstbk(KeyGenMsg::ProxyToNodeGetSTotal(locked_proxy.keygen_get_s(&msg_vec).unwrap()));
                                        let node_list = locked_proxy.node_info_vec.clone().unwrap(); 
                                        match broadcast(s_total_msg, node_list).await{
                                            Ok(_) => 
                                            {
                                                //println!("ProxySetupBroadcastMsg have send");
                                            }
                                            Err(e) => 
                                            {
                                                error!("Error!: {}, ProxyToNodeGetSTotal can not send ",e);
                                                return ;
                                            }
                                        };
                                    }
                                }
                                _ => 
                                {

                                }  
                            }
                        }
                    }
                }
                _ => 
                {

                }
                
            }
        });
    }
    Ok(())
}

//test
#[test]
fn test() 
{
   match main() 
   {
    Ok(_) =>
    {
        info!("Ok");
    }
    Err(_) => 
    {
        error!("No");
    }
   };
}

 