use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use curv::arithmetic::traits::*;
use curv::cryptographic_primitives::secret_sharing::feldman_vss::ShamirSecretSharing;
use curv::elliptic::curves::{Secp256k1, Point, Scalar};
pub type FE = Scalar<Secp256k1>;
pub type GE = Point<Secp256k1>;
use curv::BigInt;
use log::info;
use num_bigint::BigUint;
use cl_encrypt::cl::clwarpper::*;
use utilities::code::code::sha256;
use utilities::vss::vss::{pedersen_share, Vss};
use utilities::zkp::zkp::DlPcProof;
use crate::Error::{self, InvalidSS};
use message::proxy::keygen_msg::{ProxyToNodeKeyGenPhaseFiveP2PMsg, ProxyToNodeKeyGenPhaseThreeP2PMsg};
use message::node::keygen_msg::{ NodeKeyGenPhaseOneBroadcastMsg, NodeToProxyKeyGenPhaseFourP2PMsg, NodeToProxyKeyGenPhaseTwoP2PMsg};
use crate::node::Node;

impl Node { 
    pub fn keygen_phase_one(&mut self) -> NodeKeyGenPhaseOneBroadcastMsg
    {
        info!("Key is generating!");
        let secret = FE::random();
        let random = FE::random();
        let commit = Point::generator() * &secret + Point::base_point2() * &random;//g_ui

        self.secret = secret;
        self.random = random;
        self.commit = commit.clone();

        NodeKeyGenPhaseOneBroadcastMsg
        {
            sender:self.id.unwrap(),
            role:self.role.clone(),
            commit
        }
    }

    pub fn keygen_phase_two(&mut self, msg_vec:&Vec<NodeKeyGenPhaseOneBroadcastMsg>)
    -> Result<NodeToProxyKeyGenPhaseTwoP2PMsg, Error>
    {
        assert_eq!(msg_vec.len(), self.threashold_param.share_counts as usize);

        // Merge and save y,y_i_map
        let mut commit_map:HashMap<u16, Point<Secp256k1>> = HashMap::new();
        for msg in msg_vec
        {
            commit_map.insert(msg.sender, msg.commit.clone());
        }
        self.commit_map = commit_map;

        let(vss, pedersen_shares, coefficients) = pedersen_share(self.threashold_param.threshold, self.threashold_param.share_counts, &self.secret, &self.random);
        self.commitments = vss.commitments.clone();
        self.secret_coefficients = coefficients.secret_coefficients;
        self.random_coefficients = coefficients.random_coefficients;

        let secret_shares = pedersen_shares.secret_shares.clone();
        let random_shares = pedersen_shares.random_shares.clone();

        let random_s_str = FE::random().to_bigint().to_string();
        let random_r_str = FE::random().to_bigint().to_string();

        let mut share_enc_map:HashMap<u16, String> = HashMap::new();
        let mut random_enc_map:HashMap<u16, String> = HashMap::new();
        let mut t_vec = String::new();
        let mut pk_vec = String::new();
        // let mut y = String::new();
        let mut t_total = BigInt::zero();
        let mut st = FE::zero();
        let mut rt = FE::zero();
        let g = GE::generator();
        let h = GE::base_point2();

        let mut s_enc_t_vec = Vec::new();
        let mut r_enc_t_vec = Vec::new();

        for node in self.node_info_vec.as_ref().unwrap()
        { 
            let id = node.id; 
            // share 1~n, vec 0~n-1
            let share = &secret_shares[id as usize - 1];
            let random = &random_shares[id as usize - 1];
            let share_str = share.to_bigint().to_string();
            let random_str = random.to_bigint().to_string();
            
            let share_enc = encrypt(node.cl_pk.clone(), share_str.clone(), random_s_str.clone());
            let random_enc = encrypt(node.cl_pk.clone(), random_str.clone(), random_r_str.clone());
            pk_vec = pk_vec + &node.cl_pk.clone() + ";";

            let t_bn = BigInt::from_hex(&sha256(&(share_enc.clone() + &random_enc.clone()))).unwrap();
            t_vec = t_vec + &t_bn.to_string() + ":";
            t_total = t_total + t_bn.clone();
            let t_fe = FE::from_bigint(&t_bn);
            st = st + share * t_fe.clone();
            rt = rt + random * t_fe;

            s_enc_t_vec.push(scal_ciphertexts(share_enc.clone(), t_bn.to_string()));
            r_enc_t_vec.push(scal_ciphertexts(random_enc.clone(), t_bn.to_string()));

            share_enc_map.insert(id, share_enc);
            random_enc_map.insert(id, random_enc);
        }

        let pc = g * st.clone() + h * rt.clone();
        t_vec.pop();
        pk_vec.pop();
        let s_sum = s_enc_t_vec.iter().skip(1).fold(s_enc_t_vec.get(0).unwrap().to_string(), |acc,v|{add_ciphertexts(acc, v.to_string())});
        let r_sum = r_enc_t_vec.iter().skip(1).fold(r_enc_t_vec.get(0).unwrap().to_string(), |acc,v|{add_ciphertexts(acc, v.to_string())});

        let enc_proof = batch_cl_pc_proof(to_hex(pc.to_bytes(true).as_ref()), s_sum.clone(), r_sum.clone(),  to_hex(h.to_bytes(true).as_ref()), random_s_str, random_r_str, st.to_bigint().to_string(), rt.to_bigint().to_string(), t_total.to_string(), t_vec.clone(), pk_vec.clone());
        
        Ok
        (
            NodeToProxyKeyGenPhaseTwoP2PMsg
            {
                sender:self.id.unwrap(),
                role:self.role.clone(),
                share_enc_map,
                random_enc_map,
                enc_proof,
                vss,
            }
        ) 
    }

    pub fn keygen_phase_four(&mut self, msg:ProxyToNodeKeyGenPhaseThreeP2PMsg) -> Result<NodeToProxyKeyGenPhaseFourP2PMsg, Error>
    {
        // Decrypt CL share
        let share = decrypt(self.cl_keypair.sk.clone(), msg.share_enc_sum);
        let random = decrypt(self.cl_keypair.sk.clone(), msg.random_enc_sum);
        let mut share_ = String::new();
        let mut random_ = String::new();
        if let Ok(decimal_num) = share.parse::<BigUint>() {
            share_ = format!("{:x}", decimal_num);
        }else{
        }
        if let Ok(decimal_num) = random.parse::<BigUint>() {
            random_ = format!("{:x}", decimal_num);
        }else{
        }
        let share_fe = FE::from(BigInt::from_hex(share_.as_str()).unwrap());
        let random_fe = FE::from(BigInt::from_hex(random_.as_str()).unwrap());
        // verify coefficient commitment
        if msg.vss_scheme_sum.pedersen_validate_share(&share_fe, &random_fe, self.id.unwrap()).is_ok()
        {
            self.sk = share_fe;
            let commitments = self.commitments.clone();
            let secret_coefficients = self.secret_coefficients.clone();
            let random_coefficients = self.random_coefficients.clone();
            let g = Point::generator();
            let s_co_g= (0..secret_coefficients.len()).map(|i| g * secret_coefficients.get(i).unwrap()).collect::<Vec<GE>>();
            let mut bt = FE::zero(); 
            let mut pc = Point::zero();
            let mut dl = Point::zero();
            for i in 0..commitments.len(){
                let t_bn = BigInt::from_hex(&sha256(&(to_hex(commitments[i].to_bytes(true).as_ref()) + &to_hex(s_co_g[i].to_bytes(true).as_ref()).clone()))).unwrap();
                let t_fe = FE::from_bigint(&t_bn);
                pc = pc + commitments[i].clone() * t_fe.clone();
                dl = dl + s_co_g[i].clone() * t_fe.clone();
                bt = bt + random_coefficients[i].clone() * t_fe.clone();
            }

            let dlog_proof = DlPcProof::dl_pc_proof(pc.clone(), dl.clone(), bt);
            let res = dlog_proof.dl_pc_verify(pc, dl);
            info!("dl verify res: {}", res);

            Ok(
                NodeToProxyKeyGenPhaseFourP2PMsg
                {
                    sender:self.id.unwrap(),
                    role:self.role.clone(),
                    s_coefficients_log: s_co_g,
                    dlog_proof
                }
            )
        }
        else
        {
            Err(InvalidSS)
        }
  
    }

    pub fn keygen_phase_six(&mut self, msg:ProxyToNodeKeyGenPhaseFiveP2PMsg) -> Result<(), Error>
    {
        let vss_scheme = Vss{ 
            parameters: 
            ShamirSecretSharing 
            { 
                threshold: self.threashold_param.threshold, 
                share_count: self.threashold_param.share_counts 
            }, 
            commitments: msg.total_s_coefficients_log.clone()
       };
       if vss_scheme.validate_share(&self.sk, self.id.unwrap()).is_ok()
       {
            let keypair_str = serde_json::to_string(&msg.total_s_coefficients_log[0]).unwrap() + "\n" + &self.sk.to_bigint().to_hex();

            let mut pk_share_vec = Vec::with_capacity(self.threashold_param.share_counts as usize);
            pk_share_vec.extend((1..=self.threashold_param.share_counts).map(|i| vss_scheme.get_point_commitment(i)));

            let pk_share_str = serde_json::to_string(&pk_share_vec).unwrap();
            
            let id = self.id.unwrap().to_string();

            let keypair_path = format!("/home/arlo/CH/keypair/keypair{}.txt", id);
            Self::write_to_file(&keypair_str, &keypair_path, "keypair");

            let pk_share_path = format!("/home/arlo/CH/pk_share/pk_share{}.txt", id);
            Self::write_to_file(&pk_share_str, &pk_share_path, "pk share");

            info!("distributed key is generated!");
            Ok(
                ()
            )
       }
       else 
       {
           Err(InvalidSS)
       }
    }

    fn write_to_file(data: &str, file_path: &str, description: &str) {
        let path = Path::new(file_path);
        let mut file = match File::create(path) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to create {}: {}", description, err);
                return;
            }
        };
    
        if let Err(err) = file.write_all(data.as_bytes()) {
            eprintln!("Failed to write to {}: {}", description, err);
        } else {
            println!("{} write in {}", description, file_path);
        }
    }
}

