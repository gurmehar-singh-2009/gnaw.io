use snafu::prelude::*;

use chacha20poly1305::{KeyInit, XChaCha20Poly1305};
use getrandom::{
    SysRng,
    rand_core::{TryRng, UnwrapErr},
};
use libcrux_ml_kem::mlkem1024::{MlKem1024PrivateKey, MlKem1024PublicKey};
use wtransport::{Endpoint, Identity, ServerConfig};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::errors::{
    ApplicationError, ChaChaCipherEstablishSnafu, FillRngBufferSnafu,
    WTransportFaultySessionReqSnafu, WTransportServerInitFailSnafu,
};

pub trait NetworkingState {}

pub struct HandshakeState {
    x25519_secret: EphemeralSecret,
    x25519_public: PublicKey,
    kem_private: MlKem1024PrivateKey,
    kem_public: MlKem1024PublicKey,
}

pub struct ConnectedState {
    cipher: XChaCha20Poly1305, 
    tx_counter: u64,
}

impl NetworkingState for HandshakeState {}
impl NetworkingState for ConnectedState {}

pub struct NetworkConnection<S: NetworkingState> {
    state: S,
}

impl NetworkConnection<HandshakeState> {
    pub fn new() -> Result<Self, ApplicationError> {
        let mut crypto_rng = UnwrapErr(SysRng);

        let mut rand_bytes = [0u8; 64];
        SysRng
            .try_fill_bytes(&mut rand_bytes)
            .context(FillRngBufferSnafu)?;

        let (kem_private, kem_public) =
            libcrux_ml_kem::mlkem1024::generate_key_pair(rand_bytes).into_parts();

        let x25519_secret = EphemeralSecret::random_from_rng(&mut crypto_rng);
        let x25519_public = PublicKey::from(&x25519_secret);

        Ok(Self {
            state: HandshakeState {
                x25519_secret,
                x25519_public,
                kem_private,
                kem_public,
            },
        })
    }

    pub fn x25519_public(&self) -> [u8; 32] {
        self.state.x25519_public.to_bytes()
    }

    pub fn kem_public(&self) -> &MlKem1024PublicKey {
        &self.state.kem_public
    }

    pub fn finish(
        self,
        their_public: [u8; 32],
    ) -> Result<NetworkConnection<ConnectedState>, ApplicationError> {
        let shared_secret = self
            .state
            .x25519_secret
            .diffie_hellman(&PublicKey::from(their_public));

        let cipher = XChaCha20Poly1305::new_from_slice(shared_secret.as_bytes())
            .context(ChaChaCipherEstablishSnafu)?;

        Ok(NetworkConnection {
            state: ConnectedState {
                cipher,
                tx_counter: 0,
            },
        })
    }
}

impl NetworkConnection<ConnectedState> {
    pub fn send(&mut self, _data: &[u8]) -> Result<(), ApplicationError> {
        self.state.tx_counter += 1;
        Ok(())
    }
}

pub struct NetworkManager {
    pending_connections: Vec<NetworkConnection<HandshakeState>>,
    active_connections: Vec<NetworkConnection<ConnectedState>>,
}

impl NetworkManager {
    pub async fn new() -> Self {
        tokio::spawn(async move {
            Self::start_server().await;
        });

        Self {
            pending_connections: Vec::with_capacity(10),
            active_connections: Vec::with_capacity(40),
        }
    }

    pub async fn start_server() -> Result<(), ApplicationError> {
        let identity = Identity::load_pemfiles("certs/cert.pem", "certs/key.pem")
            .await
            .unwrap();

        let config = ServerConfig::builder()
            .with_bind_default(4433)
            .with_identity(identity)
            .build();

        let connection = Endpoint::server(config).context(WTransportServerInitFailSnafu)?;

        let incoming_connection = connection.accept().await;
        let session_request = incoming_connection
            .await
            .context(WTransportFaultySessionReqSnafu)?;
        let request = session_request
            .accept()
            .await
            .context(WTransportFaultySessionReqSnafu)?;

        let (send_stream, mut recv_stream) = request.accept_bi().await.unwrap();

        tokio::spawn(async move {
            let mut read_buffer = [0u8; 8192];
            loop {
                let msg = recv_stream.read(&mut read_buffer).await.unwrap();

                if let None = msg {
                    // there was an error reading the message!!
                    // idk what to do if that happens, figure out later.
                }
            }
        });

        Ok(())
    }

    pub fn begin_handshake(&mut self) -> Result<usize, ApplicationError> {
        self.pending_connections.push(NetworkConnection::new()?);
        Ok(self.pending_connections.len() - 1)
    }

    pub fn promote(
        &mut self,
        pending_index: usize,
        their_public: [u8; 32],
    ) -> Result<(), ApplicationError> {
        let pending = self.pending_connections.swap_remove(pending_index);
        let active = pending.finish(their_public)?;
        self.active_connections.push(active);
        Ok(())
    }
}
