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

use std::sync::{atomic::AtomicU64, Arc};

use engula_journal::Journal;
use engula_storage::Storage;
use tokio::sync::broadcast;

use super::{update_reader::UpdateReader, update_writer::UpdateWriter};
use crate::{async_trait, KernelUpdate, Result, Sequence, UpdateReader as _};

pub struct Kernel<J, S> {
    journal: Arc<J>,
    storage: Arc<S>,
    sequence: Arc<AtomicU64>,
    update_tx: broadcast::Sender<(Sequence, KernelUpdate)>,
}

impl<J, S> Kernel<J, S>
where
    J: Journal + Send + Sync + 'static,
    S: Storage + Send + Sync + 'static,
{
    pub async fn init(journal: J, storage: S) -> Result<Self> {
        let (update_tx, update_rx) = broadcast::channel(1024);
        let journal = Arc::new(journal);
        let storage = Arc::new(storage);
        let reader = UpdateReader::new(update_rx);
        Self::subscribe_updates(reader, journal.clone(), storage.clone());
        Ok(Self {
            journal,
            storage,
            sequence: Arc::new(AtomicU64::new(0)),
            update_tx,
        })
    }

    fn subscribe_updates(mut reader: UpdateReader, journal: Arc<J>, storage: Arc<S>) {
        tokio::spawn(async move {
            loop {
                let (_, update) = reader.wait_next().await.unwrap();
                for stream in &update.add_streams {
                    journal.create_stream(stream).await.unwrap();
                }
                for stream in &update.remove_streams {
                    journal.delete_stream(stream).await.unwrap();
                }
                for bucket in &update.add_buckets {
                    storage.create_bucket(bucket).await.unwrap();
                }
                for bucket in &update.remove_buckets {
                    storage.delete_bucket(bucket).await.unwrap();
                }
            }
        });
    }
}

#[async_trait]
impl<J, S> crate::Kernel for Kernel<J, S>
where
    J: Journal + Send + Sync + 'static,
    S: Storage + Send + Sync + 'static,
{
    type RandomReader = S::RandomReader;
    type SequentialWriter = S::SequentialWriter;
    type StreamReader = J::StreamReader;
    type StreamWriter = J::StreamWriter;
    type UpdateReader = UpdateReader;
    type UpdateWriter = UpdateWriter;

    async fn new_update_reader(&self) -> Result<Self::UpdateReader> {
        let reader = UpdateReader::new(self.update_tx.subscribe());
        Ok(reader)
    }

    async fn new_update_writer(&self) -> Result<Self::UpdateWriter> {
        let writer = UpdateWriter::new(self.sequence.clone(), self.update_tx.clone());
        Ok(writer)
    }

    async fn new_stream_reader(&self, stream_name: &str) -> Result<Self::StreamReader> {
        let reader = self.journal.new_stream_reader(stream_name).await?;
        Ok(reader)
    }

    async fn new_stream_writer(&self, stream_name: &str) -> Result<Self::StreamWriter> {
        let writer = self.journal.new_stream_writer(stream_name).await?;
        Ok(writer)
    }

    async fn new_random_reader(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> Result<Self::RandomReader> {
        let reader = self
            .storage
            .new_random_reader(bucket_name, object_name)
            .await?;
        Ok(reader)
    }

    async fn new_sequential_writer(
        &self,
        bucket_name: &str,
        object_name: &str,
    ) -> Result<Self::SequentialWriter> {
        let writer = self
            .storage
            .new_sequential_writer(bucket_name, object_name)
            .await?;
        Ok(writer)
    }
}
