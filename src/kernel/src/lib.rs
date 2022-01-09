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

//! An Engula module that provides stateful environment abstractions and
//! implementations.
//!
//! # Abstraction
//!
//! [`Kernel`] is an abstraction to provide a stateful environment to storage
//! engines.
//!
//! [`Kernel`]: crate::Kernel

mod error;
mod kernel;
mod metadata;
mod update_builder;

mod local;

pub use async_trait::async_trait;
pub use engula_journal::Sequence;

pub use self::{
    error::{Error, Result},
    kernel::{Kernel, UpdateEvent, UpdateReader, UpdateWriter},
    local::MemKernel,
    metadata::{BucketUpdate, KernelUpdate},
    update_builder::{BucketUpdateBuilder, KernelUpdateBuilder},
};

#[cfg(test)]
mod tests {
    use crate::{update_builder::PartialKernelUpdateBuilder, *};

    #[tokio::test]
    async fn kernel() -> Result<()> {
        let kernel = MemKernel::open().await?;
        test_kernel(kernel).await?;
        Ok(())
    }

    async fn test_kernel<K>(kernel: K) -> Result<()>
    where
        K: Kernel,
        K::UpdateReader: Send + 'static,
    {
        let stream_name = "stream";
        let bucket_name = "bucket";
        let object_name = "object";

        let handle = {
            let expect_add_stream = KernelUpdateBuilder::default()
                .add_stream(stream_name)
                .build();
            let expect_add_bucket = KernelUpdateBuilder::default()
                .add_bucket(bucket_name)
                .build();
            let mut reader = kernel.new_update_reader().await?;
            tokio::spawn(async move {
                let update = reader.wait_next().await.unwrap();
                assert_eq!(update, (1, expect_add_stream));
                let update = reader.wait_next().await.unwrap();
                assert_eq!(update, (2, expect_add_bucket));
            })
        };
        kernel.create_stream(stream_name).await.unwrap();
        kernel.create_bucket(bucket_name).await.unwrap();
        handle.await.unwrap();

        // check if stream and bucket successfully created
        kernel.new_stream_writer(stream_name).await.unwrap();
        kernel
            .new_sequential_writer(bucket_name, object_name)
            .await
            .unwrap();

        let update = PartialKernelUpdateBuilder::default()
            .put_meta("a", "b")
            .remove_meta("b")
            .update_bucket(
                bucket_name,
                BucketUpdateBuilder::default()
                    .add_object(object_name, vec![0])
                    .build(),
            )
            .build();

        let handle = {
            let expect = update.clone();
            let mut reader = kernel.new_update_reader().await?;
            tokio::spawn(async move {
                let update = reader.wait_next().await.unwrap();
                assert_eq!(update, (3, expect.into()));
            })
        };

        let mut writer = kernel.new_update_writer().await?;
        writer.append(update).await?;
        handle.await.unwrap();

        Ok(())
    }
}
