// Copyright 2024 Tree xie.
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

use crate::config::{
    self, get_config_path, get_current_config, load_config, PingapConf, CATEGORY_LOCATION,
    CATEGORY_UPSTREAM,
};
use crate::service::{CommonServiceTask, ServiceTask};
use crate::state::restart;
use crate::{proxy, webhook};
use async_trait::async_trait;
use log::{error, info};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

async fn hot_reload(
    hot_reload_only: bool,
) -> Result<(bool, Vec<String>), Box<dyn std::error::Error>> {
    let conf = load_config(&get_config_path(), false).await?;
    conf.validate()?;
    let mut current_conf: PingapConf = get_current_config().as_ref().clone();
    let mut should_reload_server_location = false;
    for (name, server) in conf.servers.iter() {
        if let Some(old) = current_conf.servers.get_mut(name) {
            if server.locations != old.locations {
                should_reload_server_location = true;
                old.locations.clone_from(&server.locations);
            }
        }
    }
    let (updated_category_list, diff_result) = current_conf.diff(&conf);
    if !should_reload_server_location && updated_category_list.is_empty() {
        return Ok((false, vec![]));
    }

    let mut should_reload_upstream = false;
    let mut should_reload_location = false;
    let mut should_restart = false;
    for category in updated_category_list {
        match category.as_str() {
            CATEGORY_LOCATION => should_reload_location = true,
            CATEGORY_UPSTREAM => should_reload_upstream = true,
            _ => should_restart = true,
        };
    }

    if should_reload_upstream {
        match proxy::try_init_upstreams(&conf.upstreams) {
            Err(e) => {
                error!("Reload upstream fail, error: {e:?}");
            }
            Ok(()) => {
                info!("Reload upstream success");
            }
        };
    }
    if should_reload_location {
        match proxy::try_init_locations(&conf.locations) {
            Err(e) => {
                error!("Reload location fail, error: {e:?}");
            }
            Ok(()) => {
                info!("Reload location success");
            }
        };
    }
    if should_reload_server_location {
        match proxy::try_init_server_locations(&conf.servers, &conf.locations) {
            Err(e) => {
                error!("Reload server location fail, error: {e:?}");
            }
            Ok(()) => {
                info!("Reload server location success");
            }
        };
    }

    if hot_reload_only {
        // update current config only hot reload config updated
        // the next check will not trigger hot reload
        if !should_restart {
            config::set_current_config(&conf);
        }
        return Ok((false, vec![]));
    }

    config::set_current_config(&conf);
    if should_restart {
        return Ok((true, diff_result));
    }

    Ok((false, vec![]))
}

struct AutoRestart {
    restart_unit: u32,
    count: AtomicU32,
}

pub fn new_auto_restart_service(interval: Duration) -> CommonServiceTask {
    let mut restart_unit = 1_u32;
    let unit = Duration::from_secs(30);
    if interval > unit {
        restart_unit = (interval.as_secs() / unit.as_secs()) as u32;
    }

    CommonServiceTask::new(
        "Auto restart checker",
        interval.min(unit),
        AutoRestart {
            restart_unit,
            count: AtomicU32::new(0),
        },
    )
}

#[async_trait]
impl ServiceTask for AutoRestart {
    async fn run(&self) -> Option<bool> {
        let count = self.count.fetch_add(1, Ordering::Relaxed);
        let hot_reload_only = if count > 0 && self.restart_unit > 1 {
            count % self.restart_unit != 0
        } else {
            true
        };
        match hot_reload(hot_reload_only).await {
            Ok((should_restart, diff_result)) => {
                if should_restart {
                    webhook::send(webhook::SendNotificationParams {
                        level: webhook::NotificationLevel::Info,
                        category: webhook::NotificationCategory::DiffConfig,
                        msg: diff_result.join("\n"),
                    });
                    restart();
                }
            }
            Err(e) => {
                error!("Auto restart validate fail, {e}");
            }
        }
        None
    }
    fn description(&self) -> String {
        "pingap will be restart if config changed".to_string()
    }
}
