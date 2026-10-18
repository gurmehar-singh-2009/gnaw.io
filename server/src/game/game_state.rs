use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;

use crate::game::{
    entity::entity_manager::EntityManager, network::network_manager::NetworkManager,
};

pub struct GameState {
    network_manager: NetworkManager,
    entity_manager: EntityManager,
    tick: Arc<Notify>,
}

impl GameState {
    pub fn start_game(network_manager: NetworkManager, entity_manager: EntityManager) -> Self {
        let tick = Arc::new(Notify::new());
        let tick_task = tick.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));
            loop {
                interval.tick().await;

                tick_task.notify_waiters();
            }
        });

        Self {
            network_manager,
            entity_manager,
            tick,
        }
    }

    pub async fn await_tick(&self) {
        self.tick.notified().await;
    }
}
