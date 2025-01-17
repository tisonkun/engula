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

use std::path::PathBuf;

use object_engine_filestore::SequentialWrite;
use object_engine_master::{proto::*, Bucket, Master, Tenant};

use crate::{async_trait, Result};

#[derive(Clone)]
pub struct Env {
    master: Master,
}

impl Env {
    pub async fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let master = Master::open(path).await?;
        Ok(Self { master })
    }
}

#[async_trait]
impl super::Env for Env {
    type TenantEnv = TenantEnv;

    async fn tenant(&self, name: &str) -> Result<Self::TenantEnv> {
        let tenant = self.master.tenant(name).await?;
        Ok(TenantEnv { tenant })
    }

    async fn handle_batch(&self, req: BatchRequest) -> Result<BatchResponse> {
        self.master.handle_batch(req).await
    }
}

#[derive(Clone)]
pub struct TenantEnv {
    tenant: Tenant,
}

#[async_trait]
impl super::TenantEnv for TenantEnv {
    type BucketEnv = BucketEnv;

    fn name(&self) -> &str {
        self.tenant.name()
    }

    async fn bucket(&self, name: &str) -> Result<Self::BucketEnv> {
        let bucket = self.tenant.bucket(name).await?;
        Ok(BucketEnv { bucket })
    }
}

#[derive(Clone)]
pub struct BucketEnv {
    bucket: Bucket,
}

#[async_trait]
impl super::BucketEnv for BucketEnv {
    fn name(&self) -> &str {
        self.bucket.name()
    }

    fn tenant(&self) -> &str {
        self.bucket.tenant()
    }

    async fn new_sequential_writer(&self, name: &str) -> Result<Box<dyn SequentialWrite>> {
        self.bucket.new_sequential_writer(name).await
    }
}
