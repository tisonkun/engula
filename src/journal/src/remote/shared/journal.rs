// Copyright 2022 The Engula Authors.
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

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use async_trait::async_trait;
use engula_futures::stream::batch::VecResultStream;
use futures::Stream;

use super::{Master, StreamReader, StreamWriter};
use crate::{Error, Result};

#[derive(Debug)]
pub struct EpochStateStream {}

impl Stream for EpochStateStream {
    type Item = super::EpochState;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        todo!();
    }
}

#[derive(Debug)]
pub struct Client<M: Master> {
    _master: M,
}

#[async_trait]
impl<M> crate::Journal for Client<M>
where
    M: Master + Send + Sync,
{
    type StreamLister = VecResultStream<String, Error>;
    type StreamReader = StreamReader;
    type StreamWriter = StreamWriter;

    async fn list_streams(&self) -> Result<Self::StreamLister> {
        todo!();
    }

    async fn create_stream(&self, _name: &str) -> Result<()> {
        todo!();
    }

    async fn delete_stream(&self, _name: &str) -> Result<()> {
        todo!();
    }

    async fn new_stream_reader(&self, _name: &str) -> Result<Self::StreamReader> {
        todo!();
    }

    async fn new_stream_writer(&self, _name: &str) -> Result<Self::StreamWriter> {
        todo!();
    }
}

impl<M> super::LeaderBasedJournal for Client<M>
where
    M: Master + Send + Sync,
{
    type StateStream = EpochStateStream;

    fn current_state(&self, _stream_name: &str) -> Result<super::EpochState> {
        todo!();
    }

    fn subscribe_state(&self, _stream_name: &str) -> Result<Self::StateStream> {
        todo!();
    }
}
