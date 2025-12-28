//! Worker handle and spawning functionality

use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

use crate::db::async_bridge::{DbCommand, DbResponse};

use super::DbWorker;

/// Handle to the spawned worker thread
pub struct DbWorkerHandle {
    /// Channel to send commands to the worker
    pub command_tx: Sender<DbCommand>,
    /// Channel to receive responses from the worker
    pub response_rx: Receiver<DbResponse>,
    /// Handle to the worker thread (for joining on shutdown)
    thread_handle: Option<JoinHandle<()>>,
}

impl DbWorkerHandle {
    /// Send a command to the worker
    #[allow(clippy::result_large_err)]
    pub fn send(&self, cmd: DbCommand) -> Result<(), std::sync::mpsc::SendError<DbCommand>> {
        self.command_tx.send(cmd)
    }

    /// Try to receive a response without blocking
    pub fn try_recv(&self) -> Result<DbResponse, std::sync::mpsc::TryRecvError> {
        self.response_rx.try_recv()
    }

    /// Shutdown the worker and wait for it to finish
    pub fn shutdown(mut self) {
        let _ = self.command_tx.send(DbCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for DbWorkerHandle {
    fn drop(&mut self) {
        // Send shutdown signal if we still have the handle
        let _ = self.command_tx.send(DbCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spawn a new database worker thread
///
/// Returns a handle that can be used to send commands and receive responses.
pub fn spawn_db_worker() -> DbWorkerHandle {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    let (resp_tx, resp_rx) = std::sync::mpsc::channel();

    let handle = thread::Builder::new()
        .name("db-worker".to_string())
        .spawn(move || {
            let worker = DbWorker::new(cmd_rx, resp_tx);
            worker.run();
        })
        .expect("Failed to spawn db-worker thread");

    DbWorkerHandle {
        command_tx: cmd_tx,
        response_rx: resp_rx,
        thread_handle: Some(handle),
    }
}
