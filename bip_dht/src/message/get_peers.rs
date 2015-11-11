use std::borrow::{Cow, IntoCow};
use std::collections::{BTreeMap};

use bip_bencode::{Bencode, BencodeConvert, Dictionary};

use message::{self};
use message::compact_info::{CompactNodeInfo, CompactValueInfo};
use message::request::{self, RequestValidate};
use message::response::{self, ResponseValidate};
use error::{DhtResult, DhtErrorKind, DhtError};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GetPeersRequest<'a> {
    trans_id:  &'a [u8],
    node_id:   &'a [u8],
    info_hash: &'a [u8]
}

impl<'a> GetPeersRequest<'a> {
    pub fn new(trans_id: &'a [u8], node_id: &'a [u8], info_hash: &'a [u8]) -> DhtResult<GetPeersRequest<'a>> {
        let validate = RequestValidate::new(&trans_id);
        try!(validate.validate_node_id(&node_id));
        try!(validate.validate_node_id(&info_hash));
        
        Ok(GetPeersRequest{ trans_id: trans_id, node_id: node_id, info_hash: info_hash })
    }

    pub fn from_parts(rqst_root: &Dictionary<'a, Bencode<'a>>, trans_id: &'a [u8]) -> DhtResult<GetPeersRequest<'a>> {
        let validate = RequestValidate::new(trans_id);
        let node_id = try!(validate.lookup_and_convert_bytes(rqst_root, message::NODE_ID_KEY));
        let info_hash = try!(validate.lookup_and_convert_bytes(rqst_root, message::INFO_HASH_KEY));
        
        GetPeersRequest::new(trans_id, node_id, info_hash)
    }
    
    pub fn transaction_id(&self) -> &'a [u8] {
        &self.trans_id
    }
    
    pub fn node_id(&self) -> &'a [u8] {
        self.node_id
    }
    
    pub fn info_hash(&self) -> &'a [u8] {
        self.info_hash
    }
    
    pub fn encode(&self) -> Vec<u8> {
        (ben_map!{
            //message::CLIENT_TYPE_KEY => ben_bytes!(dht::CLIENT_IDENTIFICATION),
            message::TRANSACTION_ID_KEY => ben_bytes!(self.trans_id),
            message::MESSAGE_TYPE_KEY => ben_bytes!(message::REQUEST_TYPE_KEY),
            message::REQUEST_TYPE_KEY => ben_bytes!(request::GET_PEERS_TYPE_KEY),
            request::REQUEST_ARGS_KEY => ben_map!{
                message::NODE_ID_KEY => ben_bytes!(self.node_id),
                message::INFO_HASH_KEY => ben_bytes!(self.info_hash)
            }
        }).encode()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum CompactInfoType<'a> {
    Nodes(CompactNodeInfo<'a>),
    Values(CompactValueInfo<'a>),
    Both(CompactNodeInfo<'a>, CompactValueInfo<'a>)
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct GetPeersResponse<'a> {
    trans_id:  &'a [u8],
    node_id:   &'a [u8],
    // It looks like bootstrap nodes don't provide a nodes key, probably
    // because they are only used for bootstraping and not to announce to.
    token:     Option<&'a [u8]>,
    info_type: CompactInfoType<'a>
}

impl<'a> GetPeersResponse<'a> {
    pub fn new(trans_id: &'a [u8], node_id: &'a [u8], token: Option<&'a [u8]>, info_type: CompactInfoType<'a>)
        -> DhtResult<GetPeersResponse<'a>> {
        let validate = ResponseValidate::new(&trans_id);
        try!(validate.validate_node_id(&node_id));
        
        Ok(GetPeersResponse{ trans_id: trans_id, node_id: node_id, token: token, info_type: info_type })
    }

    pub fn from_parts(rsp_root: &'a Dictionary<'a, Bencode<'a>>, trans_id: &'a [u8])
        -> DhtResult<GetPeersResponse<'a>> {
        let validate = ResponseValidate::new(trans_id);
        let node_id = try!(validate.lookup_and_convert_bytes(rsp_root, message::NODE_ID_KEY));
        let token = validate.lookup_and_convert_bytes(rsp_root, message::TOKEN_KEY).ok();
        
        let maybe_nodes = validate.lookup_and_convert_bytes(rsp_root, message::NODES_KEY);
        let maybe_values = validate.lookup_and_convert_list(rsp_root, message::VALUES_KEY);
        
        // TODO: Check if nodes in the wild actually send a 2d array of bytes as values or if they
        // stick with the more compact single byte array like that used for nodes.
        let info_type = match (maybe_nodes, maybe_values) {
            (Ok(nodes), Ok(values)) => {
                let nodes_info = try!(validate.validate_nodes(nodes));
                let values_info = try!(validate.validate_values(values));
                CompactInfoType::Both(nodes_info, values_info)
            },
            (Ok(nodes), Err(_)) => {
                let nodes_info = try!(validate.validate_nodes(nodes));
                CompactInfoType::Nodes(nodes_info)
            },
            (Err(_), Ok(values)) => {
                let values_info = try!(validate.validate_values(values));
                CompactInfoType::Values(values_info)
            },
            (Err(_), Err(_)) => {
                return Err(DhtError::new(DhtErrorKind::InvalidResponse,
                    "Failed To Find nodes Or values In Node Response"))
            }
        };
        
        GetPeersResponse::new(trans_id, node_id, token, info_type)
    }
    
    pub fn transaction_id(&self) -> &'a [u8] {
        self.trans_id
    }
    
    pub fn node_id(&self) -> &'a [u8] {
        self.node_id
    }
    
    pub fn token(&self) -> Option<&'a [u8]> {
        self.token
    }
    
    pub fn info_type(&self) -> CompactInfoType<'a> {
        self.info_type
    }
    
    pub fn encode(&self) -> Vec<u8> {
        let mut response_args = BTreeMap::new();
        
        response_args.insert(message::NODE_ID_KEY, ben_bytes!(self.node_id));
        match self.token {
            Some(token) => {
                response_args.insert(message::TOKEN_KEY, ben_bytes!(token));
            },
            None => ()
        };
        
        match self.info_type {
            CompactInfoType::Nodes(nodes) => {
                response_args.insert(message::NODES_KEY, ben_bytes!(nodes.nodes()));
            },
            CompactInfoType::Values(values) => {
                response_args.insert(message::VALUES_KEY, Bencode::List(values.values().to_vec()));
            },
            CompactInfoType::Both(nodes, values) => {
                response_args.insert(message::NODES_KEY, ben_bytes!(nodes.nodes()));
                response_args.insert(message::VALUES_KEY, Bencode::List(values.values().to_vec()));
            }
        };
        
        (ben_map!{
            //message::CLIENT_TYPE_KEY => ben_bytes!(dht::CLIENT_IDENTIFICATION),
            message::TRANSACTION_ID_KEY => ben_bytes!(self.trans_id),
            message::MESSAGE_TYPE_KEY => ben_bytes!(message::RESPONSE_TYPE_KEY),
            message::REQUEST_TYPE_KEY => ben_bytes!(request::GET_PEERS_TYPE_KEY),
            response::RESPONSE_ARGS_KEY => Bencode::Dict(response_args)
        }).encode()
    }
}