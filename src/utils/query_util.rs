use std::net::TcpStream;
use capnp::capability::Promise;
use capnp_rpc::{rpc_twoparty_capnp, twoparty, RpcSystem};

pub(crate) enum Query {
    IoTDevice,
}

pub(crate) struct IoTDevice {
    pub(crate) name: String,
    pub(crate) behavior: String,
}

pub(crate) struct QueryManager {
    query_server_address: String,
}

impl QueryManager {
    pub(crate) fn new(query_server_address: String) -> Self {
        Self {
            query_server_address,
        }
    }

    pub(crate) fn query_iot_device(&self) {
        let query_server_address = self.query_server_address.clone();
        let stream = TcpStream::connect(query_server_address).unwrap();
        stream.set_nodelay(true).unwrap();
    }

    pub(crate) fn query_catalogue_order(&self) {
        let query_server_address = self.query_server_address.clone();
        let stream = TcpStream::connect(query_server_address).unwrap();
        stream.set_nodelay(true).unwrap();
    }

}
