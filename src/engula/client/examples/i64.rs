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

use anyhow::Result;
use engula_client::{Universe, I64};

#[tokio::main]
async fn main() -> Result<()> {
    let url = "http://localhost:21716";
    let uv = Universe::connect(url).await?;
    let db = uv.create_database("i64").await?;
    let co = db.create_collection::<I64>("i64").await?;

    co.set("o", 1).await?;
    println!("o = {:?}", co.get("o").await?);
    co.object("o").add(2).await?;
    println!("o = {:?}", co.get("o").await?);
    co.object("o").sub(3).await?;
    println!("o = {:?}", co.get("o").await?);

    let mut txn = co.object("o").begin();
    txn.add(1).add(2).sub(1);
    txn.commit().await?;
    println!("o = {:?}", co.object("o").load().await?);

    Ok(())
}
