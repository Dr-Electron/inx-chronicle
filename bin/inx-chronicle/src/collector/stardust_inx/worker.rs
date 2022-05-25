// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;

use async_trait::async_trait;
use chronicle::{
    db::model::stardust::block::BlockId,
    runtime::{Actor, ActorContext, ActorError, Addr, HandleEvent, Report, Sender, SpawnActor},
};
use inx::{client::InxClient, proto::NoParams, tonic::Channel};

use super::{
    config::InxConfig,
    error::InxWorkerError,
    listener::{InxListener, StardustInxListenerError},
};
use crate::collector::{
    solidifier::Solidifier,
    stardust_inx::{MilestoneState, RequestedBlock},
    Collector,
};

#[derive(Debug)]
pub struct InxWorker {
    config: InxConfig,
}

impl InxWorker {
    pub fn new(config: InxConfig) -> Self {
        Self { config }
    }
}

pub struct Inx;

impl Inx {
    /// Creates an [`InxClient`] by connecting to the endpoint specified in `inx_config`.
    async fn connect(inx_config: &InxConfig) -> Result<InxClient<Channel>, InxWorkerError> {
        let url = url::Url::parse(&inx_config.connect_url)?;

        if url.scheme() != "http" {
            return Err(InxWorkerError::InvalidAddress(inx_config.connect_url.clone()));
        }

        InxClient::connect(inx_config.connect_url.clone())
            .await
            .map_err(InxWorkerError::ConnectionError)
    }
}

#[async_trait]
impl Actor for InxWorker {
    type State = InxClient<Channel>;
    type Error = InxWorkerError;

    async fn init(&mut self, cx: &mut ActorContext<Self>) -> Result<Self::State, Self::Error> {
        log::info!("Connecting to INX at bind address `{}`.", self.config.connect_url);
        let mut inx_client = Inx::connect(&self.config).await?;

        log::info!("Connected to INX.");
        let node_status = inx_client.read_node_status(NoParams {}).await?.into_inner();

        if !node_status.is_healthy {
            log::warn!("Node is unhealthy.");
        }
        log::info!("Node is at ledger index `{}`.", node_status.ledger_index);

        cx.spawn_child(InxListener::new(inx_client.clone())).await;

        Ok(inx_client)
    }
}

#[async_trait]
impl HandleEvent<Report<InxListener>> for InxWorker {
    async fn handle_event(
        &mut self,
        cx: &mut ActorContext<Self>,
        event: Report<InxListener>,
        inx_client: &mut Self::State,
    ) -> Result<(), Self::Error> {
        match &event {
            Report::Success(_) => {
                cx.shutdown();
            }
            Report::Error(report) => match &report.error {
                ActorError::Result(e) => match e {
                    StardustInxListenerError::SubscriptionFailed(_) => {
                        cx.shutdown();
                    }
                    StardustInxListenerError::Runtime(_) => {
                        cx.shutdown();
                    }
                    StardustInxListenerError::MissingCollector => {
                        cx.delay(SpawnActor::new(InxListener::new(inx_client.clone())), None)?;
                    }
                },
                ActorError::Panic | ActorError::Aborted => {
                    cx.shutdown();
                }
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum InxRequestType {
    Block(BlockId),
    Metadata(BlockId),
}

#[derive(Debug)]
pub struct InxRequest {
    request_type: InxRequestType,
    solidifier_addr: Addr<Solidifier>,
    ms_state: MilestoneState,
}

impl InxRequest {
    pub fn get_block(block_id: BlockId, solidifier_addr: Addr<Solidifier>, ms_state: MilestoneState) -> Self {
        Self {
            request_type: InxRequestType::Block(block_id),
            solidifier_addr,
            ms_state,
        }
    }

    pub fn get_metadata(block_id: BlockId, solidifier_addr: Addr<Solidifier>, ms_state: MilestoneState) -> Self {
        Self {
            request_type: InxRequestType::Metadata(block_id),
            solidifier_addr,
            ms_state,
        }
    }
}

#[async_trait]
impl HandleEvent<InxRequest> for InxWorker {
    async fn handle_event(
        &mut self,
        cx: &mut ActorContext<Self>,
        InxRequest {
            request_type,
            solidifier_addr,
            mut ms_state,
        }: InxRequest,
        inx_client: &mut Self::State,
    ) -> Result<(), Self::Error> {
        match request_type {
            InxRequestType::Block(block_id) => {
                match (
                    inx_client
                        .read_block(inx::proto::BlockId {
                            id: block_id.0.clone().into(),
                        })
                        .await,
                    inx_client
                        .read_block_metadata(inx::proto::BlockId { id: block_id.0.into() })
                        .await,
                ) {
                    (Ok(raw), Ok(metadata)) => {
                        let (raw, metadata) = (raw.into_inner(), metadata.into_inner());
                        cx.addr::<Collector>()
                            .await
                            .send(RequestedBlock::new(Some(raw), metadata, solidifier_addr, ms_state))
                            .map_err(|_| InxWorkerError::MissingCollector)?;
                    }
                    (Err(e), Ok(metadata)) => {
                        let metadata = metadata.into_inner();
                        // If this isn't a block we care about, don't worry, be happy
                        if metadata.milestone_index != ms_state.milestone_index || !metadata.solid {
                            ms_state.process_queue.pop_front();
                            solidifier_addr.send(ms_state)?;
                        } else {
                            log::warn!("Failed to read block: {:?}", e);
                        }
                    }
                    (_, Err(e)) => {
                        log::warn!("Failed to read metadata: {:?}", e);
                    }
                }
            }
            InxRequestType::Metadata(block_id) => {
                if let Ok(metadata) = inx_client
                    .read_block_metadata(inx::proto::BlockId { id: block_id.0.into() })
                    .await
                {
                    let metadata = metadata.into_inner();
                    cx.addr::<Collector>()
                        .await
                        .send(RequestedBlock::new(None, metadata, solidifier_addr, ms_state))
                        .map_err(|_| InxWorkerError::MissingCollector)?;
                }
            }
        }

        Ok(())
    }
}