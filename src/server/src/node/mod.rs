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

pub mod group_engine;
pub mod replica;
mod route_table;
pub mod state_engine;

use std::{collections::HashMap, sync::Arc};

use engula_api::server::v1::GroupDesc;
use futures::lock::Mutex;
use tracing::debug;

pub use self::{
    group_engine::GroupEngine,
    replica::Replica,
    route_table::{RaftRouteTable, ReplicaRouteTable},
    state_engine::StateEngine,
};
use crate::{runtime::Executor, serverpb::v1::ReplicaState, Result};

#[derive(Clone)]
struct ReplicaInfo {
    group_id: u64,
    state: ReplicaState,
}

/// A structure holds the states of node. Eg create replica.
#[derive(Default)]
struct NodeState
where
    Self: Send,
{
    replicas: HashMap<u64, ReplicaInfo>,
}

#[allow(unused)]
#[derive(Clone)]
pub struct Node
where
    Self: Send + Sync,
{
    raw_db: Arc<rocksdb::DB>,
    executor: Executor,
    state_engine: StateEngine,
    raft_route_table: RaftRouteTable,
    replica_route_table: ReplicaRouteTable,

    /// `NodeState` of this node, the lock is used to ensure serialization of create/terminate
    /// replica operations.
    node_state: Arc<Mutex<NodeState>>,
}

#[allow(unused)]
impl Node {
    pub fn new(raw_db: Arc<rocksdb::DB>, state_engine: StateEngine, executor: Executor) -> Self {
        Node {
            raw_db,
            executor,
            state_engine,
            raft_route_table: RaftRouteTable::new(),
            replica_route_table: ReplicaRouteTable::new(),
            node_state: Arc::new(Mutex::new(NodeState::default())),
        }
    }

    pub async fn recover(&self) -> Result<()> {
        let mut node_state = self.node_state.lock().await;
        debug_assert!(
            node_state.replicas.is_empty(),
            "some replicas are serving before recovery?"
        );

        let it = self.state_engine.iterate_replica_states().await;
        for (group_id, replica_id, state) in it {
            if self
                .start_replica_with_state(group_id, replica_id, state)
                .await?
                .is_none()
            {
                panic!(
                    "metadata is inconsistent, group {} replica {} not exists",
                    group_id, replica_id
                );
            };
            let replica_info = ReplicaInfo { group_id, state };
            node_state.replicas.insert(replica_id, replica_info);
        }

        Ok(())
    }

    /// Create a replica.
    ///
    /// The replica state is determined by the `GroupDesc`.
    ///
    /// NOTE: This function is idempotent.
    pub async fn create_replica(
        &self,
        replica_id: u64,
        group: GroupDesc,
        recovered: bool,
    ) -> Result<()> {
        let mut node_state = self.node_state.lock().await;
        if node_state.replicas.contains_key(&replica_id) {
            debug!(replica = replica_id, "replica already exists");
            return Ok(());
        }

        let group_id = group.id;
        let group_engine = GroupEngine::create(self.raw_db.clone(), &group).await?;
        Replica::create(replica_id, group_engine, &group).await?;
        let replica_state = if group.replicas.is_empty() {
            ReplicaState::Pending
        } else {
            ReplicaState::Initial
        };
        self.state_engine
            .save_replica_state(group_id, replica_id, replica_state)
            .await?;

        if recovered {
            let replica_info = ReplicaInfo {
                group_id,
                state: replica_state,
            };
            node_state.replicas.insert(replica_id, replica_info);
        }

        Ok(())
    }

    /// Terminate specified replica.
    pub async fn terminate_replica(&self, replica_id: u64) -> Result<()> {
        todo!()
    }

    /// Open replica and start serving raft requests.
    pub async fn start_replica(&self, replica_id: u64) -> Result<Option<()>> {
        let node_state = self.node_state.lock().await;
        let info = match node_state.replicas.get(&replica_id) {
            Some(info) => info.clone(),
            None => return Ok(None),
        };

        self.start_replica_with_state(info.group_id, replica_id, info.state)
            .await?
            .expect("replica state exists but group are missed?");

        Ok(Some(()))
    }

    async fn start_replica_with_state(
        &self,
        group_id: u64,
        replica_id: u64,
        state: ReplicaState,
    ) -> Result<Option<()>> {
        let group_engine = match GroupEngine::open(group_id, self.raw_db.clone()).await? {
            Some(group_engine) => group_engine,
            None => return Ok(None),
        };

        let replica = Replica::open(group_id, replica_id, group_engine).await?;
        self.replica_route_table.update(Arc::new(replica));
        // FIXME(walter) set raft sender.
        // self.raft_route_table.upsert(replica_id, ());

        // TODO(walter) start replica workers.

        Ok(Some(()))
    }

    #[inline]
    pub fn replica_table(&self) -> &ReplicaRouteTable {
        &self.replica_route_table
    }

    #[inline]
    pub fn raft_route_table(&self) -> &RaftRouteTable {
        &self.raft_route_table
    }

    #[inline]
    pub fn state_engine(&self) -> &StateEngine {
        &self.state_engine
    }

    #[inline]
    pub fn executor(&self) -> &Executor {
        &self.executor
    }
}

#[cfg(test)]
mod tests {
    use engula_api::server::v1::ReplicaDesc;
    use tempdir::TempDir;

    use super::*;
    use crate::runtime::ExecutorOwner;

    fn create_node(executor: Executor) -> Node {
        let tmp_dir = TempDir::new("rocksdb").unwrap();

        use crate::bootstrap::open_engine;

        let db = open_engine(tmp_dir).unwrap();
        let db = Arc::new(db);
        let state_engine = StateEngine::new(db.clone()).unwrap();
        Node::new(db, state_engine, executor)
    }

    async fn replica_state(node: Node, replica_id: u64) -> Option<ReplicaState> {
        node.state_engine()
            .iterate_replica_states()
            .await
            .filter(|(_, id, _)| *id == replica_id)
            .map(|(_, _, state)| state)
            .next()
    }

    #[test]
    fn create_pending_replica() {
        let executor_owner = ExecutorOwner::new(1);
        let executor = executor_owner.executor();
        let node = create_node(executor.clone());

        let group_id = 2;
        let replica_id = 2;
        let group = GroupDesc {
            id: group_id,
            shards: vec![],
            replicas: vec![],
        };

        executor.block_on(async {
            node.create_replica(replica_id, group, false).await.unwrap();

            assert!(matches!(
                replica_state(node, replica_id).await,
                Some(ReplicaState::Pending),
            ));
        });
    }

    #[test]
    fn create_replica() {
        let executor_owner = ExecutorOwner::new(1);
        let executor = executor_owner.executor();
        let node = create_node(executor.clone());

        let group_id = 2;
        let replica_id = 2;
        let group = GroupDesc {
            id: group_id,
            shards: vec![],
            replicas: vec![ReplicaDesc {
                id: replica_id,
                node_id: 1,
            }],
        };

        executor.block_on(async {
            node.create_replica(replica_id, group, false).await.unwrap();

            assert!(matches!(
                replica_state(node, replica_id).await,
                Some(ReplicaState::Initial),
            ));
        });
    }

    #[test]
    fn recover() {
        let executor_owner = ExecutorOwner::new(1);
        let executor = executor_owner.executor();
        let node = create_node(executor.clone());

        let group_id = 2;
        let replica_id = 2;
        let group = GroupDesc {
            id: group_id,
            shards: vec![],
            replicas: vec![],
        };

        executor.block_on(async {
            node.create_replica(replica_id, group, false).await.unwrap();
        });

        drop(node);
        let node = create_node(executor.clone());
        executor.block_on(async {
            node.recover().await.unwrap();
        })
    }
}