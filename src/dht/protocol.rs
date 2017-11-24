// Copyright 2016 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic protocol bits for implementing custom protocols.

use super::{GenericId, DHTNode};


/// Payload in the request.
pub enum RequestPayload<TId> {
    Ping,
    FindNode(TId),
}

/// Request structure.
pub struct Request<TId, TAddr> {
    pub caller: DHTNode<TId, TAddr>,
    pub request_id: TId,
    pub payload: RequestPayload<TId>
}

/// Payload in the response.
pub enum ResponsePayload<TId, TAddr> {
    NodesFound(Vec<DHTNode<TId, TAddr>>),
    RouteFound(TAddr),
    NoResult
}

/// Response structure.
pub struct Response<TId, TAddr> {
    pub request: Request<TId, TAddr>,
    pub responder: DHTNode<TId, TAddr>,
    pub payload: ResponsePayload<TId, TAddr>
}