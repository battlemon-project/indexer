use crate::Result as BattlemonResult;
use battlemon_near_json_rpc_client_wrapper::JsonRpcWrapper;
use near_crypto::{InMemorySigner, PublicKey, SecretKey};
use near_lake_framework::near_indexer_primitives::types::AccountId;
use near_lake_framework::LakeConfig;
use serde::Deserialize;
use sqlx::postgres::PgConnectOptions;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct AppSettings {
    pub contracts: ContractSettings,
    pub database: DatabaseSettings,
    pub aws: AwsSettings,
    pub near_credentials: NearCredsSettings,
}

#[derive(Deserialize)]
pub struct ContractSettings {
    pub top_contract_id: AccountId,
    pub nft_contract_id: AccountId,
    pub market_contract_id: AccountId,
}

#[derive(Deserialize, Clone)]
pub struct NearCredsSettings {
    pub account_id: AccountId,
    pub public_key: PublicKey,
    pub private_key: SecretKey,
}

impl From<NearCredsSettings> for InMemorySigner {
    fn from(near_creds: NearCredsSettings) -> Self {
        Self {
            account_id: near_creds.account_id,
            public_key: near_creds.public_key,
            secret_key: near_creds.private_key,
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct AwsSettings {
    pub s3_endpoint: Option<String>,
    pub s3_bucket_name: String,
    pub s3_region_name: String,
    pub start_block_height: u64,
    pub start_from_last_block: bool,
}

impl AwsSettings {
    pub async fn lake_config(&self, rpc_client: &JsonRpcWrapper) -> BattlemonResult<LakeConfig> {
        let mut ret = LakeConfig {
            s3_endpoint: self.s3_endpoint.clone(),
            s3_bucket_name: self.s3_bucket_name.clone(),
            s3_region_name: self.s3_region_name.clone(),
            start_block_height: self.start_block_height,
        };

        if self.start_from_last_block {
            ret.start_block_height = rpc_client.final_block_height().await?
        }

        Ok(ret)
    }
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub db_name: String,
}

impl DatabaseSettings {
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.db_name)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(&self.password)
            .port(self.port)
    }
}

#[tracing::instrument(
    name = "Loading configuration from file `config.yaml`",
    fields(id = %Uuid::new_v4())
)]
pub fn load_config() -> Result<AppSettings, config::ConfigError> {
    let mut settings = config::Config::default();
    settings.merge(config::File::with_name("config"))?;
    settings.try_into()
}
