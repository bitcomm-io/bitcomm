mod login;
mod logout;
mod pingpong;

use bytes::Bytes;
use s2n_quic::stream::SendStream;

use std::sync::Arc;
// use tokio::io::AsyncWriteExt;
use crate::object::gram::{
    command::CommandGram, BitCommand, BitcommFlag,
};

// use super::client::{get_cd_by_key, ClientPoolManager};

pub async fn process_command_data<'a>(
    reqcmdbuff: &Arc<Bytes>,
    reqcmdgram: &Arc<CommandGram>,
    stm: &Arc<tokio::sync::Mutex<SendStream>>,
) {
    match reqcmdgram.command() {
        BitCommand::LOGIN_COMMAND => {
            login::process_command_login(reqcmdbuff, reqcmdgram, stm).await
        }
        BitCommand::LOGOUT_COMMAND => {
            logout::process_command_logout(reqcmdbuff, reqcmdgram, stm).await
        }
        // BitCommand::SEND_PING =>
        //     pingpong::process_command_pingpong(reqcmdbuff, reqcmdgram, stm).await,
        _ => {}
    }
}
//
pub async fn process_pingpong<'a>(
    pingpong: &Arc<BitcommFlag>,
    stm: &Arc<tokio::sync::Mutex<SendStream>>,
) {
    pingpong::process_pingpong(pingpong, stm).await;
}