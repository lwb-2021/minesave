mod tcp;

use std::io::Write;

use crate::{
    backup::{MinecraftSaveCollection, MinecraftSaveVersion},
    error::{MyError, Result},
};
use log::{debug, error, info};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Action {
    List = 0,
    Backup = 1,
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
        "backup type: 0=Full 1=IncreasementFile 2=IncreasementData 3=Snapshot 255=Default"
    )?;
    Ok(())
}

pub async fn handle_request(data: ApiData) -> Result<()> {
    debug!("request received: {:?}", data);
    match data.action {
        Action::List => {
            todo!()
        }
        Action::Backup => {
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
            let version = {
                MinecraftSaveVersion::create(
                    unsafe { std::mem::transmute_copy(&t) },
                    data.target,
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
            MinecraftSaveCollection::global().lock().unwrap().save()?;
        }
        #[allow(unreachable_patterns)]
        i => {
            error!("Unexpected request {:?}", data);
            return Err(MyError::IllegalArgument {
                name: "action_id".to_string(),
                value: format!("{:?}", i),
                expected: "0, 1".to_string(),
            });
        }
    }
    Ok(())
}
