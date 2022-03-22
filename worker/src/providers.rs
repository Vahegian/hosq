use std::sync::{Arc, Mutex};

use rocket::{
    fairing::{Fairing, Info, Kind},
    tokio, Orbit, Rocket,
};
use web3::{transports::WebSocket, Error, Web3};

use crate::types::{config::Provider, State, Web3Node};

#[derive(Debug, Clone)]
pub struct Providers {}

impl Providers {
    pub async fn create_provider(&self, url: &str) -> Result<Web3<WebSocket>, Error> {
        let transport = web3::transports::WebSocket::new(url).await?;
        Ok(Web3::new(transport))
    }

    pub async fn get_block_num(&self, socket: &Web3<WebSocket>) -> Result<u64, Error> {
        Ok(socket.eth().block_number().await?.as_u64())
    }

    pub async fn get_providers(&self, providers: Vec<Provider>) -> Result<Vec<Web3Node>, Error> {
        let mut providers_manage = vec![];
        for provider in providers {
            let socket = self.create_provider(&(provider.provider)).await?;
            let chain_id = socket.eth().chain_id().await?.as_u64() as i64;
            let latest_block = self.get_block_num(&socket).await? as i64;

            providers_manage.push(Web3Node {
                contract_address: provider.contract_address.to_owned(),
                url: provider.provider.to_owned(),
                chain_name: provider.chain_name.to_owned(),
                start_block: provider.start_block,
                block_time_sec: provider.block_time_sec,
                update_interval_sec: provider.update_interval_sec,
                provider_id: provider.provider_id,
                chain_id,
                log_update_sec: provider.log_update_sec,
                batch_size: provider.batch_size,
                web3: Arc::new(Mutex::new(socket)),
                latest_block: Arc::new(Mutex::new(Some(latest_block))),
                skip_old: provider.skip_old,
                keep_alive: provider.keep_alive,
            });
        }
        Ok(providers_manage)
    }
}

#[rocket::async_trait]
impl Fairing for Providers {
    fn info(&self) -> Info {
        Info {
            name: "Run contract watcher service",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let providers = rocket.state::<State>().unwrap().providers.clone();
        // let shutdown = rocket.shutdown();

        for provider in &*providers {
            let (p, this) = (provider.clone(), self.clone());
            tokio::spawn(async move {
                loop {
                    let web3 = { p.web3.clone().lock().unwrap().clone() };
                    let bn = match web3.eth().block_number().await {
                        Ok(v) => v.as_u64() as i64,
                        Err(e) => {
                            error!("Error getting block number for {}: {:?}", &p.chain_name, e);
                            // r_off.notify();
                            info!(
                                "CHAIN '{}' '{}' > Creating new connection",
                                &p.chain_name, p.chain_id
                            );
                            let new_socket = this.create_provider(&p.url).await.unwrap();
                            {
                                let mut socket = p.web3.lock().unwrap();
                                *socket = new_socket;
                            }
                            continue;
                        }
                    };

                    {
                        let mut data = p.latest_block.lock().unwrap();
                        *data = Some(bn);
                    }

                    info!(
                        "CHAIN '{}' - '{}' > Socket is alive at block '{}'",
                        p.chain_name, p.chain_id, bn
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(p.update_interval_sec))
                        .await;
                }
            });
        }
    }
}
