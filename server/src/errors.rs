use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ApplicationError {
    #[snafu(display("Failed to fill a random buffer."))]
    FillRngBufferError { source: getrandom::Error },

    #[snafu(display("ChaCha Cipher construction error."))]
    ChaChaCipherEstablishError {
        source: chacha20poly1305::aead::common::InvalidLength,
    },

    #[snafu(display("Failed to setup WebTransport server."))]
    WTransportServerInitFail { source: std::io::Error },

    #[snafu(display("Failed to accept faulty session request."))]
    WTransportFaultySessionReq {
        source: wtransport::error::ConnectionError,
    },
}
