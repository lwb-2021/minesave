mod tcp;

use std::io::Write;

use crate::{
    backup::{
        MinecraftSave, MinecraftSaveCollection, MinecraftSaveVersion, MinecraftSaveVersionType,
    },
    error::{MyError, Result},
};
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
    writeln!(
        stream,
        "Json & Plain text(split arguments with &, use '\\&' to replace '&') are supported"
    )?;
    writeln!(
        stream,
        "List: action_id=0, [filter=<regex applied to name>] -> Result<[{{name: string, versions: [{{type: int, description: string}}]}}]>"
    )?;
    writeln!(
        stream,
        "Backup: action_id=1, id=?, type=?(0: Full, 1: Increasement, 2: Snapshot) -> Result"
    )?;
    Ok(())
}

pub async fn handle_request(data: ApiData) -> Result<()> {
    println!("Request received: {:?}", data);
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
            println!("Backup request recognized: id={} type={}", id, t);
            let data = MinecraftSaveCollection::global().lock().unwrap().saves[&id].clone();
            let mut version = {
                MinecraftSaveVersion::create(
                    unsafe { std::mem::transmute_copy(&t) },
                    data.target,
                    data.id.to_string(),
                    "".to_string(),
                )
                .await?
            };

            let prev = {
                MinecraftSaveCollection::global()
                    .lock()
                    .unwrap()
                    .saves
                    .get_mut(&id)
                    .unwrap()
                    .latest_version
                    .take()
            };
            version.prev = prev.map(Box::new);
            MinecraftSaveCollection::global()
                .lock()
                .unwrap()
                .saves
                .get_mut(&id)
                .unwrap()
                .latest_version = Some(version);
        }
        #[allow(unreachable_patterns)]
        i => {
            return Err(MyError::IllegalArgument {
                name: "action_id".to_string(),
                value: format!("{:?}", i),
                expected: "0, 1".to_string(),
            });
        }
    }
    Ok(())
}
