// Copyright 2021 The Engula Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

tonic::include_proto!("engula.journal.v1");

use crate::{Error, Timestamp};

pub fn serialize_ts(ts: &Timestamp) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(ts).map_err(Error::from)
}

pub fn deserialize_ts(v: &[u8]) -> Result<Timestamp, Error> {
    serde_json::from_slice(v).map_err(Error::from)
}