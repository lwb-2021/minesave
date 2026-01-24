mod tcp;

use std::io::Write;

use crate::{
    backup::{MinecraftSaveCollection, MinecraftSaveVersion},
    error::{MyError, Result},
    globals::MACHINE,
};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Action {
    List = 0,
    Backup = 1,
    Recover = 2,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiData {
    pub action: Action,
    pub payload: Vec<String>,
}

pub fn write_api_help<W: Write>(stream: &mut W) -> Result<()> {
    writeln!(stream, "MineSave API help")?;
    writeln!(stream, "Json & Command are supported")?;
    writeln!(stream, "Command example: backup --filter=\"New World\"")?;
    writeln!(
        stream,
        "List: action_id=0, [filter=<regex applied to name>] -> Result<[{{name: string, versions: [{{type: int, description: string}}]}}]>"
    )?;
    writeln!(
        stream,
        "Backup: action_id=1, id=?, type=<backup type> -> Result"
    )?;
    writeln!(
        stream,
        "backup type: 0=Default 1=IncreasementData 2=Snapshot 255=FollowSettings"
    )?;
    writeln!(stream, "Recover: action_id=2, id=?, version_index=?")?;
    Ok(())
}


pub async fn handle_request(data: ApiData) -> Result<()> {
    debug!("request received: {:?}", data);
    match data.action {
        Action::List => {
            todo!()
        }
        Action::Backup => api_backup(data).await,
        Action::Recover => api_recover(data).await,
        #[allow(unreachable_patterns)]
        i => {
            error!("Unexpected request {:?}", data);
            Err(MyError::IllegalArgument {
                name: "action_id".to_string(),
                value: format!("{:?}", i),
                expected: "0, 1".to_string(),
            })
        }
    }
}

async fn api_backup(data: ApiData) -> Result<()> {
    let id: String = data.payload[0].clone();
    let t: u8 = data.payload[1]
        .parse()
        .map_err(|_| MyError::IllegalArgument {
            name: "type".to_string(),
            value: data.payload[1].to_string(),
            expected: "int".to_string(),
        })?;
    info!("backup: id={} type={}", id, t);
    let data = MinecraftSaveCollection::global().lock().unwrap().saves[&id].clone();
    let prev = {
        MinecraftSaveCollection::global()
            .lock()
            .unwrap()
            .saves
            .get_mut(&id)
            .unwrap()
            .latest_version
            .take()
            .map(Box::new)
    };
    if !data.target.contains_key(MACHINE.as_str()) {
        return Err(MyError::Other(anyhow::anyhow!(
            "Unable to find target path on local machine"
        )));
    }
    let version = {
        MinecraftSaveVersion::create(
            unsafe { std::mem::transmute_copy(&t) },
            data.target.get(MACHINE.as_str()).unwrap(),
            data.id.to_string(),
            "".to_string(),
            prev,
        )
        .await?
    };

    MinecraftSaveCollection::global()
        .lock()
        .unwrap()
        .saves
        .get_mut(&id)
        .unwrap()
        .latest_version = Some(version);
    MinecraftSaveCollection::global().lock().unwrap().save()
}

async fn api_recover(data: ApiData) -> Result<()> {
    let id = data.payload[0].to_string();
    let index: usize = data.payload[1]
        .parse()
        .map_err(|_| MyError::IllegalArgument {
            name: "version_index".to_string(),
            value: data.payload[1].to_string(),
            expected: "int".to_string(),
        })?;
    let mut cnt = index;
    let (latest, save_name, target_path) = {
        let global = MinecraftSaveCollection::global();
        let collection = global.lock().unwrap();
        let save = &collection.saves[&id];
        let mut latest = save
            .latest_version
            .as_ref()
            .ok_or(MyError::IllegalArgument {
                name: "version_index".to_string(),
                value: data.payload[1].to_string(),
                expected: "index out of bound".to_string(),
            })?;
        while cnt != 0 {
            if let Some(tmp) = &latest.prev.as_ref() {
                latest = tmp;
                cnt -= 1;
            } else {
                return Err(MyError::IllegalArgument {
                    name: "version_index".to_string(),
                    value: data.payload[1].to_string(),
                    expected: "index out of bound".to_string(),
                });
            }
        }
        info!(
            "recover: id={}, index={}, version_creation_time={}",
            id,
            index,
            latest.time.as_millis()
        );
        let target_path = save.target.get(MACHINE.as_str()).unwrap();
        (latest.clone(), save.name.clone(), target_path.clone())
    };
    latest
        .recover(target_path.with_file_name(save_name.to_string() + "-recover"))
        .await?;
    Ok(())
}
