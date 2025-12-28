//! Tests for the database worker

use super::*;
use crate::db::async_bridge::{ConnectionParams, DbCommand, DbResponse};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

/// Maximum time to wait for a response in tests
const TEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Helper function to wait for a response with timeout
fn wait_for_response(handle: &DbWorkerHandle) -> DbResponse {
    let start = Instant::now();
    loop {
        match handle.try_recv() {
            Ok(resp) => return resp,
            Err(mpsc::TryRecvError::Empty) => {
                if start.elapsed() > TEST_TIMEOUT {
                    panic!(
                        "Test timeout: no response received within {:?}",
                        TEST_TIMEOUT
                    );
                }
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                panic!("Worker disconnected unexpectedly");
            }
        }
    }
}

#[test]
fn test_worker_shutdown() {
    let (cmd_tx, cmd_rx) = mpsc::channel();
    let (resp_tx, _resp_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let worker = DbWorker::new(cmd_rx, resp_tx);
        worker.run();
    });

    // Send shutdown command
    cmd_tx.send(DbCommand::Shutdown).unwrap();

    // Worker should exit gracefully
    handle.join().expect("Worker thread panicked");
}

#[test]
fn test_worker_channel_close() {
    let (cmd_tx, cmd_rx) = mpsc::channel::<DbCommand>();
    let (resp_tx, _resp_rx) = mpsc::channel();

    let handle = thread::spawn(move || {
        let worker = DbWorker::new(cmd_rx, resp_tx);
        worker.run();
    });

    // Drop the sender to close the channel
    drop(cmd_tx);

    // Worker should exit gracefully when channel closes
    handle.join().expect("Worker thread panicked");
}

#[test]
fn test_spawn_db_worker() {
    let handle = spawn_db_worker();

    // Shutdown cleanly
    handle.shutdown();
}

#[test]
fn test_worker_handle_drop() {
    let handle = spawn_db_worker();

    // Just drop it - should shutdown cleanly via Drop impl
    drop(handle);
}

#[test]
fn test_fetch_tables_connection_error() {
    let handle = spawn_db_worker();

    // Send a command with invalid connection params
    let invalid_conn = ConnectionParams {
        host: "invalid-host-that-does-not-exist.local".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };

    handle
        .send(DbCommand::FetchTables {
            request_id: 1,
            connection: invalid_conn,
            schema: Some("public".to_string()),
            target: (0, 0),
        })
        .unwrap();

    // Wait for response with timeout
    let response = wait_for_response(&handle);

    // Should get an error response
    match response {
        DbResponse::TablesLoaded {
            request_id, result, ..
        } => {
            assert_eq!(request_id, 1);
            assert!(result.is_err());
        }
        _ => panic!("Expected TablesLoaded response"),
    }

    handle.shutdown();
}

#[test]
fn test_fetch_table_details_connection_error() {
    let handle = spawn_db_worker();

    let invalid_conn = ConnectionParams {
        host: "invalid-host-that-does-not-exist.local".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };

    handle
        .send(DbCommand::FetchTableDetails {
            request_id: 2,
            connection: invalid_conn,
            table_name: "users".to_string(),
            schema: Some("public".to_string()),
            target: (0, 0, 0),
        })
        .unwrap();

    // Wait for response with timeout
    let response = wait_for_response(&handle);

    match response {
        DbResponse::TableDetailsLoaded {
            request_id, result, ..
        } => {
            assert_eq!(request_id, 2);
            assert!(result.is_err());
        }
        _ => panic!("Expected TableDetailsLoaded response"),
    }

    handle.shutdown();
}

#[test]
fn test_execute_query_connection_error() {
    let handle = spawn_db_worker();

    let invalid_conn = ConnectionParams {
        host: "invalid-host-that-does-not-exist.local".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };

    handle
        .send(DbCommand::ExecuteQuery {
            request_id: 3,
            connection: invalid_conn,
            query: "SELECT 1".to_string(),
            project_idx: 0,
        })
        .unwrap();

    // Wait for response with timeout
    let response = wait_for_response(&handle);

    match response {
        DbResponse::QueryExecuted {
            request_id, result, ..
        } => {
            assert_eq!(request_id, 3);
            assert!(result.is_err());
        }
        _ => panic!("Expected QueryExecuted response"),
    }

    handle.shutdown();
}

#[test]
fn test_multiple_commands() {
    let handle = spawn_db_worker();

    let invalid_conn = ConnectionParams {
        host: "invalid-host".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        username: "testuser".to_string(),
        password: "testpass".to_string(),
    };

    // Send multiple commands
    for i in 0..3 {
        handle
            .send(DbCommand::FetchTables {
                request_id: i,
                connection: invalid_conn.clone(),
                schema: None,
                target: (0, i as usize),
            })
            .unwrap();
    }

    // Collect all responses with timeout
    let mut responses = Vec::new();
    for _ in 0..3 {
        responses.push(wait_for_response(&handle));
    }

    // Should have received all 3 responses
    assert_eq!(responses.len(), 3);

    // All should be TablesLoaded with errors
    for resp in responses {
        match resp {
            DbResponse::TablesLoaded { result, .. } => {
                assert!(result.is_err());
            }
            _ => panic!("Expected TablesLoaded response"),
        }
    }

    handle.shutdown();
}
